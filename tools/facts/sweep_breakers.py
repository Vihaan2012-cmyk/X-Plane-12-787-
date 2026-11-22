"""Mechanical sweep: every breaker / switching-device ID (CE, CK, CT + 7 digits) in the text and OCR files.

Usage: python -E -P sweep_breakers.py <refs_dir> <out_csv>

For each occurrence we keep the line, the column segment holding the ID (layout text splits columns on
2+ spaces), the text after the ID in that segment (name candidate), the segment before it, and any
amp value in the same segment. Output: one CSV row per ID with every candidate and where it was seen.
"""
import collections
import csv
import glob
import os
import re
import sys

REFS, OUT = sys.argv[1], sys.argv[2]
ID = re.compile(r"\bC[EKT][0-9]{7}\b")
AMP = re.compile(r"(?<![\w.])(\d{1,3}(?:\.\d)?)\s?(?:A|AMP|AMPS)\b")
SEG = re.compile(r"\S(?:.*?\S)??(?=\s{2,}|$)")

files = sorted(glob.glob(os.path.join(REFS, "text", "*.txt"))) + sorted(glob.glob(os.path.join(REFS, "ocr", "fig_*.txt")))
seen = collections.defaultdict(lambda: {"names": collections.Counter(), "before": collections.Counter(),
                                        "amps": collections.Counter(), "where": [], "n": 0})
for path in files:
    tag = os.path.basename(path)
    for ln, line in enumerate(open(path, encoding="utf-8", errors="replace"), 1):
        if "C" not in line or not ID.search(line):
            continue
        line = line.rstrip("\r\n")
        segs = [(m.start(), m.end(), m.group()) for m in SEG.finditer(line)]
        for m in ID.finditer(line):
            rec = seen[m.group()]
            rec["n"] += 1
            if len(rec["where"]) < 8:
                rec["where"].append(f"{tag}:{ln}")
            si = next((i for i, s in enumerate(segs) if s[0] <= m.start() < s[1]), None)
            if si is None:
                continue
            s0, s1, seg = segs[si]
            after = line[m.end():s1].strip(" ()-–:")
            # a following ID in the same segment ends the name
            after = ID.split(after)[0].strip()
            if after and not ID.fullmatch(after):
                rec["names"][after] += 1
            if si > 0:
                rec["before"][segs[si - 1][2]] += 1
            for a in AMP.finditer(seg):
                rec["amps"][a.group(1)] += 1

with open(OUT, "w", newline="", encoding="utf-8") as fo:
    w = csv.writer(fo)
    w.writerow(["id", "prefix", "ata", "n", "name_candidates", "before_candidates", "amps", "where"])
    for k in sorted(seen):
        r = seen[k]
        w.writerow([k, k[:2], k[2:4], r["n"],
                    " | ".join(f"{n} ({c})" for n, c in r["names"].most_common(4)),
                    " | ".join(f"{n} ({c})" for n, c in r["before"].most_common(3)),
                    " | ".join(f"{n}A ({c})" for n, c in r["amps"].most_common(3)),
                    " ".join(r["where"])])

pref = collections.Counter(k[:2] for k in seen)
named = sum(1 for r in seen.values() if r["names"])
ata = collections.Counter(k[2:4] for k in seen)
print(f"files {len(files)} · unique IDs {len(seen)} · by prefix {dict(pref)} · with a same-segment name {named}")
print("by first two digits:", dict(sorted(ata.items())))
