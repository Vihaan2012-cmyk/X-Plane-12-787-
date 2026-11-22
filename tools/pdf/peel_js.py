"""Statically de-obfuscate a PDF's layered JavaScript (unescape -> run-length -> eval chains) WITHOUT executing it.

Usage: python -I peel_js.py <pdf>
"""
import io
import re
import sys

from pypdf import PdfReader

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
r = PdfReader(sys.argv[1], strict=False)
scripts = []
for pg in r.pages:
    for a in pg.get("/Annots", []) or []:
        aa = a.get_object().get("/AA")
        if aa is None:
            continue
        for _, v in aa.get_object().items():
            v = v.get_object()
            if v.get("/S") == "/JavaScript":
                js = v["/JS"].get_object()
                s = js.get_data().decode("latin-1") if hasattr(js, "get_data") else str(js)
                if "unescape" in s and s not in scripts:
                    scripts.append(s)
print("obfuscated scripts:", len(scripts))


def js_unescape(s):
    s = re.sub(r"%u([0-9a-fA-F]{4})", lambda m: chr(int(m.group(1), 16)), s)
    return re.sub(r"%([0-9a-fA-F]{2})", lambda m: chr(int(m.group(1), 16)), s)


def js_literal(s):
    s = re.sub(r"\\x([0-9a-fA-F]{2})", lambda m: chr(int(m.group(1), 16)), s)
    s = re.sub(r"\\u([0-9a-fA-F]{4})", lambda m: chr(int(m.group(1), 16)), s)
    return s.replace("\\'", "'").replace('\\"', '"').replace("\\\\", "\\")


def z9b(s):
    """The script's own expander: '^' followed by digits N becomes N backslashes."""
    out, i = [], 0
    while True:
        j = s.find("^", i)
        if j == -1:
            out.append(s[i:])
            return "".join(out)
        out.append(s[i:j])
        k, num = j + 1, ""
        while k < len(s) and s[k].isdigit():
            num += s[k]
            k += 1
        out.append("\\" * int(num or 0))
        i = k


PAT = re.compile(r"(\w+)\s*=\s*unescape\('((?:[^'\\]|\\.)*)'\)", re.S)
for n, code in enumerate(scripts, 1):
    print(f"\n##### script {n}: {len(code)} chars")
    layer = 0
    while layer < 12:
        m = PAT.search(code)
        if not m:
            break
        name, lit = m.group(1), m.group(2)
        val = js_unescape(js_literal(lit))
        if re.search(name + r"\s*=\s*__z9b\(" + name + r"\)", code):
            val = z9b(val)
        layer += 1
        print(f"layer {layer}: {name} -> {len(val)} chars; eval={('eval(' in val)} unescape={('unescape(' in val)}")
        code = val
    flat = " ".join(code.split())
    print("--- innermost, first 1500 chars ---")
    print(flat[:1500])
    print("--- API references ---")
    print(sorted(set(re.findall(r"\b(?:app|this|util|event|Net|SOAP|Collab|console|host|global|identity|security|spell|media)\.[A-Za-z_]+", code))))

    # Generic final peel: keep decoding %XX and \xNN escapes until the text stops changing.
    prev = None
    while prev != code:
        prev = code
        code = js_unescape(code)
        code = re.sub(r"\\x([0-9a-fA-F]{2})", lambda m: chr(int(m.group(1), 16)), code)
    flat = " ".join(code.split())
    print("--- fully decoded, first 2500 chars ---")
    print(flat[:2500])
    print("--- API / risky references ---")
    print(sorted(set(re.findall(r"\b(?:app|util|event|Net|SOAP|Collab|console|host|global|identity|security|spell|media)\.[A-Za-z_]+", code))))
    print("risky:", sorted(set(re.findall(r"(launchURL|submitForm|importDataObject|exportDataObject|mailDoc|openDoc|getURL|util\.printf|Net\.HTTP|SOAP\.|app\.launch)", code))))
