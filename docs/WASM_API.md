# ZKPlex WASM API Documentation

## Overview

This document provides complete documentation for the ZKPlex WASM API, including compilation instructions, all available methods, TypeScript bindings, and usage examples.

## Table of Contents

1. [Building WASM](#building-wasm)
2. [Installation](#installation)
3. [API Methods](#api-methods)
4. [TypeScript API](#typescript-api)
5. [Usage Examples](#usage-examples)
6. [CLI vs WASM Comparison](#cli-vs-wasm-comparison)
7. [Performance](#performance)
8. [Error Handling](#error-handling)

## Building WASM

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Add wasm32 target
rustup target add wasm32-unknown-unknown
```

### Build for Web

```bash
# Build for web browsers
wasm-pack build --target web

# Build with optimizations
wasm-pack build --target web --release

# Output: pkg/zkplex_core.js, pkg/zkplex_core_bg.wasm
```

### Build for Node.js

```bash
# Build for Node.js
wasm-pack build --target nodejs

# Output: pkg/zkplex_core.js (CommonJS), pkg/zkplex_core_bg.wasm
```

### Build for Bundlers

```bash
# Build for webpack/rollup
wasm-pack build --target bundler
```

### Output Files

After building, you'll have:

```
pkg/
├── zkplex_core.js          # JavaScript bindings
├── zkplex_core_bg.wasm     # WebAssembly binary
├── zkplex_core.d.ts        # TypeScript definitions
└── package.json            # NPM package manifest
```

## Installation

### Browser (ES Modules)

```html
<script type="module">
  import init, { zircon_to_json } from './pkg/zkplex_core.js';

  await init();

  const json = zircon_to_json("1/age:25/-/age>=18");
  console.log(json);
</script>
```

### Node.js

```javascript
// CommonJS
const { zircon_to_json } = require('./pkg/zkplex_core.js');

// ES Modules
import init, { zircon_to_json } from './pkg/zkplex_core.js';
await init();
```

### With TypeScript Wrapper

```typescript
import { ZKPlex } from './zkplex.js';

const zkp = await ZKPlex.init();
const json = zkp.zirconToJson("1/age:25/-/age>=18");
```

## API Methods

### Workflow Overview

ZKPlex WASM API follows a step-by-step workflow similar to the CLI:

**Proof Generation:**
1. `parse_zircon()` or `parse_json()` → Parse to Program
2. `apply_overrides()` → Replace placeholders (optional)
3. `program_to_request()` → Convert to ProveRequest
4. `prove()` → Generate proof

**Verification:**
1. `response_to_verify_request()` → Extract verification data
2. `verify()` → Verify proof

**Estimation:**
1. `parse_zircon()` or `parse_json()` → Parse to Program
2. `program_to_estimate_request()` → Convert to EstimateRequest
3. `estimate()` → Get circuit requirements

### Core Methods

#### `prove(request_json: string) -> string`

Generate a zero-knowledge proof.

**Parameters:**
- `request_json`: JSON string with circuit and signals (ProveRequest format)

**Returns:** JSON string with proof, verification context, and public signals (ProveResponse format)

**Example:**
```javascript
import { prove } from './pkg/zkplex_core.js';

const request = JSON.stringify({
  circuit: ["age >= 18"],
  signals: {
    age: { value: "25", public: false },
    result: { public: true }  // Output signal
  },
  strategy: "auto"
});

const response = JSON.parse(prove(request));
console.log("Proof:", response.proof);
console.log("Verification Context:", response.verify_context);
console.log("Public Signals:", response.public_signals);
```

#### `verify(request_json: string) -> string`

Verify a zero-knowledge proof.

**Parameters:**
- `request_json`: JSON string with proof, verification context, and public signals (VerifyRequest format)

**Returns:** JSON string with verification result (VerifyResponse format)

**Example:**
```javascript
import { verify } from './pkg/zkplex_core.js';

const request = JSON.stringify({
  version: response.version,
  proof: response.proof,
  verify_context: response.verify_context,
  public_signals: response.public_signals
});

const result = JSON.parse(verify(request));
console.log("Valid:", result.valid);
```

#### `estimate(request_json: string) -> string`

Estimate circuit requirements.

**Parameters:**
- `request_json`: JSON string with circuit and signals (same format as ProveRequest)

**Returns:** JSON string with estimation metrics (EstimateResponse format)

**Example:**
```javascript
import { estimate } from './pkg/zkplex_core.js';

const request = JSON.stringify({
  circuit: ["(A + B) * C > D"],
  signals: {
    A: { value: "10", public: false },
    B: { value: "20", public: false },
    C: { value: "2", public: false },
    D: { value: "50", public: true }
  }
});

const estimation = JSON.parse(estimate(request));
console.log("Required k:", estimation.k);
console.log("Estimated rows:", estimation.estimated_rows);
console.log("Proof size:", estimation.proof_size_bytes, "bytes");
console.log("Complexity:", estimation.complexity);
```

### Step-by-Step Workflow Methods

#### `parse_zircon(zircon: string) -> string`

Parse Zircon format to Program (Step 1 for Zircon input).

**Parameters:**
- `zircon`: Zircon format string (e.g., `"1/age:?/-/age>=18"`)

**Returns:** JSON string representation of Program

**Example:**
```javascript
import { parse_zircon } from './pkg/zkplex_core.js';

const program = parse_zircon("1/age:?/-/age>=18");
console.log(JSON.parse(program));
// {
//   version: 1,
//   secret: { age: { value: "?" } },
//   circuit: ["age>=18"]
// }
```

#### `parse_json(json: string) -> string`

Parse and validate JSON Program (Step 1 for JSON input).

**Parameters:**
- `json`: JSON string representation of Program

**Returns:** Normalized JSON string representation of Program

**Example:**
```javascript
import { parse_json } from './pkg/zkplex_core.js';

const input = JSON.stringify({
  version: 1,
  secret: { age: { value: "?" } },
  circuit: ["age>=18"]
});

const program = parse_json(input);
```

#### `apply_overrides(program_json: string, overrides_json: string) -> string`

Apply signal overrides to a Program (Step 2 - replaces placeholders).

**Parameters:**
- `program_json`: JSON string representation of Program
- `overrides_json`: JSON object with signal overrides

**Returns:** Updated JSON string representation of Program

**Example:**
```javascript
import { parse_zircon, apply_overrides } from './pkg/zkplex_core.js';

// Step 1: Parse Zircon template with placeholders
const program = parse_zircon("1/age:?/-/age>=18");

// Step 2: Apply overrides to replace "?"
const overrides = JSON.stringify({
  age: { value: "25", public: false }
});

const updated = apply_overrides(program, overrides);
console.log(JSON.parse(updated));
// { version: 1, secret: { age: { value: "25" } }, circuit: ["age>=18"] }
```

#### `program_to_request(program_json: string, strategy?: string) -> string`

Convert Program to ProveRequest (Step 3 for proof generation).

**Parameters:**
- `program_json`: JSON string representation of Program
- `strategy`: Optional proof strategy ("auto", "boolean", "lookup", "bitd")

**Returns:** JSON string representation of ProveRequest

**Example:**
```javascript
import { parse_zircon, program_to_request, prove } from './pkg/zkplex_core.js';

// Step 1: Parse Zircon
const program = parse_zircon("1/age:25/-/age>=18");

// Step 2: Convert to ProveRequest
const request = program_to_request(program, "auto");

// Step 3: Generate proof
const proof = prove(request);
```

#### `program_to_estimate_request(program_json: string, strategy?: string) -> string`

Convert Program to EstimateRequest (Step 3 for estimation).

**Parameters:**
- `program_json`: JSON string representation of Program
- `strategy`: Optional proof strategy ("auto", "boolean", "lookup", "bitd")

**Returns:** JSON string representation of EstimateRequest (same as ProveRequest)

**Example:**
```javascript
import { parse_zircon, program_to_estimate_request, estimate } from './pkg/zkplex_core.js';

// Step 1: Parse Zircon
const program = parse_zircon("1/age:25/-/age>=18");

// Step 2: Convert to EstimateRequest
const request = program_to_estimate_request(program, "auto");

// Step 3: Estimate circuit
const estimation = estimate(request);
console.log(JSON.parse(estimation));
```

#### `response_to_verify_request(prove_response_json: string) -> string`

Extract verification data from ProveResponse (Step 1 for verification).

**Parameters:**
- `prove_response_json`: JSON string representation of ProveResponse

**Returns:** JSON string representation of VerifyRequest

**Example:**
```javascript
import { prove, response_to_verify_request, verify } from './pkg/zkplex_core.js';

// Step 1: Generate proof
const proveResponse = prove(request);

// Step 2: Convert to VerifyRequest
const verifyRequest = response_to_verify_request(proveResponse);

// Step 3: Verify proof
const result = verify(verifyRequest);
console.log(JSON.parse(result).valid);  // true
```

### Format Conversion Methods

#### `zircon_to_json(zircon: string) -> string`

Convert Zircon format to JSON.

**Parameters:**
- `zircon`: Zircon format string (e.g., `"1/age:25/-/age>=18"`)

**Returns:** JSON string representation

**Example:**
```javascript
import { zircon_to_json } from './pkg/zkplex_core.js';

const json = zircon_to_json("1/age:25/-/age>=18");
console.log(JSON.parse(json));
// {
//   version: 1,
//   secret: { age: { value: "25" } },
//   circuit: ["age>=18"]
// }
```

#### `json_to_zircon(json: string) -> string`

Convert JSON to Zircon format.

**Parameters:**
- `json`: JSON string representation of program

**Returns:** Zircon format string

**Example:**
```javascript
import { json_to_zircon } from './pkg/zkplex_core.js';

const json = JSON.stringify({
  version: 1,
  secret: { age: { value: "25" } },
  circuit: ["age>=18"]
});

const zircon = json_to_zircon(json);
console.log(zircon); // "1/age:25/-/age>=18"
```

#### `estimate(request_json: string) -> string`

Estimate circuit requirements.

**Parameters:**
- `request_json`: JSON string with circuit and signals

**Returns:** JSON string with estimation metrics

**Example:**
```javascript
import { estimate } from './pkg/zkplex_core.js';

const request = JSON.stringify({
  circuit: "A > B",
  signals: {
    A: { value: "10", public: false },
    B: { value: "5", public: true }
  }
});

const metrics = JSON.parse(estimate(request));
console.log("K:", metrics.k);
console.log("Constraints:", metrics.estimated_constraints);
console.log("Proof size:", metrics.proof_size_bytes, "bytes");
```

#### `estimate_constraints(zircon: string) -> number`

Estimate constraint count for Zircon program.

**Parameters:**
- `zircon`: Zircon format string

**Returns:** Number of estimated constraints

**Example:**
```javascript
import { estimate_constraints } from './pkg/zkplex_core.js';

const count = estimate_constraints("1/balance:1000/-/balance>100");
console.log("Constraints:", count); // 68
```

#### `generate_circuit(zircon: string) -> string`

Generate circuit information from Zircon.

**Parameters:**
- `zircon`: Zircon format string

**Returns:** JSON string with circuit info

**Example:**
```javascript
import { generate_circuit } from './pkg/zkplex_core.js';

const circuit = JSON.parse(generate_circuit("1/A:10,B:20/-/A>B"));
console.log("Constraints:", circuit.num_constraints);
console.log("Variables:", circuit.num_variables);
console.log("Complexity:", circuit.complexity);
```

## TypeScript API

The TypeScript wrapper provides a clean, type-safe interface:

### Installation

```typescript
import { ZKPlex } from './zkplex.js';
```

### Initialization

```typescript
const zkp = await ZKPlex.init();
```

### Proof Generation

```typescript
const proof = await zkp.prove({
  circuit: "age >= 18",
  signals: {
    age: { value: "25", public: false }
  }
});
```

### Proof Verification

```typescript
const output = await zkp.verify({
  proof: proof.proof,
  verification_context: proof.verification_context,
  public_signals: proof.public_signals
});

console.log("Valid:", output.valid);
```

### Format Conversions

```typescript
// Zircon <-> JSON
const json = zkp.zirconToJson("1/age:25/-/age>=18");
const zircon = zkp.jsonToZircon(json);
```

### Circuit Analysis

```typescript
// Estimate constraints
const count = zkp.estimateConstraints("1/balance:1000/-/balance>100");

// Generate circuit info
const circuit = zkp.generateCircuit("1/A:10,B:20/-/sum<==A+B;sum>25");
console.log("Constraints:", circuit.num_constraints);

// Full estimation
const metrics = await zkp.estimate({
  circuit: "A > B",
  signals: {
    A: { value: "10", public: false },
    B: { value: "5", public: true }
  }
});
```

## Usage Examples

### Complete Workflow Example: Template-Based Proof Generation

This example demonstrates the full workflow using placeholders and step-by-step methods:

```javascript
import {
  parse_zircon,
  apply_overrides,
  program_to_request,
  prove,
  response_to_verify_request,
  verify
} from './pkg/zkplex_core.js';

// Step 1: Parse Zircon template with placeholders
const template = "1/age:?,name:?/threshold:18/-/age>=threshold";
const program = parse_zircon(template);
console.log("Program:", JSON.parse(program));
// {
//   version: 1,
//   secret: { age: { value: "?" }, name: { value: "?" } },
//   public: { threshold: { value: "18" } },
//   circuit: ["age>=threshold"]
// }

// Step 2: Apply signal overrides (replace placeholders)
const overrides = JSON.stringify({
  age: { value: "25", public: false },
  name: { value: "Alice", public: false }
});
const filledProgram = apply_overrides(program, overrides);
console.log("Filled Program:", JSON.parse(filledProgram));
// {
//   version: 1,
//   secret: { age: { value: "25" }, name: { value: "Alice" } },
//   public: { threshold: { value: "18" } },
//   circuit: ["age>=threshold"]
// }

// Step 3: Convert Program to ProveRequest
const proveRequest = program_to_request(filledProgram, "auto");
console.log("ProveRequest ready");

// Step 4: Generate proof
const proveResponse = prove(proveRequest);
const proof = JSON.parse(proveResponse);
console.log("Proof generated:", proof.proof.substring(0, 50) + "...");
console.log("Public signals:", proof.public_signals);

// Step 5: Convert ProveResponse to VerifyRequest
const verifyRequest = response_to_verify_request(proveResponse);
console.log("VerifyRequest ready");

// Step 6: Verify proof
const verifyResponse = verify(verifyRequest);
const result = JSON.parse(verifyResponse);
console.log("Proof valid:", result.valid);  // true
```

**Output:**
```
Program: { version: 1, secret: { age: { value: "?" }, ... }, ... }
Filled Program: { version: 1, secret: { age: { value: "25" }, ... }, ... }
ProveRequest ready
Proof generated: 9jqo^BlbD-BleB1DJ+Blm0Ci*9+Bnp9B...
Public signals: { threshold: { value: "18", encoding: null } }
VerifyRequest ready
Proof valid: true
```

### Complete Workflow Example: Estimation

```javascript
import {
  parse_zircon,
  program_to_estimate_request,
  estimate
} from './pkg/zkplex_core.js';

// Step 1: Parse Zircon
const program = parse_zircon("1/balance:1000,amount:50/fee:5/-/(balance-amount)>fee");

// Step 2: Convert to EstimateRequest
const estimateRequest = program_to_estimate_request(program, "auto");

// Step 3: Get estimation
const estimation = JSON.parse(estimate(estimateRequest));
console.log("Circuit estimation:");
console.log("  Complexity:", estimation.complexity);
console.log("  Required k:", estimation.k);
console.log("  Estimated rows:", estimation.estimated_rows);
console.log("  Proof size:", estimation.proof_size_bytes, "bytes");
console.log("  Operations:", estimation.operation_count);
console.log("  Comparisons:", estimation.comparison_count);
```

### Example 1: Age Verification

```typescript
import { ZKPlex } from './zkplex.js';

const zkp = await ZKPlex.init();

// Generate proof - circuit must evaluate to true (1) for proof to be generated
// If age < 18, proof generation will fail with ConstraintSystemFailure
const proof = await zkp.prove({
  circuit: ["age >= 18"],
  signals: {
    age: { value: "25", public: false }
  }
});

// Proof exists = age is >= 18
console.log("Proof generated successfully - age is >= 18");
```

### Example 2: Balance Check

```typescript
const proof = await zkp.prove({
  circuit: "balance > threshold",
  signals: {
    balance: { value: "1000", public: false },
    threshold: { value: "100", public: true }
  }
});

// Verify
const valid = await zkp.verify({
  proof: proof.proof,
  verification_context: proof.verification_context,
  public_signals: proof.public_signals
});
```

### Example 3: Range Proof

```typescript
// Proof only generates if value is in range [100, 200]
const proof = await zkp.prove({
  circuit: ["value >= minVal", "value <= maxVal"],
  signals: {
    value: { value: "150", public: false },
    minVal: { value: "100", public: true },
    maxVal: { value: "200", public: true }
  }
});

// Proof exists = value is in range
console.log("Proof generated successfully - value is in range [100, 200]");
console.log("Range bounds:", proof.public_signals.minVal.value, "-", proof.public_signals.maxVal.value);
```

### Example 4: Ethereum Address

```typescript
const proof = await zkp.prove({
  circuit: "myAddress == targetAddress",
  signals: {
    myAddress: {
      value: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
      encoding: "hex",
      public: false
    },
    targetAddress: {
      value: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
      encoding: "hex",
      public: true
    }
  }
});
```

### Example 5: Preprocessing with Hash

```typescript
// Using Zircon format with preprocessing
const zircon = "1/password:secret123/-/hash<==sha256(password{%s})/hash==expectedHash";

// Estimate constraints
const count = zkp.estimateConstraints(zircon);
console.log("Constraints (with SHA-256):", count); // ~1000+

// Generate circuit info
const circuit = zkp.generateCircuit(zircon);
console.log("Complexity:", circuit.complexity);
```

### Example 6: Format Conversion Pipeline

```typescript
// Start with Zircon
const zircon = "1/A:10,B:20/-/sum<==A+B;sum>25";

// Convert to JSON
const json = zkp.zirconToJson(zircon);
console.log("JSON:", json);
```

## CLI vs WASM Comparison

Complete feature parity between CLI and WASM:

| Feature | CLI | WASM | Method |
|---------|-----|------|--------|
| **Format Conversion** ||||
| Zircon → JSON | ✅ | ✅ | `zircon_to_json()` |
| JSON → Zircon | ✅ | ✅ | `json_to_zircon()` |
| **Circuit Analysis** ||||
| Estimate constraints | ✅ | ✅ | `estimate_constraints()` |
| Generate circuit | ✅ | ✅ | `generate_circuit()` |
| Full estimation | ✅ | ✅ | `estimate()` |
| **Proof Operations** ||||
| Generate proof | ✅ | ✅ | `prove()` |
| Verify proof | ✅ | ✅ | `verify()` |

### Usage Comparison

**WASM:**
```javascript
const count = estimate_constraints("1/age:25/-/age>=18");
// Returns: 68
```

**TypeScript:**
```typescript
const count = zkp.estimateConstraints("1/age:25/-/age>=18");
// Returns: 68
```

For CLI usage examples, see [CLI Documentation](CLI.md).

## Performance

### Bundle Size

| Component | Size (gzipped) |
|-----------|----------------|
| WASM binary | ~150KB |
| JS glue code | ~10KB |
| TypeScript wrapper | ~5KB |
| **Total** | **~165KB** |

### Operation Times

| Operation | Time | Notes |
|-----------|------|-------|
| Parse Zircon | <1ms | Simple programs |
| Parse Zircon | ~5ms | Complex with preprocessing |
| Estimate constraints | <1ms | Without circuit generation |
| Generate circuit | ~10ms | Full circuit info |
| Prove (simple) | ~100ms | age >= 18 |
| Prove (complex) | ~500ms | Multiple constraints |
| Verify | ~50ms | Typical |

### Memory Usage

| Operation | Memory |
|-----------|--------|
| Module load | ~2MB |
| Small circuit | ~5MB |
| Medium circuit | ~50MB |
| Large circuit | ~200MB |

## Error Handling

### Try-Catch Pattern

```typescript
try {
  const json = zkp.zirconToJson("invalid format");
} catch (error) {
  console.error("Parse error:", error.message);
}
```

### Common Errors

**Invalid Zircon format:**
```javascript
// Error: Failed to parse zircon format: expected 4 or 5 parts
zircon_to_json("invalid");
```

**Undefined variable:**
```javascript
// Error: Undefined variable: B
zircon_to_json("1/A:10/-/B>5");
```

**Value too large for ordering:**
```javascript
// Error: Value exceeds 2^64 for ordering comparison
const huge = "9aE476sH92Vc7DMC8bZNpe1xNNNy1fNjFpCGvfMuZMwM";
zircon_to_json(`1/addr:${huge}:base58/-/addr>0`);
```

### Error Response Format

Verification errors return structured responses:

```typescript
{
  valid: false,
  error: "Verification failed: ..."
}
```

## Advanced Usage

### React Integration

```tsx
import { ZKPlex } from './zkplex.js';
import { useState, useEffect } from 'react';

function ZirconConverter() {
  const [zkp, setZkp] = useState<ZKPlex | null>(null);
  const [zircon, setZircon] = useState("1/age:25/-/age>=18");
  const [output, setOutput] = useState(null);

  useEffect(() => {
    ZKPlex.init().then(setZkp);
  }, []);

  const convert = () => {
    if (!zkp) return;

    try {
      const json = zkp.zirconToJson(zircon);
      const count = zkp.estimateConstraints(zircon);
      setOutput({ json, count });
    } catch (error) {
      setOutput({ error: error.message });
    }
  };

  return (
    <div>
      <textarea value={zircon} onChange={e => setZircon(e.target.value)} />
      <button onClick={convert}>Convert</button>
      {output && <pre>{JSON.stringify(output, null, 2)}</pre>}
    </div>
  );
}
```

### Node.js Script

```javascript
#!/usr/bin/env node

const { ZKPlex } = require('./zkplex.js');

async function main() {
  const zkp = await ZKPlex.init();

  const zircon = process.argv[2];
  if (!zircon) {
    console.error('Usage: node script.js <zircon>');
    process.exit(1);
  }

  try {
    const json = zkp.zirconToJson(zircon);
    const count = zkp.estimateConstraints(zircon);

    console.log('JSON:', JSON.stringify(json, null, 2));
    console.log('Constraints:', count);
  } catch (error) {
    console.error('Error:', error.message);
    process.exit(1);
  }
}

main();
```

### Layout Methods

#### `get_layout(program_json: string, strategy?: string) -> string`

Get complete circuit layout information as JSON.

**Parameters:**
- `program_json`: JSON string representing a Program
- `strategy` (optional): Strategy to use ("auto", "boolean", "lookup", "bitd")

**Returns:** JSON string with complete circuit layout information including:
- Circuit parameters (k, total_rows, max_bits)
- Row layout breakdown (range table, circuit gates, unused rows)
- Resource requirements (params size, proof size, VK size)
- Signal information (secret and public signals)
- Operation breakdown (arithmetic, comparisons, preprocessing)
- Column configuration (advice, instance, selector, fixed columns)
- Gate type breakdown (arithmetic, comparison, preprocessing gates)
- Lookup table information (table sizes, overhead percentage)
- Memory usage estimates (prover and verifier memory requirements)
- Complexity analysis (timing estimates, optimization suggestions)

**Example:**
```javascript
import { get_layout } from './pkg/zkplex_core.js';

const program = JSON.stringify({
  version: 1,
  secret: { age: { value: "25", encoding: "Decimal" } },
  public: { result: { value: null, encoding: "Decimal" } },
  preprocess: [],
  circuit: ["age>=18"]
});

const layoutJson = get_layout(program, "auto");
const layout = JSON.parse(layoutJson);

console.log("Circuit:", layout.circuit);
console.log("Strategy:", layout.strategy);
console.log("Parameters:");
console.log("  k =", layout.parameters.k);
console.log("  Total rows =", layout.parameters.total_rows);
console.log("  Range check bits =", layout.parameters.max_bits);

console.log("\nRow Layout:");
console.log("  Circuit rows:", layout.row_layout.circuit_rows, 
            `(${layout.row_layout.circuit_percent.toFixed(1)}%)`);
console.log("  Unused rows:", layout.row_layout.unused_rows,
            `(${layout.row_layout.unused_percent.toFixed(1)}%)`);
console.log("  Total utilization:", 
            `${layout.row_layout.utilization_percent.toFixed(1)}%`);

console.log("\nResources:");
console.log("  Proving Key:", layout.resources.params_size_kb, "KB");
console.log("  Proof size:", layout.resources.proof_size_kb.toFixed(1), "KB");
console.log("  Verification Key:", layout.resources.vk_size_kb.toFixed(1), "KB");

console.log("\nMemory Usage:");
console.log("  Prover:", layout.memory.prover.total_mb.toFixed(1), "MB");
console.log("  Verifier:", layout.memory.verifier.total_kb.toFixed(1), "KB");

console.log("\nComplexity:");
console.log("  Overall:", layout.complexity.overall);
console.log("  Prover time:", layout.complexity.prover_time);
console.log("  Verifier time:", layout.complexity.verifier_time);

if (layout.complexity.optimization_suggestions.length > 0) {
  console.log("\nOptimization Suggestions:");
  layout.complexity.optimization_suggestions.forEach(s => {
    console.log("  •", s);
  });
}
```

#### `get_layout_ascii(program_json: string, strategy?: string) -> string`

Get circuit layout as ASCII art visualization.

**Parameters:**
- `program_json`: JSON string representing a Program
- `strategy` (optional): Strategy to use ("auto", "boolean", "lookup", "bitd")

**Returns:** ASCII art string with complete circuit layout visualization

**Example:**
```javascript
import { get_layout_ascii } from './pkg/zkplex_core.js';

const program = JSON.stringify({
  version: 1,
  secret: { 
    A: { value: "10", encoding: "Decimal" },
    B: { value: "20", encoding: "Decimal" },
    C: { value: "2", encoding: "Decimal" }
  },
  public: { 
    computed_value: { value: "60", encoding: "Decimal" },
    threshold: { value: "50", encoding: "Decimal" },
    check: { value: null, encoding: "Decimal" }
  },
  preprocess: [],
  circuit: ["computed_value<==(A+B)*C", "computed_value>threshold"]
});

const ascii = get_layout_ascii(program, "auto");
console.log(ascii);

// Output:
// ╔════════════════════════════════════════════════════════════╗
// ║          ZKPlex Circuit Layout Visualization               ║
// ╚════════════════════════════════════════════════════════════╝
//
// Circuit: computed_value<==(A+B)*C; computed_value>threshold
// Strategy: auto - Adaptive strategy (automatically chooses optimal based on circuit)
//
// Parameters:
//   k = 11 (2^11 = 2048 total rows)
//   Range check bits: 16
//
// Row Layout:
//
// ┌────────────────────────────────────────────────────────────┐
// │                     RANGE CHECK TABLE                      │
// │              16-bit lookup (rows 0-65535)                  │
// │                   65536 rows (3.2%)                        │
// ├────────────────────────────────────────────────────────────┤
// │                       CIRCUIT GATES                        │
// │                     rows 65536-66559                       │
// │                    1024 rows (50.0%)                       │
// │                      Arithmetic: 2                         │
// │                      Comparisons: 1                        │
// ├────────────────────────────────────────────────────────────┤
// │                      UNUSED (padding)                      │
// │                     rows 66560-2047                        │
// │                    958 rows (46.8%)                        │
// └────────────────────────────────────────────────────────────┘
//
// Utilization:
// [█▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░░░░░░░░░░░░░░░░░░░░░░░░░░░░]
//  █ Range Table  ▓ Circuit Gates  ░ Unused
//
// Total utilization: 53.2% (1088/2048 rows)
//
// ... (continues with all 9 sections)
```

**Use Cases:**
- Display layout in terminal or logs
- Generate reports
- Debug circuit structure
- Analyze resource requirements before proof generation
- Visualize circuit complexity for optimization

**Layout Sections:**
1. **Row Layout** - Visual diagram showing range check table, circuit gates, and unused rows
2. **Resource Requirements** - Proving key, proof size, verification key sizes
3. **Signal Information** - Secret and public signal breakdown
4. **Operation Breakdown** - Arithmetic, comparison, and preprocessing operations
5. **Column Configuration** - Advice, instance, selector, and fixed columns
6. **Gate Type Breakdown** - Detailed breakdown by gate types
7. **Lookup Table Information** - Table sizes and overhead (if using lookup strategy)
8. **Memory Usage Estimate** - Prover and verifier memory requirements
9. **Complexity Analysis** - Timing estimates and optimization suggestions


## See Also

- **[CLI Documentation](CLI.md)** - Command-line interface reference
- **[Zircon Documentation](zircon/)** - Zircon format specification
- **[Examples](zircon/EXAMPLES.md)** - Usage examples
- **[Best Practices](zircon/BEST_PRACTICES.md)** - Optimization tips