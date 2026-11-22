const MIN_RELAX: f64 = 0.02;
const RELAXED_BUDGET_FACTOR: usize = 5;
use crate::load::load_law;
use crate::network::{Element, ElementId, LoadKind, Network, NodeId, NodeKind};
use solver_common::{Complex, LuError, LuOptions, Scalar, SparseLu, SparseMatrix};
use std::f64::consts::PI;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SolverOptions {
    pub gmin: f64,
    pub low_voltage_fraction: f64,
    pub max_inner: usize,
    pub max_outer: usize,
    pub inner_tolerance: f64,
    pub outer_tolerance: f64,
}

impl Default for SolverOptions {
    fn default() -> Self {
        SolverOptions {
            gmin: 1e-9,
            low_voltage_fraction: 0.5,
            max_inner: 200,
            max_outer: 200,
            inner_tolerance: 1e-10,
            outer_tolerance: 1e-9,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SolveError {
    Singular { island: usize },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConvergenceReport {
    pub converged: bool,
    pub outer_iterations: usize,
    pub max_inner_iterations: usize,
    pub outer_residual: f64,
    pub inner_residual: f64,
    pub islands_refactored: usize,
    pub islands_factored: usize,
    pub topology_rebuilt: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IslandInfo {
    pub nodes: Vec<NodeId>,
    pub kind: NodeKind,
    pub live: bool,
    pub frequency_hz: f64,
    pub frequency_conflict: bool,
}

#[derive(Clone, Debug)]
pub struct Solution {
    voltages: Vec<[Complex; 4]>,
    currents: Vec<[Complex; 4]>,
    powers: Vec<Complex>,
    converter_output: Vec<f64>,
    island_of_node: Vec<usize>,
    islands: Vec<IslandInfo>,
    pub report: ConvergenceReport,
}

impl Solution {
    pub fn dc_voltage(&self, n: NodeId) -> f64 {
        self.voltages[n.0][0].re
    }

    pub fn phase_voltages(&self, n: NodeId) -> [Complex; 4] {
        self.voltages[n.0]
    }

    pub fn line_voltage(&self, n: NodeId, a: usize, b: usize) -> Complex {
        self.voltages[n.0][a] - self.voltages[n.0][b]
    }

    pub fn element_currents(&self, e: ElementId) -> [Complex; 4] {
        self.currents[e.0]
    }

    pub fn element_power(&self, e: ElementId) -> Complex {
        self.powers[e.0]
    }

    pub fn converter_output_power(&self, e: ElementId) -> f64 {
        self.converter_output[e.0]
    }

    pub fn island_of(&self, n: NodeId) -> usize {
        self.island_of_node[n.0]
    }

    pub fn island(&self, i: usize) -> &IslandInfo {
        &self.islands[i]
    }

    pub fn island_count(&self) -> usize {
        self.islands.len()
    }

    pub fn frequency_of(&self, n: NodeId) -> f64 {
        self.islands[self.island_of_node[n.0]].frequency_hz
    }

    pub fn is_live(&self, n: NodeId) -> bool {
        self.islands[self.island_of_node[n.0]].live
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Part {
    Whole,
    RectIn,
    RectPos,
    RectNeg,
    AtuIn,
    AtuOut,
}

#[derive(Clone, Debug)]
struct Island {
    nodes: Vec<usize>,
    kind: NodeKind,
    parts: Vec<(usize, Part)>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct ConvState {
    s_in: [Complex; 3],
    emf_dc: f64,
    emf_ac: [Complex; 3],
    freq: f64,
    p_out: f64,
}

struct Cache<T: Scalar> {
    matrix: SparseMatrix<T>,
    lu: SparseLu<T>,
}

#[derive(Default)]
struct Caches {
    ac: Vec<Option<Cache<Complex>>>,
    dc: Vec<Option<Cache<f64>>>,
}

pub struct Solver {
    options: SolverOptions,
    version: Option<u64>,
    islands: Vec<Island>,
    island_of_node: Vec<usize>,
    caches: Caches,
}

fn find_root(parent: &mut [usize], mut x: usize) -> usize {
    while parent[x] != x {
        parent[x] = parent[parent[x]];
        x = parent[x];
    }
    x
}

fn build_islands(net: &Network) -> (Vec<Island>, Vec<usize>) {
    let n = net.node_count();
    let mut parent: Vec<usize> = (0..n).collect();
    for e in net.elements() {
        let pair = match *e {
            Element::Resistor { a, b, .. } | Element::Impedance { a, b, .. } => Some((a, b)),
            Element::Switch {
                a, b, closed: true, ..
            } => Some((a, b)),
            _ => None,
        };
        if let Some((a, b)) = pair {
            let ra = find_root(&mut parent, a.0);
            let rb = find_root(&mut parent, b.0);
            if ra != rb {
                let (lo, hi) = if ra < rb { (ra, rb) } else { (rb, ra) };
                parent[hi] = lo;
            }
        }
    }
    let mut island_of_root = vec![usize::MAX; n];
    let mut island_of_node = vec![0; n];
    let mut islands: Vec<Island> = Vec::new();
    for v in 0..n {
        let r = find_root(&mut parent, v);
        if island_of_root[r] == usize::MAX {
            island_of_root[r] = islands.len();
            islands.push(Island {
                nodes: Vec::new(),
                kind: net.node_kind(NodeId(v)),
                parts: Vec::new(),
            });
        }
        let i = island_of_root[r];
        islands[i].nodes.push(v);
        island_of_node[v] = i;
    }
    for (ei, e) in net.elements().iter().enumerate() {
        let mut add = |node: NodeId, part: Part| {
            islands[island_of_node[node.0]].parts.push((ei, part));
        };
        match *e {
            Element::Resistor { a, .. } | Element::Impedance { a, .. } => add(a, Part::Whole),
            Element::Switch { a, closed, .. } => {
                if closed {
                    add(a, Part::Whole)
                }
            }
            Element::DcSource { node, .. }
            | Element::AcSource { node, .. }
            | Element::DcCurrentSource { node, .. }
            | Element::AcCurrentSource { node, .. }
            | Element::DcLoad { node, .. }
            | Element::AcLoad { node, .. } => add(node, Part::Whole),
            Element::Rectifier { ac, pos, neg, .. } => {
                add(ac, Part::RectIn);
                add(pos, Part::RectPos);
                if let Some(nn) = neg {
                    add(nn, Part::RectNeg);
                }
            }
            Element::Autotransformer { input, output, .. } => {
                add(input, Part::AtuIn);
                add(output, Part::AtuOut);
            }
        }
    }
    (islands, island_of_node)
}

fn phase_angle(p: usize) -> f64 {
    -(p as f64) * 2.0 * PI / 3.0
}

fn conv<T: Scalar>(z: Complex) -> T {
    T::from_parts(z.re, z.im)
}

fn back<T: Scalar>(t: T) -> Complex {
    Complex::new(t.real(), t.imag())
}

fn stamp<T: Scalar>(trip: &mut Vec<(usize, usize, T)>, a: Option<usize>, b: Option<usize>, y: T) {
    if let Some(a) = a {
        trip.push((a, a, y));
    }
    if let Some(b) = b {
        trip.push((b, b, y));
    }
    if let (Some(a), Some(b)) = (a, b) {
        trip.push((a, b, -y));
        trip.push((b, a, -y));
    }
}

fn rail_sources(r_out: f64, emf: f64, has_neg: bool) -> (f64, f64) {
    if has_neg {
        (emf / 2.0, r_out / 2.0)
    } else {
        (emf, r_out)
    }
}

fn island_live(net: &Network, isl: &Island, st: &[ConvState]) -> bool {
    isl.parts
        .iter()
        .any(|&(ei, part)| match (net.elements()[ei], part) {
            (Element::DcSource { e, .. }, _) => e != 0.0,
            (Element::AcSource { emf, .. }, _) => emf > 0.0,
            (Element::DcCurrentSource { amps, .. }, _) => amps != 0.0,
            (Element::AcCurrentSource { amps, .. }, _) => amps != 0.0,
            (Element::Rectifier { .. }, Part::RectPos | Part::RectNeg) => st[ei].emf_dc > 0.0,
            (Element::Autotransformer { .. }, Part::AtuOut) => st[ei].emf_ac.iter().any(|z| z.abs() > 0.0),
            _ => false,
        })
}

struct DynItem {
    a: usize,
    b: Option<usize>,
    kind: LoadKind,
}

fn merge_dyns(dyns: Vec<DynItem>) -> Vec<DynItem> {
    let mut out: Vec<DynItem> = Vec::with_capacity(dyns.len());
    let mut index: std::collections::BTreeMap<(usize, Option<usize>, u64), usize> =
        std::collections::BTreeMap::new();
    for d in dyns {
        match d.kind {
            LoadKind::Impedance { .. } => {}
            LoadKind::Power { p, q, v_nominal } => {
                let key = (d.a, d.b, v_nominal.to_bits());
                match index.get(&key) {
                    Some(&k) => {
                        if let LoadKind::Power {
                            p: ref mut pp,
                            q: ref mut qq,
                            ..
                        } = out[k].kind
                        {
                            *pp += p;
                            *qq += q;
                        }
                    }
                    None => {
                        index.insert(key, out.len());
                        out.push(d);
                    }
                }
            }
            LoadKind::Current { .. } => out.push(d),
        }
    }
    out
}

struct RealSys<'a> {
    base: &'a [(usize, usize, f64)],
    rhs: &'a [f64],
    dyns: &'a [DynItem],
    w: usize,
    ac: bool,
    frac: f64,
}

fn expand<T: Scalar>(trip: &[(usize, usize, T)], w: usize) -> Vec<(usize, usize, f64)> {
    let mut out = Vec::with_capacity(trip.len() * w * w);
    for &(i, j, y) in trip {
        let (g, b) = (y.real(), y.imag());
        if w == 1 {
            out.push((i, j, g));
        } else {
            out.push((2 * i, 2 * j, g));
            out.push((2 * i, 2 * j + 1, -b));
            out.push((2 * i + 1, 2 * j, b));
            out.push((2 * i + 1, 2 * j + 1, g));
        }
    }
    out
}

fn split<T: Scalar>(x: &[T], w: usize) -> Vec<f64> {
    if w == 1 {
        x.iter().map(|t| t.real()).collect()
    } else {
        x.iter().flat_map(|t| [t.real(), t.imag()]).collect()
    }
}

fn join<T: Scalar>(x: &[f64], w: usize) -> Vec<T> {
    if w == 1 {
        x.iter().map(|r| conv::<T>(Complex::new(*r, 0.0))).collect()
    } else {
        x.chunks(2).map(|c| conv::<T>(Complex::new(c[0], c[1]))).collect()
    }
}

fn real_solve(jt: &[(usize, usize, f64)], b: &[f64]) -> Option<Vec<f64>> {
    let lu = SparseLu::factor(&SparseMatrix::from_triplets(b.len(), jt), LuOptions::default()).ok()?;
    Some(lu.solve(b))
}

fn stamp_block(jt: &mut Vec<(usize, usize, f64)>, a: usize, b: Option<usize>, w: usize, j: [[f64; 2]; 2]) {
    for (r, row) in j.iter().enumerate().take(w) {
        for (c, val) in row.iter().enumerate().take(w) {
            jt.push((a * w + r, a * w + c, *val));
            if let Some(b) = b {
                jt.push((b * w + r, b * w + c, *val));
                jt.push((a * w + r, b * w + c, -*val));
                jt.push((b * w + r, a * w + c, -*val));
            }
        }
    }
}

fn at(v: &[f64], w: usize, a: usize) -> Complex {
    if w == 1 {
        Complex::new(v[a], 0.0)
    } else {
        Complex::new(v[2 * a], v[2 * a + 1])
    }
}

fn load_corr(kind: LoadKind, vd: Complex, ac: bool, frac: f64) -> Complex {
    let (y0, i) = load_law(kind, vd, ac, frac);
    i - y0 * vd
}

fn mismatch(sys: &RealSys, v: &[f64]) -> Vec<f64> {
    let w = sys.w;
    let mut f: Vec<f64> = sys.rhs.iter().map(|r| -r).collect();
    for &(i, j, y) in sys.base {
        f[i] += y * v[j];
    }
    for d in sys.dyns {
        let vd = at(v, w, d.a) - d.b.map_or(Complex::ZERO, |b| at(v, w, b));
        let c = load_corr(d.kind, vd, sys.ac, sys.frac);
        f[d.a * w] += c.re;
        if w == 2 {
            f[d.a * w + 1] += c.im;
        }
        if let Some(b) = d.b {
            f[b * w] -= c.re;
            if w == 2 {
                f[b * w + 1] -= c.im;
            }
        }
    }
    f
}

fn norm_inf(x: &[f64]) -> f64 {
    x.iter().fold(0.0f64, |a, b| a.max(b.abs()))
}

fn low_branch_start(sys: &RealSys) -> Option<Vec<f64>> {
    let mut jt = sys.base.to_vec();
    let f = sys.frac.max(1e-3);
    for d in sys.dyns {
        let (y0, _) = load_law(d.kind, Complex::new(1.0, 0.0), sys.ac, sys.frac);
        let dy = y0.scale(1.0 / (f * f) - 1.0);
        stamp_block(&mut jt, d.a, d.b, sys.w, [[dy.re, -dy.im], [dy.im, dy.re]]);
    }
    real_solve(&jt, sys.rhs)
}

fn newton(sys: &RealSys, start: Vec<f64>, tol: f64) -> Option<(Vec<f64>, f64, usize)> {
    let w = sys.w;
    let mut v = start;
    let mut f = mismatch(sys, &v);
    let mut fnorm = norm_inf(&f);
    for k in 1..=60 {
        let mut jt = sys.base.to_vec();
        for d in sys.dyns {
            let vd = at(&v, w, d.a) - d.b.map_or(Complex::ZERO, |b| at(&v, w, b));
            let h = 1e-6 * vd.abs().max(1.0);
            let mut j = [[0.0; 2]; 2];
            for (c, dv) in [Complex::new(h, 0.0), Complex::new(0.0, h)]
                .into_iter()
                .enumerate()
                .take(w)
            {
                let dc = (load_corr(d.kind, vd + dv, sys.ac, sys.frac)
                    - load_corr(d.kind, vd - dv, sys.ac, sys.frac))
                .scale(0.5 / h);
                j[0][c] = dc.re;
                j[1][c] = dc.im;
            }
            stamp_block(&mut jt, d.a, d.b, w, j);
        }
        let neg: Vec<f64> = f.iter().map(|x| -x).collect();
        let dx = real_solve(&jt, &neg)?;
        let full = norm_inf(&dx) / norm_inf(&v).max(1.0);
        if full < tol {
            let done: Vec<f64> = v.iter().zip(&dx).map(|(x, d)| x + d).collect();
            return Some((done, full, k));
        }
        let mut a = 1.0f64;
        let (cand, fc, cnorm) = loop {
            let cand: Vec<f64> = v.iter().zip(&dx).map(|(x, d)| x + a * d).collect();
            let fc = mismatch(sys, &cand);
            let cnorm = norm_inf(&fc);
            if cnorm < fnorm || fnorm == 0.0 {
                break (cand, fc, cnorm);
            }
            a *= 0.5;
            if a < 1e-4 {
                return None;
            }
        };
        let step = a * full;
        v = cand;
        f = fc;
        fnorm = cnorm;
        if step < tol {
            return Some((v, step, k));
        }
    }
    None
}

struct Solved {
    values: Vec<Complex>,
    inner_iterations: usize,
    inner_residual: f64,
    refactored: bool,
    factored: bool,
}

fn solve_island<T: Scalar>(
    net: &Network,
    isl: &Island,
    index: usize,
    st: &[ConvState],
    opts: &SolverOptions,
    cache: &mut Option<Cache<T>>,
) -> Result<Solved, SolveError> {
    let ac = isl.kind == NodeKind::Ac3;
    let dim = if ac { 4 } else { 1 };
    let m = isl.nodes.len() * dim;
    let loc = |n: NodeId, t: usize| -> usize { isl.nodes.binary_search(&n.0).unwrap_or(0) * dim + t };
    let mut trip: Vec<(usize, usize, T)> = Vec::new();
    let mut rhs = vec![T::zero(); m];
    let mut dyns: Vec<DynItem> = Vec::new();
    let ac_source = |trip: &mut Vec<(usize, usize, T)>,
                     rhs: &mut Vec<T>,
                     node: NodeId,
                     emf: [Complex; 3],
                     z: Complex,
                     rng: f64| {
        let y: Complex = if z.abs() < 1e-9 {
            Complex::new(1e9, 0.0)
        } else {
            z.inv()
        };
        let n = loc(node, 3);
        for (p, e) in emf.iter().enumerate() {
            let a = loc(node, p);
            stamp(trip, Some(a), Some(n), conv::<T>(y));
            let i = *e * y;
            rhs[a] += conv::<T>(i);
            rhs[n] -= conv::<T>(i);
        }
        stamp(
            trip,
            Some(n),
            None,
            conv::<T>(Complex::new(1.0 / rng.max(1e-9), 0.0)),
        );
    };
    for &(ei, part) in &isl.parts {
        match (net.elements()[ei], part) {
            (Element::Resistor { a, b, r }, _) => {
                let y = conv::<T>(Complex::new(1.0 / r.max(1e-12), 0.0));
                for t in 0..dim {
                    stamp(&mut trip, Some(loc(a, t)), Some(loc(b, t)), y);
                }
            }
            (
                Element::Impedance {
                    a,
                    b,
                    z_phase,
                    z_neutral,
                },
                _,
            ) => {
                for t in 0..dim {
                    let z = if t == 3 { z_neutral } else { z_phase };
                    let y = if z.abs() < 1e-12 {
                        Complex::new(1e12, 0.0)
                    } else {
                        z.inv()
                    };
                    stamp(&mut trip, Some(loc(a, t)), Some(loc(b, t)), conv::<T>(y));
                }
            }
            (Element::Switch { a, b, r_closed, .. }, _) => {
                let y = conv::<T>(Complex::new(1.0 / r_closed.max(1e-12), 0.0));
                for t in 0..dim {
                    stamp(&mut trip, Some(loc(a, t)), Some(loc(b, t)), y);
                }
            }
            (Element::DcSource { node, e, r }, _) => {
                let y = 1.0 / r.max(1e-12);
                let a = loc(node, 0);
                stamp(&mut trip, Some(a), None, conv::<T>(Complex::new(y, 0.0)));
                rhs[a] += conv::<T>(Complex::new(e * y, 0.0));
            }
            (
                Element::AcSource {
                    node,
                    emf,
                    angle,
                    r,
                    x,
                    r_neutral_ground,
                    ..
                },
                _,
            ) => {
                let e = [0, 1, 2].map(|p| Complex::from_polar(emf, angle + phase_angle(p)));
                ac_source(&mut trip, &mut rhs, node, e, Complex::new(r, x), r_neutral_ground);
            }
            (Element::DcCurrentSource { node, amps }, _) => {
                rhs[loc(node, 0)] += conv::<T>(Complex::new(amps, 0.0));
            }
            (Element::AcCurrentSource { node, amps, angle }, _) => {
                for p in 0..3 {
                    let i = conv::<T>(Complex::from_polar(amps, angle + phase_angle(p)));
                    rhs[loc(node, p)] += i;
                    rhs[loc(node, 3)] -= i;
                }
            }
            (Element::DcLoad { node, kind }, _) => {
                let a = loc(node, 0);
                let (y0, _) = load_law(kind, Complex::ZERO, false, opts.low_voltage_fraction);
                stamp(&mut trip, Some(a), None, conv::<T>(y0));
                dyns.push(DynItem { a, b: None, kind });
            }
            (Element::AcLoad { node, phases }, _) => {
                for (p, kind) in phases.iter().enumerate() {
                    let a = loc(node, p);
                    let n = loc(node, 3);
                    let (y0, _) = load_law(*kind, Complex::ZERO, true, opts.low_voltage_fraction);
                    stamp(&mut trip, Some(a), Some(n), conv::<T>(y0));
                    dyns.push(DynItem {
                        a,
                        b: Some(n),
                        kind: *kind,
                    });
                }
            }
            (
                Element::Rectifier {
                    ac: node,
                    ac_v_nominal,
                    ..
                },
                Part::RectIn,
            ) => {
                for p in 0..3 {
                    let a = loc(node, p);
                    let n = loc(node, 3);
                    let s = st[ei].s_in[p];
                    let kind = LoadKind::Power {
                        p: s.re,
                        q: s.im,
                        v_nominal: ac_v_nominal,
                    };
                    let (y0, _) = load_law(kind, Complex::ZERO, true, opts.low_voltage_fraction);
                    stamp(&mut trip, Some(a), Some(n), conv::<T>(y0));
                    dyns.push(DynItem { a, b: Some(n), kind });
                }
            }
            (Element::Rectifier { pos, neg, r_out, .. }, Part::RectPos | Part::RectNeg) => {
                let (e, r) = rail_sources(r_out, st[ei].emf_dc, neg.is_some());
                let (node, e) = if part == Part::RectPos {
                    (pos, e)
                } else {
                    (neg.unwrap_or(pos), -e)
                };
                let y = 1.0 / r.max(1e-12);
                let a = loc(node, 0);
                stamp(&mut trip, Some(a), None, conv::<T>(Complex::new(y, 0.0)));
                rhs[a] += conv::<T>(Complex::new(e * y, 0.0));
            }
            (
                Element::Autotransformer {
                    input, in_v_nominal, ..
                },
                Part::AtuIn,
            ) => {
                for p in 0..3 {
                    let a = loc(input, p);
                    let n = loc(input, 3);
                    let s = st[ei].s_in[p];
                    let kind = LoadKind::Power {
                        p: s.re,
                        q: s.im,
                        v_nominal: in_v_nominal,
                    };
                    let (y0, _) = load_law(kind, Complex::ZERO, true, opts.low_voltage_fraction);
                    stamp(&mut trip, Some(a), Some(n), conv::<T>(y0));
                    dyns.push(DynItem { a, b: Some(n), kind });
                }
            }
            (
                Element::Autotransformer {
                    output,
                    r_leak,
                    x_leak,
                    r_neutral_ground,
                    ..
                },
                Part::AtuOut,
            ) => {
                ac_source(
                    &mut trip,
                    &mut rhs,
                    output,
                    st[ei].emf_ac,
                    Complex::new(r_leak, x_leak),
                    r_neutral_ground,
                );
            }
            _ => {}
        }
    }
    for i in 0..m {
        trip.push((i, i, conv::<T>(Complex::new(opts.gmin, 0.0))));
    }
    let dyns = merge_dyns(dyns);
    let matrix = SparseMatrix::from_triplets(m, &trip);
    let mut refactored = false;
    let mut factored = false;
    let lu_err = |e: LuError| -> SolveError {
        let _ = e;
        SolveError::Singular { island: index }
    };
    let need = match cache.as_ref() {
        Some(c) if c.matrix == matrix => 0,
        Some(c) if c.matrix.same_pattern(&matrix) => 1,
        _ => 2,
    };
    match need {
        0 => {}
        1 => {
            if let Some(c) = cache.as_mut() {
                c.lu.refactor(&matrix).map_err(lu_err)?;
                c.matrix = matrix;
                refactored = true;
            }
        }
        _ => {
            let lu = SparseLu::factor(&matrix, LuOptions::default()).map_err(lu_err)?;
            *cache = Some(Cache { matrix, lu });
            factored = true;
        }
    }
    let lu = match cache.as_ref() {
        Some(c) => &c.lu,
        None => return Err(SolveError::Singular { island: index }),
    };
    let mut v: Vec<T> = lu.solve(&rhs);
    let mut iterations = 1;
    let mut residual = 0.0;
    let mut relax = 1.0f64;
    if !dyns.is_empty() {
        let mut budget = opts.max_inner;
        let mut k = 0usize;
        while k < budget {
            k += 1;
            let mut r2 = rhs.clone();
            for d in &dyns {
                let va = back(v[d.a]);
                let vb = d.b.map_or(Complex::ZERO, |b| back(v[b]));
                let vd = va - vb;
                let (y0, i) = load_law(d.kind, vd, ac, opts.low_voltage_fraction);
                let corr = i - y0 * vd;
                r2[d.a] -= conv::<T>(corr);
                if let Some(b) = d.b {
                    r2[b] += conv::<T>(corr);
                }
            }
            let nv = lu.solve(&r2);
            let mut diff = 0.0f64;
            let mut mx = 1.0f64;
            for (a, b) in nv.iter().zip(&v) {
                diff = diff.max(back(*a - *b).abs());
                mx = mx.max(back(*a).abs());
            }
            let step_residual = diff / mx;
            if iterations > 1 && step_residual > residual * 0.95 {
                relax = (relax * 0.5).max(MIN_RELAX);
                budget = opts.max_inner * RELAXED_BUDGET_FACTOR;
            } else if iterations > 1 && step_residual < residual * 0.5 {
                relax = (relax * 1.5).min(1.0);
            }
            residual = step_residual;
            if relax >= 1.0 {
                v = nv;
            } else {
                for (x, n) in v.iter_mut().zip(nv.iter()) {
                    *x = *x + (*n - *x) * conv::<T>(Complex::new(relax, 0.0));
                }
            }
            iterations += 1;
            if residual < opts.inner_tolerance {
                break;
            }
        }
        if residual > opts.inner_tolerance {
            let w = if ac { 2 } else { 1 };
            let base = expand(&trip, w);
            let rhs_r = split(&rhs, w);
            let sys = RealSys {
                base: &base,
                rhs: &rhs_r,
                dyns: &dyns,
                w,
                ac,
                frac: opts.low_voltage_fraction,
            };
            for low in [false, true] {
                let start = if low {
                    match low_branch_start(&sys) {
                        Some(x) => x,
                        None => continue,
                    }
                } else {
                    split(&v, w)
                };
                if let Some((nv, r, k)) = newton(&sys, start, opts.inner_tolerance) {
                    v = join(&nv, w);
                    residual = r;
                    iterations += k;
                    break;
                }
            }
        }
    }
    Ok(Solved {
        values: v.iter().map(|x| back(*x)).collect(),
        inner_iterations: iterations,
        inner_residual: residual,
        refactored,
        factored,
    })
}

fn element_results(
    net: &Network,
    volts: &[[Complex; 4]],
    st: &[ConvState],
    opts: &SolverOptions,
) -> (Vec<[Complex; 4]>, Vec<Complex>) {
    let frac = opts.low_voltage_fraction;
    let mut cur = vec![[Complex::ZERO; 4]; net.element_count()];
    let mut pow = vec![Complex::ZERO; net.element_count()];
    let rel = |n: NodeId, p: usize| volts[n.0][p] - volts[n.0][3];
    let neutral_sum = |c: &[Complex; 4]| c[0] + c[1] + c[2];
    for (ei, e) in net.elements().iter().enumerate() {
        match *e {
            Element::Resistor { a, b, r } => {
                let dim = if net.node_kind(a) == NodeKind::Ac3 { 4 } else { 1 };
                let y = 1.0 / r.max(1e-12);
                let mut s = Complex::ZERO;
                for t in 0..dim {
                    let vd = volts[a.0][t] - volts[b.0][t];
                    cur[ei][t] = vd.scale(y);
                    s += vd * cur[ei][t].conj();
                }
                pow[ei] = s;
            }
            Element::Impedance {
                a,
                b,
                z_phase,
                z_neutral,
            } => {
                let dim = if net.node_kind(a) == NodeKind::Ac3 { 4 } else { 1 };
                let mut s = Complex::ZERO;
                for t in 0..dim {
                    let z = if t == 3 { z_neutral } else { z_phase };
                    let vd = volts[a.0][t] - volts[b.0][t];
                    cur[ei][t] = vd / z;
                    s += vd * cur[ei][t].conj();
                }
                pow[ei] = s;
            }
            Element::Switch {
                a,
                b,
                closed,
                r_closed,
            } => {
                if closed {
                    let dim = if net.node_kind(a) == NodeKind::Ac3 { 4 } else { 1 };
                    let y = 1.0 / r_closed.max(1e-12);
                    let mut s = Complex::ZERO;
                    for t in 0..dim {
                        let vd = volts[a.0][t] - volts[b.0][t];
                        cur[ei][t] = vd.scale(y);
                        s += vd * cur[ei][t].conj();
                    }
                    pow[ei] = s;
                }
            }
            Element::DcSource { node, e, r } => {
                let v = volts[node.0][0].re;
                let i = (e - v) / r.max(1e-12);
                cur[ei][0] = Complex::new(i, 0.0);
                pow[ei] = Complex::new(v * i, 0.0);
            }
            Element::AcSource {
                node,
                emf,
                angle,
                r,
                x,
                ..
            } => {
                let z = Complex::new(r, x);
                let mut s = Complex::ZERO;
                for p in 0..3 {
                    let ep = Complex::from_polar(emf, angle + phase_angle(p));
                    cur[ei][p] = (ep - rel(node, p)) / z;
                    s += rel(node, p) * cur[ei][p].conj();
                }
                cur[ei][3] = neutral_sum(&cur[ei]);
                pow[ei] = s;
            }
            Element::DcCurrentSource { node, amps } => {
                cur[ei][0] = Complex::new(amps, 0.0);
                pow[ei] = Complex::new(volts[node.0][0].re * amps, 0.0);
            }
            Element::AcCurrentSource { node, amps, angle } => {
                let mut s = Complex::ZERO;
                for p in 0..3 {
                    cur[ei][p] = Complex::from_polar(amps, angle + phase_angle(p));
                    s += rel(node, p) * cur[ei][p].conj();
                }
                cur[ei][3] = neutral_sum(&cur[ei]);
                pow[ei] = s;
            }
            Element::DcLoad { node, kind } => {
                let v = volts[node.0][0];
                let (_, i) = load_law(kind, v, false, frac);
                cur[ei][0] = i;
                pow[ei] = Complex::new(v.re * i.re, 0.0);
            }
            Element::AcLoad { node, phases } => {
                let mut s = Complex::ZERO;
                for p in 0..3 {
                    let v = rel(node, p);
                    let (_, i) = load_law(phases[p], v, true, frac);
                    cur[ei][p] = i;
                    s += v * i.conj();
                }
                cur[ei][3] = neutral_sum(&cur[ei]);
                pow[ei] = s;
            }
            Element::Rectifier { ac, ac_v_nominal, .. } => {
                let mut s = Complex::ZERO;
                for p in 0..3 {
                    let kind = LoadKind::Power {
                        p: st[ei].s_in[p].re,
                        q: st[ei].s_in[p].im,
                        v_nominal: ac_v_nominal,
                    };
                    let v = rel(ac, p);
                    let (_, i) = load_law(kind, v, true, frac);
                    cur[ei][p] = i;
                    s += v * i.conj();
                }
                cur[ei][3] = neutral_sum(&cur[ei]);
                pow[ei] = s;
            }
            Element::Autotransformer {
                input, in_v_nominal, ..
            } => {
                let mut s = Complex::ZERO;
                for p in 0..3 {
                    let kind = LoadKind::Power {
                        p: st[ei].s_in[p].re,
                        q: st[ei].s_in[p].im,
                        v_nominal: in_v_nominal,
                    };
                    let v = rel(input, p);
                    let (_, i) = load_law(kind, v, true, frac);
                    cur[ei][p] = i;
                    s += v * i.conj();
                }
                cur[ei][3] = neutral_sum(&cur[ei]);
                pow[ei] = s;
            }
        }
    }
    (cur, pow)
}

impl Default for Solver {
    fn default() -> Self {
        Solver::new(SolverOptions::default())
    }
}

impl Solver {
    pub fn new(options: SolverOptions) -> Self {
        Solver {
            options,
            version: None,
            islands: Vec::new(),
            island_of_node: Vec::new(),
            caches: Caches::default(),
        }
    }

    pub fn options(&self) -> &SolverOptions {
        &self.options
    }

    pub fn solve(&mut self, net: &Network) -> Result<Solution, SolveError> {
        let opts = self.options;
        let mut rebuilt = false;
        if self.version != Some(net.topology_version()) {
            let (islands, map) = build_islands(net);
            self.caches = Caches {
                ac: (0..islands.len()).map(|_| None).collect(),
                dc: (0..islands.len()).map(|_| None).collect(),
            };
            self.islands = islands;
            self.island_of_node = map;
            self.version = Some(net.topology_version());
            rebuilt = true;
        }
        let nn = net.node_count();
        let ne = net.element_count();
        let mut volts = vec![[Complex::ZERO; 4]; nn];
        let mut st = vec![ConvState::default(); ne];
        let mut info: Vec<IslandInfo> = self
            .islands
            .iter()
            .map(|i| IslandInfo {
                nodes: i.nodes.iter().map(|&n| NodeId(n)).collect(),
                kind: i.kind,
                live: false,
                frequency_hz: 0.0,
                frequency_conflict: false,
            })
            .collect();
        let has_conv = net
            .elements()
            .iter()
            .any(|e| matches!(e, Element::Rectifier { .. } | Element::Autotransformer { .. }));
        let mut report = ConvergenceReport {
            converged: true,
            outer_iterations: 0,
            max_inner_iterations: 0,
            outer_residual: 0.0,
            inner_residual: 0.0,
            islands_refactored: 0,
            islands_factored: 0,
            topology_rebuilt: rebuilt,
        };
        let max_outer = if has_conv { opts.max_outer } else { 1 };
        for outer in 1..=max_outer {
            let before = st.clone();
            for pass_ac in [true, false] {
                for ii in 0..self.islands.len() {
                    let isl = &self.islands[ii];
                    if (isl.kind == NodeKind::Ac3) != pass_ac {
                        continue;
                    }
                    let live = island_live(net, isl, &st);
                    info[ii].live = live;
                    let dim = if pass_ac { 4 } else { 1 };
                    let values = if live {
                        let solved = if pass_ac {
                            solve_island::<Complex>(net, isl, ii, &st, &opts, &mut self.caches.ac[ii])?
                        } else {
                            solve_island::<f64>(net, isl, ii, &st, &opts, &mut self.caches.dc[ii])?
                        };
                        report.max_inner_iterations =
                            report.max_inner_iterations.max(solved.inner_iterations);
                        report.inner_residual = report.inner_residual.max(solved.inner_residual);
                        report.islands_refactored += usize::from(solved.refactored);
                        report.islands_factored += usize::from(solved.factored);
                        solved.values
                    } else {
                        vec![Complex::ZERO; isl.nodes.len() * dim]
                    };
                    for (li, &gn) in isl.nodes.iter().enumerate() {
                        for t in 0..dim {
                            volts[gn][t] = values[li * dim + t];
                        }
                    }
                    if pass_ac {
                        let mut freq = 0.0;
                        let mut conflict = false;
                        for &(ei, part) in &isl.parts {
                            let f = match (net.elements()[ei], part) {
                                (
                                    Element::AcSource {
                                        emf, frequency_hz, ..
                                    },
                                    _,
                                ) if emf > 0.0 => frequency_hz,
                                (Element::Autotransformer { .. }, Part::AtuOut)
                                    if st[ei].emf_ac.iter().any(|z| z.abs() > 0.0) =>
                                {
                                    st[ei].freq
                                }
                                _ => continue,
                            };
                            if freq == 0.0 {
                                freq = f;
                            } else if (freq - f).abs() > 1e-9 {
                                conflict = true;
                            }
                        }
                        info[ii].frequency_hz = if live { freq } else { 0.0 };
                        info[ii].frequency_conflict = conflict;
                        update_ac_states(net, &self.islands, ii, &volts, &mut st, info[ii].frequency_hz);
                    } else {
                        update_dc_states(net, &self.islands[ii], &volts, &mut st);
                    }
                }
            }
            report.outer_iterations = outer;
            let mut resid = 0.0f64;
            for (a, b) in st.iter().zip(&before) {
                for p in 0..3 {
                    resid = resid.max((a.s_in[p] - b.s_in[p]).abs() / (1.0 + a.s_in[p].abs()));
                    resid = resid.max((a.emf_ac[p] - b.emf_ac[p]).abs() / (1.0 + a.emf_ac[p].abs()));
                }
                resid = resid.max((a.emf_dc - b.emf_dc).abs() / (1.0 + a.emf_dc.abs()));
            }
            report.outer_residual = resid;
            if !has_conv {
                break;
            }
            if resid < opts.outer_tolerance {
                break;
            }
            if outer == max_outer {
                report.converged = false;
            }
        }
        if report.inner_residual > opts.inner_tolerance * 1e3 {
            report.converged = false;
        }
        let (currents, powers) = element_results(net, &volts, &st, &opts);
        let converter_output = st.iter().map(|s| s.p_out).collect();
        Ok(Solution {
            voltages: volts,
            currents,
            powers,
            converter_output,
            island_of_node: self.island_of_node.clone(),
            islands: info,
            report,
        })
    }
}

fn update_ac_states(
    net: &Network,
    islands: &[Island],
    ii: usize,
    volts: &[[Complex; 4]],
    st: &mut [ConvState],
    freq: f64,
) {
    let rel = |n: NodeId, p: usize| volts[n.0][p] - volts[n.0][3];
    for (ei, e) in net.elements().iter().enumerate() {
        match *e {
            Element::Rectifier { ac, ratio, .. } => {
                if islands[ii].nodes.binary_search(&ac.0).is_ok() {
                    let vll = ((rel(ac, 0) - rel(ac, 1)).abs()
                        + (rel(ac, 1) - rel(ac, 2)).abs()
                        + (rel(ac, 2) - rel(ac, 0)).abs())
                        / 3.0;
                    st[ei].emf_dc = ratio * vll;
                }
            }
            Element::Autotransformer {
                input,
                output,
                ratio,
                r_leak,
                x_leak,
                efficiency,
                ..
            } => {
                if islands[ii].nodes.binary_search(&input.0).is_ok() {
                    for p in 0..3 {
                        st[ei].emf_ac[p] = rel(input, p).scale(ratio);
                    }
                    st[ei].freq = freq;
                }
                if islands[ii].nodes.binary_search(&output.0).is_ok() {
                    let z = Complex::new(r_leak, x_leak);
                    let mut total = 0.0;
                    let mut s_in = [Complex::ZERO; 3];
                    for p in 0..3 {
                        let v = rel(output, p);
                        let i = (st[ei].emf_ac[p] - v) / z;
                        let s = v * i.conj();
                        total += s.re;
                        s_in[p] = s.scale(1.0 / efficiency.max(1e-6));
                    }
                    st[ei].p_out = total;
                    st[ei].s_in = s_in;
                }
            }
            _ => {}
        }
    }
}

fn update_dc_states(net: &Network, isl: &Island, volts: &[[Complex; 4]], st: &mut [ConvState]) {
    for &(ei, part) in &isl.parts {
        if let (
            Element::Rectifier {
                pos,
                neg,
                r_out,
                efficiency,
                ..
            },
            Part::RectPos,
        ) = (net.elements()[ei], part)
        {
            let (e, r) = rail_sources(r_out, st[ei].emf_dc, neg.is_some());
            let vp = volts[pos.0][0].re;
            let mut p_out = vp * (e - vp) / r.max(1e-12);
            if let Some(nn) = neg {
                let vn = volts[nn.0][0].re;
                p_out += vn * (-e - vn) / r.max(1e-12);
            }
            let p_out = p_out.max(0.0);
            st[ei].p_out = p_out;
            let p_in = p_out / efficiency.max(1e-6) / 3.0;
            st[ei].s_in = [Complex::new(p_in, 0.0); 3];
        }
    }
}
