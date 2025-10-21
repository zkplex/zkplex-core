#!/bin/bash
# Example: Zircon Format
# Compact blockchain-optimized format for ZKP programs

set -e  # Exit on error

# Use compiled CLI if available, otherwise use Docker wrapper
if [ -f "./target/release/zkplex-cli" ]; then
    ZKPLEX="./target/release/zkplex-cli"
else
    ZKPLEX="./zkplex"
fi
TEMP_DIR="./examples/temp"

echo "================================================"
echo "ZKPlex Example: Zircon Format"
echo "================================================"
echo ""
echo "Zircon is a compact format designed for blockchain storage:"
echo "  Format: <version>/<secret>/<public>/<preprocess>/<circuit>"
echo ""

# Check if zkplex CLI exists
if [ ! -f "$ZKPLEX" ]; then
    echo "Error: $ZKPLEX not found. Run 'make cli-build' first."
    exit 1
fi

# Create temp directory
mkdir -p "$TEMP_DIR"

echo "Example 1: Age Verification"
echo "---------------------------------------"
echo "Zircon: 1/age:25/result:?/-/age>=18"
echo ""

# Create Zircon file
cat > "$TEMP_DIR/age.zrc" << 'EOF'
1/age:25/result:?/-/age>=18
EOF

echo "Estimate:"
$ZKPLEX --zircon "$TEMP_DIR/age.zrc" --estimate

echo ""
echo "Generate proof:"
PROOF_OUTPUT=$($ZKPLEX --zircon "$TEMP_DIR/age.zrc" --prove)
echo "$PROOF_OUTPUT"

# Save proof to file for verification
echo "$PROOF_OUTPUT" > "$TEMP_DIR/proof1.json"

echo ""
echo "Verify proof:"
$ZKPLEX --verify --proof "$TEMP_DIR/proof1.json"

echo ""
echo ""
echo "Example 2: Complex Circuit with Output"
echo "---------------------------------------"
echo "Zircon: 1/A:10,B:20,C:2/threshold:50,result:?/-/(A+B)*C>threshold"
echo ""

# Create Zircon file with output signal
cat > "$TEMP_DIR/threshold.zrc" << 'EOF'
1/A:10,B:20,C:2/threshold:50,result:?/-/(A+B)*C>threshold
EOF

echo "Estimate:"
$ZKPLEX --zircon "$TEMP_DIR/threshold.zrc" --estimate

echo ""
echo "Generate proof:"
PROOF_OUTPUT=$($ZKPLEX --zircon "$TEMP_DIR/threshold.zrc" --prove)
echo "$PROOF_OUTPUT"

# Save proof to file for verification
echo "$PROOF_OUTPUT" > "$TEMP_DIR/proof2.json"

echo ""
echo "Verify proof:"
$ZKPLEX --verify --proof "$TEMP_DIR/proof2.json"

echo ""
echo ""
echo "Example 3: Hash Preprocessing"
echo "---------------------------------------"
echo "Zircon: 1/secret:hello:text/target:0x2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824:hex,result:?/hash<==sha256(secret)/hash==target"
echo ""

# Create Zircon file with SHA256 preprocessing
cat > "$TEMP_DIR/hash.zrc" << 'EOF'
1/secret:hello:text/target:0x2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824:hex,result:?/hash<==sha256(secret)/hash==target
EOF

echo "Estimate:"
$ZKPLEX --zircon "$TEMP_DIR/hash.zrc" --estimate

echo ""
echo "Generate proof:"
PROOF_OUTPUT=$($ZKPLEX --zircon "$TEMP_DIR/hash.zrc" --prove)
echo "$PROOF_OUTPUT"

# Save proof to file for verification
echo "$PROOF_OUTPUT" > "$TEMP_DIR/proof3.json"

echo ""
echo "Verify proof:"
$ZKPLEX --verify --proof "$TEMP_DIR/proof3.json"

# Cleanup
rm -rf "$TEMP_DIR"

echo ""
echo ""
echo "================================================"
echo "✓ All Zircon examples completed successfully!"
echo "================================================"
echo ""
echo "Zircon Format Benefits:"
echo "  - Compact: ~50-80 bytes vs 500+ bytes JSON"
echo "  - Blockchain-optimized: Designed for on-chain storage"
echo "  - Human-readable: Easy to parse and understand"
echo "  - Complete: Contains all data needed for proof generation"
echo ""
echo "Format specification: docs/zircon/SYNTAX.md"
echo ""