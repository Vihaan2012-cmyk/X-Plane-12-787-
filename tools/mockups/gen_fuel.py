"""Generate the fuel inspector page with every individual valve, pump, check valve and sensor.

Component set comes from the verified ATA 28 / STM / LOT facts (refs/facts/facts_fuel.verified.md).
Usage: python -I gen_fuel.py <out.html>
"""
import sys

W, H = 1000, 610
g = []
a = g.append
ON, OFF, DIM, BAD = "#e8e8e8", "#444", "#888", "#e05555"


def txt(x, y, s, c="#ddd", size=10):
    a(f'<text x="{x}" y="{y}" fill="{c}" font-size="{size}">{s}</text>')


def line(x1, y1, x2, y2, on=True, w=2):
    st = f'stroke="{ON}"' if on else f'stroke="{OFF}" stroke-dasharray="4 3"'
    a(f'<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" {st} stroke-width="{w}"/>')


def valve(x, y, ident, is_open, label_dy=-8, bad=False):
    col = BAD if bad else "#ddd"
    fill = col if is_open else "none"
    a(f'<rect x="{x-5}" y="{y-5}" width="10" height="10" fill="{fill}" stroke="{col}" stroke-width="1.2"/>')
    if ident:
        txt(x - len(ident) * 3, y + label_dy, ident, DIM if not bad else BAD, 9)


def pump(x, y, ident, on):
    a(f'<circle cx="{x}" cy="{y}" r="8" fill="#111" stroke="{"#ddd" if on else "#555"}" stroke-width="1.4"/>')
    if on:
        a(f'<path d="M{x-3},{y-4} L{x+4},{y} L{x-3},{y+4} z" fill="#ddd"/>')
    txt(x - len(ident) * 3, y + 20, ident, "#bbb" if on else "#666", 9)


def check(x, y, ident=None, up=True):
    d = f"M{x-5},{y+4} L{x+5},{y+4} L{x},{y-4} z" if up else f"M{x-5},{y-4} L{x+5},{y-4} L{x},{y+4} z"
    a(f'<path d="{d}" fill="none" stroke="#999"/>')
    if ident:
        txt(x + 7, y + 4, ident, "#666", 8)


def jet(x, y, ident):
    a(f'<path d="M{x-6},{y-4} L{x+6},{y} L{x-6},{y+4}" fill="none" stroke="#bbb"/>')
    txt(x - len(ident) * 3, y + 14, ident, "#888", 8)


# ---- tanks ----
a('<rect x="40" y="200" width="340" height="170" fill="#151515" stroke="#666"/>')
a('<rect x="400" y="200" width="200" height="170" fill="#151515" stroke="#666"/>')
a('<rect x="620" y="200" width="340" height="170" fill="#151515" stroke="#666"/>')
a('<rect x="8" y="230" width="26" height="110" fill="#131313" stroke="#555"/>')
a('<rect x="966" y="230" width="26" height="110" fill="#131313" stroke="#555"/>')
txt(170, 365, "L MAIN 18 420 kg · −22.4 °C", "#ccc", 10)
txt(440, 365, "CENTER 4 110 kg · −14.0 °C", "#ccc", 10)
txt(730, 365, "R MAIN 18 380 kg · −22.1 °C", "#ccc", 10)
txt(8, 226, "surge L", "#666", 9); txt(950, 226, "surge R", "#666", 9)

# ---- engines, APU ----
for x, n, ff in [(150, "ENG L", "1 408 kg/h"), (850, "ENG R", "1 412 kg/h")]:
    a(f'<rect x="{x-50}" y="20" width="100" height="40" fill="#1a1a1a" stroke="#ddd"/>')
    txt(x - 40, 37, n, "#ddd", 11); txt(x - 40, 52, ff, DIM, 10)
a('<rect x="470" y="20" width="60" height="40" fill="#1a1a1a" stroke="#555"/>'); txt(482, 37, "APU", "#888", 11); txt(482, 52, "off", "#666", 10)
a('<rect x="560" y="20" width="96" height="40" fill="#1a1a1a" stroke="#777"/>'); txt(566, 37, "FSM", "#bbb", 10); txt(566, 52, "hold-up 100 %", DIM, 9)

# ---- spar valves, APU SOV ----
line(150, 60, 150, 170); valve(150, 100, "SPAR-L", True, -9)
line(850, 60, 850, 170); valve(850, 100, "SPAR-R", True, -9)
line(500, 60, 500, 230, on=False, w=1.5); valve(500, 100, "APU-SOV", False, -9)

# ---- engine feed manifold + crossfeed ----
line(60, 170, 470, 170, w=3); line(530, 170, 940, 170, w=3)
line(470, 170, 530, 170, on=False, w=3); valve(500, 170, "XFEED", False, -9)

# ---- L main pumps ----
for x, n in [(110, "L FWD"), (190, "L AFT")]:
    line(x, 170, x, 232, w=2); pump(x, 240, n, True)
    check(x, 214, None); txt(x + 8, 214, "PRV", "#666", 8)
check(270, 230, "suction L"); line(270, 170, 270, 226, on=False, w=1)
jet(190, 300, "WS jet L")
# R main pumps
for x, n in [(810, "R FWD"), (890, "R AFT")]:
    line(x, 170, x, 232, w=2); pump(x, 240, n, True)
    check(x, 214, None); txt(x + 8, 214, "PRV", "#666", 8)
check(730, 230, "suction R"); line(730, 170, 730, 226, on=False, w=1)
jet(890, 300, "WS jet R")
# center O/J and APU DC pump
for x, n in [(440, "O/J L"), (560, "O/J R")]:
    line(x, 170, x, 232, w=2.5); pump(x, 240, n, True); check(x, 212, None)
pump(500, 240, "APU DC", False)
# center scavenge jets + scavenge isolation valves
jet(430, 315, "SCAV jet L"); jet(570, 315, "SCAV jet R")
line(420, 315, 385, 315, on=False, w=1); valve(390, 315, "SCAV-ISO L", False, -9)
line(580, 315, 615, 315, on=False, w=1); valve(610, 315, "SCAV-ISO R", False, -9)

# ---- vent system (top inside tanks) ----
for x in (70, 90):
    valve(x, 210, None, True); txt(x - 5, 225, "VD", "#555", 7)
for x in (650, 670):
    valve(x, 210, None, True); txt(x - 5, 225, "VD", "#555", 7)
for x in (420, 440, 460, 480):
    valve(x, 210, None, True); txt(x - 5, 225, "VD", "#555", 7)
for x, n in [(340, "VF L"), (580, "VF C"), (920, "VF R")]:
    valve(x, 210, n, True, 16)
for y in (260, 280, 300):
    check(21, y, None, up=False); check(979, y, None, up=False)
txt(10, 352, "drain ×3", "#555", 8); txt(950, 352, "drain ×3", "#555", 8)
valve(21, 245, None, False); valve(979, 245, None, False)
txt(-2 + 10, 375, "PRV", "#555", 8)
jet(545, 280, "NGS ejector"); check(596, 255, "ACF ×2")
# sump drains
for x in (300, 470, 530, 700):
    a(f'<rect x="{x-4}" y="366" width="8" height="8" fill="none" stroke="#777"/>')
txt(300 - 15, 386, "sump", "#555", 8); txt(700 - 15, 386, "sump", "#555", 8); txt(482, 386, "sump ×2", "#555", 8)

# ---- refuel / jettison gallery ----
line(40, 430, 960, 430, on=False, w=2)
txt(380, 446, "refuel / jettison manifold · not in use", "#666", 9)
a('<rect x="40" y="440" width="70" height="22" fill="#1a1a1a" stroke="#555"/>'); txt(46, 455, "REFUEL ADPT", "#777", 8)
for x, n in [(100, "RF L-OB"), (300, "RF L-IB"), (460, "RF C1"), (540, "RF C2"), (700, "RF R-IB"), (900, "RF R-OB")]:
    line(x, 370, x, 430, on=False, w=1); valve(x, 400, n, False, -8)
for x, n in [(390, "DEF-ISO L"), (610, "DEF-ISO R")]:
    valve(x, 450, n, False, 18)
for x, n in [(430, "JETT-ISO L"), (570, "JETT-ISO R")]:
    line(x, 248, x, 430, on=False, w=1); valve(x, 420, n, False, -2)
for x, n in [(330, "MDV L"), (670, "MDV R")]:
    valve(x, 445, n, True, 18)
for x, n in [(150, "NOZ L"), (850, "NOZ R")]:
    line(x, 430, x, 480, on=False, w=1.5); valve(x, 480, n, False, 18)
txt(120, 512, "jettison nozzle · TE inboard of aileron", "#555", 8)

# ---- FQIS ----
for x, n in [(150, "FQDC L"), (500, "FQDC C"), (850, "FQDC R")]:
    a(f'<rect x="{x-38}" y="530" width="76" height="22" fill="#1a1a1a" stroke="#888"/>'); txt(x - 30, 545, n, "#bbb", 9)
txt(60, 572, "CAN → RDC ×2 per FQDC · channels A/B · FDS ×2 · FCS ×4 · height sensors · overfill ×6 · RTD per tank", "#777", 9)
txt(60, 594, "FOB 40 910 kg · imbalance 40 kg · flow bars from valve/pump config (not sensed) · CG 26.4 % MAC", "#888", 10)

svg = f'<svg width="{W}" height="{H}" viewBox="0 0 {W} {H}">' + "\n".join(g) + "</svg>"

rows = [
    ("SPAR-L", "engine fuel shutoff L", "ball · 28 VDC motor", "RPDU · FSM backup", "fuel ctl / fire sw", "open", "open", "100 %", "O on · C off", "2.3 s"),
    ("SPAR-R", "engine fuel shutoff R", "ball · 28 VDC motor", "RPDU · FSM backup", "fuel ctl / fire sw", "open", "open", "100 %", "O on · C off", "2.2 s"),
    ("APU-SOV", "APU shutoff", "ball · 28 VDC motor", "RPDU · FSM backup", "APUC / fire sw", "shut", "shut", "0 %", "O off · C on", "2.4 s"),
    ("XFEED", "crossfeed", "ball · 28 VDC motor", "RPDU", "FQMS / FCP", "shut", "shut", "0 %", "O off · C on", "2.1 s"),
    ("SCAV-ISO L", "scavenge isolation L", "28 VDC", "RPDU", "FQMS", "shut", "shut", "0 %", "—", "1.8 s"),
    ("JETT-ISO L", "jettison isolation L (2 in)", "28 VDC", "RPDU", "FQMS", "shut", "shut", "0 %", "—", "—"),
    ("DEF-ISO L", "defuel / isolation L", "28 VDC motor", "RPDU", "RCP / FQMS", "shut", "shut", "0 %", "—", "—"),
    ("RF L-OB", "refuel valve L outboard", "ball · 28 VDC motor", "RPDU", "FQMS", "shut", "shut", "0 %", "—", "3.0 s"),
    ("NOZ L", "jettison nozzle L (2 in)", "ball · 28 VDC motor", "RPDU, hard-wired", "NOZZLE sw", "shut", "shut", "0 %", "—", "—"),
    ("MDV L", "manifold drain L", "spring poppet", "—", "pressure", "—", "open", "—", "—", "—"),
    ("VF L", "vent float L", "float", "—", "fuel level", "—", "open", "—", "—", "—"),
]
trs = "\n".join("<tr>" + "".join(f"<td>{c}</td>" for c in r) + "</tr>" for r in rows)

nav = ('<div class="h">POWER</div><div>Electrical</div><div>Engines</div><div>APU</div><div class="on">Fuel</div><div>Hydraulics</div>'
       '<div class="h">FLIGHT</div><div>Flight controls</div><div>Gear, brakes</div><div>Autoflight</div><div>FMS</div>'
       '<div class="h">ENVIRONMENT</div><div>Air, pressurisation</div><div>Ice and rain</div><div>Fire</div>'
       '<div class="h">AVIONICS</div><div>Air data, inertial</div><div>Navigation radios</div><div>Surveillance</div><div>Communications</div><div>Displays, alerting</div><div>CCS computers</div><div>CDN network</div>'
       '<div class="h">CABIN</div><div>Lights</div><div>Doors, oxygen, water</div><div>EFB, info systems</div>')

html = f'''<!doctype html><html><head><meta charset="utf-8"><link rel="stylesheet" href="base.css"><title>Fuel</title></head><body>
<div class="top"><b>SYSTEMS</b><span>Fuel</span><span class="dim">sim time</span><span>2843.112 s</span><span class="dim">phase</span><span>cruise FL370</span><span class="dim">view</span><span>every valve, pump, check valve, sensor</span><span class="dim">components</span><span>71 (ATA 28 / STM / LOT facts)</span></div>
<div class="shell"><nav>{nav}</nav>
<div class="diag"><h2>Fuel system <span>■ open · □ shut · ○ pump (▶ running) · △ check valve · &gt; jet pump · red = disagree / failed</span></h2>{svg}</div>
<div class="side"><h2>XFEED <span>crossfeed valve · shut</span></h2><div class="pad">
<div class="kv">
 <div>type</div><div>motor-operated ball valve</div>
 <div>location</div><div>centre tank manifold · actuator in L wheel well</div>
 <div>power</div><div>28 VDC · RPDU 72 ch 9 · SSPC 5 A</div>
 <div>command</div><div>FQMS (CCS) via RPDU · FCP switch</div>
 <div>position</div><div>0 % (shut)</div>
 <div>micro-switches</div><div>OPEN off · CLOSED on</div>
 <div>motor current</div><div>0.0 A (limit switch cut power)</div>
 <div>last travel</div><div>2.1 s · 3.4 A peak</div>
 <div>actuator temp</div><div>−8 °C (wheel well)</div>
 <div>cycles</div><div>1 204</div>
 <div>actuator rev.</div><div>post SB 280015 replacement</div>
</div>
<h3>Indication logic</h3>
<div class="kv">
 <div>FCP VALVE light</div><div>off (position agrees)</div>
 <div>EICAS</div><div>FUEL CROSSFEED if disagree</div>
 <div>synoptic</div><div>white = shut · green = open/transit · amber ✕ = failed</div>
</div>
<h3>History</h3>
<table>
<tr><td>2611.0</td><td>cmd SHUT (balance done)</td><td>2.1 s</td></tr>
<tr><td>2430.4</td><td>cmd OPEN (balance)</td><td>2.2 s</td></tr>
<tr><td>0.0</td><td>power-up BIT pass</td><td></td></tr>
</table>
<h3>Failures armed</h3>
<div class="kv"><div>motor</div><div>0.00</div><div>micro-switch stuck</div><div>0.00</div><div>ball seized</div><div>0.00</div><div>external leak</div><div>0.00</div></div>
</div></div>
<div class="bottom"><h2>Every valve <span>11 of 71 shown</span></h2>
<table><tr><th>id</th><th>item</th><th>type</th><th>power</th><th>commanded by</th><th>cmd</th><th>actual</th><th>pos</th><th>switches</th><th>last travel</th></tr>
{trs}</table></div>
</div></body></html>'''
open(sys.argv[1], "w", encoding="utf-8").write(html)
print("ok")
