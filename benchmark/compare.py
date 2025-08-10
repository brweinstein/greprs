#!/usr/bin/env python3
import argparse
import subprocess
import time
import psutil
import random
import string
import tempfile
import statistics
import shutil
import sys
from pathlib import Path

class BenchmarkResult:
    def __init__(self, tool, iteration, elapsed, peak_mem, matches):
        self.tool = tool
        self.iteration = iteration
        self.elapsed = elapsed
        self.peak_mem = peak_mem
        self.matches = matches

def generate_test_files(directory: Path, num_files: int, lines_per_file: int, pattern_frequency: float):
    pattern = "TEST_PATTERN"
    for i in range(num_files):
        fp = directory / f"test_{i}.txt"
        with fp.open("w") as f:
            for _ in range(lines_per_file):
                if random.random() < pattern_frequency:
                    f.write(f"Some {pattern} in random text\n")
                else:
                    f.write("".join(random.choices(string.ascii_letters + " ", k=50)) + "\n")

    # Validation
    files = list(directory.glob("test_*.txt"))
    if len(files) != num_files:
        raise RuntimeError(f"Expected {num_files} files, found {len(files)}")
    total_lines = sum(fp.open().read().count("\n") for fp in files)
    expected = num_files * lines_per_file
    if total_lines != expected:
        raise RuntimeError(f"Expected {expected} lines total, found {total_lines}")

def run_grep_command(cmd_base, pattern, target, iteration, tool_name):
    cmd = cmd_base + [pattern, str(target)]
    start = time.time()
    try:
        proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    except FileNotFoundError:
        print(f"ERROR: `{cmd[0]}` not found. Point `--greprs-bin` at your binary or install it on PATH.")
        sys.exit(1)

    proc_ps = psutil.Process(proc.pid)
    peak_mem = 0.0
    while proc.poll() is None:
        try:
            rss_mb = proc_ps.memory_info().rss / 1024**2
            peak_mem = max(peak_mem, rss_mb)
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            break
        time.sleep(0.005)

    out, _ = proc.communicate()
    elapsed = time.time() - start
    matches = len(out.splitlines())
    return BenchmarkResult(tool_name, iteration, elapsed, peak_mem, matches)

def print_iteration_table(results):
    header = f"{'Tool':<8} {'Iter':>4} {'Time (s)':>10} {'Mem (MB)':>10} {'Matches':>8}"
    print(header)
    print("-" * len(header))
    for r in results:
        print(f"{r.tool:<8} {r.iteration:>4} {r.elapsed:>10.4f} {r.peak_mem:>10.1f} {r.matches:>8}")

def print_summary(grep_res, greprs_res):
    def stats(data, attr):
        arr = [getattr(d, attr) for d in data]
        return statistics.mean(arr), statistics.stdev(arr) if len(arr) > 1 else 0.0

    print("\n=== SUMMARY ===")
    tools = [("grep", grep_res), ("greprs", greprs_res)]
    print(f"{'Tool':<8} {'Avg Time (s)':>12} {'σ Time':>8} {'Avg Mem':>10} {'σ Mem':>8} {'Matches':>8}")
    print("-" * 60)
    for name, data in tools:
        tmean, tstd = stats(data, "elapsed")
        mmean, mstd = stats(data, "peak_mem")
        matches = data[0].matches
        print(f"{name:<8} {tmean:12.4f} {tstd:8.4f} {mmean:10.1f} {mstd:8.1f} {matches:8}")

    t_g, _ = stats(grep_res, "elapsed")
    t_r, _ = stats(greprs_res, "elapsed")
    m_g, _ = stats(grep_res, "peak_mem")
    m_r, _ = stats(greprs_res, "peak_mem")

    time_diff = (t_r / t_g - 1) * 100
    mem_diff = (m_r / m_g - 1) * 100

    print("\nRelative to grep:")
    print(f"  Time:   {'faster' if time_diff < 0 else 'slower'} by {abs(time_diff):.1f}%")
    print(f"  Memory: {'less' if mem_diff < 0 else 'more'} by {abs(mem_diff):.1f}%")

    if grep_res[0].matches != greprs_res[0].matches:
        print("\nWARNING: Match counts differ!")
        print(f"  grep   : {grep_res[0].matches}")
        print(f"  greprs : {greprs_res[0].matches}")

def main():
    parser = argparse.ArgumentParser(description="Benchmark `grep` vs your `greprs`")
    parser.add_argument("--files",      type=int,   default=100,   help="Number of test files")
    parser.add_argument("--lines",      type=int,   default=5000,  help="Lines per file")
    parser.add_argument("--freq",       type=float, default=0.01,  help="Pattern frequency (0–1)")
    parser.add_argument("--iterations", type=int,   default=5,     help="Number of benchmark runs")
    parser.add_argument("--pattern",    type=str,   default="TEST_PATTERN", help="Search pattern")
    parser.add_argument("--greprs-bin", type=str,   default="greprs",
                        help="Path to your `greprs` binary (or rely on PATH)")
    parser.add_argument("--workload",   type=str,   default="medium", 
                        choices=["small", "medium", "large"],
                        help="Workload size: small (25 files, 1K lines), medium (100 files, 5K lines), large (500 files, 10K lines)")
    args = parser.parse_args()
    
    # Adjust parameters based on workload
    if args.workload == "small":
        if args.files == 100:  # Only change if using defaults
            args.files = 25
        if args.lines == 5000:
            args.lines = 1000
    elif args.workload == "large":
        if args.files == 100:
            args.files = 500
        if args.lines == 5000:
            args.lines = 10000

    # Verify greprs binary exists (either as path or on $PATH)
    if not shutil.which(args.greprs_bin):
        print(f"ERROR: `greprs` binary not found at `{args.greprs_bin}` and not in PATH.")
        sys.exit(1)

    print("Generating test files...")
    with tempfile.TemporaryDirectory() as td:
        test_dir = Path(td)
        generate_test_files(test_dir, args.files, args.lines, args.freq)

        print(f"\nTest directory : {test_dir}")
        print(f"Files          : {args.files}")
        print(f"Lines per file : {args.lines}")
        print(f"Total lines    : {args.files * args.lines:,}")
        print(f"Pattern freq   : {args.freq * 100:.2f}%\n")

        grep_results = []
        greprs_results = []
        grep_cmd  = ["grep", "-r"]
        greprs_cmd = [args.greprs_bin, "-r"]

        print(f"Running {args.iterations} iterations...\n")
        for i in range(1, args.iterations + 1):
            g = run_grep_command(grep_cmd, args.pattern, test_dir, i, "grep")
            r = run_grep_command(greprs_cmd, args.pattern, test_dir, i, "greprs")
            grep_results.append(g)
            greprs_results.append(r)

            print(f"Iteration {i}:")
            print_iteration_table([g, r])
            print("")

        print_summary(grep_results, greprs_results)

if __name__ == "__main__":
    main()