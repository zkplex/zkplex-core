# Zircon Format - Overview

## What is Zircon?

**Zircon** is a compact, blockchain-optimized format for zero-knowledge proof programs. Named after the mineral zircon (ZrSiO₄) - one of the oldest and most durable minerals on Earth, known for its exceptional resistance to physical and chemical weathering - the format embodies these same qualities of durability, stability, and information preservation through its robust structure and efficient encoding.

Just as the mineral zircon survives extreme geological conditions and preserves crystallographic information over billions of years, Zircon format provides a stable, compact representation that minimizes on-chain storage costs while maintaining the complete integrity of ZKP program specifications across blockchain networks.

## Why Zircon?

### Key Benefits

**1. Extreme Compactness**
- Typical programs: 50-100 bytes vs 200-500 bytes in JSON
- Minimizes blockchain gas costs
- Efficient for on-chain storage

**2. Human Readable**
- Despite compactness, remains understandable
- Self-documenting structure
- Easy debugging and verification

**3. Blockchain Optimized**
- Designed specifically for on-chain use
- Direct embedding in transactions
- Fast parsing with minimal overhead

**4. Extensible**
- Version field for future enhancements
- Backward compatible evolution
- Future-proof design

## Format Structure

Zircon programs consist of slash-separated components:

### Basic Format (4 parts)
```
version/secret/public/circuit
```

### Extended Format (5 parts)
```
version/secret/public/preprocess/circuit
```

## Components at a Glance

| Component      | Required | Description                                |
|----------------|----------|--------------------------------------------|
| **version**    | ✅ | Format version number (currently `1`)      |
| **secret**     | ✅ | Secret witness signals (use `-` if empty)  |
| **public**     | ✅ | Public signals (use `-` if empty)          |
| **preprocess** | ❌ | Preprocessing operations (hash, transform) |
| **circuit**    | ✅ | ZKP circuit constraints                    |

## Simple Example

```
1/age:25/-/age>=18
```

**Breakdown**:
- Version: `1`
- Scret: `age:25` (witness is secret)
- Public: `-` (no public inputs)
- Circuit: `age>=18` (prove age is 18 or older)

**Output**: Proves age ≥ 18 without revealing actual age.

## When to Use Zircon

### ✅ Ideal Use Cases

**On-Chain Storage**
- Smart contract state
- Transaction calldata
- Blockchain parameters

**Minimal Footprint Required**
- Gas cost optimization
- Storage efficiency
- Network bandwidth

**Production Environments**
- Deployed applications
- Live blockchain systems
- High-volume scenarios

### ❌ Less Suitable For

**Development & Debugging**
- Use JSON for better readability
- Easier to edit and experiment

**Complex Templates**
- Use Circom for reusable components
- Template-based circuits

**Documentation**
- JSON or Circom more verbose
- Better for explaining logic

## Comparison with Other Formats

| Feature | Zircon | JSON | Circom |
|---------|--------|------|--------|
| **Size** | ~50 bytes | ~200 bytes | ~300 bytes |
| **Readability** | Good | Excellent | Excellent |
| **Parsing Speed** | Fastest | Fast | Slow |
| **On-Chain Cost** | Minimal | High | Very High |
| **Templates** | No | No | Yes |
| **Best For** | Production | Development | Complex circuits |

## Quick Start

### CLI Usage

```bash
# Validate Zircon program
zkplex-cli --zircon "1/A:10,B:20/-/A+B>25"

# Show program info
zkplex-cli --zircon "1/age:25/-/age>=18" --info

# Estimate circuit complexity
zkplex-cli --zircon "1/balance:1000/-/balance>100" --estimate

# Convert to JSON
zkplex-cli --zircon "1/A:10/-/A>5" --into-json
```

### WASM API

```typescript
import { zircon_to_json, json_to_zircon, prove, estimate } from './zkplex_core.js';

// Parse Zircon
const json = zircon_to_json("1/A:10,B:20/-/A+B");

// Generate proof from Zircon
const request = JSON.parse(json);
const proof = prove(JSON.stringify(request));

// Estimate complexity
const estimation = estimate(JSON.stringify(request));
```

## Core Concepts

### Signals

**Secret (Witness)**: Secret values known only to prover
```
1/secret:12345/-/...
```

**Public**: Values visible to verifier
```
1/-/threshold:100/...
```

### Preprocessing

Operations executed before main circuit:
```
1/data:hello/-/hash<==sha256(data{%x})/hash==expected
```

### Circuit

Zero-knowledge constraints that must be satisfied:
```
1/A:10,B:20/-/A+B>25;A>0;B>0
```

## Design Philosophy

### Compactness First

Every character matters. Single-character delimiters, no whitespace, minimal syntax overhead.

### Explicit Over Implicit

Encoding formats explicitly specified. Clear signal visibility (secret/public).

### Composability

Multiple constraints can be combined. Preprocessing feeds into circuit logic.

### Safety

Type-safe operations. Overflow protection. Clear value size constraints.

## Version History

### Version 1 (Current)

- Basic arithmetic operators
- Comparison operators with range proofs
- Boolean operators
- Hash preprocessing
- Multiple encodings (decimal, hex, base58, base64)

## Next Steps

- **[Syntax Guide](SYNTAX.md)** - Detailed format specification
- **[Signals](SIGNALS.md)** - Working with secret and public signals
- **[Encoding](ENCODING.md)** - Value encoding formats
- **[Preprocessing](PREPROCESSING.md)** - Hash functions and transformations
- **[Circuit](CIRCUIT.md)** - Writing ZKP constraints
- **[Operators](OPERATORS.md)** - All supported operators
- **[Examples](EXAMPLES.md)** - Real-world use cases
- **[Tools](TOOLS.md)** - CLI and API reference
- **[Best Practices](BEST_PRACTICES.md)** - Optimization and patterns

## Community & Support

- **GitHub**: [zkplex-core](https://github.com/zkplex/zkplex-core)
- **Issues**: Report bugs and request features
- **Discussions**: Ask questions and share ideas

## License

MIT OR Apache-2.0