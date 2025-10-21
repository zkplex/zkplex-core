#!/bin/bash
# Example: Preprocessing with Hash Functions
# Demonstrates using preprocessing to compute hashes before circuit evaluation

set -e  # Exit on error

# Use compiled CLI if available, otherwise use Docker wrapper
if [ -f "./target/release/zkplex-cli" ]; then
    ZKPLEX="./target/release/zkplex-cli"
else
    ZKPLEX="./zkplex"
fi
TEMP_DIR="./examples/temp"

echo "================================================"
echo "ZKPlex Example: Preprocessing"
echo "================================================"
echo ""
echo "Preprocessing allows computing values (like hashes) before circuit evaluation."
echo "This is useful for operations that would be expensive in ZK circuits."
echo ""

# Check if zkplex CLI exists
if [ ! -f "$ZKPLEX" ]; then
    echo "Error: $ZKPLEX not found. Run 'make cli-build' first."
    exit 1
fi

# Create temp directory
mkdir -p "$TEMP_DIR"

echo "Example 1: SHA256 Hash Comparison"
echo "---------------------------------------"
echo "Scenario: Prove you know a password that hashes to a target"
echo "  secret: 'hello' (hidden)"
echo "  target: 0x2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824 (public)"
echo ""
echo "Preprocessing: Compute SHA256 hash of secret"
echo "Circuit: Verify hash equals target"
echo ""

# Using CLI flags
echo "Using CLI flags:"
$ZKPLEX \
  --preprocess "hash<==sha256(secret)" \
  --circuit "hash == target" \
  --secret secret:hello:text \
  --public target:0x2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824:hex \
  --public "result:?" \
  --estimate

echo ""
echo "Generate proof:"
PROOF_OUTPUT=$($ZKPLEX \
  --preprocess "hash<==sha256(secret)" \
  --circuit "hash == target" \
  --secret secret:hello:text \
  --public target:0x2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824:hex \
  --public "result:?" \
  --prove)

echo "$PROOF_OUTPUT"
echo "$PROOF_OUTPUT" > "$TEMP_DIR/proof1.json"

echo ""
echo "Verify proof:"
$ZKPLEX --verify --proof "$TEMP_DIR/proof1.json"

echo ""
echo ""
echo "Example 2: Hash with Format Specifiers"
echo "---------------------------------------"
echo "Scenario: Prove hash computation with formatted data using arithmetic"
echo "  A = 255 (secret, formatted as hex)"
echo "  B = 1000 (secret, formatted as decimal)"
echo "  Format: sha256('ff|1000') -> compute hash -> verify hash is non-zero"
echo ""

# Create Zircon file with format specifiers
cat > "$TEMP_DIR/format_hash.zrc" << 'EOF'
1/A:255,B:1000/result:?/hash<==sha256(A{%x}|B{%d})/hash!=0
EOF

echo "Zircon format:"
cat "$TEMP_DIR/format_hash.zrc"
echo ""

echo "Format specifiers:"
echo "  {%x} = hexadecimal (A:255 -> 'ff')"
echo "  {%d} = decimal (B:1000 -> '1000')"
echo "  Combined: sha256('ff|1000')"
echo "  Constraint: hash != 0 (hash is valid, uses equality check - no range tables needed)"
echo ""

echo "Estimate:"
$ZKPLEX --zircon "$TEMP_DIR/format_hash.zrc" --estimate

echo ""
echo "Generate proof:"
PROOF_OUTPUT=$($ZKPLEX --zircon "$TEMP_DIR/format_hash.zrc" --prove)
echo "$PROOF_OUTPUT"
echo "$PROOF_OUTPUT" > "$TEMP_DIR/proof2.json"

echo ""
echo "Verify proof:"
$ZKPLEX --verify --proof "$TEMP_DIR/proof2.json"

echo ""
echo ""
echo "Example 3: Multiple Preprocessing Steps"
echo "---------------------------------------"
echo "Scenario: Chain multiple hash operations"
echo "  secret1 = 'hello' (secret)"
echo "  secret2 = 'world' (secret)"
echo "  Step 1: hash1 = sha256(secret1)"
echo "  Step 2: hash2 = sha256(secret2)"
echo "  Step 3: combined = sha256(hash1 | hash2)"
echo "  Circuit: combined == expected"
echo ""

echo "Generate proof with multiple preprocessing:"
PROOF_OUTPUT=$($ZKPLEX \
  --preprocess "hash1<==sha256(secret1)" \
  --preprocess "hash2<==sha256(secret2)" \
  --preprocess "combined<==sha256(hash1{%x}|hash2{%x})" \
  --circuit "combined == expected" \
  --secret secret1:hello:text \
  --secret secret2:world:text \
  --public expected:0x936a185caaa266bb9cbe981e9e05cb78cd732b0b3280eb944412bb6f8f8f07af:hex \
  --public "result:?" \
  --prove)

echo "$PROOF_OUTPUT"
echo "$PROOF_OUTPUT" > "$TEMP_DIR/proof3.json"

echo ""
echo "Verify proof:"
$ZKPLEX --verify --proof "$TEMP_DIR/proof3.json"

echo ""
echo ""
echo "Example 4: Hash Inequality Check"
echo "---------------------------------------"
echo "Scenario: Prove hash is not zero (valid hash output)"
echo "  secret: 'test' (hidden)"
echo "  Preprocessing: hash = sha256(secret)"
echo "  Circuit: Verify hash != 0 (uses equality check, no range tables needed)"
echo ""

echo "Generate proof:"
PROOF_OUTPUT=$($ZKPLEX \
  --preprocess "hash<==sha256(secret)" \
  --circuit "hash != 0" \
  --secret secret:test:text \
  --public "result:?" \
  --prove)

echo "$PROOF_OUTPUT"
echo "$PROOF_OUTPUT" > "$TEMP_DIR/proof4.json"

echo ""
echo "Verify proof:"
$ZKPLEX --verify --proof "$TEMP_DIR/proof4.json"

# Cleanup
rm -rf "$TEMP_DIR"

echo ""
echo ""
echo "================================================"
echo "✓ All preprocessing examples completed!"
echo "================================================"
echo ""
echo "Key Concepts Demonstrated:"
echo ""
echo "1. Hash Preprocessing"
echo "   - Compute SHA256 hash before circuit evaluation"
echo "   - Reduces circuit complexity (hash computed once)"
echo ""
echo "2. Format Specifiers"
echo "   - {%x} = hexadecimal format"
echo "   - {%d} = decimal format"
echo "   - Allows flexible data encoding in hash input"
echo ""
echo "3. Multiple Preprocessing Steps"
echo "   - Chain multiple operations"
echo "   - Each step can use results from previous steps"
echo ""
echo "4. Preprocessing in Debug Output"
echo "   - All preprocessing steps saved in proof JSON"
echo "   - Check 'debug.preprocess' field in generated proofs"
echo ""
echo "Format Specifiers Reference:"
echo "  {%x}  - hexadecimal (e.g., 255 -> 'ff')"
echo "  {%d}  - decimal (e.g., 1000 -> '1000')"
echo "  {%o}  - octal (e.g., 8 -> '10')"
echo "  {%b}  - binary (e.g., 5 -> '101')"
echo ""
echo "Documentation: docs/PREPROCESSING.md"
echo ""