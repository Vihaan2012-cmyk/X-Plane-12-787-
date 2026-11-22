"""Generate a clean meridional-section SVG of a three-spool turbofan (Trent 1000 proportions, approximate).

Geometry is defined as radius-vs-axial curves in metres, then scaled. Blades are placed per stage
between hub and tip, and streamlines are interpolated between the annulus walls, so the flow follows the gas path.
Usage: python -I gen_meridional.py <out.svg>
"""
import sys

S = 162.0          # px per metre
X0 = 185.0         # px of x = 0 (fan face)
CY = 312.0         # px of the centreline
W, H = 1000, 610


def px(x): return X0 + x * S
def py(r): return CY - r * S


def interp(curve, x):
    """Linear interpolation of r at x on a polyline [(x, r), ...]."""
    if x <= curve[0][0]:
        return curve[0][1]
    for (x0, r0), (x1, r1) in zip(curve, curve[1:]):
        if x0 <= x <= x1:
            t = (x - x0) / (x1 - x0) if x1 > x0 else 0
            return r0 + t * (r1 - r0)
    return curve[-1][1]


def path(curve, close=None):
    d = "M" + " L".join(f"{px(x):.1f},{py(r):.1f}" for x, r in curve)
    return d


def smooth(curve, n=60):
    xs = [curve[0][0] + (curve[-1][0] - curve[0][0]) * i / n for i in range(n + 1)]
    return [(x, interp(curve, x)) for x in xs]


# ---- geometry (m) ----
nac_outer = [(-0.95, 1.50), (-0.80, 1.585), (-0.40, 1.625), (0.30, 1.62), (1.20, 1.56), (2.10, 1.44), (2.55, 1.34)]
nac_inner = [(-0.95, 1.50), (-0.85, 1.42), (-0.50, 1.405), (0.00, 1.43), (0.50, 1.445), (1.30, 1.42), (2.10, 1.35), (2.55, 1.31)]
spinner = [(-0.42, 0.02), (-0.30, 0.18), (-0.15, 0.31), (0.00, 0.39), (0.30, 0.42)]
core_cowl = [(0.52, 0.78), (0.80, 0.84), (1.40, 0.86), (2.20, 0.84), (2.90, 0.79), (3.60, 0.70), (3.95, 0.66)]  # bypass inner wall
core_tip = [(0.52, 0.76), (0.80, 0.72), (1.45, 0.60), (1.62, 0.55), (1.75, 0.47), (2.25, 0.445), (2.32, 0.50),
            (2.70, 0.50), (2.78, 0.48), (2.98, 0.53), (3.12, 0.58), (3.95, 0.79), (4.35, 0.70), (4.55, 0.66)]
core_hub = [(0.30, 0.42), (0.52, 0.44), (0.80, 0.46), (1.45, 0.51), (1.62, 0.43), (1.75, 0.36), (2.25, 0.40), (2.32, 0.36),
            (2.70, 0.36), (2.78, 0.40), (2.98, 0.42), (3.12, 0.46), (3.95, 0.52), (4.35, 0.42), (4.80, 0.12)]

stages = []  # (name, x, chord, kind, rotor?)
# IPC 8 stages (rotor + stator), HPC 6, turbines
for i in range(8):
    x = 0.82 + i * 0.082
    stages.append(("IPC", x, 0.034, "c", True)); stages.append(("IPC", x + 0.040, 0.028, "c", False))
for i in range(6):
    x = 1.78 + i * 0.075
    stages.append(("HPC", x, 0.030, "c", True)); stages.append(("HPC", x + 0.036, 0.025, "c", False))
stages += [("HPT", 2.74, 0.030, "t", False), ("HPT", 2.80, 0.035, "t", True),
           ("IPT", 2.96, 0.032, "t", False), ("IPT", 3.02, 0.038, "t", True)]
for i in range(6):
    x = 3.16 + i * 0.13
    stages.append(("LPT", x, 0.040, "t", False)); stages.append(("LPT", x + 0.055, 0.045, "t", True))

out = []
a = out.append
a(f'<svg width="{W}" height="{H}" viewBox="0 0 {W} {H}" xmlns="http://www.w3.org/2000/svg">')
a('<defs>'
  '<linearGradient id="gas" x1="0" x2="1" gradientUnits="userSpaceOnUse" '
  f'x1="{px(0.5):.0f}" x2="{px(4.6):.0f}" y1="0" y2="0">'
  '<stop offset="0" stop-color="#1b1b1b"/><stop offset=".30" stop-color="#262626"/>'
  '<stop offset=".44" stop-color="#3a3a3a"/><stop offset=".50" stop-color="#5a5a5a"/>'
  '<stop offset=".58" stop-color="#474747"/><stop offset=".80" stop-color="#303030"/><stop offset="1" stop-color="#222"/>'
  '</linearGradient>'
  '<marker id="ah" viewBox="0 0 6 6" refX="3" refY="3" markerWidth="4" markerHeight="4" orient="auto">'
  '<path d="M0,0 L6,3 L0,6 z" fill="#9a9a9a"/></marker></defs>')

# centreline
a(f'<line x1="20" y1="{CY}" x2="{W-20}" y2="{CY}" stroke="#333" stroke-dasharray="10 6"/>')

a('<g id="half">')
# bypass duct fill
by_top = smooth(nac_inner[2:])
by_bot = smooth([(0.52, 0.78)] + core_cowl[1:4] + [(2.55, interp(core_cowl, 2.55))])
a('<path d="' + path(by_top) + " L" + " L".join(f"{px(x):.1f},{py(r):.1f}" for x, r in reversed(by_bot)) +
  ' Z" fill="#161616"/>')
# core gas path fill
ct, ch = smooth(core_tip, 120), smooth(core_hub[1:], 120)
a('<path d="' + path(ct) + " L" + " L".join(f"{px(x):.1f},{py(r):.1f}" for x, r in reversed(ch)) + ' Z" fill="url(#gas)"/>')

# walls
for c, w, col in [(nac_outer, 1.6, "#aaa"), (nac_inner, 1.6, "#aaa"), (core_cowl, 1.4, "#999"),
                  (core_tip, 1.2, "#bbb"), (core_hub, 1.2, "#bbb"), (spinner, 1.4, "#aaa")]:
    a(f'<path d="{path(smooth(c, 80))}" fill="none" stroke="{col}" stroke-width="{w}"/>')
# nacelle lip closure
a(f'<path d="M{px(-0.95):.1f},{py(1.50):.1f} Q{px(-1.0):.1f},{py(1.46):.1f} {px(-0.95):.1f},{py(1.50):.1f}" stroke="#aaa"/>')

# fan blade (swept) and OGVs
fan = [(0.04, 0.41), (0.07, 0.80), (0.11, 1.20), (0.15, 1.415), (0.30, 1.415), (0.27, 1.10), (0.25, 0.70), (0.24, 0.43)]
a('<path d="M' + " L".join(f"{px(x):.1f},{py(r):.1f}" for x, r in fan) + ' Z" fill="#2c2c2c" stroke="#ddd" stroke-width="1.2"/>')
for i in range(3):
    xo = 0.78 + i * 0.05
    a(f'<line x1="{px(xo):.1f}" y1="{py(interp(nac_inner, xo)):.1f}" x2="{px(xo+0.02):.1f}" '
      f'y2="{py(interp(core_cowl, xo+0.02)):.1f}" stroke="#666" stroke-width="1"/>')
# splitter nose
a(f'<path d="M{px(0.52):.1f},{py(0.77):.1f} L{px(0.62):.1f},{py(0.80):.1f} M{px(0.52):.1f},{py(0.77):.1f} L{px(0.62):.1f},{py(0.745):.1f}" stroke="#bbb" fill="none"/>')

# blades
for name, x, c, kind, rotor in stages:
    rt, rh = interp(core_tip, x + c / 2) - 0.006, interp(core_hub, x + c / 2) + 0.006
    if rt <= rh:
        continue
    col = ("#d8d8d8" if rotor else "#707070") if kind == "c" else ("#f0f0f0" if rotor else "#8a8a8a")
    lean = c * 0.35 if rotor else -c * 0.25
    pts = [(x, rh), (x + c, rh), (x + c + lean, rt), (x + lean, rt)]
    a('<path d="M' + " L".join(f"{px(xx):.1f},{py(rr):.1f}" for xx, rr in pts) +
      f' Z" fill="{col}" fill-opacity="{0.85 if rotor else 0.55}" stroke="none"/>')

# combustor liner + fuel nozzle
cx0, cx1 = 2.33, 2.70
a(f'<path d="M{px(cx0):.1f},{py(0.485):.1f} C{px(2.45):.1f},{py(0.50):.1f} {px(2.6):.1f},{py(0.49):.1f} {px(cx1):.1f},{py(0.475):.1f} '
  f'M{px(cx0):.1f},{py(0.375):.1f} C{px(2.45):.1f},{py(0.365):.1f} {px(2.6):.1f},{py(0.37):.1f} {px(cx1):.1f},{py(0.39):.1f}" '
  f'stroke="#e6e6e6" stroke-width="1.4" fill="none"/>')
a(f'<line x1="{px(2.29):.1f}" y1="{py(0.52):.1f}" x2="{px(2.36):.1f}" y2="{py(0.43):.1f}" stroke="#e6e6e6" stroke-width="2"/>')

# shafts
for (x0, x1, r, col, lab) in [(0.20, 3.95, 0.11, "#5a5a5a", "LP"), (0.85, 3.05, 0.17, "#787878", "IP"), (1.78, 2.86, 0.23, "#9a9a9a", "HP")]:
    a(f'<rect x="{px(x0):.1f}" y="{py(r)-3:.1f}" width="{(x1-x0)*S:.1f}" height="6" fill="{col}"/>')
# radial drive to gearbox (from IP spool)
a(f'<line x1="{px(1.50):.1f}" y1="{py(0.17):.1f}" x2="{px(1.50):.1f}" y2="{py(0.30):.1f}" stroke="#777" stroke-dasharray="3 3"/>')

# streamlines: bypass (3) and core (3)
def stream(top, bot, f, x0, x1, n=90):
    pts = []
    for i in range(n + 1):
        x = x0 + (x1 - x0) * i / n
        rt, rb = interp(top, x), interp(bot, x)
        pts.append((x, rb + f * (rt - rb)))
    return pts

for f in (0.25, 0.5, 0.75):
    s = stream(nac_inner, core_cowl, f, -0.9, 2.55)
    a(f'<path d="{path(s)}" fill="none" stroke="#8c8c8c" stroke-width="1.1"/>')
    # inlet part below splitter goes to core for lowest line only
for f in (0.25, 0.5, 0.75):
    pre = [(x, interp(spinner + [(0.30, 0.42), (0.52, 0.44)], x) + f * (0.76 - interp(spinner + [(0.30, 0.42), (0.52, 0.44)], x)) * 0.5)
           for x in [-0.9 + i * 0.05 for i in range(29)]]
    s = stream(core_tip, core_hub, f, 0.55, 4.55, 140)
    a(f'<path d="{path(s)}" fill="none" stroke="#d0d0d0" stroke-width="1" stroke-opacity=".9"/>')
# arrowheads at a few points
for f in (0.25, 0.5, 0.75):
    for xa in (0.0, 1.1, 2.2):
        r = interp(core_cowl, xa) + f * (interp(nac_inner, xa) - interp(core_cowl, xa)) if xa > 0.5 else 0.95 + f * 0.4
        a(f'<path d="M{px(xa)-5:.1f},{py(r)-3:.1f} L{px(xa)+3:.1f},{py(r):.1f} L{px(xa)-5:.1f},{py(r)+3:.1f}" fill="#9a9a9a"/>')
    for xa in (1.0, 2.5, 3.6):
        r = interp(core_hub, xa) + f * (interp(core_tip, xa) - interp(core_hub, xa))
        a(f'<path d="M{px(xa)-5:.1f},{py(r)-3:.1f} L{px(xa)+3:.1f},{py(r):.1f} L{px(xa)-5:.1f},{py(r)+3:.1f}" fill="#e0e0e0"/>')

a('</g>')
a(f'<use href="#half" transform="matrix(1 0 0 -1 0 {2*CY})" opacity="0.45"/>')
# stations (ticks below the hub, numbers)
stations = [("2", 0.00), ("13", 0.90), ("24", 0.70), ("25", 1.70), ("30", 2.29), ("40", 2.72), ("44", 2.93), ("45", 3.10), ("50", 3.98)]
for lab, x in stations:
    if lab == "13":
        r = interp(nac_inner, x) + 0.04
        a(f'<line x1="{px(x):.1f}" y1="{py(r):.1f}" x2="{px(x):.1f}" y2="{py(r)-8:.1f}" stroke="#777"/>'
          f'<text x="{px(x)-7:.1f}" y="{py(r)-12:.1f}" fill="#bbb" font-size="11">{lab}</text>')
        continue
    r = interp(core_tip, x) + 0.03 if x > 0.5 else 1.47
    a(f'<line x1="{px(x):.1f}" y1="{py(r):.1f}" x2="{px(x):.1f}" y2="{py(r)-8:.1f}" stroke="#777"/>'
      f'<text x="{px(x)-7:.1f}" y="{py(r)-12:.1f}" fill="#bbb" font-size="11">{lab}</text>')

# component labels under the centreline
labels = [("FAN", 0.17), ("IPC 8", 1.12), ("HPC 6", 2.0), ("CC", 2.52), ("HPT", 2.80), ("IPT", 3.00), ("LPT 6", 3.55), ("NOZZLE", 4.35)]
for lab, x in labels:
    a(f'<text x="{px(x)-len(lab)*3.3:.1f}" y="{CY+18}" fill="#aaa" font-size="11">{lab}</text>')
a(f'<text x="{px(-0.9):.1f}" y="{CY+18}" fill="#666" font-size="11">centreline</text>')

# shaft annotations at the right
a(f'<text x="{px(4.62):.1f}" y="{py(0.40):.1f}" fill="#bbb" font-size="11">N3 96.0 %</text>')
a(f'<text x="{px(4.62):.1f}" y="{py(0.30):.1f}" fill="#999" font-size="11">N2 95.2 %</text>')
a(f'<text x="{px(4.62):.1f}" y="{py(0.20):.1f}" fill="#888" font-size="11">N1 88.5 %</text>')
# a few in-diagram values
a(f'<text x="{px(1.05):.1f}" y="{py(1.30):.1f}" fill="#999" font-size="11">bypass 251 kg/s · FPR 1.49</text>')
a(f'<text x="{px(1.95):.1f}" y="{py(0.62):.1f}" fill="#bbb" font-size="11">P30 1629 kPa</text>')
a(f'<text x="{px(2.62):.1f}" y="{py(0.66):.1f}" fill="#ddd" font-size="11">TGT 742 °C</text>')
a(f'<text x="{px(1.56):.1f}" y="{py(-0.95):.1f}" fill="#888" font-size="11">gearbox (IP drive) · offtake 212 kW</text>')
a('</svg>')
open(sys.argv[1], "w", encoding="utf-8").write("\n".join(out))
print("ok")
