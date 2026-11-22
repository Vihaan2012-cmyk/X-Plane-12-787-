import csv
import glob
import os
import re
import sys
from collections import Counter

ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
FACTS = os.path.join(ROOT, "refs", "facts")
OUT = os.path.join(ROOT, "aircraft", "24-electrical")
os.makedirs(OUT, exist_ok=True)

CORE = ["elec_breakers", "elec_loads", "elec_structure", "elec_conditions", "locator", "facts_elec_gen", "facts_elec_dist"]
HEADS = ("ID", "group", "part", "Topic", "condition")


def read_table(path):
    rows = []
    header = None
    with open(path, encoding="utf-8", errors="replace") as fh:
        for line in fh:
            line = line.rstrip("\n")
            if not line.startswith("|"):
                continue
            cells = [c.strip() for c in line.strip().strip("|").split("|")]
            if all(re.fullmatch(r"-+", c) for c in cells if c):
                continue
            if header is None or (cells and cells[0] in HEADS):
                header = cells
                continue
            rows.append(dict(zip(header, cells)))
    return rows


def load_all():
    tables = {}
    for name in CORE:
        p = os.path.join(FACTS, name + ".verified.md")
        tables[name] = read_table(p) if os.path.exists(p) else []
    extra = []
    names = sorted(set(os.path.basename(p).replace(".verified", "") for p in glob.glob(os.path.join(FACTS, "elec2_*.md"))))
    for n in names:
        v = os.path.join(FACTS, n.replace(".md", ".verified.md"))
        p = v if os.path.exists(v) else os.path.join(FACTS, n)
        extra.append((n, read_table(p)))
    return tables, extra


TABLES, EXTRA = load_all()

KNOWN_CITES = set()
for rows in list(TABLES.values()) + [r for _, r in EXTRA]:
    for r in rows:
        for k in ("Cite", "cite", "sources"):
            for m in re.findall(r"[a-z_0-9]+\.txt:\d+", r.get(k, "")):
                KNOWN_CITES.add(m)


RAWMAP = {}
VERFACT = {}
for n in sorted(os.path.basename(x) for x in glob.glob(os.path.join(FACTS, "elec2_*.md"))):
    if n.endswith(".verified.md"):
        for r in read_table(os.path.join(FACTS, n)):
            VERFACT.setdefault(r.get("Fact", ""), []).append(r.get("Cite", ""))
    else:
        for r in read_table(os.path.join(FACTS, n)):
            for m in re.findall(r"[a-z_0-9]+\.txt:\d+", r.get("Cite", "")):
                RAWMAP.setdefault(m, []).append(r.get("Fact", ""))


def remap(src):
    def one(m):
        c = m.group(0)
        if c in KNOWN_CITES:
            return c
        for f in RAWMAP.get(c, []):
            for v in VERFACT.get(f, []):
                if v and v != c:
                    return v.split()[0]
        return c
    return re.sub(r"[a-z_0-9]+\.txt:\d+", one, src)


def clean(s):
    return re.sub(r"\s+", " ", (s or "").replace('"', "").replace("`", "")).strip()


def wr(name, header, rows):
    with open(os.path.join(OUT, name), "w", newline="", encoding="utf-8") as fh:
        w = csv.writer(fh)
        w.writerow(header)
        for r in rows:
            if "source" in r:
                r["source"] = remap(r["source"])
            w.writerow([r.get(h, "") for h in header])


SRC_H = ["id", "kind", "rating", "voltage", "frequency_range", "bus_fed", "grade", "source"]
sources = []


def S(id, kind, rating, volt, freq, fed, grade, src):
    sources.append(dict(id=id, kind=kind, rating=rating, voltage=volt, frequency_range=freq, bus_fed=fed, grade=grade, source=src))


for ch in ("L1", "L2", "R1", "R2"):
    S("VFSG_" + ch, "VFSG", "250 kVA", "235 VAC 3ph", "360-800 Hz", ch, "VERIFIED",
      "eps24.txt:9636 eps24.txt:9648 bus by rule four-channel-isolated eps24.txt:475")
S("ASG_L", "ASG", "225 kVA", "235 VAC 3ph", "360-440 Hz", "LTB", "VERIFIED", "eps24.txt:12527 eps24.txt:9610 eps24.txt:3046 ata24.txt:4284 eps24.txt:713")
S("ASG_R", "ASG", "225 kVA", "235 VAC 3ph", "360-440 Hz", "RTB", "VERIFIED", "eps24.txt:12527 eps24.txt:9610 eps24.txt:3046 ata24.txt:4284 eps24.txt:713")
S("RAT_GEN", "RAT generator", "10 kVA", "230 VAC 3ph", "min 380 Hz at RCB close", "BB", "VERIFIED", "ata24.txt:5155 stm.txt:8183 ata24.txt:5154 ata24.txt:428")
S("BATT_MAIN", "main battery", "50 Ah", "28 VDC", "DC", "HBB_MAIN", "VERIFIED", "ata24.txt:544 ata24.txt:588")
S("BATT_APU", "APU battery", "50 Ah", "28 VDC", "DC", "HBB_APU", "VERIFIED", "ata24.txt:544 ata24.txt:597 eps24.txt:21589")
S("EXT_L_FWD", "external power receptacle", "90 kVA", "115 VAC 3ph", "400 Hz", "AC115_L", "VERIFIED", "stm.txt:6818 eps24.txt:482 24eps.pdf p804")
S("EXT_R_FWD", "external power receptacle", "90 kVA", "115 VAC 3ph", "400 Hz", "AC115_R", "VERIFIED", "stm.txt:6818 eps24.txt:482 24eps.pdf p833")
S("EXT_AFT", "external power receptacle", "188 A max continuous", "115 VAC 3ph", "400 Hz", "ATRU_L2", "VERIFIED", "eps24.txt:4900 ata24.txt:994 eps24.txt:482")

BUS_H = ["id", "voltage", "kind", "panel", "grade", "source"]
buses = []


def B(id, volt, kind, panel, grade, src):
    buses.append(dict(id=id, voltage=volt, kind=kind, panel=panel, grade=grade, source=src))


for b, p in (("L1", "P100"), ("L2", "P100"), ("R1", "P200"), ("R2", "P200"), ("LTB", "P100"), ("RTB", "P200")):
    B(b, "235 VAC", "AC3", p, "VERIFIED", "ata24.txt:3066 ata24.txt:4647 ata24.txt:416")
B("BB", "235 VAC", "AC3", "P400", "VERIFIED", "ata24.txt:1691 eps24.txt:2759")
B("AC115_L", "115 VAC", "AC3", "P300", "APPROX", "ata24.txt:1415 ata24.txt:418 rule panel-by-side")
B("AC115_R", "115 VAC", "AC3", "P400", "APPROX", "ata24.txt:1415 ata24.txt:418 rule panel-by-side")
for b in ("HVDC_L1", "HVDC_L2", "HVDC_R1", "HVDC_R2"):
    B(b, "+/-270 VDC", "DC+-", "P700" if b.startswith("HVDC_L") else "P800", "APPROX", "ata24.txt:2417 ata24.txt:2418 ata24.txt:2436 rule panel-by-side")
for b, p in (("DC28_L", "P300"), ("DC28_R", "P400"), ("CIB", "P300"), ("FIB", "P400")):
    B(b, "28 VDC", "DC", p, "APPROX", "ata24.txt:1719 ata24.txt:422 ata24.txt:418 rule panel-by-side")
B("HBB_MAIN", "28 VDC", "DC", "P300", "VERIFIED", "ata24.txt:460 ata24.txt:588")
B("HBB_APU", "28 VDC", "DC", "P49", "VERIFIED", "ata24.txt:597")

CONV_H = ["id", "kind", "input_bus", "output_bus", "rating", "grade", "source"]
conv = []


def C(id, kind, i, o, rating, grade, src):
    conv.append(dict(id=id, kind=kind, input_bus=i, output_bus=o, rating=rating, grade=grade, source=src))


for a in ("L1", "L2", "R1", "R2"):
    C("ATRU_" + a, "ATRU", a, "HVDC_" + a, "155 kVA continuous (787-9)", "APPROX",
      "ata24.txt:6143 ata24.txt:2436 ata24.txt:2417 ata24.txt:2418 rule name-pairing")
C("ATU_L", "ATU", "L1", "AC115_L", "260 A per-unit", "VERIFIED", "ata24.txt:1408 ata24.txt:1415 eps24.txt:4945 eps24.txt:934 24eps.pdf p734")
C("ATU_R", "ATU", "R1", "AC115_R", "260 A per-unit", "VERIFIED", "ata24.txt:1408 ata24.txt:1415 eps24.txt:4945 eps24.txt:936 24eps.pdf p755")
for t, i, o, g, src in (("L", "L1", "DC28_L", "VERIFIED", "ata24.txt:1455 ata24.txt:1458 24eps.pdf p606"),
                        ("C1", "BB", "CIB", "VERIFIED", "ata24.txt:1459 ata24.txt:1587 24eps.pdf p833 elec2_r4_topology_bpcu.md"),
                        ("C2", "BB", "FIB", "VERIFIED", "ata24.txt:1459 ata24.txt:1587 elec2_r4_topology_bpcu.md"),
                        ("R", "R1", "DC28_R", "APPROX", "ata24.txt:1459 ata24.txt:1893 ata24.txt:1719 rule name-pairing")):
    C("TRU_" + t, "TRU", i, o, "", g, src)
C("SPU", "SPU", "HBB_APU", "ATRU_R2", "13.5 kW", "VERIFIED", "ata24.txt:2971 ata24.txt:2974 ata24.txt:2948")
for n, i in (("CAC_L1", "HVDC_L1"), ("CAC_L2", "HVDC_L2"), ("CAC_R1", "HVDC_R1"), ("CAC_R2", "HVDC_R2"),
             ("EMP_L", "HVDC_L1"), ("EMP_R", "HVDC_R1"), ("EMP_C1", "HVDC_L2"), ("EMP_C2", "HVDC_R2")):
    C("CMSC_" + n, "CMSC", i, "motor/start load", "110 kW (150 kW overload 3 min)", "APPROX",
      "eps24.txt:36425 ata24.txt:2560 ata24.txt:6183 rule name-pairing")

CT_H = ["id", "name", "connects_a", "connects_b", "panel", "type", "rating", "grade", "source"]
contactors = []


def K(id, name, a, b, panel, typ, rating, grade, src):
    contactors.append(dict(id=id, name=name, connects_a=a, connects_b=b, panel=panel, type=typ, rating=rating, grade=grade, source=src))


CUR = [
    ("CK2422525", "L1 GCB", "VFSG_L1", "L1", "P100", "contactor", "", "24eps.pdf p734 eps24.txt:5563"),
    ("CK2422526", "L2 GCB", "VFSG_L2", "L2", "P100", "contactor", "", "24eps.pdf p736 eps24.txt:5564"),
    ("CK2422528", "R1 GCB", "VFSG_R1", "R1", "P200", "contactor", "", "24eps.pdf p755"),
    ("CK2422529", "R2 GCB", "VFSG_R2", "R2", "P200", "contactor", "", "24eps.pdf p757"),
    ("CK2421501", "L1 BTB", "L1", "RTB", "P100", "contactor", "", "24eps.pdf p734 eps24.txt:25276"),
    ("CK2421502", "L2 BTB", "L2", "LTB", "P100", "contactor", "", "24eps.pdf p736 eps24.txt:26017"),
    ("CK2421523", "L3 BTB", "L1", "L2", "P100", "contactor", "", "24eps.pdf p734 eps24.txt:25929"),
    ("CK2421503", "R1 BTB", "R1", "LTB", "P200", "contactor", "", "24eps.pdf p755 eps24.txt:26935"),
    ("CK2421504", "R2 BTB", "R2", "RTB", "P200", "contactor", "", "24eps.pdf p757 eps24.txt:26839"),
    ("CK2421524", "R3 BTB", "R1", "R2", "P200", "contactor", "", "24eps.pdf p755 eps24.txt:26839"),
    ("CK2422527", "L APB", "ASG_L", "LTB", "P150", "contactor", "", "24eps.pdf p765 ata24.txt:4284"),
    ("CK2422530", "R APB", "ASG_R", "RTB", "P150", "contactor", "", "24eps.pdf p765 ata24.txt:4284"),
    ("CK2425501", "L ATUC", "L1", "ATU_L", "P100", "contactor", "150", "24eps.pdf p734 eps24.txt:25938"),
    ("CK2425503", "R ATUC", "R1", "ATU_R", "P200", "contactor", "150", "24eps.pdf p755 eps24.txt:26759"),
    ("CK2435501", "L1 ATRUC", "L1", "ATRU_L1", "P100", "contactor", "300", "24eps.pdf p734 eps24.txt:25945"),
    ("CK2435502", "L2 ATRUC", "L2", "ATRU_L2", "P100", "contactor", "300", "24eps.pdf p736 eps24.txt:26035"),
    ("CK2435503", "R1 ATRUC", "R1", "ATRU_R1", "P200", "contactor", "300", "24eps.pdf p755 eps24.txt:26766"),
    ("CK2435504", "R2 ATRUC", "R2", "ATRU_R2", "P200", "contactor", "300", "24eps.pdf p757 eps24.txt:26854"),
    ("CK2432507", "BB RLY", "R2", "BB", "P200", "relay", "15", "24eps.pdf p757"),
    ("CK2432502", "BB ISO RLY", "BB", "R2", "P400", "relay", "65", "24eps.pdf p833 ata24.txt:1889"),
    ("CK2441301", "L EPC", "EXT_L_FWD", "AC115_L", "P300", "contactor 3PST 115 VAC", "260", "24eps.pdf p804"),
    ("CK2441302", "R EPC", "EXT_R_FWD", "AC115_R", "P400", "contactor 3PST 115 VAC", "260", "24eps.pdf p833"),
    ("CK2441372", "LAEPC", "EXT_AFT", "ATRU_L2", "P700", "contactor", "", "eps24.txt:35829"),
    ("CK2422301", "L BSB", "ATU_L", "AC115_L", "P300", "contactor", "260", "24eps.pdf p804"),
    ("CK2422303", "R BSB", "ATU_R", "AC115_R", "P400", "contactor", "260", "24eps.pdf p833"),
    ("CK2422302", "L BTB 115", "AC115_L", "AC115_R", "P300", "contactor", "260", "24eps.pdf p804"),
    ("CK2422304", "R BTB 115", "AC115_R", "AC115_L", "P400", "contactor", "260", "24eps.pdf p833"),
    ("CK2432807", "L DCT", "DC28_L", "DC28_R", "P300", "contactor 1PST 28 VDC", "400", "24eps.pdf p804"),
    ("CK2432808", "R DCT", "DC28_R", "DC28_L", "P400", "contactor 1PST 28 VDC", "400", "24eps.pdf p833 eps24.txt:28935"),
    ("CK2432506", "L TRU RLY", "L1", "TRU_L", "P300", "contactor 3PST 235 VAC", "65", "24eps.pdf p804"),
    ("CK2432504", "R TRU RLY", "R1", "TRU_R", "P400", "contactor 3PST 235 VAC", "65", "24eps.pdf p833 eps24.txt:28932"),
    ("CK2432501", "C1 TRU ISO R", "L2", "TRU_C1", "P300", "contactor", "65", "24eps.pdf p804"),
    ("CK2432505", "C1 TRU RELAY", "BB", "TRU_C1", "P400", "contactor", "65", "24eps.pdf p833"),
    ("CK2433503", "RCB", "RAT_GEN", "BB", "P400", "contactor 3PST 235 VAC", "65", "24eps.pdf p833 eps24.txt:29051"),
    ("CK2432803", "CIT", "CIB", "FIB", "P300", "contactor 1PST NC 28 VDC", "400", "24eps.pdf p804"),
    ("CK2432804", "FIT", "FIB", "CIB", "P400", "contactor 1PST NC 28 VDC", "400", "24eps.pdf p833 eps24.txt:29760"),
    ("CK2431801", "MBR", "HBB_MAIN", "CIB", "P300", "contactor 1PST 28 VDC", "400", "24eps.pdf p804"),
    ("CK2431387", "SPUB", "SPU", "ATRU_R2", "P800", "contactor", "", "eps24.txt:35910 ata24.txt:2958"),
]
for i, n, a, b, p, t, r, s in CUR:
    K(i, n, a, b, p, t, r, "VERIFIED", s)
K("SPUC", "SPU contactor", "BATT_APU", "SPU", "P49", "contactor", "", "APPROX", "ata24.txt:2957")

OPEN_H = ["file", "item", "issue", "sources"]
openi = []


def O(f, item, issue, src):
    openi.append(dict(file=f, item=item, issue=issue, sources=src))


O("sources.csv", "BATT_MAIN/BATT_APU capacity", "50 Ah (ata24/eps24) vs 65 Ah (stm.txt:7501); 50 Ah used", "ata24.txt:544 eps24.txt:1423 stm.txt:7501")
O("conversion.csv", "TRU_R input and output", "right TRU input bus and 28 VDC bus mirrored from left TRU; not stated directly", "ata24.txt:1893 ata24.txt:1719")
O("conversion.csv", "CMSC_* identities", "only L1/L2/R1/R2 CAC, left/right EMP, center hydraulic named; EMP ids and HVDC bus mapping assumed", "ata24.txt:2592 ata24.txt:6183")
O("conversion.csv", "ATRU_* kVA", "155 kVA continuous is the 787-9 value; L2/R2 ATRUs also accept 115 VAC", "ata24.txt:6143 ata24.txt:2437")
O("buses.csv", "DC28_L/R, CIB/FIB, AC115_L/R panels", "panel by side rule; not stated per bus", "ata24.txt:418 ata24.txt:422")
O("rpdus.csv", "RPDU 74 and 72 gateway flag", "backup table labels 74 Gateway and 72 Standard; figures show 72 Gateway and 74 Standard; figures used", "eps24.txt:33402 eps24.txt:33404 eps24.txt:33016 eps24.txt:33030")
O("rpdus.csv", "RPDU 22 31 32 33 34 41 42 location", "no locator row found", "locator.verified.md")
O("contactors.csv", "BB RLY CK2432507 vs BB ISO RLY CK2432502", "both listed on the backup bus to R2 path; endpoints assumed R2 and BB", "24eps.pdf p757 24eps.pdf p833")
O("contactors.csv", "CK contactor rows from breaker sweep", "connected buses not stated in facts; connects_a and connects_b blank", "elec_breakers.verified.md")


def parse_panel(p):
    p = clean(p)
    panel = ""
    m = re.search(r"\bP\d{2,3}\b", p)
    if m:
        panel = m.group(0)
    unit = ""
    m = re.search(r"RPDU\s?(\d{2})\b", p)
    if m:
        unit = "RPDU " + m.group(1)
    elif re.search(r"[LR] SPDU", p):
        unit = re.search(r"[LR] SPDU", p).group(0)
    elif "SPDU" in p:
        unit = "SPDU"
    elif "ELCU" in p:
        unit = re.search(r"ELCU\s?\w*", p).group(0).strip()
    card = ""
    m = re.search(r"card (M\d+\s?\w*\d*)(?: / slot (\d+))?", p)
    if m:
        card = m.group(1).strip() + (" slot " + m.group(2) if m.group(2) else "")
    elif re.search(r"\bM\d{7}\b", p):
        card = re.search(r"\bM\d{7}\b", p).group(0)
    ch = ""
    m = re.search(r"\bch (\d+)", p)
    if m:
        ch = m.group(1)
    return panel, unit, card, ch


PREFIX = {"CE": "ECB", "CK": "ELCF", "CT": "TCB"}
BR_H = ["id", "name", "prefix_type", "panel", "unit", "card_module", "channel", "rating_a", "grade", "source"]
breakers = []
for r in TABLES["elec_breakers"]:
    i = clean(r.get("ID"))
    if not re.fullmatch(r"C[EKT]\d+", i):
        continue
    gr = clean(r.get("grade"))
    if "CONFLICT" in gr or "EXAMPLE" in gr:
        O("breakers.csv", i, "dropped grade " + gr, clean(r.get("sources")))
        continue
    typ = clean(r.get("type")).upper()
    pre = i[:2]
    if pre == "CK" and typ.startswith(("CONTACTOR", "RELAY")) and "ELCC" not in typ:
        continue
    panel, unit, card, ch = parse_panel(r.get("panel / unit / module", ""))
    g = "VERIFIED" if gr.startswith("NAMED") and "OCR" not in gr and "CHECK" not in gr else "APPROX"
    breakers.append(dict(id=i, name=clean(r.get("name(s)")), prefix_type=PREFIX[pre], panel=panel, unit=unit, card_module=card,
                         channel=ch, rating_a=clean(r.get("rating A")), grade=g,
                         source=clean(r.get("sources")) or "elec_breakers.verified.md"))

seen = set(c['id'] for c in contactors)
for r in TABLES["elec_breakers"]:
    i = clean(r.get("ID"))
    if not i.startswith("CK") or i in seen:
        continue
    gr = clean(r.get("grade"))
    if "CONFLICT" in gr or "EXAMPLE" in gr:
        continue
    typ = clean(r.get("type")).upper()
    if "CONTACTOR" in typ or "RELAY" in typ:
        panel, unit, card, ch = parse_panel(r.get("panel / unit / module", ""))
        t = "relay" if typ.startswith("RELAY") else "contactor"
        g = "VERIFIED" if gr.startswith("NAMED") and "OCR" not in gr and "CHECK" not in gr else "APPROX"
        seen.add(i)
        K(i, clean(r.get("name(s)")), "", "", panel, t, clean(r.get("rating A")), g, clean(r.get("sources")) or "elec_breakers.verified.md")

RESOLVED = set()
RESOLVED_K = set()
R1B = [r for n, rows in EXTRA if n.startswith("elec2_r1b") for r in rows]
for r in R1B:
    i = clean(r.get("ID"))
    if not re.fullmatch(r"C[EKT]\d{7}", i):
        continue
    typ = clean(r.get("type")).upper()
    panel, unit, card, ch = parse_panel(r.get("panel / unit / card / module / channel", ""))
    rm = re.search(r"(\d+) ?A(?![A-Za-z])", clean(r.get("rating")))
    rating = rm.group(1) if rm else ""
    nm = re.sub(r"\s*\(.*", "", clean(r.get("name")))
    src = " ".join(re.findall(r"(?:24eps\.pdf p\d+|[a-z_0-9]+\.txt:\d+)", clean(r.get("evidence (cite or pdf page)")))[:3])
    if typ.startswith("CONTACTOR"):
        if i not in seen:
            seen.add(i)
            K(i, nm, "", "", panel, "contactor", rating, "VERIFIED", src)
            RESOLVED.add(i)
            RESOLVED_K.add(i)
        continue
    kind = {"ELCC": "ELCF", "ECB": "ECB", "TCB": "TCB"}.get(typ.split(",")[0].strip())
    if kind is None:
        continue
    breakers[:] = [b for b in breakers if b["id"] != i]
    breakers.append(dict(id=i, name=nm, prefix_type=kind, panel=panel, unit=unit, card_module=card, channel=ch,
                         rating_a=rating, grade="VERIFIED", source=src))
    RESOLVED.add(i)
CURIDS = set(c[0] for c in CUR)
breakers[:] = [b for b in breakers if b["id"] not in CURIDS and b["id"] not in RESOLVED_K]
openi[:] = [o for o in openi if not (o["file"] == "breakers.csv" and o["item"] in RESOLVED)]

LD_H = ["load", "system_ata", "voltage_bus", "unit_rpdu", "breaker_id", "rating", "draw", "grade", "source"]
loads = []
for r in TABLES["elec_loads"]:
    gr = clean(r.get("grade"))
    if gr != "VERIFIED":
        if gr:
            O("loads.csv", clean(r.get("load")), "dropped grade " + gr, clean(r.get("cite")))
        continue
    panel, unit, card, ch = parse_panel(r.get("panel / unit / card / module / channel", ""))
    if unit in ("RPDU 61", "RPDU 62"):
        O("loads.csv", clean(r.get("load")), "unit " + unit + " not among the 17 RPDUs; dropped", clean(r.get("cite")))
        continue
    bid = ";".join(re.findall(r"C[EKT]\d+", clean(r.get("ID"))))
    loads.append(dict(load=clean(r.get("load")), system_ata=clean(r.get("ATA")), voltage_bus=clean(r.get("voltage / bus")),
                      unit_rpdu=unit or panel, breaker_id=bid, rating=clean(r.get("rating A")), draw=clean(r.get("draw")),
                      grade="VERIFIED", source=clean(r.get("cite"))))

RP_H = ["id", "gateway", "station", "buttline", "waterline", "location_source", "feeds", "grade", "source"]
GATEWAY = {"71", "72", "81", "82"}
ALLR = ["21", "22", "31", "32", "33", "34", "41", "42", "71", "72", "73", "74", "75", "76", "81", "82", "92"]
loc = {}
for r in TABLES["locator"]:
    m = re.fullmatch(r"RPDU (\d{2})", clean(r.get("part")))
    if m and clean(r.get("status")) == "VERIFIED" and m.group(1) not in loc:
        loc[m.group(1)] = (clean(r.get("loc 1")), clean(r.get("loc 2")), clean(r.get("loc 3")), clean(r.get("cite")))
STD_FEED = {"71": "P300 SPDU slot 5 CIB (standby)", "81": "P300 SPDU slot 5 CIB (standby)", "73": "P300 SPDU slot 3 CIB (standby)",
            "82": "P400 SPDU slot 5 FIB (standby)", "72": "P400 SPDU slot 3 FIB (standby)", "74": "P400 SPDU slot 5 FIB (standby)",
            "22": "RPDU 21 slot 2 (control backup)", "21": "RPDU 22 slot 2 (control backup)", "41": "RPDU 42 slot 3 (control backup)",
            "76": "RPDU 73 slot 3 (control backup)", "92": "RPDU 81 slot 3 (control backup)"}
FEED_SRC = {"71": "eps24.txt:33399", "81": "eps24.txt:33400", "73": "eps24.txt:33401", "82": "eps24.txt:33403", "72": "eps24.txt:33404",
            "74": "eps24.txt:33402", "22": "eps24.txt:33405", "21": "eps24.txt:33406", "41": "eps24.txt:33412", "76": "eps24.txt:33413", "92": "eps24.txt:33415"}
rpdus = []
for n in ALLR:
    l = loc.get(n)
    ins = "115 VAC 3ph + 28 VDC" if n not in ("81", "82") else "28 VDC only"
    feeds = "; ".join(x for x in (STD_FEED.get(n, ""), ins) if x)
    src = " ".join(x for x in ("eps24.txt:33016 eps24.txt:33030 ata24.txt:1960", FEED_SRC.get(n, ""), l[3] if l else "") if x)
    rpdus.append(dict(id="RPDU " + n, gateway="Y" if n in GATEWAY else "N", station=l[0] if l else "", buttline=l[1] if l else "",
                      waterline=l[2] if l else "", location_source=l[3] if l else "", feeds=feeds,
                      grade="VERIFIED" if l else "APPROX", source=src))


def finish():
    wr("sources.csv", SRC_H, sources)
    wr("buses.csv", BUS_H, buses)
    wr("conversion.csv", CONV_H, conv)
    wr("contactors.csv", CT_H, contactors)
    wr("breakers.csv", BR_H, breakers)
    wr("loads.csv", LD_H, loads)
    wr("rpdus.csv", RP_H, rpdus)
    wr("open-items.csv", OPEN_H, openi)


def validate():
    errs = []
    nodes = set(r["id"] for r in sources) | set(r["id"] for r in buses) | set(r["id"] for r in conv)
    for name, rows in (("sources", sources), ("buses", buses), ("conversion", conv), ("contactors", contactors),
                       ("breakers", breakers), ("rpdus", rpdus)):
        for k, v in Counter(r["id"] for r in rows).items():
            if v > 1:
                errs.append("dup id %s in %s x%d" % (k, name, v))
    allids = [r["id"] for r in sources] + [r["id"] for r in buses] + [r["id"] for r in conv]
    for k, v in Counter(allids).items():
        if v > 1:
            errs.append("node id clash " + k)
    for r in sources:
        if r["bus_fed"] not in nodes:
            errs.append("source %s feeds unknown %s" % (r["id"], r["bus_fed"]))
    for r in conv:
        for k in ("input_bus", "output_bus"):
            if r[k] not in nodes and r[k] != "motor/start load":
                errs.append("conv %s unknown %s" % (r["id"], r[k]))
    for r in contactors:
        for k in ("connects_a", "connects_b"):
            if r[k] and r[k] not in nodes:
                errs.append("contactor %s unknown %s" % (r["id"], r[k]))
    rp = set(r["id"] for r in rpdus)
    for n in ALLR:
        if "RPDU " + n not in rp:
            errs.append("missing RPDU " + n)
    for r in loads:
        if r["unit_rpdu"].startswith("RPDU ") and r["unit_rpdu"] not in rp:
            errs.append("load %s unknown rpdu %s" % (r["load"], r["unit_rpdu"]))
    for r in breakers:
        if r["unit"].startswith("RPDU ") and r["unit"] not in rp:
            errs.append("breaker %s unknown rpdu %s" % (r["id"], r["unit"]))
    return errs


def readme():
    def cnt(rows):
        c = Counter(r["grade"] for r in rows)
        return ", ".join("%s %d" % (k, c[k]) for k in ("VERIFIED", "DERIVED", "STANDARD", "APPROX") if c.get(k)) or "none"
    spec = [
        ("sources.csv", sources, SRC_H, "generators, batteries, RAT, external power receptacles"),
        ("buses.csv", buses, BUS_H, "AC3, DC and DC+- buses"),
        ("conversion.csv", conv, CONV_H, "ATRU, ATU, TRU, SPU, CMSC"),
        ("contactors.csv", contactors, CT_H, "named contactors and relays plus CK contactor ids from the breaker sweep (connects blank when unstated)"),
        ("breakers.csv", breakers, BR_H, "CE=ECB, CK=ELCF, CT=TCB by id prefix rule"),
        ("loads.csv", loads, LD_H, "load to bus, RPDU and breaker path"),
        ("rpdus.csv", rpdus, RP_H, "17 RPDUs, gateway flag, location (station, buttline, waterline)"),
        ("open-items.csv", openi, OPEN_H, "unresolved, conflicting or dropped rows"),
    ]
    L = ["# 24-electrical data", "", "- Generated by `tools/facts/gen_elec_data.py` from `refs/facts`; do not hand-edit.",
         "- Grades: VERIFIED, DERIVED (named rule), STANDARD, APPROX. `source` is a citation or a rule name.",
         "- Rule name-pairing: L/R and numbered names pair to the same-named bus or unit; graded APPROX.",
         "- Rule panel-by-side: left items in P100/P300/P700, right items in P200/P400/P800; graded APPROX.",
         "- breakers.csv grade: VERIFIED when the sweep grade is NAMED, else APPROX (id only, OCR or check flag).",
         "- loads.csv `unit_rpdu` holds the RPDU when stated, else the panel. Loads on RPDU 61/62 are dropped to open-items.csv.",
         "- Dropped (CONFLICT, EXAMPLE-SCREEN, CHECK) rows are listed in open-items.csv.", ""]
    for f, rows, h, d in spec:
        L.append("## " + f)
        L.append("- " + d + ".")
        L.append("- Columns: " + ", ".join(h) + ".")
        L.append("- Rows: %d" % len(rows) + ("; grades: " + cnt(rows) if "grade" in h else ""))
        L.append("")
    with open(os.path.join(OUT, "README.md"), "w", encoding="utf-8") as fh:
        fh.write("\n".join(L))
    return spec


if __name__ == "__main__":
    finish()
    errs = validate()
    spec = readme()
    for f, rows, h, d in spec:
        extra = ""
        if "grade" in h:
            extra = " " + str(dict(Counter(r["grade"] for r in rows)))
        print("%-16s %5d%s" % (f, len(rows), extra))
    used = set(m for r in sources + conv + contactors + rpdus + buses for m in re.findall(r"[a-z_0-9]+\.txt:\d+", r.get("source", "")))
    missing = sorted(used - KNOWN_CITES)
    print("elec2 files:", len(EXTRA), "cites not found in facts:", len(missing), missing[:20])
    print("validation errors:", len(errs))
    for e in errs[:40]:
        print(" ", e)
    sys.exit(1 if errs else 0)
