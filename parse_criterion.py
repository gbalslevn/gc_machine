#!/usr/bin/env python3
"""
parse_criterion.py

Parses Criterion benchmark output and produces a comparative overview table.

Usage:
    python parse_criterion.py                         # looks for ./target/criterion
    python parse_criterion.py --dir path/to/criterion # custom path
    python parse_criterion.py --format md             # markdown (default)
    python parse_criterion.py --format csv            # CSV
    python parse_criterion.py --format both           # markdown + CSV file
    python parse_criterion.py --unit ns               # ns | us | ms | auto (default)

Expected Criterion output structure:
    target/criterion/<benchmark name>/new/estimates.json
"""

import json
import os
import argparse
import csv
import sys
from pathlib import Path
from collections import defaultdict

# ── Known optimisation prefixes (order controls column order in output) ────────
OPTIMISATIONS = [
    "original",
    "free xor",
    "grr3",
    "point and permute",
    "half gates",
]

# ── Time unit helpers ──────────────────────────────────────────────────────────

UNITS = {
    "ns": (1,          "ns"),
    "us": (1_000,      "µs"),
    "ms": (1_000_000,  "ms"),
    "s":  (1_000_000_000, "s"),
}

def auto_unit(ns_value: float) -> tuple[float, str]:
    """Pick a human-friendly unit based on magnitude."""
    if ns_value >= 1_000_000_000:
        return ns_value / 1_000_000_000, "s"
    if ns_value >= 1_000_000:
        return ns_value / 1_000_000, "ms"
    if ns_value >= 1_000:
        return ns_value / 1_000, "µs"
    return ns_value, "ns"

def convert(ns_value: float, unit: str) -> tuple[float, str]:
    if unit == "auto":
        return auto_unit(ns_value)
    divisor, label = UNITS[unit]
    return ns_value / divisor, label

# ── Parsing ────────────────────────────────────────────────────────────────────

def load_estimates(criterion_dir: Path) -> dict:
    """
    Walk criterion_dir and return:
        { benchmark_name: { "mean_ns": float, "std_ns": float } }
    """
    results = {}
    for estimates_path in criterion_dir.rglob("new/estimates.json"):
        # Path structure: <criterion_dir>/<benchmark name>/new/estimates.json
        benchmark_name = estimates_path.parts[-3]  # folder two levels up
        try:
            with open(estimates_path) as f:
                data = json.load(f)
            mean_ns  = data["mean"]["point_estimate"]
            std_ns   = data["std_dev"]["point_estimate"]
            results[benchmark_name] = {"mean_ns": mean_ns, "std_ns": std_ns}
        except (KeyError, json.JSONDecodeError) as e:
            print(f"  Warning: could not parse {estimates_path}: {e}", file=sys.stderr)
    return results

def split_benchmark_name(name: str) -> tuple[str, str]:
    """
    Split e.g. "free xor gate garbling" → ("free xor", "gate garbling")
    Falls back to ("unknown", name) if no known prefix matches.
    """
    name_lower = name.lower()
    for opt in OPTIMISATIONS:
        if name_lower.startswith(opt):
            benchmark_type = name[len(opt):].strip()
            return opt, benchmark_type
    return "unknown", name

def organise(raw: dict) -> tuple[dict, list, list]:
    """
    Returns:
        table[benchmark_type][optimisation] = {"mean_ns", "std_ns"}
        sorted list of benchmark types
        sorted list of optimisations actually present
    """
    table = defaultdict(dict)
    opts_found = set()

    for name, stats in raw.items():
        opt, btype = split_benchmark_name(name)
        table[btype][opt] = stats
        opts_found.add(opt)

    # Keep column order: known optimisations first, then any unknowns alphabetically
    ordered_opts = [o for o in OPTIMISATIONS if o in opts_found]
    ordered_opts += sorted(opts_found - set(OPTIMISATIONS))

    ordered_btypes = sorted(table.keys())
    return table, ordered_btypes, ordered_opts

# ── Formatting ─────────────────────────────────────────────────────────────────

def format_cell(stats: dict | None, unit: str) -> tuple[str, str]:
    """Return (mean_str, std_str) formatted for display."""
    if stats is None:
        return "—", "—"
    mean, ulabel = convert(stats["mean_ns"], unit)
    std_ns_converted = stats["std_ns"] / (stats["mean_ns"] / mean) if mean != 0 else 0
    return f"{mean:>9.1f} {ulabel}", f"±{std_ns_converted:.1f}"

def print_markdown(table: dict, btypes: list, opts: list, unit: str):
    col_width = 22

    # Header
    header = f"{'Benchmark':<35}" + "".join(f"{o.title():>{col_width}}" for o in opts)
    print(header)
    print("-" * len(header))

    for btype in btypes:
        row_parts = [f"{btype:<35}"]
        for opt in opts:
            stats = table[btype].get(opt)
            mean_str, std_str = format_cell(stats, unit)
            cell = f"{mean_str} {std_str}"
            row_parts.append(f"{cell:>{col_width}}")
        print("".join(row_parts))

def write_csv(table: dict, btypes: list, opts: list, unit: str, path: Path):
    # Build header: benchmark, then mean+std per optimisation
    header = ["benchmark"]
    for opt in opts:
        header += [f"{opt} mean", f"{opt} std"]

    rows = []
    for btype in btypes:
        row = [btype]
        for opt in opts:
            stats = table[btype].get(opt)
            if stats is None:
                row += ["", ""]
            else:
                mean, ulabel = convert(stats["mean_ns"], unit)
                scale = stats["mean_ns"] / mean if mean != 0 else 1
                std = stats["std_ns"] / scale
                row += [f"{mean:.1f} {ulabel}", f"{std:.1f} {ulabel}"]
        rows.append(row)

    with open(path, "w", newline="") as f:
        writer = csv.writer(f)
        writer.writerow(header)
        writer.writerows(rows)

    print(f"\nCSV saved to: {path}")

# ── Speedup summary ────────────────────────────────────────────────────────────

def print_speedup_summary(table: dict, btypes: list, opts: list):
    """Print mean speedup of each optimisation vs 'original'."""
    if "original" not in opts:
        return

    print("\n── Speedup vs. Original (mean) ──")
    print(f"{'Benchmark':<35}", end="")
    other_opts = [o for o in opts if o != "original"]
    for o in other_opts:
        print(f"{o.title():>20}", end="")
    print()
    print("-" * (35 + 20 * len(other_opts)))

    for btype in btypes:
        baseline = table[btype].get("original")
        if baseline is None:
            continue
        print(f"{btype:<35}", end="")
        for opt in other_opts:
            stats = table[btype].get(opt)
            if stats is None:
                print(f"{'—':>20}", end="")
            else:
                speedup = baseline["mean_ns"] / stats["mean_ns"]
                marker = "↑" if speedup > 1 else "↓"
                print(f"{speedup:>18.2f}x {marker}", end="")
        print()

# Bench metrics helpers - Formatting memory usage and amount of protocol bytes 
def load_bench_metrics(criterion_dir: Path) -> dict:
    path = criterion_dir / "bench_metrics.json"
    if not path.exists():
        return {}
    with open(path) as f:
        return json.load(f)
    
def organise_metrics(raw_metrics: dict) -> dict:
    """Mirrors organise() but for metrics data."""
    table = defaultdict(dict)
    for name, metrics in raw_metrics.items():
        opt, btype = split_benchmark_name(name)
        table[btype][opt] = metrics
    return table

def format_bytes(n: int) -> str:
    if n >= 1_000_000:
        return f"{n/1_000_000:.1f} MB"
    if n >= 1_000:
        return f"{n/1_000:.1f} KB"
    return f"{n} B"

def format_instructions(n: int | None) -> str:
    if n is None:
        return "—"
    if n >= 1_000_000_000:
        return f"{n/1_000_000_000:.2f}B"
    if n >= 1_000_000:
        return f"{n/1_000_000:.2f}M"
    if n >= 1_000:
        return f"{n/1_000:.1f}K"
    return str(n)

def print_metrics_tables(metrics_table: dict, btypes: list, opts: list):
    if not metrics_table:
        return

    col_width = 18
    header = f"{'Benchmark':<35}" + "".join(f"{o.title():>{col_width}}" for o in opts)
    divider = "-" * len(header)

    BTYPE_LABELS = {
        "":          "AND + XOR",
        "- only AND": "only AND",
        "- only XOR": "only XOR",
    }

    for title, field, formatter in [
        ("── Protocol Bytes ──",            "protocol_bytes",         format_bytes),
        ("── Garble Memory Allocated ──",   "garble_bytes_allocated", format_bytes),
        ("── Eval Memory Allocated ──",     "eval_bytes_allocated",   format_bytes),
        ("── Garble Instructions ──",       "garble_instructions",    format_instructions),
        ("── Eval Instructions ──",         "eval_instructions",      format_instructions),
    ]:
        print(f"\n{title}")
        print(header)
        print(divider)
        for btype in btypes:
            label = BTYPE_LABELS.get(btype, btype)
            row = [f"{label:<35}"]
            for opt in opts:
                m = metrics_table.get(btype, {}).get(opt)
                val = formatter(m[field]) if m and m.get(field) is not None else "—"
                row.append(f"{val:>{col_width}}")
            print("".join(row))

# ── Entry point ────────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="Criterion benchmark comparison table")
    parser.add_argument("--unit",   choices=["auto", "ns", "us", "ms", "s"], default="auto",
                        help="Time unit (default: auto)")
    args = parser.parse_args()

    criterion_dir = Path("target/criterion")
    if not criterion_dir.exists():
        print(f"Error: directory not found: {criterion_dir}", file=sys.stderr)
        sys.exit(1)

    print(f"Scanning: {criterion_dir.resolve()}\n")
    raw = load_estimates(criterion_dir)
    if not raw:
        print("No estimates.json files found. Have you run `cargo bench` yet?", file=sys.stderr)
        sys.exit(1)

    print(f"Found {len(raw)} benchmarks.\n")
    table, btypes, opts = organise(raw)
    print_markdown(table, btypes, opts, args.unit)
    print_speedup_summary(table, btypes, opts)
    # write_csv(table, btypes, opts, args.unit, Path(args.csv_out))
    
    raw_metrics = load_bench_metrics(criterion_dir)
    if raw_metrics:
        metrics_table = organise_metrics(raw_metrics)
        metrics_btypes = sorted(metrics_table.keys())  
        print_metrics_tables(metrics_table, metrics_btypes, opts)

if __name__ == "__main__":
    main()