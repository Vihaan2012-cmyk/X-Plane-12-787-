use super::*;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 11) as f64) / ((1u64 << 53) as f64)
    }
    fn index(&mut self, n: usize) -> usize {
        ((self.next() * n as f64) as usize).min(n - 1)
    }
}

fn random_real(rng: &mut Rng, n: usize, per_row: usize, dominant: bool) -> SparseMatrix<f64> {
    let mut t = Vec::new();
    for i in 0..n {
        if dominant {
            t.push((i, i, 10.0 + rng.next()));
        }
        for _ in 0..per_row {
            let j = rng.index(n);
            t.push((i, j, rng.next() * 2.0 - 1.0));
        }
    }
    SparseMatrix::from_triplets(n, &t)
}

fn random_complex(rng: &mut Rng, n: usize, per_row: usize) -> SparseMatrix<Complex> {
    let mut t = Vec::new();
    for i in 0..n {
        t.push((i, i, Complex::new(8.0 + rng.next(), rng.next())));
        for _ in 0..per_row {
            let j = rng.index(n);
            t.push((i, j, Complex::new(rng.next() - 0.5, rng.next() - 0.5)));
        }
    }
    SparseMatrix::from_triplets(n, &t)
}

fn max_err(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| (x - y).abs()).fold(0.0, f64::max)
}

#[test]
fn complex_arithmetic() {
    let a = Complex::new(1.0, 2.0);
    let b = Complex::new(3.0, -1.0);
    let p = a * b;
    assert!((p.re - 5.0).abs() < 1e-15 && (p.im - 5.0).abs() < 1e-15);
    let q = p / b;
    assert!((q.re - 1.0).abs() < 1e-14 && (q.im - 2.0).abs() < 1e-14);
    let z = Complex::from_polar(2.0, 0.5);
    assert!((z.abs() - 2.0).abs() < 1e-14 && (z.arg() - 0.5).abs() < 1e-14);
}

#[test]
fn real_sparse_matches_dense() {
    let mut rng = Rng(1);
    for trial in 0..20 {
        let n = 5 + trial * 7;
        let m = random_real(&mut rng, n, 3, true);
        let b: Vec<f64> = (0..n).map(|_| rng.next()).collect();
        let lu = SparseLu::factor(&m, LuOptions::default()).unwrap();
        let x = lu.solve(&b);
        let xd = dense_solve(&m.to_dense(), n, &b).unwrap();
        assert!(max_err(&x, &xd) < 1e-9, "trial {trial}");
        let r = m.mul_vec(&x);
        assert!(max_err(&r, &b) < 1e-9);
    }
}

#[test]
fn pivoting_handles_zero_diagonal() {
    let mut rng = Rng(7);
    for trial in 0..20 {
        let n = 8 + trial * 3;
        let m0 = random_real(&mut rng, n, 4, false);
        let mut t = Vec::new();
        for i in 0..n {
            t.push((i, (i + 1) % n, 3.0 + rng.next()));
            for &(j, v) in m0.row(i) {
                t.push((i, j, v));
            }
        }
        let m = SparseMatrix::from_triplets(n, &t);
        let b: Vec<f64> = (0..n).map(|_| rng.next()).collect();
        let lu = SparseLu::factor(&m, LuOptions::default()).unwrap();
        let x = lu.solve(&b);
        let xd = dense_solve(&m.to_dense(), n, &b).unwrap();
        assert!(max_err(&x, &xd) < 1e-7, "trial {trial}");
    }
}

#[test]
fn complex_sparse_matches_dense() {
    let mut rng = Rng(3);
    for trial in 0..10 {
        let n = 10 + trial * 9;
        let m = random_complex(&mut rng, n, 3);
        let b: Vec<Complex> = (0..n).map(|_| Complex::new(rng.next(), rng.next())).collect();
        let lu = SparseLu::factor(&m, LuOptions::default()).unwrap();
        let x = lu.solve(&b);
        let xd = dense_solve(&m.to_dense(), n, &b).unwrap();
        for (p, q) in x.iter().zip(&xd) {
            assert!((*p - *q).abs() < 1e-9);
        }
    }
}

#[test]
fn refactor_reuses_ordering_and_matches() {
    let mut rng = Rng(11);
    let n = 60;
    let m = random_real(&mut rng, n, 3, true);
    let mut lu = SparseLu::factor(&m, LuOptions::default()).unwrap();
    let order = lu.ordering().to_vec();
    let mut t = Vec::new();
    for i in 0..n {
        for &(j, v) in m.row(i) {
            t.push((i, j, v * (1.0 + 0.3 * rng.next())));
        }
    }
    let m2 = SparseMatrix::from_triplets(n, &t);
    lu.refactor(&m2).unwrap();
    assert_eq!(lu.ordering(), order.as_slice());
    assert_eq!(lu.refactor_count(), 1);
    let b: Vec<f64> = (0..n).map(|_| rng.next()).collect();
    let x = lu.solve(&b);
    let xd = dense_solve(&m2.to_dense(), n, &b).unwrap();
    assert!(max_err(&x, &xd) < 1e-9);
}

#[test]
fn refactor_rejects_pattern_change() {
    let a = SparseMatrix::from_triplets(3, &[(0, 0, 1.0), (1, 1, 1.0), (2, 2, 1.0)]);
    let b = SparseMatrix::from_triplets(3, &[(0, 0, 1.0), (1, 1, 1.0), (2, 2, 1.0), (0, 2, 1.0)]);
    let mut lu = SparseLu::factor(&a, LuOptions::default()).unwrap();
    assert_eq!(lu.refactor(&b), Err(LuError::PatternChanged));
}

#[test]
fn refactor_repivots_when_pivot_collapses() {
    let a = SparseMatrix::from_triplets(
        3,
        &[
            (0, 0, 4.0),
            (0, 1, 1.0),
            (1, 0, 1.0),
            (1, 1, 4.0),
            (2, 2, 1.0),
            (0, 2, 1.0),
            (2, 0, 1.0),
        ],
    );
    let mut lu = SparseLu::factor(&a, LuOptions::default()).unwrap();
    let b = SparseMatrix::from_triplets(
        3,
        &[
            (0, 0, 1e-9),
            (0, 1, 1.0),
            (1, 0, 1.0),
            (1, 1, 4.0),
            (2, 2, 1.0),
            (0, 2, 1.0),
            (2, 0, 1.0),
        ],
    );
    lu.refactor(&b).unwrap();
    let rhs = [1.0, 2.0, 3.0];
    let x = lu.solve(&rhs);
    let r = b.mul_vec(&x);
    assert!(max_err(&r, &rhs) < 1e-8);
}

#[test]
fn multiple_right_hand_sides() {
    let mut rng = Rng(5);
    let n = 30;
    let m = random_real(&mut rng, n, 3, true);
    let lu = SparseLu::factor(&m, LuOptions::default()).unwrap();
    let rhs: Vec<Vec<f64>> = (0..4).map(|_| (0..n).map(|_| rng.next()).collect()).collect();
    let xs = lu.solve_many(&rhs);
    for (b, x) in rhs.iter().zip(&xs) {
        assert!(max_err(&m.mul_vec(x), b) < 1e-9);
    }
}

#[test]
fn singular_detected() {
    let t: Vec<(usize, usize, f64)> = (0..7).map(|i| (i, i, 1.0)).collect();
    let m = SparseMatrix::from_triplets(8, &t);
    assert!(matches!(
        SparseLu::factor(&m, LuOptions::default()),
        Err(LuError::Singular { .. })
    ));
}

#[test]
fn tiny_systems_use_dense() {
    let m = SparseMatrix::from_triplets(2, &[(0, 0, 2.0), (0, 1, 1.0), (1, 0, 1.0), (1, 1, 3.0)]);
    let x = solve_system(&m, &[3.0, 5.0]).unwrap();
    assert!((x[0] - 0.8).abs() < 1e-12 && (x[1] - 1.4).abs() < 1e-12);
}

#[test]
fn ordering_is_permutation_and_deterministic() {
    let mut rng = Rng(9);
    let m = random_real(&mut rng, 200, 3, true);
    let a = minimum_degree_ordering(&m);
    let b = minimum_degree_ordering(&m);
    assert_eq!(a, b);
    let mut s = a.clone();
    s.sort_unstable();
    assert_eq!(s, (0..200).collect::<Vec<_>>());
}

#[test]
fn bit_identical_repeat() {
    let mut rng = Rng(13);
    let m = random_real(&mut rng, 100, 3, true);
    let b: Vec<f64> = (0..100).map(|_| rng.next()).collect();
    let x1 = SparseLu::factor(&m, LuOptions::default()).unwrap().solve(&b);
    let x2 = SparseLu::factor(&m, LuOptions::default()).unwrap().solve(&b);
    assert_eq!(x1, x2);
}
