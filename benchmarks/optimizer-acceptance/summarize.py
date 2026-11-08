"""Preserve native observations and extract a compact CSV (Python standard library)."""
import csv
import re
import sys
import zipfile
from pathlib import Path


def duration(value):
    if value == "None":
        return "NA"
    match = re.fullmatch(r"(?:Some\()?([\d.]+)(ns|µs|ms|s)\)?", value)
    if not match:
        raise ValueError(value)
    return round(float(match[1]) * {"ns": 0.001, "µs": 1, "ms": 1000, "s": 1000000}[match[2]], 3)


def rows(root):
    for side in ("before", "after"):
        for path in sorted(root.glob(f"{side}-[0-9]*.txt")):
            text = path.read_text(encoding="utf-8-sig")
            assert "test result: ok. 1 passed" in text, path
            samples = re.split(r"(?=^MEASURE )", text, flags=re.M)[1:]
            assert len(samples) == 16, path
            for sample in samples:
                line = sample.splitlines()[0]
                label, call = re.match(r"MEASURE (\S+) call_us=(\d+)", line).groups()
                field = lambda key: re.search(rf"^    {key}: (.*),$", sample, re.M)[1]
                values = re.findall(r"^            (?:total_)?base_hundredths: (\d+),$", sample, re.M)
                accepted = int(re.search(r" accepted: (\d+)", line)[1])
                proposals = int(field("examined_programs"))
                assert accepted <= proposals
                result = dict(side=side, run=path.stem.split("-")[-1], workload=label,
                              call_us=int(call), preflight_us=duration(re.search(r"preflight: (.*?), model:", line)[1]),
                              model_us=duration(re.search(r"model: (.*?), solver:", line)[1]),
                              solver_us=duration(re.search(r"solver: (.*?), solver_calls:", line)[1]),
                              first_verified_search_us=duration(re.search(r"first_accepted: (.*?), rejected:", line)[1]),
                              checked_proposals=proposals, accepted_proposals=accepted,
                              status=field("status"), termination=field("termination_reason"),
                              enumeration_complete=field("enumeration_complete"),
                              returned_ties=len(values), best_bv=max(map(int, values)) if values else "NA",
                              optimum_us="NA", internal_nodes="NA", memory_bytes="NA", improvement_sequence="NA",
                              diagnostics=line.split("Diagnostics ", 1)[1])
                yield result


def main():
    root, destination = map(Path, sys.argv[1:])
    destination.mkdir(parents=True, exist_ok=True)
    data = list(rows(root))
    with (destination / "observations.csv").open("w", newline="", encoding="utf-8") as file:
        writer = csv.DictWriter(file, fieldnames=list(data[0]))
        writer.writeheader()
        writer.writerows(data)
    with zipfile.ZipFile(destination / "native-observations.zip", "w", zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for path in sorted(root.glob("*.txt")):
            archive.write(path, path.name)
    for label in dict.fromkeys(row["workload"] for row in data):
        parts = []
        for side in ("before", "after"):
            selected = [r for r in data if r["side"] == side and r["workload"] == label]
            first = [r["first_verified_search_us"] / 1000 for r in selected if r["first_verified_search_us"] != "NA"]
            proposals = [r["checked_proposals"] for r in selected]
            parts.append(f"{len(first)}/{len(selected)}; {min(first):.1f}–{max(first):.1f} ms" if first else "0/5; NA")
            parts[-1] += f"; {min(proposals)}–{max(proposals)} proposals"
        print(label, " | ".join(parts))


if __name__ == "__main__":
    main()
