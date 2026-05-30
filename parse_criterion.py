#!/usr/bin/env python3
"""
parse_criterion.py — Parses Criterion benchmark output and prints a comparison table.

Usage:
    python parse_criterion.py               # looks for ./target/criterion
    python parse_criterion.py --unit ns     # ns | us | ms | s | auto (default)
"""

import json, csv, sys, argparse
from pathlib import Path
from collections import defaultdict

OPTIMISATIONS = ["original", "free xor", "grr3", "point and permute", "half gates"]

UNITS = {"ns": 1, "us": 1_000, "ms": 1_000_000, "s": 1_000_000_000}

BTYPE_LABELS = {"": "AND + XOR", "- only AND": "only AND", "- only XOR": "only XOR"}


# ── Helpers ────────────────────────────────────────────────────────────────────

def convert(ns: float, unit: str) -> tuple[float, str]:
    if unit != "auto":
        return ns / UNITS[unit], unit
    for label, divisor in [("s", 1e9), ("ms", 1e6), ("µs", 1e3)]:
        if ns >= divisor:
            return ns / divisor, label
    return ns, "ns"


def format_bytes(n: int) -> str:
    for suffix, div in [("MB", 1_000_000), ("KB", 1_000)]:
        if n >= div:
            return f"{n/div:.1f} {suffix}"
    return f"{n} B"


def format_instructions(n: int | None) -> str:
    if n is None:
        return "—"
    for suffix, div in [("B", 1e9), ("M", 1e6), ("K", 1e3)]:
        if n >= div:
            return f"{n/div:.2f}{suffix}"
    return str(n)


# ── Parsing ────────────────────────────────────────────────────────────────────

def split_name(name: str) -> tuple[str, str]:
    """Split 'free xor gate garbling' → ('free xor', 'gate garbling')."""
    lower = name.lower()
    for opt in OPTIMISATIONS:
        if lower.startswith(opt):
            return opt, name[len(opt):].strip()
    return "unknown", name


def organise(raw: dict) -> tuple[dict, list, list]:
    table = defaultdict(dict)
    opts_found = set()
    for name, stats in raw.items():
        opt, btype = split_name(name)
        table[btype][opt] = stats
        opts_found.add(opt)
    ordered_opts = [o for o in OPTIMISATIONS if o in opts_found]
    ordered_opts += sorted(opts_found - set(OPTIMISATIONS))
    return table, sorted(table), ordered_opts


def load_estimates(criterion_dir: Path) -> dict:
    results = {}
    for path in criterion_dir.rglob("new/estimates.json"):
        benchmark = path.parts[-3]
        try:
            data = json.loads(path.read_text())
            results[benchmark] = {
                "mean_ns": data["mean"]["point_estimate"],
                "std_ns":  data["std_dev"]["point_estimate"],
            }
        except (KeyError, json.JSONDecodeError) as e:
            print(f"  Warning: could not parse {path}: {e}", file=sys.stderr)
    return results


def load_metrics(criterion_dir: Path) -> dict:
    path = criterion_dir / "bench_metrics.json"
    return json.loads(path.read_text()) if path.exists() else {}


# ── Printing ───────────────────────────────────────────────────────────────────

def make_header(opts: list, col: int = 22) -> str:
    return f"{'Benchmark':<35}" + "".join(f"{o.title():>{col}}" for o in opts)


def print_timing(table: dict, btypes: list, opts: list, unit: str):
    print(header := make_header(opts))
    print("-" * len(header))
    for btype in btypes:
        row = [f"{btype:<35}"]
        for opt in opts:
            s = table[btype].get(opt)
            if s is None:
                cell = "—"
            else:
                mean, ulabel = convert(s["mean_ns"], unit)
                divisor = s["mean_ns"] / mean
                std = s["std_ns"] / divisor
                cell = f"{mean:>9.1f} {ulabel} ±{std:.1f}"
            row.append(f"{cell:>22}")
        print("".join(row))


def print_speedup(table: dict, btypes: list, opts: list):
    if "original" not in opts:
        return
    others = [o for o in opts if o != "original"]
    print("\n── Speedup vs. Original (mean) ──")
    print(f"{'Benchmark':<35}" + "".join(f"{o.title():>20}" for o in others))
    print("-" * (35 + 20 * len(others)))
    for btype in btypes:
        baseline = table[btype].get("original")
        if not baseline:
            continue
        print(f"{btype:<35}", end="")
        for opt in others:
            s = table[btype].get(opt)
            if s is None:
                print(f"{'—':>20}", end="")
            else:
                speedup = baseline["mean_ns"] / s["mean_ns"]
                print(f"{speedup:>18.2f}x {'↑' if speedup > 1 else '↓'}", end="")
        print()


def print_metrics(metrics_raw: dict, opts: list):
    table = defaultdict(dict)
    for name, m in metrics_raw.items():
        opt, btype = split_name(name)
        table[btype][opt] = m
    btypes = sorted(table)

    header  = make_header(opts)
    divider = "-" * len(header)

    for title, field, fmt in [
        ("── Protocol Bytes ──",           "protocol_bytes",         format_bytes),
        ("── Garble Memory Allocated ──",  "garble_bytes_allocated", format_bytes),
        ("── Eval Memory Allocated ──",    "eval_bytes_allocated",   format_bytes),
        ("── Garble Instructions ──",      "garble_instructions",    format_instructions),
        ("── Eval Instructions ──",        "eval_instructions",      format_instructions),
    ]:
        print(f"\n{title}\n{header}\n{divider}")
        for btype in btypes:
            label = BTYPE_LABELS.get(btype, btype)
            row = [f"{label:<35}"]
            for opt in opts:
                m = table.get(btype, {}).get(opt)
                val = fmt(m[field]) if m and m.get(field) is not None else "—"
                row.append(f"{val:>18}")
            print("".join(row))


# ── Entry point ────────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="Criterion benchmark comparison table")
    parser.add_argument("--unit", choices=["auto", "ns", "us", "ms", "s"], default="auto")
    args = parser.parse_args()

    criterion_dir = Path("target/criterion")
    if not criterion_dir.exists():
        sys.exit(f"Error: directory not found: {criterion_dir}")

    print(f"Scanning: {criterion_dir.resolve()}\n")
    raw = load_estimates(criterion_dir)
    if not raw:
        sys.exit("No estimates.json files found. Have you run `cargo bench` yet?")

    print(f"Found {len(raw)} benchmarks.\n")
    table, btypes, opts = organise(raw)
    print_timing(table, btypes, opts, args.unit)
    print_speedup(table, btypes, opts)

    metrics = load_metrics(criterion_dir)
    if metrics:
        print_metrics(metrics, opts)


if __name__ == "__main__":
    main()