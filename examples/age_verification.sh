#!/bin/bash
# Example: Age Verification ZKP
# Prove that age >= 18 without revealing exact age

set -e  # Exit on error

# Use compiled CLI if available, otherwise use Docker wrapper
if [ -f "./target/release/zkplex-cli" ]; then
    ZKPLEX="./target/release/zkplex-cli"
else
    ZKPLEX="./zkplex"
fi
TEMP_DIR="./examples/temp"

echo "================================================"
echo "ZKPlex Example: Age Verification"
echo "================================================"
echo ""
echo "Scenario: Prove age >= 18 without revealing exact age (secret: 25)"
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
  --circuit "age >= 18" \
  --secret age:25 \
  --public "result:?" \
  --estimate

echo ""
echo ""
echo "Step 2: Generate proof"
echo "---------------------------------------"
PROOF_OUTPUT=$($ZKPLEX \
  --circuit "age >= 18" \
  --secret age:25 \
  --public "result:?" \
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
echo "  1. Circuit 'age >= 18' was analyzed and estimated"
echo "  2. Proof was generated with secret age=25"
echo "  3. Proof was verified without revealing the actual age"
echo ""
echo "The verifier only knows that age >= 18 is true,"
echo "but doesn't know the actual age (25)."
echo ""