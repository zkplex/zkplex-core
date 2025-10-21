# ZKPlex CLI Examples

This directory contains working examples demonstrating how to use zkplex-cli for various zero-knowledge proof scenarios.

## Prerequisites

Build the CLI in Docker first:

```bash
make cli-build
```

This creates the `./zkplex` wrapper script that runs the CLI in a Docker container.

## Available Examples

### 1. Age Verification (`age_verification.sh`)

Demonstrates proving that age >= 18 without revealing the exact age.

**Workflow:**
1. Estimate circuit requirements
2. Generate proof with secret age (25)
3. Verify proof without knowing the secret

**Run:**
```bash
./examples/age_verification.sh
```

**Key Concepts:**
- Secret signals (hidden from verifier)
- Public signals (included in proof)
- Circuit evaluation: `age >= 18`

---

### 2. Threshold Check (`threshold_check.sh`)

Proves that `(A + B) * C > threshold` and outputs the computed result as a public signal.

**Workflow:**
1. Estimate circuit with multiple secret inputs
2. Generate proof with output signal (result)
3. Verify proof with public threshold and result

**Run:**
```bash
./examples/threshold_check.sh
```

**Key Concepts:**
- Output signals (`result:?`)
- Complex expressions with arithmetic
- Multiple secret inputs (A, B, C)
- Public inputs and outputs

---

### 3. Preprocessing (`preprocessing.sh`)

Demonstrates preprocessing with hash functions and format specifiers.

**Examples in script:**
1. SHA256 hash comparison - prove knowledge of password preimage
2. Hash with format specifiers - using `{%x}` and `{%d}` formatting
3. Multiple preprocessing steps - chaining hash operations
4. Preprocessing in range proof - hash output verification

**Run:**
```bash
./examples/preprocessing.sh
```

**Key Concepts:**
- Preprocessing statements (`hash<==sha256(secret)`)
- Format specifiers (`{%x}`, `{%d}`, `{%o}`, `{%b}`)
- Chaining multiple preprocessing operations
- Reducing circuit complexity by computing expensive operations beforehand
- Preprocessing saved in debug output

---

### 4. Zircon Format (`zircon_format.sh`)

Demonstrates Zircon format - a compact blockchain-optimized format for ZKP programs.

**Examples in script:**
- Age verification: `1/age:25/-/age>=18`
- Threshold check with output: `1/A:10,B:20,C:2/threshold:50,result:?/-(A+B)*C>threshold`
- Hash preprocessing: `1/secret:hello/target:0x.../sha256(secret)==target`

**Run:**
```bash
./examples/zircon_format.sh
```

**Key Concepts:**
- Zircon format syntax
- Compact representation (50-80 bytes vs 500+ bytes JSON)
- Hash preprocessing functions
- File-based proof definitions

---

### 5. Run All Examples (`run_all.sh`)

Runs all examples sequentially with interactive prompts.

**Run:**
```bash
./examples/run_all.sh
```

---

## Understanding the Output

### Estimate Output

Shows circuit requirements before proof generation:

```
ZKPlex Circuit Estimation
=========================

Circuit: age >= 18

Complexity: medium

Circuit Parameters:
  Required k:        10
  Total rows (2^k):  1024
  Estimated rows:    512
  Row utilization:   50.0%

Operations:
  Arithmetic ops:    0
  Comparisons:       1
  Preprocessing:     0

Resource Requirements (Hardware-Independent):
  Params size:       123456 bytes (120 KB)
  Proof size:        34567 bytes (33.8 KB)
  VK size:           12345 bytes (12.1 KB)
```

### Proof Output

Contains all information needed for verification:

```json
{
  "version": 1,
  "proof": "...(ASCII85 encoded proof)...",
  "verification_context": "...(ASCII85 encoded context)...",
  "public_signals": {
    "result": {
      "value": "1",
      "encoding": null
    }
  },
  "debug": {
    "preprocess": [],
    "circuit": ["age >= 18"],
    "k": 10,
    "strategy": "auto",
    "secret_signals": ["age"],
    "output_signal": "result"
  }
}
```

### Verification Output

Simple confirmation:

```
Verifying proof from proof.json...
Circuit: age >= 18
k: 10
Strategy: auto
Public signals: {"result": PublicSignal { value: "1", encoding: None }}
Regenerating verification key...
Verifying proof...
✓ Proof is VALID
```

---

## Creating Your Own Proofs

### Step 1: Design the Circuit

Decide what you want to prove:
- What values are secret? (only you know)
- What values are public? (verifier knows)
- What is the constraint? (inequality, equality, computation)

### Step 2: Estimate Requirements

```bash
./zkplex --circuit "YOUR_CIRCUIT" \
         --secret var1:value1 \
         --public var2:value2 \
         --estimate
```

### Step 3: Generate Proof

```bash
./zkplex --circuit "YOUR_CIRCUIT" \
         --secret var1:value1 \
         --public var2:value2 \
         --prove > proof.json
```

### Step 4: Verify Proof

```bash
./zkplex --verify --proof proof.json
```

---

## Common Use Cases

### Range Proofs

Prove a value is within a range without revealing it:

```bash
./zkplex --circuit "value >= 18 AND value <= 65" \
         --secret value:42 \
         --prove
```

### Equality Proofs

Prove two values are equal without revealing them:

```bash
./zkplex --circuit "hash1 == hash2" \
         --secret hash1:$SECRET1 \
         --public hash2:$PUBLIC_HASH \
         --prove
```

### Computation Proofs

Prove a computation result without revealing inputs:

```bash
./zkplex --circuit "A + B" \
         --secret A:10 --secret B:20 \
         --public result:? \
         --prove
```

### Hash Proofs

Prove knowledge of a preimage:

```bash
./zkplex --circuit "sha256(secret) == target" \
         --secret secret:mypassword \
         --public target:0x5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8:hex \
         --prove
```

---

## Encoding Formats

ZKPlex supports multiple value encoding formats:

| Encoding | Example | Use Case |
|----------|---------|----------|
| **decimal** | `123456` | Default, simple numbers |
| **hex** | `0x1a2b3c` | Hashes, binary data |
| **base58** | `9aE476sH...` | Bitcoin/Solana addresses |
| **base64** | `SGVsbG8=` | Universal binary encoding |
| **base85** | (ASCII85) | Compact proof encoding |

**Usage:**
```bash
./zkplex --secret address:9aE476sH92Vc7DMC8bZNpe1xNNNy1fNjFpCGvfMuZMwM:base58 ...
```

---

## Proof Strategies

Choose the right strategy for your circuit:

| Strategy | Operations Supported | Best For |
|----------|---------------------|----------|
| **auto** | All | Default, adaptive |
| **boolean** | `+`, `-`, `*`, `/`, `==`, `!=`, `AND`, `OR`, `NOT` | Boolean logic |
| **lookup** | All (with lookup tables) | Faster comparisons |
| **bitd** | All (with bit decomposition) | No lookup overhead |

**Usage:**
```bash
./zkplex --circuit "..." --proof-strategy lookup --prove
```

---

## Tips

1. **Always estimate first** - Know your circuit complexity before generating proofs
2. **Use output signals** - Let the circuit compute public results (use `?` as value)
3. **Test with small values** - Verify logic before using real data
4. **Keep secrets secure** - Never commit secret values to git/blockchain
5. **Use Zircon for storage** - Compact format ideal for on-chain storage

---

## Documentation

- **[CLI Reference](../docs/CLI.md)** - Complete CLI documentation
- **[Quick Start Guide](../docs/QUICKSTART.md)** - 5-minute tutorial
- **[Zircon Syntax](../docs/zircon/SYNTAX.md)** - Zircon format specification
- **[Architecture](../docs/ARCHITECTURE.md)** - System design

---

## Getting Help

Run `./zkplex --help` for detailed usage information.

For issues or questions:
- GitHub Issues: https://github.com/zkplex/zkplex-core/issues
- Documentation: [docs/](../docs/)