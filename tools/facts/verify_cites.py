"""Check every inventory row's citation: do the row's key terms appear near the cited line?

Usage: python -I verify_cites.py <txt_dir> <inv.md> [<inv.md> ...]
Writes <inv>.checked.md next to each input. A "Check" column is added to every table row:
  OK    at least half the row's key terms (and the acronym, if any) appear within ±WIN lines
  NEAR  some terms found, below the threshold
  BAD   no terms found near the cited line
  NOCITE no line citation in the row
Also prints a summary.
"""
import io
import os
import re
import sys

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
TXT = sys.argv[1]
WIN = 40
CITE = re.compile(r"\b(stm|ata\d\d|a\d\d|locator|loc|lot_fcom|aireuropa|fctm2021|qrh)(?:\.txt)?\s*:\s*(\d+)", re.I)
ALIAS = {"loc": "locator", "fcom": "lot_fcom"}
STOP = set("the and for with from into each that this side left right center centre system systems unit units "
           "module modules data control controller power box airplane aircraft".split())
cache = {}
import json
OVR = json.load(open("quote_verified.json", encoding="utf-8")) if os.path.exists("quote_verified.json") else {}


def norm_row(r):
    return re.sub(r"\s+", " ", r.strip())


def lines_of(src):
    src = ALIAS.get(src.lower(), src.lower())
    if re.fullmatch(r"a\d\d", src):
        src = "ata" + src[1:]
    if src not in cache:
        p = os.path.join(TXT, src + ".txt")
        cache[src] = open(p, encoding="utf-8", errors="replace").read().lower().splitlines() if os.path.exists(p) else None
    return cache[src]


def terms(cells):
    text = " ".join(cells[:2])
    acr = [a.lower() for a in re.findall(r"\b[A-Z][A-Z0-9]{1,7}\b", " ".join(cells[:3]))]
    words = [w.lower() for w in re.findall(r"[A-Za-z][A-Za-z0-9/-]{3,}", text) if w.lower() not in STOP]
    return list(dict.fromkeys(acr)), list(dict.fromkeys(words))[:6]


def check(row):
    cells = [c.strip() for c in row.strip().strip("|").split("|")]
    cites = CITE.findall(row)
    if not cites:
        return "NOCITE"
    acr, words = terms(cells)
    want = acr + words
    if not want:
        return "NEAR"
    best = 0.0
    for src, ln in cites:
        L = lines_of(src)
        if not L:
            continue
        ln = int(ln)
        window = " ".join(L[max(0, ln - WIN - 1): ln + WIN])
        found = sum(1 for t in want if t in window)
        acr_ok = (not acr) or any(a in window for a in acr)
        score = found / len(want)
        if acr_ok:
            best = max(best, score)
        else:
            best = max(best, min(score, 0.49))
    return "OK" if best >= 0.5 else ("NEAR" if best > 0 else "BAD")


summary = {}
for path in sys.argv[2:]:
    out = []
    stats = {"OK": 0, "QUOTED": 0, "NEAR": 0, "BAD": 0, "NOCITE": 0}
    for line in open(path, encoding="utf-8"):
        s = line.rstrip("\n")
        if s.startswith("|") and not re.match(r"^\|\s*-", s) and "Source" not in s[:400]:
            st = "QUOTED" if norm_row(s) in OVR else check(s)
            stats[st] += 1
            out.append(s.rstrip().rstrip("|") + f"| {st} |")
        elif s.startswith("|") and re.match(r"^\|\s*-", s):
            out.append(s.rstrip().rstrip("|") + "|---|")
        elif s.startswith("|"):
            out.append(s.rstrip().rstrip("|") + "| Check |")
        else:
            out.append(s)
    dst = path[:-3] + ".checked.md"
    open(dst, "w", encoding="utf-8").write("\n".join(out) + "\n")
    summary[os.path.basename(path)] = stats
for k, v in summary.items():
    tot = sum(v.values())
    print(f"{k:22} rows={tot:4}  OK={v['OK']:4}  QUOTED={v['QUOTED']:4}  NEAR={v['NEAR']:4}  BAD={v['BAD']:4}  NOCITE={v['NOCITE']:4}")
