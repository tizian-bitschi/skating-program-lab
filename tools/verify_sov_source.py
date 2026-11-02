"""Reproduce the independent June 2 numerical fixture from the official PDF.

Uses Poppler, independently of the PyMuPDF production-data transcription.
Reads no Rust catalog or production generator input. No runtime dependency.
Default mode checks the committed fixture; --write explicitly refreshes it.
"""

import argparse
import hashlib
from pathlib import Path
import re
import subprocess


ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "tests" / "fixtures"
EDITION = "isu-2786-2026-06-02"
PDF_SHA256 = "980ca3f689c280196f1fe4e32843e97010116c8b624eb7871c9f8850561613b7"
COUNTS = (55, 52, 54, 54, 50, 55, 55, 45, 45, 52, 54, 40, 14)


def source_fixture(pdf):
    if hashlib.sha256(pdf.read_bytes()).hexdigest() != PDF_SHA256:
        raise ValueError("Wrong PDF edition/hash; never substitute the May edition")
    codes = {page: [] for page in range(2, 15)}
    for line in (FIXTURES / f"{EDITION}-source-codes.txt").read_text(encoding="utf-8").splitlines():
        if line.startswith("#"):
            continue
        page, _, _, names = line.split("|")
        codes[int(page)].extend(names.split())
    if tuple(len(codes[p]) for p in range(2, 15)) != COUNTS:
        raise ValueError("Reference code inventory does not match printed page counts")
    result = subprocess.run(
        ["pdftotext", "-layout", str(pdf), "-"],
        check=True, capture_output=True, encoding="utf-8",
    )
    pages = result.stdout.split("\f")
    if len(pages) != 15 or pages[-1].strip():
        raise ValueError("Expected 14 printed PDF pages")
    output = [
        "# document=ISU Communication 2786",
        "# version=2026-06-02",
        f"# sha256={PDF_SHA256}",
        "# Independent Poppler extraction; graphic-only 3A labels use the reviewed code inventory.",
        "table\tpage\trow\tcode\t-5\t-4\t-3\t-2\t-1\tBASE\t+1\t+2\t+3\t+4\t+5",
    ]
    for page in range(2, 15):
        rows = []
        for line in pages[page - 1].splitlines():
            cells = list(re.finditer(r"(?<![\w,])-?\d+,\d{2}(?!\d)", line))
            if len(cells) == 11:
                rows.append((line[:cells[0].start()].strip(), [c.group() for c in cells]))
        if len(rows) != COUNTS[page - 2]:
            raise ValueError(f"Page {page}: missing/extra numeric row or GOE cell")
        for ordinal, (code, (label, values)) in enumerate(zip(codes[page], rows), 1):
            if label != code and not (
                not label and page == 4 and code in {"3A", "3Ab", "3Aq", "3Aqb"}
            ):
                raise ValueError(f"Page {page} row {ordinal}: code label {label!r} != {code!r}")
            # Preserve printed cells, not a percentage or rounding reconstruction.
            numeric = [str(int(value.replace(",", ""))) for value in values]
            output.append("\t".join([EDITION, str(page), str(ordinal), code] + numeric))
    return "\n".join(output) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pdf", type=Path)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    fixture = source_fixture(args.pdf)
    destination = FIXTURES / f"{EDITION}-source-values.tsv"
    if args.write:
        destination.write_text(fixture, encoding="utf-8")
    elif destination.read_text(encoding="utf-8") != fixture:
        raise ValueError("Committed fixture differs from the independently extracted PDF cells")
    print("Verified 625 rows / 6,875 printed numeric cells against the pinned June 2 PDF")


if __name__ == "__main__":
    main()
