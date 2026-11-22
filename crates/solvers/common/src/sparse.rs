use crate::scalar::Scalar;

#[derive(Clone, Debug, PartialEq)]
pub struct SparseMatrix<T: Scalar> {
    n: usize,
    rows: Vec<Vec<(usize, T)>>,
}

impl<T: Scalar> SparseMatrix<T> {
    pub fn from_triplets(n: usize, triplets: &[(usize, usize, T)]) -> Self {
        let mut order: Vec<usize> = (0..triplets.len()).collect();
        order.sort_by_key(|&k| (triplets[k].0, triplets[k].1, k));
        let mut rows: Vec<Vec<(usize, T)>> = vec![Vec::new(); n];
        for k in order {
            let (i, j, v) = triplets[k];
            let row = &mut rows[i];
            match row.last_mut() {
                Some(last) if last.0 == j => last.1 += v,
                _ => row.push((j, v)),
            }
        }
        SparseMatrix { n, rows }
    }

    pub fn identity(n: usize) -> Self {
        SparseMatrix {
            n,
            rows: (0..n).map(|i| vec![(i, T::one())]).collect(),
        }
    }

    pub fn dim(&self) -> usize {
        self.n
    }

    pub fn row(&self, i: usize) -> &[(usize, T)] {
        &self.rows[i]
    }

    pub fn nnz(&self) -> usize {
        self.rows.iter().map(Vec::len).sum()
    }

    pub fn get(&self, i: usize, j: usize) -> T {
        match self.rows[i].binary_search_by_key(&j, |e| e.0) {
            Ok(p) => self.rows[i][p].1,
            Err(_) => T::zero(),
        }
    }

    pub fn same_pattern(&self, other: &SparseMatrix<T>) -> bool {
        self.n == other.n
            && self
                .rows
                .iter()
                .zip(&other.rows)
                .all(|(a, b)| a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.0 == y.0))
    }

    pub fn mul_vec(&self, x: &[T]) -> Vec<T> {
        self.rows
            .iter()
            .map(|row| {
                let mut s = T::zero();
                for &(j, v) in row {
                    s += v * x[j];
                }
                s
            })
            .collect()
    }

    pub fn to_dense(&self) -> Vec<T> {
        let mut d = vec![T::zero(); self.n * self.n];
        for (i, row) in self.rows.iter().enumerate() {
            for &(j, v) in row {
                d[i * self.n + j] = v;
            }
        }
        d
    }

    pub(crate) fn rows(&self) -> &[Vec<(usize, T)>] {
        &self.rows
    }
}
