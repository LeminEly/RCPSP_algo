#!/usr/bin/env python3
import os
import sys
import subprocess
import shutil
import tempfile

# Configuration
DATASET_DIR = "data/j60.sm"
SOLUTIONS_FILE = "data/j60hrs.sm"
SOLVER_BIN = "./target/release/super_solver"
RESULTS_DIR = "results"

def parse_open_instances(solutions_path):
    """
    Parses the solutions file and identifies instances where LB < UB.
    Expected format: Group Instance ... LB UB
    """
    if not os.path.exists(solutions_path):
        print(f"❌ Error: '{solutions_path}' is missing.")
        sys.exit(1)

    open_instances = []
    try:
        with open(solutions_path, "r") as f:
            for line in f:
                line = line.strip()
                if not line or line.startswith("*") or line.startswith("#"):
                    continue
                
                parts = line.split()
                # PSPLIB hrs format usually has at least 4-5 columns:
                # Group Instance [some_vals] LB UB
                if len(parts) >= 4:
                    try:
                        g = parts[0]
                        i = parts[1]
                        # Assume last column is UB and second to last is LB
                        lb = int(parts[-2])
                        ub = int(parts[-1])
                        
                        if lb < ub:
                            # Construct the instance name used in the dataset directory
                            # Example: Group 1, Instance 1 -> j601_1
                            instance_name = f"j60{g}_{i}"
                            open_instances.append(instance_name)
                    except (ValueError, IndexError):
                        continue
    except Exception as e:
        print(f"❌ Error reading '{solutions_path}': {e}")
        sys.exit(1)

    return open_instances

def main():
    print("==================================================")
    print("🚀 RCPSP Filtered Benchmarking (Challenge Mode)")
    print("==================================================")

    # 1. Parse j60hrs.sm
    print(f"🔍 Parsing {SOLUTIONS_FILE}...")
    open_instance_names = parse_open_instances(SOLUTIONS_FILE)

    # 2. Extract and check
    total_open = len(open_instance_names)
    if total_open == 0:
        print(f"❌ Error: No open instances (LB < UB) found in {SOLUTIONS_FILE}.")
        sys.exit(1)

    print(f"✅ Total number of open instances: {total_open}")
    print("--------------------------------------------------")

    # 3. Build the solver
    print("📦 Building solver in release mode...")
    try:
        subprocess.run(["cargo", "build", "--release"], check=True, capture_output=True)
    except subprocess.CalledProcessError as e:
        print(f"❌ Error: Failed to build solver.\n{e.stderr.decode()}")
        sys.exit(1)

    if not os.path.exists(SOLVER_BIN):
        print(f"❌ Error: Solver binary not found at {SOLVER_BIN}")
        sys.exit(1)

    # 4. Filter and prepare execution
    # We create a temporary base directory and a nested 'j60.sm' directory
    # to ensure the Rust solver correctly detects the dataset name as 'j60'.
    with tempfile.TemporaryDirectory() as base_tmp:
        tmp_dir = os.path.join(base_tmp, "j60.sm")
        os.makedirs(tmp_dir)
        
        linked_count = 0
        for name in open_instance_names:
            src_file = os.path.join(DATASET_DIR, f"{name}.sm")
            if os.path.exists(src_file):
                os.symlink(os.path.abspath(src_file), os.path.join(tmp_dir, f"{name}.sm"))
                linked_count += 1
            else:
                # Some instances in the solutions file might not be in the dataset dir
                # (e.g., j60 vs j120) - we only care about j60 here.
                continue

        if linked_count == 0:
            print(f"❌ Error: None of the {total_open} open instances were found in {DATASET_DIR}.")
            sys.exit(1)

        print(f"🏃 Executing solver on {linked_count} instances...")
        # (Optional: Print names if requested or if count is small)
        for name in open_instance_names:
            if os.path.exists(os.path.join(tmp_dir, f"{name}.sm")):
                print(f"   ▶ Running: {name}")

        # 5. Run the solver on the filtered directory
        try:
            cmd = [
                SOLVER_BIN,
                "--dataset", tmp_dir,
                "--solutions", SOLUTIONS_FILE
            ]
            subprocess.run(cmd, check=True)
        except subprocess.CalledProcessError as e:
            print(f"\n❌ Error during solver execution: {e}")
            sys.exit(1)

    print("\n==================================================")
    print("✅ Challenge benchmarking complete!")
    print(f"Results saved in '{RESULTS_DIR}/resultats_finaux.txt'.")
    print("==================================================")

if __name__ == "__main__":
    main()
