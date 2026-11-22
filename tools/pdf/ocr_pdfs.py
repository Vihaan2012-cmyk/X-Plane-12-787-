"""OCR scanned PDFs into searchable text files, one page at a time and in parallel.

Usage: python -I ocr_pdfs.py <out_dir> <work_dir> <pdf> [<pdf> ...]

Each page is rendered with pdftoppm at 300 dpi in grayscale, then read by Tesseract.
Output: <out_dir>/<pdf name>.txt with a "=== page N ===" line before each page.
Pages already done are cached in work_dir, so a rerun resumes where it stopped.
"""
import concurrent.futures as cf
import os
import subprocess
import sys

TESS = os.path.join(os.environ["LOCALAPPDATA"], "Programs", "Tesseract-OCR", "tesseract.exe")
PDFTOPPM = "pdftoppm"
OUT, WORK, PDFS = sys.argv[1], sys.argv[2], sys.argv[3:]
ENV = dict(os.environ, OMP_THREAD_LIMIT="1")
NO_WINDOW = 0x08000000  # CREATE_NO_WINDOW


def page_count(pdf):
    from pypdf import PdfReader
    return len(PdfReader(pdf, strict=False).pages)


def ocr_page(pdf, tag, n):
    txt = os.path.join(WORK, f"{tag}-{n:05d}.txt")
    if os.path.exists(txt):
        return n, True
    stem = os.path.join(WORK, f"{tag}-{n:05d}")
    subprocess.run([PDFTOPPM, "-r", "300", "-gray", "-png", "-f", str(n), "-l", str(n), "-singlefile", pdf, stem],
                   check=True, capture_output=True, creationflags=NO_WINDOW)
    png = stem + ".png"
    # Tesseract writes <base>.txt itself; write to a temp base, then rename, so an interrupted page never looks done.
    tmp = stem + ".part"
    subprocess.run([TESS, png, tmp, "--psm", "3", "-l", "eng"], check=True, capture_output=True,
                   env=ENV, creationflags=NO_WINDOW)
    os.replace(tmp + ".txt", txt)
    os.remove(png)
    return n, False


def main():
    os.makedirs(OUT, exist_ok=True)
    os.makedirs(WORK, exist_ok=True)
    workers = max(1, (os.cpu_count() or 4) - 4)
    for pdf in PDFS:
        name = os.path.basename(pdf)
        tag = "".join(c if c.isalnum() else "_" for c in os.path.splitext(name)[0])[:40]
        n = page_count(pdf)
        print(f"{name}: {n} pages, {workers} workers", flush=True)
        done = 0
        with cf.ThreadPoolExecutor(workers) as ex:
            futs = [ex.submit(ocr_page, pdf, tag, i) for i in range(1, n + 1)]
            for f in cf.as_completed(futs):
                try:
                    f.result()
                except Exception as e:
                    print(f"  page failed: {e}", flush=True)
                done += 1
                if done % 200 == 0 or done == n:
                    print(f"  {done}/{n}", flush=True)
        out = os.path.join(OUT, name + ".txt")
        with open(out, "w", encoding="utf-8") as fo:
            for i in range(1, n + 1):
                p = os.path.join(WORK, f"{tag}-{i:05d}.txt")
                fo.write(f"\n=== page {i} ===\n")
                if os.path.exists(p):
                    fo.write(open(p, encoding="utf-8", errors="replace").read())
                else:
                    fo.write("[OCR FAILED]\n")
        print(f"  wrote {out}", flush=True)
    print("done", flush=True)


main()
