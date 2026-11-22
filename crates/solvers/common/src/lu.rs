use crate::dense::dense_solve;
use crate::ordering::minimum_degree_ordering;
use crate::scalar::Scalar;
use crate::sparse::SparseMatrix;
use std::collections::BTreeSet;

const DENSE_LIMIT: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LuError {
    Singular { step: usize },
    PatternChanged,
    DimensionMismatch,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LuOptions {
    pub pivot_threshold: f64,
    pub singular_tolerance: f64,
    pub refactor_threshold: f64,
}

impl Default for LuOptions {
    fn default() -> Self {
        LuOptions {
            pivot_threshold: 0.1,
            singular_tolerance: 1e-300,
            refactor_threshold: 1e-3,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SparseLu<T: Scalar> {
    n: usize,
    options: LuOptions,
    col_order: Vec<usize>,
    pivot_rows: Vec<usize>,
    l: Vec<Vec<(usize, T)>>,
    u: Vec<Vec<(usize, T)>>,
    u_diag: Vec<T>,
    pattern: SparseMatrix<T>,
    refactor_count: usize,
    repivot_count: usize,
}

fn find<T: Scalar>(row: &[(usize, T)], c: usize) -> Option<T> {
    row.binary_search_by_key(&c, |e| e.0).ok().map(|p| row[p].1)
}

fn merge_sub<T: Scalar>(
    target: &[(usize, T)],
    pivot: &[(usize, T)],
    f: T,
    skip: usize,
    fill: &mut Vec<usize>,
) -> Vec<(usize, T)> {
    let mut out = Vec::with_capacity(target.len() + pivot.len());
    let (mut i, mut j) = (0, 0);
    while i < target.len() || j < pivot.len() {
        let tc = target.get(i).map_or(usize::MAX, |e| e.0);
        let pc = pivot.get(j).map_or(usize::MAX, |e| e.0);
        if tc == pc {
            if tc != skip {
                out.push((tc, target[i].1 - f * pivot[j].1));
            }
            i += 1;
            j += 1;
        } else if tc < pc {
            out.push(target[i]);
            i += 1;
        } else {
            if pc != skip {
                out.push((pc, -(f * pivot[j].1)));
                fill.push(pc);
            }
            j += 1;
        }
    }
    out
}

impl<T: Scalar> SparseLu<T> {
    pub fn factor(m: &SparseMatrix<T>, options: LuOptions) -> Result<Self, LuError> {
        let order = minimum_degree_ordering(m);
        Self::factor_with_order(m, order, options)
    }

    pub fn factor_with_order(
        m: &SparseMatrix<T>,
        col_order: Vec<usize>,
        options: LuOptions,
    ) -> Result<Self, LuError> {
        if col_order.len() != m.dim() {
            return Err(LuError::DimensionMismatch);
        }
        let mut lu = SparseLu {
            n: m.dim(),
            options,
            col_order,
            pivot_rows: Vec::new(),
            l: Vec::new(),
            u: Vec::new(),
            u_diag: Vec::new(),
            pattern: m.clone(),
            refactor_count: 0,
            repivot_count: 0,
        };
        lu.eliminate(m, false)?;
        Ok(lu)
    }

    pub fn refactor(&mut self, m: &SparseMatrix<T>) -> Result<(), LuError> {
        if !self.pattern.same_pattern(m) {
            return Err(LuError::PatternChanged);
        }
        self.refactor_count += 1;
        self.pattern = m.clone();
        if self.eliminate(m, true).is_err() {
            self.repivot_count += 1;
            self.eliminate(m, false)?;
        }
        Ok(())
    }

    pub fn dim(&self) -> usize {
        self.n
    }

    pub fn ordering(&self) -> &[usize] {
        &self.col_order
    }

    pub fn refactor_count(&self) -> usize {
        self.refactor_count
    }

    pub fn repivot_count(&self) -> usize {
        self.repivot_count
    }

    pub fn fill_in(&self) -> usize {
        self.l.iter().map(Vec::len).sum::<usize>() + self.u.iter().map(Vec::len).sum::<usize>()
    }

    #[allow(clippy::needless_range_loop)]
    fn eliminate(&mut self, m: &SparseMatrix<T>, fixed: bool) -> Result<(), LuError> {
        let n = self.n;
        let mut rows: Vec<Vec<(usize, T)>> = m.rows().to_vec();
        let mut col_rows: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); n];
        for (i, row) in rows.iter().enumerate() {
            for &(j, _) in row {
                col_rows[j].insert(i);
            }
        }
        let old_pivots = self.pivot_rows.clone();
        let mut pivot_rows = Vec::with_capacity(n);
        let mut l = Vec::with_capacity(n);
        let mut u = Vec::with_capacity(n);
        let mut u_diag = Vec::with_capacity(n);
        let opts = self.options;
        for k in 0..n {
            let c = self.col_order[k];
            let cands: Vec<usize> = col_rows[c].iter().copied().collect();
            let mut maxabs = 0.0f64;
            for &r in &cands {
                let v = find(&rows[r], c).map_or(0.0, |x| x.magnitude());
                if v > maxabs {
                    maxabs = v;
                }
            }
            if maxabs <= opts.singular_tolerance || !maxabs.is_finite() {
                return Err(LuError::Singular { step: k });
            }
            let p = if fixed {
                let p = old_pivots[k];
                let v = find(&rows[p], c).map_or(0.0, |x| x.magnitude());
                if v < opts.refactor_threshold * maxabs {
                    return Err(LuError::Singular { step: k });
                }
                p
            } else {
                let mut best: Option<(usize, usize)> = None;
                let mut diag_ok = false;
                for &r in &cands {
                    let v = find(&rows[r], c).map_or(0.0, |x| x.magnitude());
                    if v >= opts.pivot_threshold * maxabs {
                        if r == c {
                            diag_ok = true;
                        }
                        let key = (rows[r].len(), r);
                        if best.is_none_or(|b| key < b) {
                            best = Some(key);
                        }
                    }
                }
                if diag_ok {
                    c
                } else {
                    best.map_or(cands[0], |b| b.1)
                }
            };
            let prow = std::mem::take(&mut rows[p]);
            for &(j, _) in &prow {
                col_rows[j].remove(&p);
            }
            let piv = find(&prow, c).unwrap_or_else(T::zero);
            let mut lk = Vec::new();
            for &r in &cands {
                if r == p {
                    continue;
                }
                let arc = find(&rows[r], c).unwrap_or_else(T::zero);
                let f = arc / piv;
                let mut fill = Vec::new();
                let new_row = merge_sub(&rows[r], &prow, f, c, &mut fill);
                rows[r] = new_row;
                col_rows[c].remove(&r);
                for j in fill {
                    col_rows[j].insert(r);
                }
                lk.push((r, f));
            }
            pivot_rows.push(p);
            l.push(lk);
            u.push(prow.iter().copied().filter(|e| e.0 != c).collect());
            u_diag.push(piv);
        }
        self.pivot_rows = pivot_rows;
        self.l = l;
        self.u = u;
        self.u_diag = u_diag;
        Ok(())
    }

    pub fn solve(&self, b: &[T]) -> Vec<T> {
        let mut y = b.to_vec();
        for k in 0..self.n {
            let yp = y[self.pivot_rows[k]];
            for &(r, f) in &self.l[k] {
                y[r] -= f * yp;
            }
        }
        let mut x = vec![T::zero(); self.n];
        for k in (0..self.n).rev() {
            let mut s = y[self.pivot_rows[k]];
            for &(j, v) in &self.u[k] {
                s -= v * x[j];
            }
            x[self.col_order[k]] = s / self.u_diag[k];
        }
        x
    }

    pub fn solve_many(&self, rhs: &[Vec<T>]) -> Vec<Vec<T>> {
        rhs.iter().map(|b| self.solve(b)).collect()
    }
}

pub fn solve_system<T: Scalar>(m: &SparseMatrix<T>, b: &[T]) -> Result<Vec<T>, LuError> {
    if m.dim() <= DENSE_LIMIT {
        dense_solve(&m.to_dense(), m.dim(), b)
    } else {
        Ok(SparseLu::factor(m, LuOptions::default())?.solve(b))
    }
}
