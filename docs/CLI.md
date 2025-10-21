# ZKPlex CLI Documentation

## Overview

`zkplex-cli` is a command-line tool for working with zero-knowledge proofs using the ZKPlex framework. It supports format conversions, circuit analysis, proof generation, and verification.

## Table of Contents

1. [Installation](#installation)
2. [Quick Start](#quick-start)
3. [Input Formats](#input-formats)
4. [Operations](#operations)
5. [Signal Definitions](#signal-definitions)
6. [Proof Generation](#proof-generation)
7. [Proof Verification](#proof-verification)
8. [Format Conversion](#format-conversion)
9. [Circuit Analysis](#circuit-analysis)
10. [Examples](#examples)
11. [Advanced Usage](#advanced-usage)

## Installation

```bash
# Build the CLI
cargo build --release

# The binary will be at:
# target/release/zkplex-cli

# Optionally, install system-wide
cargo install --path .
```

## Quick Start

```bash
# Generate a simple proof
zkplex-cli --circuit "A + B == result" \
           --secret A:5 \
           --secret B:3 \
           --public result:8 \
           --prove

# Verify a proof
zkplex-cli --verify --proof proof.json

# Convert Zircon to JSON
zkplex-cli --zircon "1/age:25/-/age>=18" --into-json

# Estimate circuit constraints
zkplex-cli --zircon "1/balance:1000/-/balance>100" --estimate
```

## Input Formats

The CLI supports three input formats for circuit definitions:

### 1. Zircon Format (`-z, --zircon`)

Compact blockchain-optimized format:

```bash
zkplex-cli --zircon "1/age:25/-/age>=18"
```

**Format structure**: `version/secret/public/[preprocessing/]circuit`

See [Zircon Documentation](zircon/) for full specification.

### 2. JSON Format (`-j, --json`)

Structured JSON format:

```bash
zkplex-cli --json program.json --into-zircon
```

### 3. Direct Circuit (`--circuit`)

Direct circuit expression (for proof generation):

```bash
zkplex-cli --circuit "age >= 18" --secret age:25 --prove
```

## Operations

### Help

```bash
zkplex-cli --help
zkplex-cli -h
```

### Format Conversion

Convert between formats:

```bash
# To JSON
zkplex-cli --zircon input.zrc --into-json

# To Zircon
zkplex-cli --json input.json --into-zircon
```

### Circuit Analysis

```bash
# Show program information
zkplex-cli --zircon "1/A:10,B:20/-/A+B>25" --info

# Estimate constraints
zkplex-cli --zircon "1/balance:1000/-/balance>100" --estimate
```

### Proof Generation

```bash
zkplex-cli --circuit "circuit" \
           --secret name:value \
           --public name:value \
           --prove \
           --proof output.json
```

### Proof Verification

```bash
zkplex-cli --verify --proof proof.json
```

## Signal Definitions

Signals are inputs to the circuit. They can be secret (private witness) or public (part of the proof).

### Secret Signals (`-s, --secret`)

Secret signals are NOT included in the proof and remain private:

```bash
--secret name:value
--secret name:value:encoding
```

**Examples:**
```bash
--secret age:25
--secret password:secret123:base58
--secret address:0x1a2b3c:hex
```

### Public Signals (`-p, --public`)

Public signals ARE included in the proof and can be verified:

```bash
--public name:value
--public name:value:encoding
```

**Examples:**
```bash
--public threshold:18
--public expectedHash:0xabcd:hex
--public targetAddress:9aE476sH92Vc7DMC8bZNpe1xNNNy1fNjFpCGvfMuZMwM:base58
```

### Important Notes

- **At least one public signal is REQUIRED** for proof generation
- Use short aliases: `-s` for `--secret`, `-p` for `--public`
- Multiple signals can be specified by repeating the flag
- Secret signals are used during proving but NOT saved in the proof file

### Value Encodings

Supported encodings:

| Encoding | Aliases | Example | Use Case |
|----------|---------|---------|----------|
| `decimal` | (default) | `12345` | Numbers |
| `hex` | `hex` | `0x1a2b3c` | Ethereum addresses, hashes |
| `base58` | `base58`, `b58` | `5HpHagT65T...` | Solana/Bitcoin addresses |
| `base64` | `base64`, `b64` | `SGVsbG8=` | Universal encoding |
| `base85` | `base85`, `b85` | `9jqo^` | Compact encoding |

**Example with encoding:**
```bash
zkplex-cli --circuit "hash == expected" \
           --secret hash:0xdeadbeef:hex \
           --public expected:3735928559 \
           --prove
```

## Proof Generation

### Basic Proof

```bash
zkplex-cli --circuit "age >= 18" \
           --secret age:25 \
           --public threshold:18 \
           --prove
```

### Proof with Multiple Signals

```bash
zkplex-cli --circuit "A + B == sum" \
           --secret A:5 \
           --secret B:3 \
           --public sum:8 \
           --prove \
           --proof my_proof.json
```

### Proof from Zircon File

```bash
# Create Zircon file with placeholders
echo '1/A:?,B:?/sum:?/A+B==sum' > sum.zrc

# Generate proof by providing values
zkplex-cli --zircon sum.zrc \
           --secret A:10 \
           --secret B:20 \
           --public sum:30 \
           --prove
```

### Proof Strategy Options

Control the circuit strategy for optimization:

```bash
--proof-strategy auto     # Adaptive strategy (automatically chooses optimal based on circuit)
--proof-strategy boolean  # Base strategy (arithmetic, equality, and boolean operations)
--proof-strategy lookup   # Full comparison support with lookup tables
--proof-strategy bitd     # Full comparison support with bit decomposition
```

**Strategy Comparison:**

| Strategy | Supported Operations | Use Case |
|----------|---------------------|----------|
| **auto** | All operations (adaptive selection) | Default choice - automatically selects optimal strategy |
| **boolean** | `+`, `-`, `*`, `/`, `==`, `!=`, `AND`, `OR`, `NOT` | Circuits without range comparisons - smallest proofs |
| **lookup** | `+`, `-`, `*`, `/`, `==`, `!=`, `AND`, `OR`, `NOT`, `>`, `<`, `>=`, `<=` | Fast proving with comparisons (efficient for ≤16-bit values) |
| **bitd** | `+`, `-`, `*`, `/`, `==`, `!=`, `AND`, `OR`, `NOT`, `>`, `<`, `>=`, `<=` | Comparisons with larger values (more efficient for >16-bit values) |

**Examples:**

```bash
# Boolean operations (without range comparisons) - use boolean
zkplex-cli --circuit "(A == 15) AND (B != 20)" \
           --secret A:15 --secret B:18 \
           --public result:1 \
           --prove \
           --proof-strategy boolean

# Range checks - use lookup for faster proving
zkplex-cli --circuit "value > 100" \
           --secret value:150 \
           --public threshold:100 \
           --prove \
           --proof-strategy lookup
```

### Output

Proof is saved as JSON with all necessary information:

```json
{
  "proof": "base85_encoded_proof_data...",
  "verification_context": "base85_encoded_context...",
  "public_signals": {
    "sum": {
      "value": "8"
    }
  }
}
```

**Note:** The proof only generates if the circuit evaluates to true (1). If the circuit evaluates to false, proof generation will fail with `ConstraintSystemFailure`.

## Proof Verification

### Basic Verification

```bash
zkplex-cli --verify --proof proof.json
```

**Output:**
```
Proof verification: true
```

**Note:** The CLI may display additional information about the circuit evaluation, but this is not part of the proof itself. The proof only verifies that the circuit constraints are satisfied.

### Verification from File

```bash
# Verify proof file
zkplex-cli --verify --proof output.json

# Short form
zkplex-cli --verify output.json
```

### Verification Outputs

**Success:**
```
Proof verification: true
Output: 1
```

**Failure:**
```
Proof verification: false
Error: Verification failed: invalid proof
```

## Format Conversion

### Zircon to JSON

```bash
zkplex-cli --zircon "1/age:25/-/age>=18" --into-json
```

**Output:**
```json
{
  "version": 1,
  "secret": {
    "age": {
      "value": "25"
    }
  },
  "public": {},
  "circuit": ["age>=18"]
}
```

### JSON to Zircon

```bash
zkplex-cli --json input.json --into-zircon
```

### File Input/Output

```bash
# Read from file
zkplex-cli --zircon examples/age_check.zrc --into-json

# Output to file
zkplex-cli --zircon input.zrc --into-json > output.json
```

## Circuit Analysis

### Show Program Information

```bash
zkplex-cli --zircon "1/A:10/B:20/hash<==sha256(A{%x})/A>B" --info
```

**Output:**
```
Program Information:
  Version: 1
  Secret signals: A
  Public signals: B
  Preprocessing: hash<==sha256(A{%x})
  Circuit: A>B
```

### Estimate Constraints

```bash
zkplex-cli --zircon "1/balance:1000/-/balance>100" --estimate
```

**Output:**
```
Estimated constraints: 68
K parameter: 10
Estimated proof size: ~1024 bytes
```

### Complex Circuit Analysis

```bash
zkplex-cli --zircon "1/A:10,B:20/-/sum<==A+B;prod<==A*B;sum>prod" --estimate
```

**Output:**
```
Estimated constraints: 137
K parameter: 11
Estimated proof size: ~1536 bytes
```

## Examples

### Example 1: Age Verification

Prove age >= 18 without revealing exact age:

```bash
zkplex-cli --circuit "age >= 18" \
           --secret age:25 \
           --public threshold:18 \
           --prove \
           --proof age_proof.json

# Verify
zkplex-cli --verify --proof age_proof.json
```

### Example 2: Range Proof

Prove value is in range [100, 200]:

```bash
zkplex-cli --circuit "(value >= 100) AND (value <= 200)" \
           --secret value:150 \
           --public min:100 \
           --public max:200 \
           --prove
```

### Example 3: Ethereum Address Comparison

```bash
zkplex-cli --circuit "myAddr == targetAddr" \
           --secret myAddr:0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb:hex \
           --public targetAddr:0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb:hex \
           --prove
```

### Example 4: Base58 Encoded Value

```bash
zkplex-cli --circuit "hash == expected" \
           --secret hash:3J98t1WpEZ73CNmYviecrnyiWrnqRhWNLy:base58 \
           --public expected:100 \
           --prove
```

### Example 5: Multiple Operations

```bash
zkplex-cli --circuit "(A + B) * C > threshold" \
           --secret A:10 \
           --secret B:20 \
           --secret C:2 \
           --public threshold:50 \
           --prove
```

### Example 6: Using Zircon Files

```bash
# Create Zircon file
cat > balance.zrc << 'EOF'
1/balance:?/threshold:100/balance>threshold
EOF

# Generate proof
zkplex-cli --zircon balance.zrc \
           --secret balance:1000 \
           --prove

# Verify
zkplex-cli --verify --proof proof.json
```

### Example 7: Format Conversion Pipeline

```bash
# Start with Zircon
echo '1/A:10,B:20/-/A+B>25' > input.zrc

# Convert to JSON
zkplex-cli --zircon input.zrc --into-json > program.json

## Advanced Usage

### Using Placeholders in Zircon Files

Placeholders (`?`) allow you to define circuit structure separately from values:

```bash
# Create template
echo '1/A:?,B:?/sum:?/A+B==sum' > template.zrc

# Use with different values
zkplex-cli --zircon template.zrc \
           --secret A:5 --secret B:3 --public sum:8 \
           --prove --proof proof1.json

zkplex-cli --zircon template.zrc \
           --secret A:100 --secret B:200 --public sum:300 \
           --prove --proof proof2.json
```

### Batch Verification

```bash
# Verify multiple proofs
for proof in proofs/*.json; do
  echo "Verifying $proof"
  zkplex-cli --verify --proof "$proof"
done
```

### Integration with Shell Scripts

```bash
#!/bin/bash

# Generate proof
zkplex-cli --circuit "balance > 100" \
           --secret balance:1000 \
           --public threshold:100 \
           --prove \
           --proof balance.json

# Check verification output
if zkplex-cli --verify --proof balance.json | grep -q "true"; then
  echo "✓ Proof verified"
  exit 0
else
  echo "✗ Proof verification failed"
  exit 1
fi
```

### Circuit Estimation Before Proving

```bash
# Estimate first
ESTIMATE=$(zkplex-cli --circuit "complex_circuit" --estimate)
echo "$ESTIMATE"

# Check if acceptable, then prove
if [[ "$ESTIMATE" =~ "constraints: "([0-9]+) ]]; then
  CONSTRAINTS="${BASH_REMATCH[1]}"
  if [ "$CONSTRAINTS" -lt 1000 ]; then
    echo "Constraints acceptable, generating proof..."
    zkplex-cli --circuit "complex_circuit" ... --prove
  else
    echo "Circuit too complex ($CONSTRAINTS constraints)"
  fi
fi
```

## Command Reference

### Global Options

| Option | Short | Description |
|--------|-------|-------------|
| `--help` | `-h` | Show help information |

### Input Format Options (choose one)

| Option | Short | Argument | Description |
|--------|-------|----------|-------------|
| `--zircon` | `-z` | TEXT\|FILE | Input in Zircon format |
| `--json` | `-j` | TEXT\|FILE | Input in JSON format |
| `--circuit` | | TEXT | Direct circuit expression |

### Output Format Options

| Option | Description |
|--------|-------------|
| `--into-json` | Convert to JSON format |
| `--into-zircon` | Convert to Zircon format |

### Circuit Analysis Options

| Option | Short | Description |
|--------|-------|-------------|
| `--info` | `-i` | Show program information |
| `--estimate` | `-e` | Estimate circuit requirements |

### Proof Options

| Option | Short | Argument | Description |
|--------|-------|----------|-------------|
| `--secret` | `-s` | name:value[:enc] | Secret signal (repeatable) |
| `--public` | `-p` | name:value[:enc] | Public signal (repeatable) |
| `--prove` | | | Generate a proof |
| `--verify` | | | Verify a proof |
| `--proof` | | FILE | Proof file path |
| `--proof-strategy` | | STRATEGY | Circuit strategy (auto\|boolean\|lookup\|bitd) |

## Troubleshooting

### Error: "Signal has placeholder '?' but no value provided"

**Cause:** Zircon file contains `?` placeholder but no value was provided via CLI.

**Solution:** Provide value using `--secret` or `--public`:
```bash
zkplex-cli --zircon "1/A:?/-/A>10" --secret A:15 --prove
```

### Error: "At least one public signal is required"

**Cause:** Proof generation requires at least one public signal.

**Solution:** Add a public signal:
```bash
# ✗ Wrong - no public signals
zkplex-cli --circuit "A + B" --secret A:5 --secret B:3 --prove

# ✓ Correct - has public signal
zkplex-cli --circuit "A + B == sum" \
           --secret A:5 --secret B:3 --public sum:8 --prove
```

### Error: "Verification failed"

**Causes:**
- Proof was tampered with
- Wrong verification key
- Incorrect public inputs
- Proof doesn't match the circuit

**Solution:** Ensure proof file is valid and untampered.

### Error: "Value exceeds 2^64 for ordering comparison"

**Cause:** Using ordering operators (`>`, `<`, `>=`, `<=`) with values >= 2^64.

**Solution:** Use equality operators (`==`, `!=`) for large values:
```bash
# ✗ Wrong - value too large for ordering
zkplex-cli --circuit "addr > 0" --secret addr:9aE...ZMwM:base58 --prove

# ✓ Correct - use equality
zkplex-cli --circuit "addr == target" \
           --secret addr:9aE...ZMwM:base58 \
           --public target:9aE...ZMwM:base58 --prove
```

### Error: "Failed to parse zircon format"

**Cause:** Invalid Zircon format syntax.

**Solution:** Check format structure: `version/secret/public/[preprocessing/]circuit`
```bash
# ✗ Wrong
zkplex-cli --zircon "age:25/age>=18" --prove

# ✓ Correct
zkplex-cli --zircon "1/age:25/-/age>=18" --prove
```

## Performance Tips

1. **Use `--estimate` first** for complex circuits to understand cost
2. **Choose proof strategy** based on your circuit:
   - `auto`: Default choice - automatically selects optimal strategy
   - `boolean`: Circuits without range comparisons - smallest proofs
   - `lookup`: Fast proving with comparisons (efficient for ≤16-bit values)
   - `bitd`: Comparisons with larger values (more efficient for >16-bit values)
3. **Minimize public signals** to reduce proof size
4. **Use preprocessing** to optimize repeated operations
5. **Keep circuits simple** - fewer operations = faster proving

## See Also

- **[WASM API Documentation](WASM_API.md)** - JavaScript/TypeScript API
- **[Zircon Documentation](zircon/)** - Zircon format specification
- **[Examples](zircon/EXAMPLES.md)** - More usage examples
- **[Best Practices](zircon/BEST_PRACTICES.md)** - Optimization and security tips
- **[Quick Start Guide](../QUICKSTART.md)** - Getting started guide