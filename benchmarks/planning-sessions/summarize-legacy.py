"""Validate retained #140 legacy observations and reproduce three-run medians."""
import csv
import hashlib
import itertools
import statistics
import sys
from pathlib import Path

root = Path(sys.argv[1])
rows = list(csv.DictReader((root / "observations.csv").open()))
profiles = ["Bronze", "Silver", "Gold", "Masters", "Women", "Men"]
scenarios = ["empty", "required", "excluded", "impossible"]
expected = set(itertools.product(["baseline", "current"], profiles, scenarios, ["1", "2", "3"]))
keys = [(r["edition"], r["profile"], r["scenario"], r["run"]) for r in rows]
assert len(keys) == len(set(keys)) == 144 and set(keys) == expected
maxima = {"Bronze": (1100, 1100, 1100), "Silver": (1480, 1470, 1480), "Gold": (2420, 2420, 2280), "Masters": (3340, 3280, 3340), "Women": (13310, 12020, 13310), "Men": (13310, 12020, 13310)}
for r in rows:
    assert r["passed"] == "true" and r["exit_code"] == "0"
    manifest = root / f"{r['profile']}-{r['scenario']}.request.txt"
    assert hashlib.sha256(manifest.read_bytes()).hexdigest().upper() == r["request_sha256"]
    if r["scenario"] == "impossible":
        assert r["outcome"] == "Infeasible" and r["alternatives"] == "0"
    else:
        assert r["outcome"] == "ProvenMaximum"
        assert int(r["maximum_hundredths"]) == maxima[r["profile"]][scenarios.index(r["scenario"])]
        neutral = r["edition"] == "current" and r["profile"] == "Bronze" and r["scenario"] == "required"
        assert (r["alternatives"], r["alternative_status"]) == (("4", "Exhausted") if neutral else ("5", "LimitReached"))
metrics = ["proof_us", "alternatives_us", "total_us"]
fields = ["profile", "scenario"] + [f"{prefix}_{m}" for m in metrics for prefix in ["baseline", "current", "ratio"]]
with (root / "summary.csv").open("w", newline="") as output:
    writer = csv.DictWriter(output, fieldnames=fields)
    writer.writeheader()
    for profile in sorted(profiles):
        for scenario in sorted(scenarios):
            result = {"profile": profile, "scenario": scenario}
            for metric in metrics:
                values = {}
                for edition in ["baseline", "current"]:
                    group = [r[metric] for r in rows if (r["profile"], r["scenario"], r["edition"]) == (profile, scenario, edition)]
                    assert len(group) == 3
                    values[edition] = statistics.median(int(x) for x in group) if all(group) else None
                    result[f"{edition}_{metric}"] = values[edition]
                if values["baseline"]:
                    result[f"ratio_{metric}"] = f"{values['current'] / values['baseline']:.3f}"
            writer.writerow(result)
print("Validated 144 observations, exact matrix, hashes, independent maxima and statuses; wrote summary.csv")
