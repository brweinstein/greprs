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

def generate_test_files(directory: Path, num_files: int, lines_per_file: int, pattern_frequency: float, 
                       include_binary: bool = False, include_subdirs: bool = True):
    """Generate test files with various formats and structures."""
    pattern = "TEST_PATTERN"
    
    # Create subdirectories if requested
    subdirs = [directory]
    if include_subdirs and num_files > 10:
        subdir1 = directory / "subdir1"
        subdir2 = directory / "subdir2" 
        subdir1.mkdir()
        subdir2.mkdir()
        subdirs.extend([subdir1, subdir2])
    
    files_per_dir = num_files // len(subdirs)
    file_extensions = [".txt", ".log", ".rs", ".py", ".md"]
    
    total_files = 0
    for i, base_dir in enumerate(subdirs):
        start_idx = i * files_per_dir
        end_idx = start_idx + files_per_dir
        if i == len(subdirs) - 1:  # Last directory gets remaining files
            end_idx = num_files
            
        for j in range(start_idx, end_idx):
            ext = random.choice(file_extensions)
            fp = base_dir / f"test_{j}{ext}"
            
            with fp.open("w") as f:
                for line_num in range(lines_per_file):
                    if random.random() < pattern_frequency:
                        contexts = [
                            f"Error: {pattern} occurred at line {line_num}",
                            f"Found {pattern} in processing",
                            f"DEBUG: {pattern} validation successful",
                            f"Warning: {pattern} deprecated",
                        ]
                        f.write(random.choice(contexts) + "\n")
                    else:
                        # Generate realistic-looking log/code content
                        content_types = [
                            f"INFO: Processing item {line_num} completed successfully",
                            f"DEBUG: Function call_handler() returned status=OK",
                            f"WARN: Cache miss for key 'item_{random.randint(1000, 9999)}'",
                            f"// TODO: Implement better error handling here",
                            f"let result = process_data(input_{line_num});",
                            "".join(random.choices(string.ascii_letters + " \t", k=random.randint(20, 80))),
                        ]
                        f.write(random.choice(content_types) + "\n")
            total_files += 1
            
        # Add some binary files if requested
        if include_binary and i == 0:
            binary_file = base_dir / f"binary_{i}.bin"
            with binary_file.open("wb") as f:
                # Write some binary data with occasional text
                for _ in range(100):
                    if random.random() < 0.1:
                        f.write(f"TEXT_{pattern}_HERE\n".encode())
                    else:
                        f.write(bytes(random.randint(0, 255) for _ in range(50)))

    # Validation
    all_files = list(directory.rglob("test_*"))
    if len(all_files) != total_files:
        raise RuntimeError(f"Expected {total_files} files, found {len(all_files)}")

def run_grep_command(cmd_base, pattern, target, iteration, tool_name, extra_args=None):
    """Run grep command with optional extra arguments for feature testing."""
    cmd = cmd_base.copy()
    if extra_args:
        cmd.extend(extra_args)
    cmd.extend([pattern, str(target)])
    
    start = time.time()
    try:
        proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
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

    out, err = proc.communicate()
    elapsed = time.time() - start
    matches = len(out.splitlines()) if out else 0
    return BenchmarkResult(tool_name, iteration, elapsed, peak_mem, matches)

def run_feature_tests(greprs_bin, test_dir):
    """Test various greprs features for compatibility."""
    print("\n=== FEATURE COMPATIBILITY TESTS ===")
    
    test_cases = [
        {
            "name": "Case insensitive search",
            "args": ["-i"],
            "pattern": "test_pattern",
            "description": "Should find TEST_PATTERN with -i flag"
        },
        {
            "name": "Line numbers",
            "args": ["-n"],
            "pattern": "TEST_PATTERN", 
            "description": "Should show line numbers"
        },
        {
            "name": "Count matches",
            "args": ["-c"],
            "pattern": "TEST_PATTERN",
            "description": "Should show count of matches per file"
        },
        {
            "name": "Files with matches",
            "args": ["-l"],
            "pattern": "TEST_PATTERN",
            "description": "Should list files containing matches"
        },
        {
            "name": "Only matching parts",
            "args": ["-o"],
            "pattern": "TEST_PATTERN",
            "description": "Should show only the matched text"
        },
        {
            "name": "Context lines",
            "args": ["-C", "2"],
            "pattern": "TEST_PATTERN",
            "description": "Should show 2 lines before and after matches"
        },
        {
            "name": "Include pattern",
            "args": ["--include=*.rs"],
            "pattern": "TEST_PATTERN",
            "description": "Should only search .rs files"
        },
        {
            "name": "Exclude pattern", 
            "args": ["--exclude=*.log"],
            "pattern": "TEST_PATTERN",
            "description": "Should exclude .log files"
        }
    ]
    
    results = []
    for test_case in test_cases:
        print(f"\nTesting: {test_case['name']}")
        print(f"Description: {test_case['description']}")
        
        # Test with both grep and greprs
        for tool, cmd_base in [("grep", ["grep", "-r"]), ("greprs", [greprs_bin, "-r"])]:
            try:
                result = run_grep_command(
                    cmd_base, 
                    test_case["pattern"], 
                    test_dir, 
                    1, 
                    tool, 
                    test_case["args"]
                )
                print(f"  {tool:>6}: {result.matches:>4} matches, {result.elapsed:.4f}s")
                
                if tool == "greprs":
                    results.append((test_case["name"], result.matches))
                    
            except Exception as e:
                print(f"  {tool:>6}: ERROR - {e}")
    
    return results

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

    print("\n=== PERFORMANCE SUMMARY ===")
    tools = [("grep", grep_res), ("greprs", greprs_res)]
    print(f"{'Tool':<8} {'Avg Time (s)':>12} {'σ Time':>8} {'Avg Mem':>10} {'σ Mem':>8} {'Matches':>8}")
    print("-" * 64)
    for name, data in tools:
        tmean, tstd = stats(data, "elapsed")
        mmean, mstd = stats(data, "peak_mem")
        matches = data[0].matches if data else 0
        print(f"{name:<8} {tmean:12.4f} {tstd:8.4f} {mmean:10.1f} {mstd:8.1f} {matches:8}")

    if grep_res and greprs_res:
        t_g, _ = stats(grep_res, "elapsed")
        t_r, _ = stats(greprs_res, "elapsed")
        m_g, _ = stats(grep_res, "peak_mem")
        m_r, _ = stats(greprs_res, "peak_mem")

        time_diff = (t_r / t_g - 1) * 100 if t_g > 0 else 0
        mem_diff = (m_r / m_g - 1) * 100 if m_g > 0 else 0

        print(f"\nRelative to grep:")
        print(f"  Time:   {'faster' if time_diff < 0 else 'slower'} by {abs(time_diff):.1f}%")
        print(f"  Memory: {'less' if mem_diff < 0 else 'more'} by {abs(mem_diff):.1f}%")

        if grep_res[0].matches != greprs_res[0].matches:
            print(f"\nWARNING: Match counts differ!")
            print(f"  grep   : {grep_res[0].matches}")
            print(f"  greprs : {greprs_res[0].matches}")

def main():
    parser = argparse.ArgumentParser(description="Benchmark `grep` vs your `greprs`")
    parser.add_argument("--files",      type=int,   default=100,   help="Number of test files")
    parser.add_argument("--lines",      type=int,   default=5000,  help="Lines per file")
    parser.add_argument("--freq",       type=float, default=0.01,  help="Pattern frequency (0–1)")
    parser.add_argument("--iterations", type=int,   default=100,     help="Number of benchmark runs")
    parser.add_argument("--pattern",    type=str,   default="TEST_PATTERN", help="Search pattern")
    parser.add_argument("--greprs-bin", type=str,   default="greprs",
                        help="Path to your `greprs` binary (or rely on PATH)")
    parser.add_argument("--workload",   type=str,   default="medium", 
                        choices=["small", "medium", "large", "xlarge"],
                        help="Workload size")
    parser.add_argument("--test-features", action="store_true",
                        help="Run feature compatibility tests")
    parser.add_argument("--include-binary", action="store_true",
                        help="Include binary files in test data")
    args = parser.parse_args()
    
    # Adjust parameters based on workload
    workload_configs = {
        "small": (25, 1000),
        "medium": (100, 5000), 
        "large": (500, 10000),
        "xlarge": (1000, 20000)
    }
    
    if args.workload in workload_configs:
        if args.files == 100:  # Using default
            args.files = workload_configs[args.workload][0]
        if args.lines == 5000:  # Using default
            args.lines = workload_configs[args.workload][1]

    # Verify greprs binary exists
    if not shutil.which(args.greprs_bin):
        print(f"ERROR: `greprs` binary not found at `{args.greprs_bin}` and not in PATH.")
        sys.exit(1)

    print("Generating test files...")
    with tempfile.TemporaryDirectory() as td:
        test_dir = Path(td)
        generate_test_files(
            test_dir, 
            args.files, 
            args.lines, 
            args.freq,
            include_binary=args.include_binary,
            include_subdirs=True
        )

        print(f"\nTest directory : {test_dir}")
        print(f"Files          : {args.files}")
        print(f"Lines per file : {args.lines}")
        print(f"Total lines    : {args.files * args.lines:,}")
        print(f"Pattern freq   : {args.freq * 100:.2f}%")
        print(f"Workload       : {args.workload}")

        # Run feature tests if requested
        if args.test_features:
            feature_results = run_feature_tests(args.greprs_bin, test_dir)
            print(f"\nFeature tests completed: {len(feature_results)} tests run")

        # Run performance benchmarks
        print(f"\nRunning {args.iterations} performance iterations...\n")
        
        grep_results = []
        greprs_results = []
        grep_cmd = ["grep", "-r"]
        greprs_cmd = [args.greprs_bin, "-r"]

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