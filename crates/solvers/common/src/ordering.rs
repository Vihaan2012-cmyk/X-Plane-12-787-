use crate::scalar::Scalar;
use crate::sparse::SparseMatrix;
use std::collections::BTreeSet;

pub fn minimum_degree_ordering<T: Scalar>(m: &SparseMatrix<T>) -> Vec<usize> {
    let n = m.dim();
    let mut adj: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); n];
    for i in 0..n {
        for &(j, _) in m.row(i) {
            if i != j {
                adj[i].insert(j);
                adj[j].insert(i);
            }
        }
    }
    let mut queue: BTreeSet<(usize, usize)> = (0..n).map(|i| (adj[i].len(), i)).collect();
    let mut order = Vec::with_capacity(n);
    while let Some(&(deg, v)) = queue.iter().next() {
        queue.remove(&(deg, v));
        order.push(v);
        let nbrs: Vec<usize> = adj[v].iter().copied().collect();
        for &a in &nbrs {
            queue.remove(&(adj[a].len(), a));
            adj[a].remove(&v);
        }
        for (x, &a) in nbrs.iter().enumerate() {
            for &b in &nbrs[x + 1..] {
                adj[a].insert(b);
                adj[b].insert(a);
            }
        }
        for &a in &nbrs {
            queue.insert((adj[a].len(), a));
        }
        adj[v].clear();
    }
    order
}
