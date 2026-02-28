#!/bin/bash

# Configuration
DATASET_PATH=${1:-"data/j60.sm"}
# Extract basename (e.g., j60.sm or j30)
DATASET_NAME=$(basename "$DATASET_PATH" .sm)
# Default solutions file (e.g., data/j60hrs.sm)
SOLUTIONS=${2:-"data/${DATASET_NAME}hrs.sm"}

echo "=================================================="
echo "🚀 RCPSP High-Performance Solver Benchmark"
echo "=================================================="

if ! command -v cargo &> /dev/null
then
    echo "❌ Error: 'cargo' could not be found. Please install Rust."
    exit 1
fi

if [[ "$DATASET_NAME" == "j60" ]]; then
    echo "💡 Tip: For the RCPSP Challenge (filtered J60), you can also use: ./bench_j60.py"
    echo ""
fi

echo "📦 Building project in release mode..."
cargo build --release

echo "🏃 Running solver on $DATASET_NAME instances..."
./target/release/super_solver --dataset "$DATASET_PATH" --solutions "$SOLUTIONS"

echo ""
echo "✅ Benchmarking complete. Results saved in 'results/resultats_finaux.txt'."
echo "=================================================="
