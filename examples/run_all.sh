#!/bin/bash
# Run all ZKPlex examples

set -e  # Exit on error

echo ""
echo "========================================"
echo "   ZKPlex CLI Examples"
echo "========================================"
echo ""

# Check if zkplex CLI exists
if [ ! -f "./zkplex" ]; then
    echo "Error: ./zkplex not found."
    echo "Please run 'make cli-build' first to build the CLI."
    exit 1
fi

# Get CLI version
echo "zkplex version:"
./zkplex --version
echo ""

# Run examples
echo ""
echo "Running examples..."
echo ""

# Example 1: Age Verification
if [ -f "./examples/age_verification.sh" ]; then
    bash ./examples/age_verification.sh
    echo ""
    read -p "Press Enter to continue to next example..."
    echo ""
fi

# Example 2: Threshold Check
if [ -f "./examples/threshold_check.sh" ]; then
    bash ./examples/threshold_check.sh
    echo ""
    read -p "Press Enter to continue to next example..."
    echo ""
fi

# Example 3: Preprocessing
if [ -f "./examples/preprocessing.sh" ]; then
    bash ./examples/preprocessing.sh
    echo ""
    read -p "Press Enter to continue to next example..."
    echo ""
fi

# Example 4: Zircon Format
if [ -f "./examples/zircon_format.sh" ]; then
    bash ./examples/zircon_format.sh
    echo ""
fi

echo ""
echo "========================================"
echo "   All examples completed!"
echo "========================================"
echo ""
echo "Learn more:"
echo "  - Documentation: docs/"
echo "  - Zircon format: docs/zircon/SYNTAX.md"
echo "  - CLI reference: docs/CLI.md"
echo ""