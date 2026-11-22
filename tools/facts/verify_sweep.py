"""Check every row the sweep agents wrote: is the quote really at the cited line, and are the row's key tokens there?

Usage: python -E -P verify_sweep.py <refs_dir> <out_dir>

For each markdown table row with `quote` and `cite` columns:
  VERIFIED   quote found within 2 lines of the cite
  REANCHORED quote found elsewhere in the same file (cite moved to the nearest match)
  MISSING    quote not in the file
Whitespace is collapsed on both sides; "/" in a quote may stand for "|".
Token check (TOKENS ok/bad): breaker IDs, RPDU numbers and the amp rating in the row must appear within
3 lines of the anchored line. Writes <name>.checked.md next to each input and prints a summary.
"""
import glob
import os
import re
import sys

REFS, OUT = sys.argv[1], sys.argv[2]
WS = re.compile(r"\s+")
IDRE = re.compile(r"\bC[EKT][0-9]{7}\b")
RPDU = re.compile(r"RPDU ?([0-9]{2})")
cache = {}


def norm(s):
    return WS.sub(" ", s).strip()


def lines_of(fname):
    if fname not in cache:
        d = "ocr" if fname.startswith("fig_") else "text"
        p = os.path.join(REFS, d, fname)
        cache[fname] = [norm(l) for l in open(p, encoding="utf-8", errors="replace")] if os.path.exists(p) else None
    return cache[fname]


def find(lines, quote, near):
    cands = {quote, quote.replace("/", "|")}
    hits = [i for i, l in enumerate(lines) if any(c in l for c in cands)]
    if not hits:
        lq = quote.lower()
        hits = [i for i, l in enumerate(lines) if lq in l.lower()]
    if not hits:
        return None
    return min(hits, key=lambda i: abs(i - near))


def split_row(line):
    cells = line.strip().strip("|").split("|")
    return [c.strip() for c in cells]


totals = {}
for md in sorted(glob.glob(os.path.join(OUT, "*.md"))):
    if md.endswith(".checked.md"):
        continue
    name = os.path.basename(md)[:-3]
    out, header, qi, ci = [], None, None, None
    st = {"VERIFIED": 0, "REANCHORED": 0, "MISSING": 0, "NOFILE": 0, "tokens_bad": 0, "rows": 0}
    for raw in open(md, encoding="utf-8", errors="replace"):
        line = raw.rstrip("\n")
        if line.startswith("|") and header is None:
            header = [h.lower() for h in split_row(line)]
            qi = header.index("quote") if "quote" in header else None
            ci = header.index("cite") if "cite" in header else None
            out.append(line + (" status | tokens |" if qi is not None else ""))
            continue
        if line.startswith("|") and set(line.replace("|", "").strip()) <= set("-: "):
            out.append(line + ("---|---|" if qi is not None else ""))
            continue
        if not line.startswith("|"):
            header = None
            out.append(line)
            continue
        cells = split_row(line)
        if qi is None or ci is None or len(cells) <= max(qi, ci):
            out.append(line)
            continue
        st["rows"] += 1
        cidx = [j for j, c in enumerate(cells) if re.search(r"\.txt:\d+", c)]
        cj = cidx[-1] if cidx else ci
        qj = cj - 1
        raw_quote = cells[qj].strip('"“”`')
        quote, cite = norm(raw_quote), cells[cj].strip("` ")
        m = re.match(r"([\w.\-]+\.txt):(\d+)", cite)
        status, tok = "MISSING", "-"
        if m:
            fname, ln = m.group(1), int(m.group(2))
            lines = lines_of(fname)
            if lines is None:
                status = "NOFILE"
            else:
                at = find(lines, quote, ln - 1) if quote else None
                if at is None and quote:
                    frags = [norm(f) for f in re.split(r"\s{2,}", raw_quote) if f.strip()]
                    spots = [find(lines, f, ln - 1) for f in frags]
                    if len(frags) > 1 and all(x is not None and abs(x - (ln - 1)) <= 3 for x in spots):
                        at = spots[0]
                if at is not None:
                    status = "VERIFIED" if abs(at - (ln - 1)) <= 2 else "REANCHORED"
                    if status == "REANCHORED":
                        cells[cj] = f"{fname}:{at + 1}"
                    win = re.sub(r"[()]", "", " ".join(lines[max(0, at - 3): at + 4]))
                    row = " | ".join(c for j, c in enumerate(cells) if j not in (qj, cj))
                    need = set(IDRE.findall(row)) | {f"RPDU {n}" for n in RPDU.findall(row)}
                    bad = [t for t in need if t not in win and t.replace(" ", "") not in win]
                    tok = "ok" if not bad else "bad: " + ",".join(sorted(bad))
                    if bad:
                        st["tokens_bad"] += 1
        st[status] += 1
        out.append("| " + " | ".join(cells) + f" | {status} | {tok} |")
    open(md[:-3] + ".checked.md", "w", encoding="utf-8").write("\n".join(out) + "\n")
    totals[name] = st
    print(f"{name:12s} rows {st['rows']:5d}  verified {st['VERIFIED']:5d}  reanchored {st['REANCHORED']:4d}  "
          f"missing {st['MISSING']:4d}  nofile {st['NOFILE']:3d}  token-mismatch {st['tokens_bad']:4d}")
s = {k: sum(t[k] for t in totals.values()) for k in ("rows", "VERIFIED", "REANCHORED", "MISSING", "NOFILE", "tokens_bad")}
print("TOTAL", s)
