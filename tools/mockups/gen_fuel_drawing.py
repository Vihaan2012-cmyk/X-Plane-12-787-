"""Fuel system engineering drawing: 787-9 left wing + centre tank, plan view, true scale.

Geometry: semi-span 30.06 m (span 60.12 m, airport planning doc), leading-edge sweep ~34 deg,
front spar 15 % chord, rear spar 62 % chord, ribs ~0.76 m apart from side of body (rib 1) outboard.
Part locations use the rib numbers stated in the verified ATA 28 facts. Anything else is APPROX.
Usage: python -I gen_fuel_drawing.py <out.html>
"""
import math
import sys

S = 24.0                 # px per metre (true scale both axes)
CX = 945.0               # px of aircraft centreline
TOPY = 70.0              # px of the wing apex leading edge (chordwise 0)
W, H = 1000, 610
TANLE = math.tan(math.radians(34.0))
SOB = 2.88               # side of body (m from centreline)
TIP = 30.06


def chord(s):
    """Wing chord (m) vs spanwise station s (m from centreline), simple cranked planform."""
    if s <= SOB:
        return 11.8
    if s <= 9.8:
        return 11.8 - (s - SOB) * (11.8 - 7.6) / (9.8 - SOB)
    return 7.6 - (s - 9.8) * (7.6 - 1.9) / (TIP - 9.8)


def le(s): return s * TANLE
def te(s): return le(s) + chord(s)
def fs(s): return le(s) + 0.15 * chord(s)
def rs(s): return le(s) + 0.62 * chord(s)
def X(s): return CX - s * S
def Y(c): return TOPY + c * S
def rib_s(n): return SOB + 0.5 + (n - 1) * 0.76


out = []
a = out.append
parts = 0


def t(x, y, s, col="#9a9a9a", size=9, anchor="start"):
    a(f'<text x="{x:.1f}" y="{y:.1f}" fill="{col}" font-size="{size}" text-anchor="{anchor}">{s}</text>')


def ln(x1, y1, x2, y2, col="#6a6a6a", w=0.8, dash=None):
    d = f' stroke-dasharray="{dash}"' if dash else ""
    a(f'<line x1="{x1:.1f}" y1="{y1:.1f}" x2="{x2:.1f}" y2="{y2:.1f}" stroke="{col}" stroke-width="{w}"{d}/>')


def poly(pts, col="#6a6a6a", w=0.8, fill="none", dash=None):
    d = f' stroke-dasharray="{dash}"' if dash else ""
    a('<path d="M' + " L".join(f"{x:.1f},{y:.1f}" for x, y in pts) + f'" fill="{fill}" stroke="{col}" stroke-width="{w}"{d}/>')


def pipe(points_sc, col="#f2f2f2", w=1.6, live=True, coupling_every=1.4):
    """Pipe through (s, c) points in metres; couplings generated every N metres (APPROX rule)."""
    global parts
    pts = [(X(s), Y(c)) for s, c in points_sc]
    poly(pts, col if live else "#5a5a5a", w, dash=None if live else "4 3")
    # couplings
    for (s0, c0), (s1, c1) in zip(points_sc, points_sc[1:]):
        L = math.hypot(s1 - s0, c1 - c0)
        n = max(1, int(L / coupling_every))
        for k in range(1, n + 1):
            f = k / (n + 1)
            s, c = s0 + f * (s1 - s0), c0 + f * (c1 - c0)
            dx, dy = (c1 - c0), -(s1 - s0)
            m = math.hypot(dx, dy) or 1
            ux, uy = dx / m * 3.2, dy / m * 3.2
            ln(X(s) - ux, Y(c) + uy, X(s) + ux, Y(c) - uy, "#bdbdbd", 0.9)
            parts += 2   # coupling + bonding jumper
        parts += 1        # pipe segment


def valve(s, c, live, num=None):
    global parts
    x, y = X(s), Y(c)
    a(f'<path d="M{x-5},{y-4} L{x+5},{y+4} L{x+5},{y-4} L{x-5},{y+4} Z" fill="{"#f2f2f2" if live else "#111"}" stroke="#f2f2f2" stroke-width="0.9"/>')
    parts += 8   # body, ball, seals, actuator motor, gearbox, 2 micro-switches, manual lever
    if num:
        callout(x, y, num)


def pump(s, c, live, num=None):
    global parts
    x, y = X(s), Y(c)
    a(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="5.5" fill="#111" stroke="#f2f2f2" stroke-width="1"/>')
    if live:
        a(f'<path d="M{x-2},{y-3} L{x+3},{y} L{x-2},{y+3} z" fill="#f2f2f2"/>')
    parts += 11
    if num:
        callout(x, y, num)


def check(s, c):
    global parts
    x, y = X(s), Y(c)
    a(f'<path d="M{x-4},{y+3} L{x+4},{y+3} L{x},{y-3} z" fill="none" stroke="#c8c8c8" stroke-width="0.8"/>')
    parts += 3


def probe(s, c):
    global parts
    a(f'<circle cx="{X(s):.1f}" cy="{Y(c):.1f}" r="1.8" fill="#c8c8c8"/>')
    parts += 4   # probe, connector, harness segment, clamp


callouts = []


def callout(x, y, num, dx=-22, dy=-16):
    callouts.append((x, y, num, dx, dy))


# ---------- structure ----------
span_pts = [i * 0.25 for i in range(int(TIP / 0.25) + 1)] + [TIP]
# fuselage
ln(X(0), 40, X(0), 600, "#3e3e3e", 0.8, "14 4 2 4")
ln(X(SOB), 40, X(SOB), 600, "#7a7a7a", 1.0)
t(X(0) - 4, 36, "℄ A/C", "#777", 9, "end"); t(X(SOB) + 3, 36, "SOB", "#777", 9)
# wing outline
outline = [(X(s), Y(le(s))) for s in span_pts if s >= SOB] + [(X(TIP), Y(te(TIP)))] + \
          [(X(s), Y(te(s))) for s in reversed(span_pts) if s >= SOB]
poly(outline, "#a8a8a8", 1.2)
# centre wing box (inside fuselage)
poly([(X(0), Y(fs(SOB))), (X(SOB), Y(fs(SOB))), (X(SOB), Y(rs(SOB))), (X(0), Y(rs(SOB)))], "#8a8a8a", 1.0)
# spars
poly([(X(s), Y(fs(s))) for s in span_pts if s >= SOB and s <= 28.6], "#cfcfcf", 1.1)
poly([(X(s), Y(rs(s))) for s in span_pts if s >= SOB and s <= 28.6], "#cfcfcf", 1.1)
t(X(16) + 6, Y(fs(16)) - 6, "FRONT SPAR", "#8a8a8a", 9)
t(X(16) + 6, Y(rs(16)) + 13, "REAR SPAR", "#8a8a8a", 9)
# ribs (streamwise)
for n in range(1, 36):
    s = rib_s(n)
    if s > 28.6:
        break
    heavy = n in (1, 31, 32, 34)
    ln(X(s), Y(fs(s)), X(s), Y(rs(s)), "#9a9a9a" if heavy else "#3a3a3a", 1.0 if heavy else 0.6)
    if n % 5 == 0 or n in (1, 9, 12, 32):
        t(X(s), Y(fs(s)) - 4, str(n), "#7a7a7a", 8, "middle")
t(X(rib_s(1)) + 2, Y(fs(rib_s(1))) - 14, "RIB", "#6a6a6a", 8)
# tank boundaries labels
t(X(13) , Y(fs(13) + 2.6), "LEFT MAIN TANK", "#bdbdbd", 10, "middle")
t(X(1.45), Y(fs(SOB) + 2.0), "CENTER", "#bdbdbd", 10, "middle")
t(X(1.45), Y(fs(SOB) + 2.6), "TANK", "#bdbdbd", 10, "middle")
t(X(27.1), Y(fs(27.1) + 0.2), "SURGE", "#9a9a9a", 8, "middle")
# engine nacelle (under wing, forward)
es = 9.6
a(f'<rect x="{X(es)-34:.1f}" y="{Y(le(es))-150:.1f}" width="68" height="170" rx="30" fill="none" stroke="#7a7a7a" stroke-width="1"/>')
t(X(es), Y(le(es)) - 156, "ENG L (nacelle, below wing)", "#8a8a8a", 9, "middle")
# aileron / flaps hint
ln(X(21), Y(te(21)), X(26.5), Y(te(26.5)), "#5a5a5a", 3)
t(X(23.8), Y(te(23.8)) + 14, "aileron", "#666", 8, "middle")

# ---------- engine feed ----------
r9, r12 = rib_s(9), rib_s(12)
feed = [(0.6, rs(0.6) - 0.35), (SOB, rs(SOB) - 0.35), (r9, rs(r9) - 0.35), (r12, rs(r12) - 0.35)]
pipe(feed)
pump(r9, rs(r9) - 0.35, True, 1); pump(r12, rs(r12) - 0.35, True, 2)
check(r9 + 0.45, rs(r9 + 0.45) - 0.9); check(r12 + 0.45, rs(r12 + 0.45) - 0.9)   # PRVs (fwd rib 11, aft inboard rib 10, APPROX)
pump(1.6, rs(1.6) - 0.6, True, 3)                       # O/J L
valve(0.6, rs(0.6) - 0.35, False, 4)                    # crossfeed (centre tank manifold)
# spar valve and feed to engine
sv = (es, rs(es) - 0.35)
valve(*sv, True, 5)
pipe([sv, (es, le(es) - 2.0)], w=1.6)
# suction feed check valve
check(rib_s(6), rs(rib_s(6)) - 0.7)
# APU feed (dead)
pipe([(0.6, rs(0.6) - 0.35), (0.2, rs(0.2) + 0.6), (0.2, rs(0.2) + 9.5)], live=False)
valve(0.2, rs(0.2) + 0.9, False, 6); pump(0.9, rs(0.9) - 1.1, False)
t(X(0.2) + 4, Y(rs(0.2) + 9.6), "to APU ↓", "#6a6a6a", 8, "end")

# ---------- scavenge ----------
pipe([(1.9, rs(1.9) - 1.4), (SOB + 0.4, rs(SOB + 0.4) - 1.4)], live=False, coupling_every=2.0)
valve(SOB + 0.4, rs(SOB + 0.4) - 1.4, False, 7)
a(f'<path d="M{X(1.9)-6},{Y(rs(1.9)-1.4)-4} L{X(1.9)+6},{Y(rs(1.9)-1.4)} L{X(1.9)-6},{Y(rs(1.9)-1.4)+4}" fill="none" stroke="#c8c8c8"/>'); parts += 5
wx, wy = X(r12) - 10, Y(rs(r12) - 1.2)
a(f'<path d="M{wx-6},{wy-4} L{wx+6},{wy} L{wx-6},{wy+4}" fill="none" stroke="#c8c8c8"/>'); parts += 5
t(wx - 8, wy + 13, "water scav jet", "#777", 8)

# ---------- refuel / jettison manifold along front spar ----------
r5, r13, r20, r22 = rib_s(5), rib_s(13), rib_s(20), rib_s(22)
refuel = [(0.8, fs(0.8) + 0.45), (SOB, fs(SOB) + 0.45), (r5, fs(r5) + 0.45), (r13, fs(r13) + 0.45), (r20 + 0.5, fs(r20 + 0.5) + 0.45)]
pipe(refuel, live=False)
valve(0.8, fs(0.8) + 0.45, False, 8); valve(2.0, fs(2.0) + 0.45, False)
valve(r5, fs(r5) + 0.45, False, 9); valve(r20 + 0.5, fs(r20 + 0.5) + 0.45, False, 10)
a(f'<rect x="{X(r13)-9:.1f}" y="{Y(fs(r13))-16:.1f}" width="18" height="10" fill="#111" stroke="#bdbdbd"/>'); parts += 12
t(X(r13), Y(fs(r13)) - 20, "refuel adapters ×2 · RCP", "#8a8a8a", 8, "middle")
valve(SOB + 0.6, fs(SOB + 0.6) + 1.0, True, 11)                 # manifold drain valve between ribs 1 and 2
# defuel / isolation (feed ↔ refuel manifold)
pipe([(SOB - 0.4, fs(SOB - 0.4) + 0.45), (SOB - 0.4, rs(SOB - 0.4) - 0.35)], live=False, coupling_every=1.8)
valve(SOB - 0.4, (fs(SOB) + rs(SOB)) / 2, False, 12)
# jettison: manifold → nozzle at TE inboard of aileron, plus jettison isolation at O/J outlet
pipe([(r20 + 0.5, fs(r20 + 0.5) + 0.45), (r22, rs(r22) + 0.3), (20.6, te(20.6) - 0.2)], live=False)
valve(20.6, te(20.6) - 0.2, False, 13)
valve(1.6, rs(1.6) - 1.5, False, 14)

# ---------- vent system ----------
r32 = rib_s(32)
pipe([(1.0, fs(1.0) + 2.2), (SOB, fs(SOB) + 2.4), (r32, fs(r32) + 1.0)], col="#9a9a9a", w=0.9, coupling_every=2.2)
pipe([(rib_s(2), rs(rib_s(2)) - 2.2), (r32 - 0.3, rs(r32 - 0.3) - 1.0)], col="#9a9a9a", w=0.9, coupling_every=2.2)
for n in (6, 18):
    valve(rib_s(n), fs(rib_s(n)) + 1.9 if n == 6 else fs(rib_s(n)) + 1.5, True)
valve(r32, fs(r32) + 1.0, True, 15)                          # vent float valve
for k in range(3):
    check(r32 + 0.25, rs(r32 + 0.25) - 0.4 - k * 0.55)        # surge drain checks ×3
a(f'<path d="M{X(rib_s(33)):.1f},{Y(rs(rib_s(33)) - 0.2):.1f} l-8,6 l16,0 z" fill="none" stroke="#bdbdbd"/>'); parts += 6
t(X(rib_s(33)) - 12, Y(rs(rib_s(33))) + 18, "NACA scoop · flame arrestor · relief", "#777", 8, "end")

# ---------- FQIS probes ----------
for n in range(2, 31, 3):
    s = rib_s(n)
    for f in (0.28, 0.55, 0.80):
        probe(s, fs(s) + f * (rs(s) - fs(s)))
for s in (0.6, 1.4, 2.2):
    for f in (0.3, 0.7):
        probe(s, fs(SOB) + f * (rs(SOB) - fs(SOB)))
# harness to FQDC
a(f'<rect x="{X(1.2)-20:.1f}" y="{Y(rs(SOB))+30:.1f}" width="40" height="14" fill="#111" stroke="#bdbdbd"/>'); parts += 14
t(X(1.2), Y(rs(SOB)) + 40, "FQDC L", "#bdbdbd", 8, "middle")
ln(X(1.2), Y(rs(SOB)) + 30, X(1.2), Y(rs(SOB)) - 2, "#8a8a8a", 0.7, "2 2")
poly([(X(rib_s(29)), Y(rs(rib_s(29)) - 0.25)), (X(SOB), Y(rs(SOB) - 0.25))], "#7a7a7a", 0.7, dash="2 2")
t(X(18), Y(rs(18) - 0.25) + 11, "FQIS in-tank harness", "#6a6a6a", 8, "middle")
# sump drains
for s, c in [(rib_s(3), rs(rib_s(3)) - 0.5), (1.0, rs(1.0) - 0.2), (2.3, rs(2.3) - 0.2), (r32 + 0.8, rs(r32 + 0.8) - 0.3)]:
    a(f'<rect x="{X(s)-2.5:.1f}" y="{Y(c)-2.5:.1f}" width="5" height="5" fill="none" stroke="#bdbdbd" stroke-width="0.8"/>'); parts += 4

# ---------- electrical feeds (to fuselage) ----------
for s, c in [(r9, rs(r9) - 0.35), (r12, rs(r12) - 0.35), (es, rs(es) - 0.35)]:
    poly([(X(s), Y(c) + 7), (X(s), Y(rs(s)) + 1.4 * S), (X(SOB), Y(rs(SOB)) + 1.4 * S)], "#5a5a5a", 0.7, dash="1 3")
    parts += 9   # SSPC/ELCU channel, wire segments, connectors, clamps
t(X(6.0), Y(rs(6.0)) + 1.4 * S + 12, "power: ELCU / RPDU feeds (dotted)", "#666", 8)

# ---------- detail A inset: boost pump ----------
ix, iy = 30, 70
a(f'<rect x="{ix}" y="{iy}" width="300" height="200" fill="#121212" stroke="#7a7a7a"/>')
t(ix + 8, iy + 16, "DETAIL A · L AFT BOOST PUMP (rib 12, rear spar)", "#d8d8d8", 9)
a(f'<rect x="{ix+40}" y="{iy+40}" width="90" height="120" rx="6" fill="none" stroke="#cfcfcf"/>')          # canister
a(f'<rect x="{ix+55}" y="{iy+55}" width="60" height="50" fill="none" stroke="#bdbdbd"/>')                  # motor
for k in range(5):
    ln(ix + 60 + k * 12, iy + 58, ix + 60 + k * 12, iy + 102, "#5a5a5a", 0.6)
a(f'<ellipse cx="{ix+85}" cy="{iy+125}" rx="28" ry="9" fill="none" stroke="#f2f2f2"/>')                   # impeller
ln(ix + 50, iy + 150, ix + 120, iy + 150, "#9a9a9a", 1, "2 2")                                          # inlet screen
a(f'<circle cx="{ix+64}" cy="{iy+112}" r="3" fill="none" stroke="#bdbdbd"/><circle cx="{ix+106}" cy="{iy+112}" r="3" fill="none" stroke="#bdbdbd"/>')
a(f'<path d="M{ix+130},{iy+70} L{ix+160},{iy+70}" stroke="#f2f2f2" stroke-width="2"/>')
a(f'<path d="M{ix+150},{iy+64} l8,6 l-8,6" fill="none" stroke="#cfcfcf"/>')
labels = [("1 motor (235 VAC, 3-ph)", iy + 62), ("2 rotor bearings ×2", iy + 82), ("3 impeller", iy + 102),
          ("4 inlet screen ~30 in²", iy + 122), ("5 discharge check 0.5 psid", iy + 142), ("6 PRV 16 psid", iy + 162),
          ("7 thermistor (180 °F trip)", iy + 182)]
for txt_, yy in labels:
    t(ix + 172, yy, txt_, "#9a9a9a", 8)
t(ix + 8, iy + 194, "+ pressure switch LRU · liquid-ring reprime · guillotine housing check", "#6a6a6a", 8)
ln(ix + 300, iy + 100, X(r12) - 6, Y(rs(r12) - 0.35) - 4, "#5a5a5a", 0.6, "3 3")

# ---------- callout bubbles ----------
for x, y, num, dx, dy in callouts:
    ln(x, y, x + dx, y + dy, "#6a6a6a", 0.6)
    a(f'<circle cx="{x+dx:.1f}" cy="{y+dy:.1f}" r="7" fill="#121212" stroke="#bdbdbd" stroke-width="0.8"/>')
    t(x + dx, y + dy + 3, str(num), "#e8e8e8", 8, "middle")

# ---------- title block, scale bar, north arrow ----------
bx, by = 330, 548
a(f'<rect x="{bx}" y="{by}" width="420" height="52" fill="#121212" stroke="#7a7a7a"/>')
ln(bx, by + 18, bx + 420, by + 18, "#3a3a3a")
t(bx + 8, by + 13, "FUEL SYSTEM · LEFT WING AND CENTER TANK · PLAN VIEW (LOOKING DOWN)", "#e8e8e8", 9)
t(bx + 8, by + 32, f"787-9 · true scale 1:{int(1000/S*0.0254*1000/25.4)} on screen ({S:.0f} px/m) · sheet 1 of 4 · R wing mirror", "#9a9a9a", 8)
t(bx + 8, by + 46, f"parts in this view: {parts:,} · fuel system 2,413 · aircraft 48,9xx (APPROX until full lists)", "#9a9a9a", 8)
sx0 = 770
ln(sx0, 590, sx0 + 5 * S, 590, "#bdbdbd", 1.2)
for k in range(6):
    ln(sx0 + k * S, 586, sx0 + k * S, 594, "#bdbdbd", 0.8)
t(sx0, 604, "0", "#8a8a8a", 8); t(sx0 + 5 * S, 604, "5 m", "#8a8a8a", 8, "middle")
a(f'<path d="M975,90 l-6,14 l6,-4 l6,4 z" fill="#bdbdbd"/>'); t(975, 84, "FWD", "#8a8a8a", 8, "middle")

svg = f'<svg width="{W}" height="{H}" viewBox="0 0 {W} {H}">' + "\n".join(out) + "</svg>"

legend = [
    (1, "L FWD boost pump", "rib 9 · rear spar", "on · 6.1 A · 38.2 psi"), (2, "L AFT boost pump", "rib 12 · rear spar", "on · 5.9 A · 37.8 psi"),
    (3, "CTR O/J pump L", "rear spar · ctr tank", "on · 41.0 psi"), (4, "crossfeed valve", "ctr manifold · act. L wheel well", "shut"),
    (5, "spar valve L", "rear spar at pylon", "open"), (6, "APU shutoff valve", "ctr tank · act. L wheel well", "shut"),
    (7, "scavenge isolation L", "SOB", "shut"), (8, "refuel valve C1", "front spar · ctr", "shut"),
    (9, "refuel valve L-IB", "front spar · rib 5", "shut"), (10, "refuel valve L-OB", "front spar · outbd rib 20", "shut"),
    (11, "manifold drain valve L", "ribs 1–2", "open"), (12, "defuel / isolation L", "SOB", "shut"),
    (13, "jettison nozzle L", "TE inbd of aileron", "shut"), (14, "jettison isolation L", "O/J outlet", "shut"),
    (15, "vent float valve L", "rib 32 vent port", "open"),
]
rows = "\n".join(f"<tr><td>{n}</td><td>{p}</td><td>{loc}</td><td>{st}</td></tr>" for n, p, loc, st in legend)

nav = ('<div class="h">POWER</div><div>Electrical</div><div>Engines</div><div>APU</div><div class="on">Fuel</div><div>Hydraulics</div>'
       '<div class="h">FLIGHT</div><div>Flight controls</div><div>Gear, brakes</div><div>Autoflight</div><div>FMS</div>'
       '<div class="h">ENVIRONMENT</div><div>Air, pressurisation</div><div>Ice and rain</div><div>Fire</div>'
       '<div class="h">AVIONICS</div><div>Air data, inertial</div><div>Navigation radios</div><div>Surveillance</div><div>Communications</div><div>Displays, alerting</div><div>CCS computers</div><div>CDN network</div>'
       '<div class="h">CABIN</div><div>Lights</div><div>Doors, oxygen, water</div><div>EFB, info systems</div>')

html = f'''<!doctype html><html><head><meta charset="utf-8"><link rel="stylesheet" href="base.css"><title>Fuel drawing</title>
<style>.tree div{{padding:1px 0}}.tree .i1{{padding-left:12px}}.tree .i2{{padding-left:24px}}.tree .i3{{padding-left:36px}}.tree .s{{color:#fff}}</style></head><body>
<div class="top"><b>SYSTEMS</b><span>Fuel</span><span class="dim">sim time</span><span>2843.112 s</span><span class="dim">view</span><span>sheet 1 · L wing + centre · plan</span><span class="dim">zoom</span><span>wing › bay › part › sub-part</span></div>
<div class="shell"><nav>{nav}</nav>
<div class="diag" style="background:#121212">{svg}</div>
<div class="side"><h2>L AFT boost pump › impeller <span>part 2 · sub-part 3</span></h2><div class="pad">
<div class="tree">
 <div>Fuel</div><div class="i1">Left main tank · bay rib 11–12</div><div class="i2">L AFT boost pump (LRU)</div>
 <div class="i3">motor stator · rotor</div><div class="i3">rotor bearings ×2</div><div class="i3 s">▸ impeller</div><div class="i3">inlet screen</div>
 <div class="i3">discharge check valve</div><div class="i3">PRV</div><div class="i3">thermistor</div><div class="i3">pressure switch (LRU)</div>
 <div class="i3">liquid-ring reprime · housing check</div><div class="i3">ELCU channel · wire ×4 · connector ×2</div>
</div>
<h3>Impeller</h3>
<div class="kv">
 <div>speed</div><div>11 790 rpm (VF 398 Hz)</div>
 <div>head rise</div><div>37.8 psi at 0.00 kg/s (FWD pump carries feed)</div>
 <div>inlet NPSH</div><div>2.8 psi margin · −22.4 °C fuel</div>
 <div>cavitation erosion</div><div>0.03</div>
 <div>blade tip wear</div><div>0.06 · efficiency −0.8 %</div>
 <div>run hours</div><div>14 210</div>
 <div>failures armed</div><div>erosion 0.00 · blade loss 0.00</div>
</div>
<h3>Pump (LRU)</h3>
<div class="kv">
 <div>power</div><div>235 VAC · ELCU · 5.9 A</div><div>pressure switch</div><div>HIGH (resets ≥ 9.5 psig)</div>
 <div>motor temp</div><div>31 °C · trip 180 °F</div><div>auto-off logic</div><div>10 s low-press in 60 s (ground only)</div>
</div>
</div></div>
<div class="bottom"><h2>Callouts <span>15 of {parts:,} parts in view · click a part to drill in</span></h2>
<table><tr><th>#</th><th>part</th><th>location</th><th>state</th></tr>{rows}</table></div>
</div></body></html>'''
open(sys.argv[1], "w", encoding="utf-8").write(html)
print("parts in view:", parts)
