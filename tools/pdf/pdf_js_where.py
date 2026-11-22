"""For each PDF, find every raw /JS or /JavaScript token and report whether it falls
inside binary stream data (a coincidental byte match) or in object syntax (real).
Also list any embedded files with their names, sizes and first bytes.

Usage: python -I pdf_js_where.py <file> [<file> ...]
"""
import re
import sys

from pypdf import PdfReader

TOK = re.compile(rb"/(JS|JavaScript)(?![A-Za-z0-9#])")

for path in sys.argv[1:]:
    data = open(path, "rb").read()
    # Byte ranges of stream bodies.
    spans = []
    for m in re.finditer(rb"stream\r?\n", data):
        end = data.find(b"endstream", m.end())
        if end != -1:
            spans.append((m.end(), end))
    spans.sort()

    def in_stream(pos):
        lo, hi = 0, len(spans) - 1
        while lo <= hi:
            mid = (lo + hi) // 2
            s, e = spans[mid]
            if pos < s:
                hi = mid - 1
            elif pos >= e:
                lo = mid + 1
            else:
                return True
        return False

    real = []
    coincidental = 0
    for m in TOK.finditer(data):
        if in_stream(m.start()):
            coincidental += 1
        else:
            ctx = data[max(0, m.start() - 60): m.end() + 60]
            real.append(ctx)
    name = path.replace("\\", "/").split("/")[-1]
    print(f"== {name}: {coincidental} inside stream data, {len(real)} in object syntax")
    for c in real[:5]:
        print("   ", c)
    try:
        r = PdfReader(path, strict=False)
        root = r.trailer["/Root"].get_object()
        names = root.get("/Names")
        if names is not None:
            ef = names.get_object().get("/EmbeddedFiles")
            if ef is not None:
                arr = ef.get_object().get("/Names", [])
                for i in range(0, len(arr), 2):
                    fname = str(arr[i])
                    spec = arr[i + 1].get_object()
                    f = spec.get("/EF", {}).get("/F")
                    body = f.get_object().get_data() if f is not None else b""
                    print(f"   embedded file: {fname!r}  {len(body):,} bytes  starts {body[:8]!r}")
    except Exception as e:
        print("   (embedded-file check failed:", e, ")")
