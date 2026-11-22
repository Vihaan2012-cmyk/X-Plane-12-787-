"""Static safety scan for PDFs (the same idea as pdfid, plus a structural walk).

Usage: python -I pdf_safety.py <dir> [<dir> ...]

For every file:
  1. Magic bytes: it must really be a PDF, not an EXE or ZIP renamed to .pdf.
  2. Raw keyword scan. /#xx hex-escaped names are decoded first, so obfuscated
     names count too. Compressed object streams (/ObjStm) are decompressed and
     scanned as well.
  3. Structural walk with pypdf: document-level OpenAction and AA, named
     JavaScript, embedded files, page AA, and annotation actions.

Nothing in a file is executed. Prints one verdict line per file plus details for anything flagged.
"""
import os
import re
import sys
import zlib

from pypdf import PdfReader
from pypdf.generic import DictionaryObject, IndirectObject

RISKY = ["JS", "JavaScript", "Launch", "EmbeddedFile", "EmbeddedFiles", "RichMedia", "XFA",
         "SubmitForm", "ImportData", "GoToE", "GoToR"]
INFO = ["OpenAction", "AA", "AcroForm", "URI", "Encrypt", "ObjStm"]
NAME = re.compile(rb"/([A-Za-z0-9#]{1,40})")
HEX = re.compile(rb"#([0-9A-Fa-f]{2})")
OBJSTM = re.compile(rb"/Type\s*/ObjStm[^>]*?>>\s*stream\r?\n", re.S)


def decode_name(raw: bytes) -> str:
    return HEX.sub(lambda m: bytes([int(m.group(1), 16)]), raw).decode("latin-1")


def count_names(buf: bytes, counts: dict, obfuscated: set):
    for m in NAME.finditer(buf):
        raw = m.group(1)
        name = decode_name(raw) if b"#" in raw else raw.decode("latin-1")
        if name in counts:
            counts[name] += 1
            if b"#" in raw:
                obfuscated.add(name)


def raw_scan(path):
    counts = {k: 0 for k in RISKY + INFO}
    obf = set()
    with open(path, "rb") as f:
        data = f.read()
    head = data[:1024]
    magic_ok = b"%PDF" in head
    magic_note = "" if magic_ok else f"not a PDF: starts with {head[:8]!r}"
    count_names(data, counts, obf)
    # Decompress object streams, where names can hide from a plain byte scan.
    for m in OBJSTM.finditer(data):
        start = m.end()
        end = data.find(b"endstream", start)
        if end == -1:
            continue
        try:
            count_names(zlib.decompress(data[start:end]), counts, obf)
        except Exception:
            pass
    return magic_ok, magic_note, counts, obf


def action_types(obj, depth=0):
    """Collect the /S types of an action dictionary and of its /Next chain."""
    out = []
    if depth > 5:
        return out
    if isinstance(obj, IndirectObject):
        obj = obj.get_object()
    if isinstance(obj, DictionaryObject):
        s = obj.get("/S")
        if s is not None:
            out.append(str(s))
        nxt = obj.get("/Next")
        if nxt is not None:
            items = nxt if isinstance(nxt, list) else [nxt]
            for n in items:
                out += action_types(n, depth + 1)
    return out


def structural(path):
    findings = []
    r = PdfReader(path, strict=False)
    if r.is_encrypted:
        try:
            r.decrypt("")
        except Exception:
            findings.append("encrypted (owner password): structure partly hidden")
    root = r.trailer["/Root"].get_object()
    oa = root.get("/OpenAction")
    if oa is not None:
        t = action_types(oa)
        if any(x in ("/JavaScript", "/Launch", "/SubmitForm", "/ImportData", "/GoToE", "/GoToR") for x in t):
            findings.append(f"RISK OpenAction runs {t}")
    if root.get("/AA") is not None:
        findings.append("RISK document-level additional actions (/AA)")
    names = root.get("/Names")
    if names is not None:
        names = names.get_object()
        if names.get("/JavaScript") is not None:
            findings.append("RISK named JavaScript in /Names")
        if names.get("/EmbeddedFiles") is not None:
            findings.append("RISK embedded files in /Names")
    acro = root.get("/AcroForm")
    if acro is not None and acro.get_object().get("/XFA") is not None:
        findings.append("RISK XFA form")
    link_uri = 0
    for i, page in enumerate(r.pages):
        if page.get("/AA") is not None:
            findings.append(f"RISK page {i + 1} additional actions")
        annots = page.get("/Annots")
        if annots is None:
            continue
        for a in annots.get_object():
            a = a.get_object()
            sub = str(a.get("/Subtype"))
            if sub in ("/FileAttachment", "/RichMedia", "/Screen", "/Movie", "/Sound"):
                findings.append(f"RISK page {i + 1} annotation {sub}")
            act = a.get("/A")
            if act is not None:
                for t in action_types(act):
                    if t == "/URI":
                        link_uri += 1
                    elif t in ("/JavaScript", "/Launch", "/SubmitForm", "/ImportData", "/GoToE", "/GoToR"):
                        findings.append(f"RISK page {i + 1} link action {t}")
            if a.get("/AA") is not None:
                findings.append(f"RISK page {i + 1} annotation additional actions")
    return findings, link_uri, len(r.pages)


def main():
    files = []
    for d in sys.argv[1:]:
        for name in sorted(os.listdir(d)):
            if name.lower().endswith(".pdf"):
                files.append(os.path.join(d, name))
    flagged = 0
    for p in files:
        name = os.path.basename(p)
        try:
            ok, note, counts, obf = raw_scan(p)
            try:
                findings, uris, pages = structural(p)
            except Exception as e:
                findings, uris, pages = [f"structure walk failed: {type(e).__name__}: {e}"], 0, "?"
            risky_raw = {k: v for k, v in counts.items() if k in RISKY and v}
            bad = (not ok) or any(f.startswith("RISK") for f in findings) or bool(obf)
            verdict = "FLAG" if bad else ("CHECK" if risky_raw else "clean")
            if bad:
                flagged += 1
            info = {k: v for k, v in counts.items() if k in INFO and v}
            print(f"{verdict:5}  {name}  pages={pages}  links={uris}  info={info}  raw-risky={risky_raw}")
            if note:
                print("        " + note)
            if obf:
                print(f"        obfuscated names: {sorted(obf)}")
            for f in findings:
                print("        " + f)
        except Exception as e:
            flagged += 1
            print(f"ERROR  {name}  {type(e).__name__}: {e}")
    print(f"\n{len(files)} PDFs scanned, {flagged} flagged")


main()
