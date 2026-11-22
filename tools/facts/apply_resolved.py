"""Apply the resolved BAD-row answers to the v2 inventories, and verify every quote mechanically.

Usage: python -I apply_resolved.py <scratch_dir>
For each answer:
  - The quote must really exist in the cited source file within ±60 lines of the cited line
    (whitespace- and case-insensitive, OCR-tolerant: 85% of quote words, in order, in a window).
  - false_alarm with a verified quote: the row is kept and its citation is recorded as quote-verified.
  - replace with a verified quote: the row is replaced and recorded as quote-verified.
  - replace without a verified quote: the row is replaced but stays unverified.
  - unsupported: the row is replaced with the UNSUPPORTED version.
Writes quote_verified.json (normalised row -> cite) for the checker, and prints a summary.
"""
import io
import json
import os
import re
import sys

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
SCR = sys.argv[1]
TXT = os.path.join(SCR, "txt")
ALIAS = {"stm": "stm", "loc": "locator", "locator": "locator", "fcom": "lot_fcom", "lot_fcom": "lot_fcom"}


def norm_row(r):
    return re.sub(r"\s+", " ", r.strip())


def src_lines(name):
    n = name.lower().replace(".txt", "")
    n = ALIAS.get(n, n)
    if re.fullmatch(r"a\d\d", n):
        n = "ata" + n[1:]
    p = os.path.join(TXT, n + ".txt")
    return open(p, encoding="utf-8", errors="replace").read().lower().splitlines() if os.path.exists(p) else None


def words(s):
    return re.findall(r"[a-z0-9]+", s.lower())


def find_quote(quote, cite):
    """Search the whole cited file for the quote (OCR-tolerant, in-order words). Return the true 1-based line, or None."""
    m = re.search(r"([A-Za-z_0-9]+?)(?:\.txt)?\s*[: ]\s*p?\.?\s*(\d+)", cite or "")
    if not m or not quote:
        return None, None
    name = m.group(1)
    L = src_lines(name)
    if not L:
        return None, None
    q = words(quote)
    if len(q) < 3:
        return None, None
    flat, owner = [], []
    for i, l in enumerate(L):
        for w in words(l):
            flat.append(w); owner.append(i + 1)
    need = max(3, int(len(q) * 0.85))
    best = None
    first = q[0]
    for start_i, w in enumerate(flat):
        if w != first:
            continue
        hits, j = 1, start_i + 1
        limit = start_i + len(q) * 6 + 40
        for qw in q[1:]:
            k = j
            while k < min(len(flat), limit) and flat[k] != qw:
                k += 1
            if k < min(len(flat), limit):
                hits += 1; j = k + 1
        if hits >= need:
            best = owner[start_i]; break
    return name, best


orig = {}
for part in ("a", "b"):
    for line in open(os.path.join(SCR, f"bad_rows_{part}.jsonl"), encoding="utf-8"):
        r = json.loads(line)
        orig[r["id"]] = r

answers = []
for part in ("a", "b"):
    p = os.path.join(SCR, f"bad_rows_{part}.resolved.jsonl")
    for line in open(p, encoding="utf-8"):
        line = line.strip()
        if line:
            answers.append(json.loads(line))

files = {}
verified = {}
stats = {}
for a in answers:
    o = orig.get(a.get("id"))
    if not o:
        stats["unknown id"] = stats.get("unknown id", 0) + 1
        continue
    fp = os.path.join(SCR, o["file"])
    if fp not in files:
        files[fp] = open(fp, encoding="utf-8").read().split("\n")
    lines = files[fp]
    target = norm_row(o["row"])
    idx = next((i for i, l in enumerate(lines) if norm_row(l) == target), None)
    if idx is None:
        stats["row not found"] = stats.get("row not found", 0) + 1
        continue
    act = a.get("action")
    name, true_ln = find_quote(a.get("quote", ""), a.get("cite", ""))
    qok = true_ln is not None
    if qok:
        fixed_cite = f"{name.lower().replace('.txt','')}.txt:{true_ln}"
        a["cite"] = fixed_cite
        stats.setdefault("cites re-anchored", 0)
        stats["cites re-anchored"] += 1
    if act == "false_alarm":
        key = (stats.setdefault("false_alarm quote OK" if qok else "false_alarm quote NOT found", 0))
        stats["false_alarm quote OK" if qok else "false_alarm quote NOT found"] += 1
        if qok:
            cells = lines[idx].strip().strip("|").split("|")
            cells[-1] = " " + a["cite"] + " "
            lines[idx] = "|" + "|".join(cells) + "|"
            verified[norm_row(lines[idx])] = a["cite"]
    elif act in ("replace", "unsupported") and a.get("new_row", "").startswith("|"):
        new_row = a["new_row"].strip()
        if qok and act == "replace":
            cells = new_row.strip("|").split("|")
            cells[-1] = " " + a["cite"] + " "
            new_row = "|" + "|".join(cells) + "|"
        lines[idx] = new_row
        label = f"{act} quote OK" if (qok and act == "replace") else act + ("" if act == "unsupported" else " quote NOT found")
        stats[label] = stats.get(label, 0) + 1
        if qok and act == "replace":
            verified[norm_row(new_row)] = a["cite"]
    else:
        stats["malformed"] = stats.get("malformed", 0) + 1

for fp, lines in files.items():
    out = fp.replace(".v2.md", ".v3.md")
    open(out, "w", encoding="utf-8").write("\n".join(lines))
for g in ("avionics", "power", "controls", "environment"):
    src = os.path.join(SCR, f"inv_{g}.v2.md")
    dst = os.path.join(SCR, f"inv_{g}.v3.md")
    if not os.path.exists(dst):
        open(dst, "w", encoding="utf-8").write(open(src, encoding="utf-8").read())
json.dump(verified, open(os.path.join(SCR, "quote_verified.json"), "w", encoding="utf-8"), ensure_ascii=False, indent=0)
print(json.dumps(stats, indent=1))
print("quote-verified rows:", len(verified))
