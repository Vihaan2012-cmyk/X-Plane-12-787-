"""Merge the checked sweep tables into fact files for the electrical plan.

Usage: python -E -P merge_sweep.py <refs_dir>

Reads refs/facts/sweep/out/*.checked.md and refs/facts/sweep/breakers_raw.csv. Writes to refs/facts/:
  elec_breakers.verified.md   one row per breaker / switching-device ID
  elec_loads.verified.md      load -> power path rows without an ID
  elec_structure.verified.md  counts and structure rows
  elec_conditions.verified.md power conditions + MMEL rows (copied)
  locator.verified.md         part locations (copied)
Only rows whose quote was found in the source (VERIFIED) are kept. Rows whose ID/RPDU token was not
near the quote get grade CHECK. Placeholder IDs from the CBIC example screens are dropped.
"""
import collections
import csv
import difflib
import glob
import os
import re
import sys

REFS = sys.argv[1]
OUT = os.path.join(REFS, "facts", "sweep", "out")
FACTS = os.path.join(REFS, "facts")
IDRE = re.compile(r"\bC[EKT] ?[0-9]{7}\b")
PLACEHOLDER = re.compile(r"^(CE21007\d\d|CK2100780|CT2100704)$")
CITE = re.compile(r"\.txt:\d+")
SKIP_NAMES = {"", "unpaired", "ocr-damaged", "-", "?"}


def cells_of(line):
    return [c.strip() for c in line.strip().strip("|").split("|")]


def tables(path):
    """Yield (table_title, header, rows) for each table in a checked md file."""
    title, header, rows = None, None, []
    for raw in open(path, encoding="utf-8", errors="replace"):
        line = raw.rstrip("\n")
        if line.startswith("## "):
            if header:
                yield title, header, rows
            title, header, rows = line[3:].strip(), None, []
        elif line.startswith("|"):
            c = cells_of(line)
            if header is None:
                header = [h.lower() for h in c]
            elif set("".join(c)) <= set("-: "):
                continue
            else:
                rows.append(c)
        elif header and rows and not line.strip():
            yield title, header, rows
            header, rows = None, []
    if header:
        yield title, header, rows


def split_row(c):
    status, tokens = c[-2], c[-1]
    cidx = [j for j, x in enumerate(c[:-2]) if CITE.search(x)]
    cj = cidx[-1] if cidx else len(c) - 3
    return c[:cj - 1], c[cj - 1], c[cj], status, tokens


breakers = collections.defaultdict(lambda: {"names": collections.Counter(), "paths": set(), "types": set(),
                                            "ratings": set(), "cites": [], "check": False, "example": False})
loads, structure, kept, dropped = [], [], collections.Counter(), collections.Counter()

for md in sorted(glob.glob(os.path.join(OUT, "*.checked.md"))):
    group = os.path.basename(md)[:-len(".checked.md")]
    if group in ("conditions", "locator"):
        continue
    for title, header, rows in tables(md):
        t = (title or "").lower()
        for c in rows:
            if len(c) < 4:
                continue
            fields, quote, cite, status, tokens = split_row(c)
            if status != "VERIFIED":
                dropped["not verified"] += 1
                continue
            check = tokens.startswith("bad")
            if "count" in t or "structure" in t:
                structure.append((group, fields, quote, cite, check))
                kept["structure"] += 1
                continue
            load = fields[0] if fields else ""
            rest = fields[4:] if len(fields) > 4 else []
            ids = [i.replace(" ", "") for i in IDRE.findall(" ".join(fields))]
            if any(PLACEHOLDER.match(i) for i in ids) or re.search(r"(BREAKER|CONTACTOR) NAME", load, re.I):
                dropped["placeholder"] += 1
                continue
            rating = next((x for x in rest if re.fullmatch(r"\d+(\.\d+)?\s*A?", x)), "")
            if ids:
                for i in ids:
                    b = breakers[i]
                    nm = re.sub(r"\s+", " ", load).strip()
                    if nm.lower() not in SKIP_NAMES and not IDRE.fullmatch(nm) and not nm.lower().startswith(("unpaired", "ocr-damaged")):
                        b["names"][nm] += 1
                    if len(fields) > 3 and fields[3]:
                        b["paths"].add(fields[3])
                    if len(rest) > 1 and rest[1]:
                        b["types"].add(rest[1])
                    if rating:
                        b["ratings"].add(rating.rstrip("A ").strip())
                    b["cites"].append(cite)
                    b["check"] |= check
                    b["example"] |= ("example" in " ".join(fields).lower()) or "eps24.txt:65" in cite or "eps24.txt:338" in cite
                kept["breaker rows"] += 1
            else:
                loads.append((group, fields, quote, cite, check))
                kept["load rows"] += 1

# mechanical layout candidates (secondary evidence)
raw = {r["id"]: r for r in csv.DictReader(open(os.path.join(FACTS, "sweep", "breakers_raw.csv"), encoding="utf-8"))}
for i, r in raw.items():
    if PLACEHOLDER.match(i):
        continue
    b = breakers[i]
    b.setdefault("layout", r["name_candidates"])
    b["ocr_only"] = all(w.startswith("fig_") for w in r["where"].split())
    if not b["cites"]:
        b["cites"] = r["where"].split()[:3]


def norm_name(n):
    return re.sub(r"[^A-Z0-9]", "", n.upper())


grades = collections.Counter()
rows_out = []
for i in sorted(breakers):
    b = breakers[i]
    names = [n for n, _ in b["names"].most_common()]
    groups = []
    for n in names:
        k = norm_name(n).replace("0", "O")
        for g in groups:
            if k.startswith(g[0]) or g[0].startswith(k) or difflib.SequenceMatcher(None, k, g[0]).ratio() >= 0.75:
                break
        else:
            groups.append((k, n))
    if names and len(groups) > 1:
        grade = "CONFLICT"
    elif names:
        grade = "NAMED"
    elif b.get("layout"):
        grade = "LAYOUT"          # name only from the mechanical same-segment pass
    else:
        grade = "ID-ONLY"
    if b.get("ocr_only"):
        grade += " · OCR"
    if b["check"]:
        grade += " · CHECK"
    if b["example"]:
        grade += " · EXAMPLE-SCREEN"
    grades[grade.split(" ·")[0]] += 1
    rows_out.append(f"| {i} | {i[:2]} | {i[2:4]} | {'; '.join(names[:4]) or b.get('layout', '')[:60]} | "
                    f"{'; '.join(sorted(b['paths']))[:90]} | {'; '.join(sorted(b['types']))[:40]} | "
                    f"{'; '.join(sorted(b['ratings']))} | {grade} | {' '.join(b['cites'][:3])} |")

hdr = ("| ID | prefix | first 2 digits | name(s) | panel / unit / module | type | rating A | grade | sources |\n"
       "|---|---|---|---|---|---|---|---|---|\n")
open(os.path.join(FACTS, "elec_breakers.verified.md"), "w", encoding="utf-8").write(
    "# Electrical: breaker and switching-device IDs (sweep 2026-11-19)\n\n"
    "Grades: NAMED (one name, quote verified) · CONFLICT (several names) · LAYOUT (name from text layout only) · "
    "ID-ONLY · OCR (seen only in OCR'd figures) · CHECK (token not next to quote) · EXAMPLE-SCREEN (CBIC demo dialog).\n\n"
    + hdr + "\n".join(rows_out) + "\n")


def dump(name, title, rows, cols):
    with open(os.path.join(FACTS, name), "w", encoding="utf-8") as fo:
        fo.write(f"# {title} (sweep 2026-11-19)\n\n| group | " + " | ".join(cols) + " | quote | cite | grade |\n|"
                 + "---|" * (len(cols) + 4) + "\n")
        for g, f, q, c, chk in rows:
            f = (f + [""] * len(cols))[:len(cols)]
            fo.write(f"| {g} | " + " | ".join(f) + f" | {q} | {c} | {'CHECK' if chk else 'VERIFIED'} |\n")


dump("elec_loads.verified.md", "Electrical: load to power path", loads,
     ["load", "ATA", "voltage / bus", "panel / unit / card / module / channel", "ID", "type", "rating A", "draw"])
dump("elec_structure.verified.md", "Electrical: counts and structure", structure, ["item", "value"])
for src, dst in (("conditions", "elec_conditions.verified.md"), ("locator", "locator.verified.md")):
    open(os.path.join(FACTS, dst), "w", encoding="utf-8").write(open(os.path.join(OUT, src + ".checked.md"), encoding="utf-8").read())

# coverage
rp = collections.defaultdict(set)
for g, f, q, c, chk in loads:
    for n in re.findall(r"RPDU ?([0-9]{2})", " | ".join(f[:4])):
        rp[n].add(f[0])
print("kept", dict(kept), "dropped", dict(dropped))
print("breaker IDs", len(breakers), dict(grades))
print("rated IDs", sum(1 for b in breakers.values() if b["ratings"]))
print("loads tied to an RPDU:", sum(len(v) for v in rp.values()), {k: len(v) for k, v in sorted(rp.items())})
