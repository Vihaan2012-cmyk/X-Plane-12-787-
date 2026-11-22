# Tools

Python runs with `python -I` (or `-E -P` when it needs user packages such as PyMuPDF). Input documents are untrusted, so they are always passed as arguments and never run from their own folder.

## `xtask/` (Rust)
- `cargo xtask layers`: checks the crate layering (systems → solvers → core).
- `cargo xtask check`: layering, then fmt, clippy (warnings are errors) and tests.

## `pdf/`
- `pdf_safety.py`, `pdf_js_where.py`, `peel_js.py`: static malware checks on incoming PDFs.
- `pdf_inventory.py`: page count, title and text hints for a folder of PDFs.
- `ocr_pdfs.py`: OCR scanned PDFs, page by page, with caching.
- `ocr_figures.py`: OCR only the pages with raster figures, to catch diagram labels.

## `facts/`
- `verify_facts.py`: checks fact-file quotes against the text extracts and re-anchors line numbers.
- `verify_cites.py`, `merge_inventory.py`, `apply_resolved.py`: the computer and data-flow inventory pipeline.
- `make_hits.sh` → sweep agents → `verify_sweep.py` → `merge_sweep.py`: the electrical sweep. Writes `refs/facts/elec_*.verified.md`.
- `sweep_breakers.py`: mechanical pass for every breaker ID (CE/CK/CT + 7 digits).

## `mockups/`
- `gen_fuel.py`, `gen_fuel_drawing.py`, `gen_meridional.py`: inspector mockup generators (`docs/mockups/`).
