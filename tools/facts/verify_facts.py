"""Verify fact tables (| Topic | Fact | Quote | Cite |): find each quote in the cited file and re-anchor the line.

Usage: python -I verify_facts.py <txt_dir> <facts.md> [<facts.md> ...]
Writes <facts>.verified.md with the Cite column corrected and a Check column:
  VERIFIED  quote found (OCR-tolerant, 85% of words in order); cite re-anchored to the true line
  MISSING   quote not found anywhere in the cited file
  NOCITE    no usable cite or quote
"""
import io
import os
import re
import sys

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
TXT = sys.argv[1]
ALIAS = {"loc": "locator", "fcom": "lot_fcom", "stm": "stm"}
cache = {}


def load(name):
    n = name.lower().replace(".txt", "")
    n = ALIAS.get(n, n)
    if re.fullmatch(r"a\d\d", n):
        n = "ata" + n[1:]
    if n not in cache:
        p = os.path.join(TXT, n + ".txt")
        if not os.path.exists(p):
            cache[n] = None
        else:
            lines = open(p, encoding="utf-8", errors="replace").read().lower().splitlines()
            flat, owner = [], []
            for i, l in enumerate(lines):
                for w in re.findall(r"[a-z0-9]+", l):
                    flat.append(w)
                    owner.append(i + 1)
            cache[n] = (flat, owner)
    return n, cache[n]


def find(quote, fname):
    n, data = load(fname)
    if not data:
        return n, None
    flat, owner = data
    q = re.findall(r"[a-z0-9]+", quote.lower())
    if len(q) < 3:
        return n, None
    need = max(3, int(len(q) * 0.85))
    for s, w in enumerate(flat):
        if w != q[0]:
            continue
        hits, j, lim = 1, s + 1, min(len(flat), s + len(q) * 6 + 40)
        for qw in q[1:]:
            k = j
            while k < lim and flat[k] != qw:
                k += 1
            if k < lim:
                hits += 1
                j = k + 1
        if hits >= need:
            return n, owner[s]
    return n, None


for path in sys.argv[2:]:
    out, stats = [], {"VERIFIED": 0, "MISSING": 0, "NOCITE": 0}
    for line in open(path, encoding="utf-8"):
        s = line.rstrip("\n")
        if not s.startswith("|"):
            out.append(s)
            continue
        cells = [c.strip() for c in s.strip().strip("|").split("|")]
        if re.match(r"^-+$", cells[0].replace(":", "")) or cells[0].lower() == "topic":
            out.append(s.rstrip().rstrip("|") + ("|---|" if cells[0].startswith("-") else "| Check |"))
            continue
        if len(cells) < 4:
            out.append(s)
            continue
        quote, cite = cells[-2].strip('"“”'), cells[-1]
        m = re.search(r"([A-Za-z_0-9]+?)(?:\.txt)?\s*:\s*\d+", cite)
        if not m:
            stats["NOCITE"] += 1
            out.append(s.rstrip().rstrip("|") + "| NOCITE |")
            continue
        name, ln = find(quote, m.group(1))
        if ln:
            stats["VERIFIED"] += 1
            cells[-1] = f"{name}.txt:{ln}"
            out.append("| " + " | ".join(cells) + " | VERIFIED |")
        else:
            stats["MISSING"] += 1
            out.append(s.rstrip().rstrip("|") + "| MISSING |")
    dst = path[:-3] + ".verified.md"
    open(dst, "w", encoding="utf-8").write("\n".join(out) + "\n")
    print(f"{os.path.basename(path):18} {stats}")
