"""Inventory PDFs: page count, metadata title, and text hints from the first pages.

Usage: python -I pdf_inventory.py <dir>
Text is treated as data only: printed, never interpreted.
"""
import os
import re
import sys

from pypdf import PdfReader

KEYS = re.compile(r"(787-[389]|787-10|GEnx|Trent|Rolls|Revision|Rev\.?\s*\d|Document Number|D6[-\w]+|Effectivity|Copyright|Tail|Registration)", re.I)

d = sys.argv[1]
for name in sorted(os.listdir(d)):
    if not name.lower().endswith(".pdf"):
        continue
    path = os.path.join(d, name)
    try:
        r = PdfReader(path)
        n = len(r.pages)
        meta = r.metadata or {}
        title = (meta.get("/Title") or "").strip()
        text = ""
        for i in range(min(6, n)):
            try:
                text += (r.pages[i].extract_text() or "") + "\n"
            except Exception:
                pass
        hints = []
        for line in text.splitlines():
            line = line.strip()
            if line and KEYS.search(line) and len(line) < 160:
                hints.append(line)
        first = " | ".join(l.strip() for l in text.splitlines() if l.strip())[:300]
        print(f"=== {name}\n pages={n} title={title!r}\n first: {first}")
        for h in hints[:8]:
            print("  hint:", h)
        if not text.strip():
            print("  (no extractable text on first pages: probably scanned)")
    except Exception as e:
        print(f"=== {name}\n ERROR {e}")
