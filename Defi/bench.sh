#!/bin/bash

# Configuration
DATASET=${1:-"data/j60.sm"}
SOLUTIONS="data/j60hrs.sm"

echo "=================================================="
echo "🚀 RCPSP High-Performance Solver Benchmark"
echo "=================================================="

if ! command -v cargo &> /dev/null
then
    echo "❌ Error: 'cargo' could not be found. Please install Rust."
    exit 1
fi

echo "📦 Building project in release mode..."
cargo build --release

echo "🏃 Running solver on j60 instances..."
./target/release/super_solver --dataset "$DATASET" --solutions "$SOLUTIONS"

echo ""
echo "✅ Benchmarking complete. Results saved in 'results/resultats_finaux.txt'."
echo "=================================================="
