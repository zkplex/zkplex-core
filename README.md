# /// ZKPlex ///

ZKPlex is a WASM-compilable Rust library for creating and verifying zero-knowledge proofs based on mathematical circuits. Built on Halo2 with range proof support, it provides multiple format options including **Zircon** - a blockchain-optimized compact format designed for durability and efficiency.

**Key Features:** Natural arithmetic syntax, cryptographically secure comparisons, multiple value encodings (hex, base58, base64), preprocessing with hash functions, and full TypeScript support.

## Features

### Core Capabilities
- **Zircon Format** - Blockchain-optimized compact format for ZKP programs
- **Rich Operations** - Arithmetic (`+`, `-`, `*`, `/`), Comparisons (`>`, `<`, `==`, `!=`, `>=`, `<=`), Boolean (`AND`, `OR`, `NOT`)
- **Cryptographically Secure Comparisons** - Using range proofs and is_zero gadgets
- **Multiple Value Formats** - Decimal, Hexadecimal, Base58, Base64
- **Preprocessing** - Hash functions (SHA-1/256/512, SHA3-256/512, MD5, BLAKE2b/3, Keccak-256, RIPEMD-160) with value concatenation
- **Public/Secret Signals** - Full control over what information is revealed

### Developer Experience
- **CLI Tool** - Command-line interface for format conversion and circuit analysis
- **WASM Support** - Use from JavaScript/TypeScript in browsers or Node.js
- **TypeScript Definitions** - Full type safety for JS/TS projects
- **React Play** - Interactive UI for creating and verifying proofs
- **Docker Support** - Reproducible builds and easy deployment

### Input Formats
- **Zircon** - Compact blockchain format: `1/age:25/-/age>=18`
- **JSON** - Structured format with metadata
- **Direct Circuit** - Natural arithmetic syntax

## Documentation

### Getting Started
- **[Quick Start Guide](docs/QUICKSTART.md)** - CLI quick start (5 minutes)
- **[Docker Guide](docs/DOCKER.md)** - Docker build and deployment
- **[Development Guide](docs/DEVELOPMENT.md)** - Development workflow and cache busting

### Core Documentation
- **[Architecture](docs/ARCHITECTURE.md)** - System architecture and design
- **[Design Philosophy](docs/DESIGN_PHILOSOPHY.md)** - Design principles and comparisons
- **[CLI Reference](docs/CLI.md)** - Complete CLI documentation
- **[WASM API](docs/WASM_API.md)** - JavaScript/TypeScript API reference

### Zircon Format
- **[Overview](docs/zircon/OVERVIEW.md)** - Introduction to Zircon format
- **[Syntax](docs/zircon/SYNTAX.md)** - Format specification and grammar
- **[Signals](docs/zircon/SIGNALS.md)** - Secret and public signals
- **[Encoding](docs/zircon/ENCODING.md)** - Value encoding formats (hex, base58, base64)
- **[Operators](docs/zircon/OPERATORS.md)** - All operators and precedence
- **[Preprocessing](docs/zircon/PREPROCESSING.md)** - Hash functions and transformations
- **[Circuit](docs/zircon/CIRCUIT.md)** - Circuit constraints and patterns
- **[Examples](docs/zircon/EXAMPLES.md)** - Real-world examples
- **[Tools](docs/zircon/TOOLS.md)** - CLI and API reference
- **[Best Practices](docs/zircon/BEST_PRACTICES.md)** - Optimization and security

## Docker Architecture

ZKPlex uses a two-stage Docker architecture that separates WASM compilation from Play development. This approach provides reproducible builds, eliminates local environment dependencies, and enables efficient hot-reloading during development.

### How WASM is Transferred to Play

```
┌─────────────────────────────────────────────────────┐
│  Host Machine                                       │
│                                                     │
│  1. WASM Build Stage:                               │
│     ┌──────────────────┐                            │
│     │ Docker Container │  wasm-pack build           │
│     │  (Rust + wasm)   │ ───────────────►           │
│     └──────────────────┘                            │
│              │                                      │
│              │ docker cp                            │
│              ▼                                      │
│     ┌──────────────────┐                            │
│     │   ./pkg/         │  ◄── WASM artifacts        │
│     │  ├─ .js          │                            │
│     │  ├─ .wasm        │                            │
│     │  └─ .d.ts        │                            │
│     └──────────────────┘                            │
│              ▲                                      │
│              │ mount (docker -v)                    │
│              │                                      │
│  2. Play Stage:                                     │
│     ┌──────────────────┐                            │
│     │ Docker Container │                            │
│     │  (Node + Vite)   │                            │
│     │                  │                            │
│     │  /app/pkg/ ◄─────┼── mounts ./pkg             │
│     │  /app/src/ ◄─────┼── mounts ./play/src        │
│     └──────────────────┘                            │
│              │                                      │
│              │ http://localhost:5173                │
│              ▼                                      │
│     ┌──────────────────┐                            │
│     │    Browser       │                            │
│     └──────────────────┘                            │
└─────────────────────────────────────────────────────┘
```

### Build Pipeline

**Stage 1: WASM Compilation** (`Dockerfile.wasm`)

1. Creates a Rust container with wasm-pack
2. Compiles zkplex-core to WebAssembly
3. Outputs artifacts to `/output` inside container
4. Artifacts are copied to host `./pkg/` directory via `docker cp`

**Stage 2: Play Development** (`Dockerfile.play`)

1. Creates a Node.js container with Vite dev server
2. Mounts `./pkg/` from host (read-only) - contains WASM artifacts
3. Mounts `./play/src/` from host (read-only) - enables hot reload
4. Exposes port 5173 for browser access

### Key Benefits

✅ **Separation of Concerns**
- Heavy Rust container (~2GB) only used for building
- Lightweight Node container for development server

✅ **Artifact Reusability**
- WASM built once, play restarts independently
- No need to rebuild WASM on every code change

✅ **Hot Reload for Play**
- Source files mounted via volume (`-v`)
- Vite watches changes and updates browser instantly

✅ **Reproducible Builds**
- Same build output on any machine
- No dependency on local Rust/Node versions
- Perfect for CI/CD pipelines

### Development Workflow

**When you change Rust code:**
```bash
make wasm-build    # Rebuild WASM
make play-stop     # Stop play
make play-dev      # Restart with new WASM
```

**When you change Play code:**
```bash
# Nothing needed! Hot reload works automatically
# Files are mounted, Vite detects changes
```

**Production build:**
```bash
make wasm-build    # Build WASM module
make play-build    # Build Play for production
# Output: play/dist/
# Preview: cd play && npm run preview
```

**Available Make Commands:**

| Command | Description |
|---------|-------------|
| `make wasm-build` | Build WASM module in Docker and copy to ./pkg/ |
| `make cli-build` | Build CLI in Docker (use ./zkplex to run) |
| `make play-dev` | Run Play dev server in Docker |
| `make play-stop` | Stop Play Docker container |
| `make play-restart` | Restart Play (stop + start) |
| `make play-build` | Build Play for production |
| `make clean-play` | Remove Play build artifacts (dist/) |
| `make clean-wasm` | Remove WASM artifacts |
| `make clean-cli` | Remove CLI artifacts |
| `make clean` | Remove all Docker containers and images |

For complete Docker documentation, see [docs/DOCKER.md](docs/DOCKER.md).

## Quick Start

### Play (Recommended for First-Time Users)

```bash
# 1. Build WASM in Docker
make wasm-build

# 2. Start Play
make play-dev

# 3. Open http://localhost:5173 in your browser
```

The Play provides an interactive UI for creating and verifying proofs.

### CLI Tool

#### Docker (Recommended)

```bash
# 1. Build CLI in Docker
make cli-build

# 2. Run CLI with ./zkplex wrapper script
./zkplex --version
./zkplex --help

# 3. Generate a proof (prove age >= 18 without revealing exact age)
./zkplex --circuit "age >= 18" --secret age:25 --prove

# 4. Or use Zircon format
echo "1/age:25/-/age>=18" > proof.zrc
./zkplex --zircon proof.zrc --prove
```

The `./zkplex` wrapper script automatically runs the CLI in a Docker container with your current directory mounted, so you can access files seamlessly.

#### Local Build

```bash
# Build the CLI
cargo build --release

# Generate a proof (prove age >= 18 without revealing exact age)
./target/release/zkplex-cli \
  --circuit "age >= 18" \
  --secret age:25 \
  --prove

# Or use Zircon format
echo "1/age:25/-/age>=18" > proof.zrc
./target/release/zkplex-cli --zircon proof.zrc --prove
```

**📖 Complete CLI Guide:** [docs/QUICKSTART.md](docs/QUICKSTART.md)

### WASM API (JavaScript/TypeScript)

```bash
# Build WASM (Docker recommended)
make wasm-build

# Or build locally
wasm-pack build --target web
```

```typescript
import init, { prove, verify, version } from './pkg/zkplex_core.js';

await init();

console.log('zkplex-core version:', version());

// Generate proof
const proveRequest = {
  circuit: ["age >= 18"],
  signals: {
    age: { value: "25", public: false }
  }
};

const proof = JSON.parse(prove(JSON.stringify(proveRequest)));
console.log("Proof:", proof.proof);

// Verify proof
const verifyRequest = {
  proof: proof.proof,
  verification_context: proof.verification_context,
  public_signals: proof.public_signals
};

const result = JSON.parse(verify(JSON.stringify(verifyRequest)));
console.log("Valid:", result.valid);  // true
```

**📖 Complete WASM API Guide:** [docs/WASM_API.md](docs/WASM_API.md)

## Examples

### Age Verification

Prove you're over 18 without revealing your exact age:

```javascript
const request = {
  circuit: ["age >= 18"],
  signals: {
    age: { value: "25", public: false }
  }
};

const response = JSON.parse(prove(JSON.stringify(request)));
// Proof generated successfully - circuit evaluated to true
```

### Solana Address Ownership

Prove you know the secret key for a specific Solana address:

```javascript
const request = {
  circuit: ["myAddress == targetAddress"],
  signals: {
    myAddress: {
      value: "9aE476sH92Vc7DMC8bZNpe1xNNNy1fNjFpCGvfMuZMwM",
      encoding: "base58",
      public: false  // Keep your address secret
    },
    targetAddress: {
      value: "9aE476sH92Vc7DMC8bZNpe1xNNNy1fNjFpCGvfMuZMwM",
      encoding: "base58",
      public: true
    }
  }
};

const response = JSON.parse(prove(JSON.stringify(request)));
// Proof generated successfully - addresses match
```

**Note**: Equality comparisons work with arbitrary-precision values (32-byte addresses). Ordering comparisons (`>`, `<`) require values < 2^64.

### Complex Circuit

Prove `(A + B) * C > threshold` without revealing A, B, C:

```javascript
const request = {
  circuit: ["(A + B) * C > threshold"],
  signals: {
    A: { value: "10", public: false },
    B: { value: "20", public: false },
    C: { value: "2", public: false },
    threshold: { value: "50", public: true }
  }
};

const response = JSON.parse(prove(JSON.stringify(request)));
// Proof generated successfully - threshold check passed (60 > 50)
```

More examples: [docs/zircon/EXAMPLES.md](docs/zircon/EXAMPLES.md)

## How Verification Works

One of the fundamental properties of zero-knowledge proofs is that **verification does not require access to secret witness values**. ZKPlex implements this carefully:

### Verification Context

When a proof is generated, ZKPlex creates a `verification_context` containing:

- **Circuit structure**: The circuit constraints (without secret values)
- **Secret signal names**: Names only, no values
- **Public signal names**: Including the output signal name
- **Circuit metadata**: `k` parameter, strategy, and `cached_max_bits`

The `cached_max_bits` field is critical for circuits with range checks (`>`, `<`, `>=`, `<=`):
- Stores the maximum bit width of all values (e.g., 16 bits for values up to 65535)
- Ensures verifier uses the same lookup table size as the prover
- Without this, verification would fail with table size mismatch errors

### Verification Process

1. **Decode verification context** from the proof
2. **Reconstruct circuit** using only public signals and circuit structure
3. **Secret signals are NOT evaluated** - they remain as placeholders (`value: None`)
4. **Restore metadata** like `cached_max_bits` to match prover's configuration
5. **Verify cryptographic proof** using Halo2's verification algorithm

This ensures that:
- ✅ Verifier never learns secret values
- ✅ Circuit constraints are cryptographically verified
- ✅ Only public inputs and the output are revealed

For technical details, see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md#verification-without-secret-values).

## Proof Strategies

ZKPlex supports multiple proof strategies optimized for different use cases:

| Strategy | Supports | Use Case                             |
|----------|----------|--------------------------------------|
| **auto** | All operations (adaptive selection) | Default choice - automatically selects optimal strategy |
| **boolean** | `+`, `-`, `*`, `/`, `==`, `!=`, `AND`, `OR`, `NOT` | Circuits without range comparisons - smallest proofs |
| **lookup** | `+`, `-`, `*`, `/`, `==`, `!=`, `AND`, `OR`, `NOT`, `>`, `<`, `>=`, `<=` | Fast proving with comparisons (efficient for ≤16-bit values) |
| **bitd** | `+`, `-`, `*`, `/`, `==`, `!=`, `AND`, `OR`, `NOT`, `>`, `<`, `>=`, `<=` | Comparisons with larger values (more efficient for >16-bit values) |

**Note**: All strategies produce ~30-40 KB proofs due to Halo2's IPA commitment overhead.

**Example:**
```typescript
// Boolean strategy example (without range comparisons)
const request = {
  circuit: ["(A == 15) AND (B != 20)"],
  signals: {
    A: { value: "15", public: false },
    B: { value: "18", public: false },
    result: { public: true }  // Output signal - value will be computed
  },
  strategy: "boolean"  // Base strategy for boolean/equality operations
};
// Proof will contain result = 1 (true)
```

## Operator Precedence

Operators are evaluated in this order (highest to lowest priority):

1. **Parentheses** `()` - Force evaluation order
2. **Multiplication/Division** `*`, `/` - Left to right
3. **Addition/Subtraction** `+`, `-` - Left to right
4. **Comparison** `>`, `<`, `>=`, `<=`, `==`, `!=` - Left to right
5. **Boolean AND** `&&`, `AND` - Left to right
6. **Boolean OR** `||`, `OR` - Left to right

**Example:**
```
A + B > C + D    // Evaluates as (A + B) > (C + D)
                 // NOT as A + (B > C) + D
```

## Important Notes

### Comparison Operators

- **Equality operators** (`==`, `!=`) work with arbitrary-precision values of any size
- **Ordering operators** (`>`, `<`, `>=`, `<=`) require values in range [0, 2^64) due to range proof constraints
- Use equality operators for comparing large values like Solana addresses (32 bytes)

### Supported Operators

**✅ Fully Supported:**
- Arithmetic: `+`, `-`, `*`, `/`
- Comparison: `>`, `<`, `>=`, `<=`, `==`, `!=`
- Boolean: `AND`/`&&`, `OR`/`||`, `NOT`/`!`
- Grouping: `()` parentheses

**❌ Not Yet Supported:**
- Bitwise operators: `&`, `|`, `^`, `~`, `>>`, `<<`
- Power operator: `**`
- Modulo: `%`
- Compound assignments: `+=`, `-=`, etc.

### Feature Roadmap

Features could be added to ZKPlex:

| Operator             |                  | Priority  | Complexity                       |
|----------------------|------------------|-----------|----------------------------------|
| Power                | **               | 🔥 HIGH   | Medium - exponentiation in field |
| Modulo               | %                | 🔥 HIGH   | Medium - modular arithmetic      |
| Integer division     | \                | 🟡 MEDIUM | Easy - similar to /              |
| Bitwise AND          | &                | 🟡 MEDIUM | Hard - needs bit decomposition   |
| Bitwise OR           | |                | 🟡 MEDIUM | Hard - needs bit decomposition   |
| Bitwise XOR          | ^                | 🟡 MEDIUM | Hard - needs bit decomposition   |
| Bitwise NOT          | ~                | 🟢 LOW    | Hard - needs bit decomposition   |
| Left shift           | <<               | 🟢 LOW    | Medium - multiply by 2^n         |
| Right shift          | >>               | 🟢 LOW    | Medium - divide by 2^n           |
| Ternary              | ? :              | 🟡 MEDIUM | Medium - conditional selection   |
| Compound assignments | +=, -=, *=, etc. | 🟢 LOW    | Easy - syntactic sugar           |

**Tier 1 (High Priority):**
- `%` Modulo operator - remainder after division
- `**` Power operator - exponentiation
- `\` Integer division - division with floor rounding

**Tier 2 (Medium Priority):**
- `? :` Ternary operator - conditional expressions
- `^` Bitwise XOR - useful for cryptographic operations
- Array support - fixed-size arrays for batch operations

**Tier 3 (Lower Priority):**
- `<<` `>>` Shift operators - bit shifting operations
- Multi-dimensional arrays
- For loops with constraints

**Optimization Opportunities:**
- Reduce boolean operator constraints (currently 6 for AND, can be optimized to 3 when inputs are known binary)
- Add more lookup table optimizations for common operations
- Support for custom hint functions

Request features or vote on priorities at [GitHub Issues](https://github.com/zkplex/zkplex-core/issues).

## Performance

| Operation | Constraints | Notes |
|-----------|-------------|-------|
| Addition | 1 | Custom gate |
| Multiplication | 1 | Custom gate |
| Equality (==) | 3 | is_zero gadget |
| Greater (>) | ~68 | 64-bit range check + is_zero |
| GreaterEqual (>=) | ~65 | 64-bit range check only |

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT OR Apache-2.0

## References

- [Halo2 Documentation](https://zcash.github.io/halo2/)
- [Zero-Knowledge Proofs](https://en.wikipedia.org/wiki/Zero-knowledge_proof)
- [Range Proofs](https://eprint.iacr.org/2017/1066.pdf)