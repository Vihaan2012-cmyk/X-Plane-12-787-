"""OCR only the pages that carry raster figures, to catch labels the PDF text layer misses.

Usage: python -I ocr_figures.py <out_dir> <work_dir> <tag>=<pdf> [<tag>=<pdf> ...]

A page counts as a figure page when its raster images cover >= 8 % of the page.
Pages render at 300 dpi grey (PyMuPDF) and go through Tesseract with --psm 11 (sparse text,
best for diagram labels). Output: <out_dir>/fig_<tag>.txt with "=== page N ===" markers.
Done pages are cached in work_dir, so a rerun resumes.
"""
import concurrent.futures as cf
import os
import subprocess
import sys

import fitz

TESS = os.path.join(os.environ["LOCALAPPDATA"], "Programs", "Tesseract-OCR", "tesseract.exe")
ENV = dict(os.environ, OMP_THREAD_LIMIT="1")
NO_WINDOW = 0x08000000
OUT, WORK = sys.argv[1], sys.argv[2]
JOBS = [a.split("=", 1) for a in sys.argv[3:]]
os.makedirs(OUT, exist_ok=True)
os.makedirs(WORK, exist_ok=True)


def figure_pages(doc):
    pages = []
    for p in doc:
        area = abs(p.rect)
        cover = 0.0
        for info in p.get_image_info():
            r = fitz.Rect(info["bbox"]) & p.rect
            cover += abs(r)
        if area and cover / area >= 0.08:
            pages.append(p.number + 1)
    return pages


def ocr(png, txt):
    tmp = txt[:-4] + ".part"
    subprocess.run([TESS, png, tmp, "--psm", "11", "-l", "eng"], check=True, capture_output=True,
                   env=ENV, creationflags=NO_WINDOW)
    os.replace(tmp + ".txt", txt)
    os.remove(png)


for tag, pdf in JOBS:
    doc = fitz.open(pdf)
    pages = figure_pages(doc)
    print(f"{tag}: {len(pages)} figure pages of {doc.page_count}", flush=True)
    futs = []
    with cf.ThreadPoolExecutor(max_workers=28) as pool:
        for n in pages:
            txt = os.path.join(WORK, f"{tag}-{n:05d}.txt")
            if os.path.exists(txt):
                continue
            png = os.path.join(WORK, f"{tag}-{n:05d}.png")
            doc[n - 1].get_pixmap(dpi=300, colorspace=fitz.csGRAY).save(png)
            futs.append(pool.submit(ocr, png, txt))
            while sum(not f.done() for f in futs) > 56:     # keep temp PNGs bounded
                cf.wait(futs, return_when=cf.FIRST_COMPLETED)
        for f in cf.as_completed(futs):
            f.result()
    with open(os.path.join(OUT, f"fig_{tag}.txt"), "w", encoding="utf-8") as fo:
        for n in pages:
            fo.write(f"=== page {n} ===\n")
            fo.write(open(os.path.join(WORK, f"{tag}-{n:05d}.txt"), encoding="utf-8", errors="replace").read())
    print(f"{tag}: done", flush=True)
