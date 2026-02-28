# 🚀 RCPSP High-Performance Solver

[![Rust](https://img.shields.io/badge/rust-2021%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A state-of-the-art **High-Performance Solver** for the **Resource-Constrained Project Scheduling Problem (RCPSP)**, implemented in **Rust** for maximum speed and parallel efficiency.

---

## 🎯 Project Objective

The goal of this project is to provide a robust and extremely fast optimization engine capable of solving complex scheduling problems under resource constraints. 

By leveraging an **Islands Model Genetic Algorithm**, the solver explores multiple evolutionary paths in parallel, ensuring high-quality solutions for large-scale instances like `j60` from the PSPLIB dataset in a fraction of a second.

---

## ✨ Key Features

- **🦀 Pure Rust Implementation**: Zero overhead, memory-safe, and blazingly fast.
- **🏝️ Islands Model GA**: Parallel populations with scheduled migrations to prevent local optima stagnation.
- **⚡ Double Justification (DJ)**: Advanced schedule compaction heuristic for improved makespans.
- **🧬 Critical Path Crossover**: Specialized genetic operator that preserves bottleneck activity sequences.
- **🧵 Multi-Core Optimization**: Fully parallelized architecture using the `Rayon` data parallelism library.

---

## 🛠️ Installation Guide

Follow the instructions below for your specific operating system to get the solver up and running.

### 1. Install Rust (Required for all systems)

The easiest way to install Rust is via `rustup`. Open your terminal and run:

| Operating System | Command |
| :--- | :--- |
| **Linux / macOS** | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **Windows** | Download and run [rustup-init.exe](https://rustup.rs/) |

*After installation, restart your terminal or run `source $HOME/.cargo/env`.*

### 2. Build the Project

```bash
# Clone the repository
git clone https://github.com/LeminEly/RCPSP_algo.git
cd Defi

# Compile in release mode (Crucial for performance!)
cargo build --release
```

---

## 🏃 Running the Solver

We provide a convenient benchmark script to run the solver on the included datasets.

```bash
# Set execution permissions (Linux/macOS)
chmod +x bench.sh

# Run on the j60 dataset (Default)
./bench.sh

# Run on the j30 dataset
./bench.sh data/j30

# Advanced: Specify a custom dataset path and solutions file
./bench.sh data/my_dataset data/my_solutions.sm
```

> [!NOTE]
> The script automatically detects the dataset name and attempts to find the corresponding solutions file in the `data/` directory. Detailed logs are generated in `results/resultats_finaux.txt`.

---

## 📊 Performance & Results (j60 Dataset)

Our solver consistently achieves near-optimal results with extreme efficiency.

| Metric | Result |
| :--- | :--- |
| **Average solve time** | **~0.35 seconds** |
| **Dataset size** | **480 instances** |
| **Convergence rate** | **~76%** (Matched or beaten state-of-the-art) |

---

## 📂 Repository Structure

```text
.
├── src/                # 🦀 Rust Source code (Islands Model GA, scheduling)
├── data/               # 📂 Datasets (j30, j60.sm, etc.)
├── results/            # 📝 Benchmark output directory
├── Cargo.toml          # 📦 Dependencies (Rayon, Clap, Rand)
└── bench.sh            # 🏃 Dynamic benchmarking script
```

---

## 🤝 Contribution & License

This project was developed as part of the DEVCORE challenge.  
📜 Licensed under the **MIT License**.
