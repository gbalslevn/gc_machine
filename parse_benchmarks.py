import json, csv, sys, argparse
from pathlib import Path
from collections import defaultdict

UNITS = {"ns": 1, "us": 1_000, "ms": 1_000_000, "s": 1_000_000_000}
OPTIMISATIONS = ["original", "point and permute", "grr3", "free xor", "half gates"]
BTYPE_LABELS = {"": "AND + XOR", "- only AND": "AND", "- only XOR": "XOR"}

# Gate-type suffixes (standard benchmarks)
GATE_SUFFIXES = {"- only AND", "- only XOR", ""}

# Conditional optimisation base names (without trailing underscore variant)
CONDITIONAL_OPTS = {"naive", "stacked"}
CONDITIONALS = ["naive", "stacked"]

# Branch-outcome suffixes used by naive/stacked conditional benchmarks
BRANCH_OUTCOMES = ["- winning", "- equal", "- loosing"]
# Human-readable labels for branch outcomes
BRANCH_LABELS = {
    "- winning": "Winning",
    "- equal":   "Equal",
    "- loosing": "Loosing",
    "":          "No branch",   # the bare naive/stacked entries (tiny base case)
}


# ── Helpers ────────────────────────────────────────────────────────────────────

def convert(ns: float) -> tuple[float, str]:
    for label, divisor in [("s", 1e9), ("ms", 1e6), ("µs", 1e3)]:
        if ns >= divisor:
            return ns / divisor, label
    return ns, "ns"


def format_bytes(n: int) -> str:
    for suffix, div in [("MB", 1_048_576), ("KB", 1_024)]:
        if n >= div:
            return f"{n/div:.2f} {suffix}"
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
    """
    Returns (optimisation_key, remainder_suffix).

    For standard benchmarks the remainder is a gate-type suffix such as
    '- only AND', '- only XOR', or '' (mixed).

    For conditional benchmarks the remainder is a branch-outcome suffix such as
    '- winning', '- equal', '- loosing', or '' (bare base entry).
    """
    lower = name.lower()
    for opt in OPTIMISATIONS:
        if lower.startswith(opt):
            return opt, name[len(opt):].strip()

    # Match conditional variants: 'naive_conditional' or 'stacked_conditional'
    # (with or without the '_conditional' suffix in the raw key).
    for cond in CONDITIONALS:
        # Accept both 'naive' and 'naive_conditional' as the base token.
        for token in (cond + "_conditional", cond):
            if lower.startswith(token):
                remainder = name[len(token):].strip()
                return cond, remainder

    return "unknown", name


def is_branch_outcome_suffix(suffix: str) -> bool:
    return suffix in BRANCH_OUTCOMES or suffix == ""


def organise(raw: dict):
    """
    Separates entries into:
      - table / opts:          standard garbling-optimisation benchmarks
      - cond_table / cond_opts: naive & stacked conditional benchmarks
    """
    table      = defaultdict(dict)   # gate_suffix  -> opt    -> stats
    cond_table = defaultdict(dict)   # branch_outcome -> cond_opt -> stats
    opts_found      = set()
    cond_opts_found = set()

    for name, stats in raw.items():
        opt, suffix = split_name(name)

        if opt in CONDITIONAL_OPTS:
            # suffix is a branch outcome (- winning / - equal / - loosing / "")
            cond_table[suffix][opt] = stats
            cond_opts_found.add(opt)
        elif opt != "unknown":
            table[suffix][opt] = stats
            opts_found.add(opt)

    ordered_opts = [o for o in OPTIMISATIONS if o in opts_found]
    ordered_opts += sorted(opts_found - set(OPTIMISATIONS))

    # Order branch outcomes canonically
    outcome_order = BRANCH_OUTCOMES + [""]
    ordered_outcomes = [o for o in outcome_order if o in cond_table]

    return (table, sorted(table), ordered_opts,
            cond_table, ordered_outcomes, sorted(cond_opts_found))


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


def print_timing(table: dict, btypes: list, opts: list):
    print(header := make_header(opts))
    print("-" * len(header))
    for btype in btypes:
        row = [f"{btype:<35}"]
        for opt in opts:
            s = table[btype].get(opt)
            if s is None:
                cell = "—"
            else:
                mean, ulabel = convert(s["mean_ns"])
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


def _print_metrics_section(title: str, field: str, fmt, table: dict,
                            btypes: list, opts: list,
                            row_label_fn=None, col: int = 22):
    header  = make_header(opts, col)
    divider = "-" * len(header)
    print(f"\n{title}\n{header}\n{divider}")
    for btype in btypes:
        if row_label_fn:
            label = row_label_fn(btype)
        else:
            label = BTYPE_LABELS.get(btype, btype)
        row = [f"{label:<35}"]
        for opt in opts:
            m   = table.get(btype, {}).get(opt)
            val = fmt(m[field]) if m and m.get(field) is not None else "—"
            row.append(f"{val:>{col}}")
        print("".join(row))


def print_metrics(metrics_raw: dict, opts: list):
    # ── Separate standard vs conditional entries ───────────────────────────────
    std_metrics  = {}
    cond_metrics = {}
    for name, m in metrics_raw.items():
        opt, suffix = split_name(name)
        if opt in CONDITIONAL_OPTS:
            cond_metrics[name] = (opt, suffix, m)
        elif opt != "unknown":
            std_metrics[name] = (opt, suffix, m)

    # ── Standard optimisation tables ──────────────────────────────────────────
    table = defaultdict(dict)
    for name, (opt, suffix, m) in std_metrics.items():
        table[suffix][opt] = m
    btypes = sorted(table)

    for title, field, fmt in [
        ("── Protocol Bytes ──",          "protocol_bytes",         format_bytes),
        ("── Garble Memory Allocated ──", "garble_bytes_allocated", format_bytes),
        ("── Eval Memory Allocated ──",   "eval_bytes_allocated",   format_bytes),
        ("── Garble Instructions ──",     "garble_instructions",    format_instructions),
        ("── Eval Instructions ──",       "eval_instructions",      format_instructions),
    ]:
        _print_metrics_section(title, field, fmt, table, btypes, opts)

    # ── Conditional benchmarks ────────────────────────────────────────────────
    if not cond_metrics:
        return

    # Rebuild as: branch_outcome -> cond_opt -> metrics
    cond_table     = defaultdict(dict)
    cond_opts_found = set()
    for name, (opt, suffix, m) in cond_metrics.items():
        cond_table[suffix][opt] = m
        cond_opts_found.add(opt)

    cond_opts     = sorted(cond_opts_found)
    outcome_order = BRANCH_OUTCOMES + [""]
    cond_btypes   = [o for o in outcome_order if o in cond_table]

    print("\n\n── Conditional Benchmark Metrics ──")
    for title, field, fmt in [
        ("── Protocol Bytes ──",          "protocol_bytes",         format_bytes),
        ("── Garble Memory Allocated ──", "garble_bytes_allocated", format_bytes),
        ("── Eval Memory Allocated ──",   "eval_bytes_allocated",   format_bytes),
        ("── Garble Instructions ──",     "garble_instructions",    format_instructions),
        ("── Eval Instructions ──",       "eval_instructions",      format_instructions),
    ]:
        _print_metrics_section(
            title, field, fmt,
            cond_table, cond_btypes, cond_opts,
            row_label_fn=lambda b: BRANCH_LABELS.get(b, b),
        )


# ── Entry point ────────────────────────────────────────────────────────────────

def main():
    criterion_dir = Path("target/criterion")
    if not criterion_dir.exists():
        sys.exit(f"Error: directory not found: {criterion_dir}")

    print(f"Scanning: {criterion_dir.resolve()}\n")
    raw = load_estimates(criterion_dir)
    if not raw:
        sys.exit("No estimates.json files found. Have you run `cargo bench` yet?")

    print(f"Found {len(raw)} benchmarks.\n")
    table, btypes, opts, cond_table, cond_btypes, cond_opts = organise(raw)

    # ── Standard benchmarks ───────────────────────────────────────────────────
    print_timing(table, btypes, opts)
    print_speedup(table, btypes, opts)

    # ── Conditional benchmarks (timing) ───────────────────────────────────────
    if cond_table:
        print("\n── Conditional benchmarks ──")
        # Use BRANCH_LABELS as row labels by temporarily remapping keys
        display_table = {
            BRANCH_LABELS.get(k, k): v for k, v in cond_table.items()
        }
        display_btypes = [BRANCH_LABELS.get(b, b) for b in cond_btypes]
        print_timing(display_table, display_btypes, cond_opts)

    # ── Metrics (if present) ──────────────────────────────────────────────────
    metrics = load_metrics(criterion_dir)
    if metrics:
        print_metrics(metrics, opts)


if __name__ == "__main__":
    main()