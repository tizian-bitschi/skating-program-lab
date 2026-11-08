"""Verify the complete observation matrix and summarize it using only stdlib."""
import csv
import hashlib
import statistics
import sys
import zipfile
from collections import Counter
from pathlib import Path

directory = Path(sys.argv[1])
rows = list(csv.DictReader((directory / "observations.csv").open(newline="")))
profiles = ["Bronze", "Silver", "Gold", "Masters", "Women", "Men"]
scenarios = ["empty", "required", "excluded", "impossible"]
phases = ["Domain", "Model", "Solve", "IncumbentFound", "Reconstruction",
          "MaximumProved", "Alternatives", "PostproofReconstruction"]
# Source-derived values and worked component bounds are explained in the report.
expected = {
    "Bronze": [1100, 1100, 1100, None],
    "Silver": [1430, 1420, 1430, None],
    "Gold": [2420, 2420, 2280, None],
    "Masters": [3340, 3280, 3340, None],
    "Women": [13310, 12020, 13310, None],
    "Men": [13310, 12020, 13310, None],
}
counts = Counter((r["profile"], r["scenario"], r["cancel_phase"]) for r in rows)
assert set(counts) == {
    (p, s, "") for p in profiles for s in scenarios
} | {(p, "empty", phase) for p in profiles for phase in phases}
assert set(counts.values()) == {3}, counts
assert len(rows) == 216
raw_path = directory / "raw-process-logs.zip"
raw = zipfile.ZipFile(raw_path) if raw_path.exists() else None
if raw:
    assert len(raw.namelist()) == 432
for row in rows:
    assert row["passed"] == "true" and row["exit_code"] == "0", row
    assert int(row["peak_working_set_bytes"]) > 0, row
    path = directory / f'{row["profile"]}-{row["scenario"]}.request.txt'
    assert hashlib.sha256(path.read_bytes()).hexdigest().upper() == row["request_sha256"], row
    if raw:
        label = f'{row["profile"]}-{row["scenario"]}-{row["cancel_phase"] or "continue"}-{row["run"]}'
        stdout = raw.read(f"{label}.stdout.txt").decode("utf-8-sig").splitlines()
        assert stdout[0] == "REQUEST\t" + path.read_text().rstrip("\n"), row
        metric = next(line[7:] for line in stdout if line.startswith("METRIC\t"))
        assert next(csv.reader([metric])) == list(row.values())[:15], row
        assert not raw.read(f"{label}.stderr.txt"), row
    phase = row["cancel_phase"]
    maximum = expected[row["profile"]][scenarios.index(row["scenario"])]
    if phase:
        assert row["cancel_response_us"], row
        if phase in ["MaximumProved", "Alternatives", "PostproofReconstruction"]:
            assert row["outcome"] == "ProvenMaximum", row
            assert row["alternative_status"] == "Cancelled", row
            assert int(row["maximum_hundredths"]) == expected[row["profile"]][0], row
            assert 1 <= int(row["alternatives"]) <= 5, row
        else:
            assert row["outcome"] == "Cancelled" and row["alternatives"] == "0", row
            assert not row["maximum_hundredths"], row
    elif maximum is None:
        assert row["outcome"] == "Infeasible", row
        assert not row["proof_us"] and not row["maximum_hundredths"], row
    else:
        assert row["outcome"] == "ProvenMaximum", row
        assert int(row["maximum_hundredths"]) == maximum, row
        assert int(row["alternatives"]) == 5 and row["alternative_status"] == "LimitReached", row
        assert 0 <= int(row["first_valid_us"]) <= int(row["proof_us"]) <= int(row["total_us"]), row
if raw:
    raw.close()


def median_ms(group, key):
    values = [int(r[key]) / 1000 for r in group if r[key]]
    return f"{statistics.median(values):.3f}" if values else "—"


lines = ["# Native Release observations", "",
         "All 216 processes passed: 72 primary requests and 144 cancellation requests.",
         "Three fresh-process repetitions per identical request; no watchdog failures.", "",
         "Times below are median milliseconds. Range is minimum–maximum total time.",
         "Memory is maximum lifetime peak working set across the three runs, in MiB.", "",
         "| Profile | Case | BV | Domain | Model | First valid | Proof | Alternatives | Total | Total range | Peak MiB |",
         "| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |"]
for profile in profiles:
    for scenario in scenarios:
        group = [r for r in rows if r["profile"] == profile and r["scenario"] == scenario and not r["cancel_phase"]]
        times = [int(r["total_us"]) / 1000 for r in group]
        memory = max(int(r["peak_working_set_bytes"]) for r in group) / 2**20
        value = group[0]["maximum_hundredths"] or "Infeasible"
        metrics = " | ".join(median_ms(group, key) for key in ["domain_us", "model_us", "first_valid_us", "proof_us", "alternatives_us", "total_us"])
        lines.append(f"| {profile} | {scenario} | {value} | {metrics} | {min(times):.3f}–{max(times):.3f} | {memory:.2f} |")
lines += ["", "## Cancellation response", "",
          "Trigger-to-return time in milliseconds over all profiles and repetitions.", "",
          "| Trigger | Median | Maximum | Worst profile |", "| --- | ---: | ---: | --- |"]
for phase in phases:
    group = [r for r in rows if r["cancel_phase"] == phase]
    worst = max(group, key=lambda r: int(r["cancel_response_us"]))
    lines.append(f'| {phase} | {median_ms(group, "cancel_response_us")} | {int(worst["cancel_response_us"]) / 1000:.3f} | {worst["profile"]} |')
(directory / "summary.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
print("Verified all 216 rows, 24 exact manifest hashes, independent expected values and cancellation outcomes.")
