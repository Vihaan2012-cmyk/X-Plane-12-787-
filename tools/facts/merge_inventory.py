"""Merge the checked inventory files into docs/systems/computers.md and docs/systems/data-flows.md.

Usage: python -I merge_inventory.py <scratch_dir> <docs_dir> <suffix>
suffix: ".checked.md" (first pass) or ".v2.checked.md" (after the fix pass)
"""
import io
import os
import re
import sys

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
SCR, DOCS, SUF = sys.argv[1], sys.argv[2], sys.argv[3]
GROUPS = [("avionics", "Avionics and computing"), ("power", "Electrical, engines, APU, fuel"),
          ("controls", "Hydraulics, landing gear, flight controls"),
          ("environment", "Environmental, ice and rain, fire, cabin, lights, doors, cargo")]


def sections(text):
    """Split markdown into (heading, body) pairs at '## ' headings."""
    parts = re.split(r"(?m)^## +", text)
    out = []
    for p in parts[1:]:
        head, _, body = p.partition("\n")
        out.append((head.strip(), body.strip()))
    return out


def pick(secs, *keys):
    return [(h, b) for h, b in secs if any(k in h.lower() for k in keys)]


def count(body, status):
    return len(re.findall(rf"\|\s*{status}\s*\|\s*$", body, re.M))


comp = ["# Every computer on the 787-9 (Trent 1000)", "",
        "Generated from the documents by reading agents, then checked automatically. **Check** column:",
        "- **OK:** the row's terms appear at its cited line.",
        "- **QUOTED:** a second pass found a verbatim quote at the citation, confirmed by script.",
        "- **NEAR:** a partial match; read it before relying on it.",
        "- **BAD:** the citation doesn't support the row; it needs fixing.",
        "",
        "Citations point at the text extracts in `refs/text/` (`stm` = System Technical Manual, `ataNN` / `ANN` = 147 Technic ATA book, `loc` = Locator, `lot_fcom` = LOT FCOM). The model for each box is [shared-computer.md](shared-computer.md).", ""]
flows = ["# Data flows (first ICD input)", "",
         "Every documented data movement between boxes. This seeds [shared-data.md](shared-data.md). The Check column has the same meaning as in [computers.md](computers.md).", ""]
RES = os.path.join(SCR, "resolved_conflicts.md")
if os.path.exists(RES):
    comp += [open(RES, encoding="utf-8").read().rstrip(), ""]
totals = {"OK": 0, "QUOTED": 0, "NEAR": 0, "BAD": 0, "NOCITE": 0}
for key, title in GROUPS:
    path = os.path.join(SCR, f"inv_{key}{SUF}")
    secs = sections(open(path, encoding="utf-8").read())
    comp += [f"## {title}", ""]
    for h, b in pick(secs, "boxes"):
        comp += [b, ""]
        for s in totals:
            totals[s] += count(b, s)
    for h, b in pick(secs, "hosted", "surfaces", "exterior lights", "ratings"):
        comp += [f"### {re.sub(r'^[0-9]+\\. *', '', h)}", "", b, ""]
    for h, b in pick(secs, "conflict", "gap"):
        comp += [f"### {re.sub(r'^[0-9]+\\. *', '', h)}", "", b, ""]
    flows += [f"## {title}", ""]
    for h, b in pick(secs, "data flows"):
        flows += [b, ""]

open(os.path.join(DOCS, "computers.md"), "w", encoding="utf-8").write("\n".join(comp))
open(os.path.join(DOCS, "data-flows.md"), "w", encoding="utf-8").write("\n".join(flows))
print("rows by check status (boxes only):", totals)
