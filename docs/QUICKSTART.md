# ZKPlex Quick Start Guide

This guide will help you quickly get started with ZKPlex for generating and verifying zero-knowledge proofs.

## Installation

```bash
cargo build --release
```

The CLI tool will be available at `target/release/zkplex-cli`.

## Basic Concepts

ZKPlex supports multiple input formats:
- **Circuit**: Direct circuit (lowest level)
- **Zircon**: Compact blockchain-optimized format
- **JSON**: Structured format with metadata

## Quick Examples

### 1. Simple Proof Generation (Circuit Format)

```bash
# Generate a proof that you know two numbers that sum to 10
zkplex-cli --circuit "A+B==sum" \
  --secret A:3 \
  --secret B:7 \
  --public sum:10 \
  --prove \
  --proof my_proof.json

# Using short aliases: -s for --secret, -p for --public
zkplex-cli --circuit "A+B==sum" \
  -s A:3 \
  -s B:7 \
  -p sum:10 \
  --prove \
  --proof my_proof.json
```

### 2. Using Zircon Format

```bash
# Create a Zircon file
echo '1/A:3,B:7/C:10/A+B==C' > check.zrc

# Generate proof
zkplex-cli --zircon check.zrc \
  --prove \
  --proof result.json
```

### 3. Using Placeholders

Use `?` as a placeholder for values you want to provide at runtime:

```bash
# Create Zircon file with placeholders
echo '1/A:?,B:?/C:10/A+B==C' > check.zrc

# Provide values via command line
zkplex-cli --zircon check.zrc \
  --secret A:5 \
  --secret B:5 \
  --prove \
  --proof result.json
```

### 4. Verify a Proof

```bash
# Basic verification
zkplex-cli --verify result.json

# Get JSON output
zkplex-cli --verify result.json --into-json
```

## Common Use Cases

### Password Verification (with encoding)

```bash
# Create program with base58 encoded password
cat > password.zrc << 'EOF'
1/password:?:base58/expected:5Kd3NBUAdUnhyzenEwVLy9pBKxSwXvE9FMPyR4UKZvpe6E3AgLr:base58/password==expected
EOF

# Generate proof with your password
zkplex-cli --zircon password.zrc \
  --secret password:5Kd3NBUAdUnhyzenEwVLy9pBKxSwXvE9FMPyR4UKZvpe6E3AgLr:base58 \
  --prove \
  --proof password_proof.json

# Verify the proof
zkplex-cli --verify password_proof.json --into-json
```

### Range Proof

```bash
# Prove a value is within a range (need at least one public signal)
zkplex-cli --circuit "(value >= min) AND (value <= max)" \
  -s value:50 \
  -p min:10 \
  -p max:100 \
  --prove \
  --proof range_proof.json
```

### Hash Preimage

```bash
# Using JSON format
cat > hash.json << 'EOF'
{
  "version": 1,
  "secret": {
    "secret": {
      "value": "?",
      "encoding": "hex"
    }
  },
  "public": {
    "hash": {
      "value": "0x1234abcd",
      "encoding": "hex"
    }
  },
  "circuit": ["HASH(secret) == hash"]
}
EOF

# Generate proof
zkplex-cli --json hash.json \
  --secret secret:0xdeadbeef \
  --prove \
  --proof hash_proof.json
```

## Proof Structure

Generated proofs are self-contained JSON files that include everything needed for verification:

```json
{
  "proof": "base85_encoded_proof...",
  "verification_context": "base85_encoded_context...",
  "public_signals": {
    "C": {
      "value": "10"
    }
  }
}
```

No separate verification key file is needed - all verification info is in `verification_context`!

**Note:** The proof only generates if the circuit evaluates to true (1). If the circuit constraints are not satisfied, proof generation will fail with `ConstraintSystemFailure`.

## Estimation

Before generating a proof, you can estimate the computational cost:

```bash
# Text output
zkplex-cli --circuit "A+B==C" --estimate

# JSON output
zkplex-cli --circuit "A+B==C" --estimate --into-json
```

## Preprocessing

For complex circuits, you can use preprocessing to optimize the circuit:

```bash
zkplex-cli --circuit "A+B==C" \
  --preprocess "X=A*2,Y=B*2" \
  --secret A:5 \
  --secret B:10 \
  --prove \
  --proof optimized.json
```

## Format Conversion

Convert between different formats:

```bash
# Zircon to JSON
zkplex-cli --zircon input.zrc --json output.json

# JSON to Zircon
zkplex-cli --json input.json --zircon output.zrc
```

## Supported Encodings

- `decimal` (default): Regular numbers
- `hex`: Hexadecimal (prefix with 0x)
- `base58`: Bitcoin-style base58
- `base64`: Standard base64

Example with explicit encoding:

```bash
zkplex-cli --circuit "A==B" \
  --secret A:0x1a2b:hex \
  --public B:6699:decimal \
  --prove
```

## Verification Output

### Text Format (default)

```
Proof verification: true
```

### JSON Format (--into-json)

**Success:**
```json
{
  "valid": true
}
```

**Failure (proof invalid):**
```json
{
  "valid": false,
  "error": "Verification failed: ..."
}
```

**Note:** The CLI may display additional information about the circuit evaluation during proof generation, but this is not part of the proof structure itself. The proof only verifies that the circuit constraints were satisfied.

## CLI Arguments Reference

### Input Formats (choose one for proof generation)
- `--circuit <circuit>` - Direct circuit
- `--zircon <file>` - Zircon format file
- `--json <file>` - JSON format file (for conversion only)

### Operations (choose one)
- `--prove` - Generate a proof
- `--verify <proof.json>` - Verify a proof
- `--estimate` - Estimate computational cost

### Signal Values
- `-s, --secret <name:value[:encoding]>` - Secret input (can be repeated)
- `-p, --public <name:value[:encoding]>` - Public input (can be repeated)
  - **Note**: At least one public signal is REQUIRED for proof generation

### Optional Parameters
- `--preprocess <circuit>` - Preprocessing operations
- `--proof <file>` - Output file for proof (default: proof.json)
- `--into-json` - Output results in JSON format

### Conversion Outputs
- `--json <file>` - Convert to JSON
- `--zircon <file>` - Convert to Zircon

## Tips and Best Practices

1. **Use placeholders (`?`)** for values you want to keep separate from the circuit definition
2. **Use `--into-json`** when integrating with other tools or scripts
3. **Estimate first** for complex circuits to understand computational requirements
4. **Use preprocessing** to optimize repeated operations
5. **Store proofs** in a secure location - they contain verification keys and public inputs
6. **Use appropriate encodings** for different data types (base58 for keys, hex for hashes, etc.)

## Troubleshooting

### "Signal has placeholder '?' but no value provided"
You forgot to provide a value for a placeholder. Add `--secret name:value` or `--public name:value`.

### "ConstraintSystemFailure"
The circuit constraints were not satisfied. This means the circuit evaluated to false (0) instead of true (1). Check that your input values satisfy the circuit conditions.

### "Verification failed"
The proof is invalid. This could mean:
- The proof was tampered with
- Wrong verification key
- Incorrect public inputs

### "Signal has invalid value"
The value format doesn't match the specified encoding. Check your encoding specification.

## Next Steps

- **[📖 Complete CLI Documentation](docs/CLI.md)** - Full reference with all options and examples
- **[Zircon Documentation](docs/zircon/)** - Zircon format specification
- **[WASM API Documentation](docs/WASM_API.md)** - JavaScript/TypeScript API
- Check example programs in the `examples/` directory

## Support

For issues and questions:
- GitHub Issues: [zkplex-core/issues](https://github.com/zkplex/zkplex-core/issues)
- Documentation: `docs/` directory
- Examples: `examples/` directory