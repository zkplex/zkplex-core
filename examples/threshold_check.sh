#!/bin/bash
# Example: Threshold Check with Output Signal
# Prove that (A + B) * C > threshold and reveal the computed value

set -e  # Exit on error

# Use compiled CLI if available, otherwise use Docker wrapper
if [ -f "./target/release/zkplex-cli" ]; then
    ZKPLEX="./target/release/zkplex-cli"
else
    ZKPLEX="./zkplex"
fi
TEMP_DIR="./examples/temp"

echo "================================================"
echo "ZKPlex Example: Threshold Check"
echo "================================================"
echo ""
echo "Scenario: Prove (A + B) * C > threshold and reveal the computed value"
echo "  A = 10 (secret)"
echo "  B = 20 (secret)"
echo "  C = 2 (secret)"
echo "  threshold = 50 (public input)"
echo "  computed_value = 60 (public input - prover reveals this)"
echo "  check = ? (output signal - will be 1 if true)"
echo ""
echo "Expected: (10 + 20) * 2 = 60 > 50 ✓ → check = 1 (true)"
echo ""

# Check if zkplex CLI exists
if [ ! -f "$ZKPLEX" ]; then
    echo "Error: $ZKPLEX not found. Run 'make cli-build' first."
    exit 1
fi

# Create temp directory
mkdir -p "$TEMP_DIR"

echo "Step 1: Estimate circuit requirements"
echo "---------------------------------------"
$ZKPLEX \
  --zircon "1/A:10,B:20,C:2/computed_value:60,threshold:50,check:?/-/computed_value<==(A+B)*C;computed_value>threshold" \
  --estimate

echo ""
echo ""
echo "Step 2: Generate proof"
echo "---------------------------------------"
PROOF_OUTPUT=$($ZKPLEX \
  --zircon "1/A:10,B:20,C:2/computed_value:60,threshold:50,check:?/-/computed_value<==(A+B)*C;computed_value>threshold" \
  --prove)

echo "$PROOF_OUTPUT"

# Save proof to file
echo "$PROOF_OUTPUT" > "$TEMP_DIR/proof.json"

echo ""
echo ""
echo "Step 3: Verify proof"
echo "---------------------------------------"
$ZKPLEX --verify --proof "$TEMP_DIR/proof.json"

# Cleanup
rm -rf "$TEMP_DIR"

echo ""
echo ""
echo "================================================"
echo "✓ Example completed successfully!"
echo "================================================"
echo ""
echo "What happened:"
echo "  1. Circuit has 2 constraints:"
echo "     - computed_value == (A+B)*C (proves 60 is correct)"
echo "     - computed_value > threshold (proves 60 > 50)"
echo "  2. Proof generated with secrets A, B, C hidden"
echo "  3. Output signal 'check' = 1 (the comparison result)"
echo "  4. Proof verified successfully"
echo ""
echo "The verifier knows:"
echo "  - threshold = 50 (public input)"
echo "  - computed_value = 60 (public input - prover revealed)"
echo "  - check = 1 (output signal - comparison is true)"
echo "  - The constraints are satisfied"
echo ""
echo "The verifier does NOT know:"
echo "  - A = 10 (secret)"
echo "  - B = 20 (secret)"
echo "  - C = 2 (secret)"
echo "  - How the value 60 was computed from A, B, C"
echo ""