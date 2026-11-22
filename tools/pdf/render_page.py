"""Render PDF pages to PNG so an agent can look at a figure.

Usage:
  python -E -P render_page.py <pdf> <page> <out.png> [dpi]          one page (1-based)
  python -E -P render_page.py --line <text.txt> <line>                which PDF page holds that line of a
                                                                      pdftotext extract (counts form feeds)
Text extract -> PDF (in refs/incoming/): eps24.txt -> 24eps.pdf, ata24.txt -> "ATA 24.pdf",
stm.txt -> "787- System Technical Manual by Boeing RR1K.pdf", locator.txt -> "B787 Locator - 787 Systems.pdf",
faa_2014_0773.txt -> FAA-2014-0773-0002_attachment_1.pdf, ataNN.txt -> "ATA NN.pdf" (some names have a
trailing space: "ATA 21 .pdf", "ATA 27 .pdf", "ATA 28 .pdf", "ATA 34 .pdf", "ATA 38 .pdf"),
lot_fcom.txt -> "LOT Polish Airlines B787-8_-9 Flight Crew Operations Manual.pdf" (pages marked "=== page N ===").
"""
import sys

if sys.argv[1] == "--line":
    path, line = sys.argv[2], int(sys.argv[3])
    page, marker = 1, None
    with open(path, encoding="utf-8", errors="replace") as f:
        for i, l in enumerate(f, 1):
            if l.startswith("=== page "):
                marker = int(l.split()[2])
            if i >= line:
                break
            page += l.count("\f")
    print(marker if marker is not None else page)
    sys.exit(0)

import pymupdf

pdf, page, out = sys.argv[1], int(sys.argv[2]), sys.argv[3]
dpi = int(sys.argv[4]) if len(sys.argv) > 4 else 130
doc = pymupdf.open(pdf)
doc[page - 1].get_pixmap(dpi=dpi).save(out)
print(out)
