"""Compare verified baseline/current primary requests without hiding regressions."""
import csv
import statistics
import sys
from pathlib import Path

baseline, current = map(Path, sys.argv[1:3])
groups = []
for directory in (baseline, current):
    rows = list(csv.DictReader((directory / "observations.csv").open(newline="")))
    assert len(rows) == 216 and all(r["passed"] == "true" for r in rows)
    groups.append({(p, s): [r for r in rows if (r["profile"], r["scenario"]) == (p, s) and not r["cancel_phase"]]
                   for p in ["Bronze", "Silver", "Gold", "Masters", "Women", "Men"]
                   for s in ["empty", "required", "excluded", "impossible"]})

lines = ["# Identical-request native Release comparison", "",
         "Three serial fresh processes per case. Times are medians in milliseconds; ratio is current/baseline.",
         "Proof includes validated representative reconstruction. Alternatives are measured separately.", "",
         "| Profile | Case | BV before→after | Proof before→after | Ratio | Alternatives before→after | Ratio |",
         "| --- | --- | --- | ---: | ---: | ---: | ---: |"]


def median(rows, key):
    values = [int(r[key]) / 1000 for r in rows if r[key]]
    return statistics.median(values) if values else None


def compare(a, b, key):
    old, new = median(a, key), median(b, key)
    if old is None or new is None:
        assert old is new
        return "— | —"
    return f"{old:.3f}→{new:.3f} | {new / old:.2f}×" if old else f"{old:.3f}→{new:.3f} | —"


for key, a in groups[0].items():
    b = groups[1][key]
    assert len(a) == len(b) == 3
    assert {r["request_sha256"] for r in a} == {r["request_sha256"] for r in b}, key
    p, s = key
    assert (baseline / f"{p}-{s}.request.txt").read_bytes() == (current / f"{p}-{s}.request.txt").read_bytes()
    lines.append(f'| {p} | {s} | {a[0]["maximum_hundredths"] or "Infeasible"}→{b[0]["maximum_hundredths"] or "Infeasible"} | '
                 + compare(a, b, "proof_us") + " | " + compare(a, b, "alternatives_us") + " |")
(current / "comparison.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
print("Verified all 24 identical request manifests and compared proof/alternatives separately.")
