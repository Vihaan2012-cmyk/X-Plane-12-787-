# solver_common (b789-solver-common)

## Types
- `Complex { re, im }`: `new`, `from_polar`, `conj`, `abs`, `arg`, `norm_sqr`, `scale`, `inv`, `is_finite`, consts `ZERO`, `ONE`; ops `+ - * / neg += -=`, `* f64`.
- `trait Scalar`: `zero`, `one`, `magnitude`, `from_parts(re, im)`, `real`, `imag`. Implemented for `f64` and `Complex`.
- `SparseMatrix<T>` (row-sorted): `from_triplets(n, &[(i, j, v)])` (duplicates summed in input order), `identity`, `dim`, `row`, `nnz`, `get`, `same_pattern`, `mul_vec`, `to_dense`.
- `LuOptions { pivot_threshold = 0.1, singular_tolerance, refactor_threshold = 1e-3 }`.
- `LuError { Singular { step }, PatternChanged, DimensionMismatch }`.
- `SparseLu<T>`: sparse right-looking LU, threshold partial pivoting (diagonal preferred, then fewest entries, then lowest row id).
  - `factor(&m, opts)`: minimum-degree ordering plus numeric factorisation.
  - `factor_with_order(&m, order, opts)`.
  - `refactor(&mut self, &m)`: same pattern, new values; reuses ordering and pivot sequence, re-pivots automatically if a pivot collapses.
  - `solve(&b)`, `solve_many(&[Vec<T>])`.
  - `dim`, `ordering`, `refactor_count`, `repivot_count`, `fill_in`.

## Functions
- `minimum_degree_ordering(&m)`: deterministic (degree, id) minimum degree on the symmetrised pattern.
- `dense_solve(&a_rowmajor, n, &b)`: dense Gaussian elimination, partial pivoting.
- `solve_system(&m, &b)`: dense for n <= 6, sparse LU otherwise.
