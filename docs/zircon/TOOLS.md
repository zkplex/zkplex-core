# Zircon Format - Tools

## Overview

This document covers the tools available for working with Zircon format:

1. **CLI Tool** - Command-line interface for ZKPlex
2. **WASM API** - WebAssembly bindings for browser/Node.js
3. **Rust Library** - Direct Rust crate usage

## CLI Tool: zkplex-cli

### Installation

```bash
# Build from source
cargo build --release

# Binary location
./target/release/zkplex-cli
```

### Basic Usage

```bash
zkplex-cli [OPTIONS]
```

### Format Conversion Options

#### Zircon Input

**From string**:
```bash
zkplex-cli -z "1/age:25/-/age>=18"
```

**From file**:
```bash
zkplex-cli -z examples/age_check.zircon
```

**Aliases**: `-z`, `--zircon`

#### JSON Input

**From string**:
```bash
zkplex-cli -j '{"version":1,"secret":{"age":25},"circuit":"age>=18"}'
```

**From file**:
```bash
zkplex-cli -j examples/age_check.json
```

**Aliases**: `-j`, `--json`

### Output Format Options

#### To Zircon

```bash
zkplex-cli -j input.json --into-zircon
```

**Output**: Zircon format string

#### To JSON

```bash
zkplex-cli -z input.zircon --into-json
```

**Output**: JSON representation

### Circuit Operations

#### Generate Circuit

```bash
zkplex-cli -z "1/A:10,B:20/-/A>B" --circuit
```

**Output**: R1CS constraints in JSON format

**Structure**:
```json
{
  "constraints": [...],
  "num_constraints": 68,
  "num_variables": 5
}
```

#### Estimate Constraints

```bash
zkplex-cli -z "1/A:10,B:20/-/A>B" --estimate
```

**Output**: Constraint count estimate

**Example**:
```
Estimated constraints: ~68
Breakdown:
  - A > B: 68 constraints (64-bit range check)
```

#### Combined: Circuit + Estimate

```bash
zkplex-cli -z "1/A:10,B:20/-/A>B" --circuit --estimate
```

**Output**: Full circuit JSON + constraint estimate

### Proof Operations

#### Generate Proof

ZKPlex supports **four input formats** for proof generation:

##### 1. Direct Circuit Expression

```bash
zkplex-cli --circuit "A + B > 100" \
  --secret A:50 \
  --public B:60 \
  --prove \
  --proof output.json
```

##### 2. Zircon Format (File or String)

```bash
# From file with placeholders
echo '1/A:?,B:20/-/A+B' > program.zrc
zkplex-cli --zircon program.zrc \
  --secret A:30 \
  --prove \
  --proof output.json

# Direct string
zkplex-cli --zircon "1/A:10,B:20/-/A+B>100" \
  --prove \
  --proof output.json
```

##### 3. JSON Format

```bash
# Create JSON program with placeholders
cat > program.json <<'EOF'
{
  "version": 1,
  "secret": {
    "secret": {
      "value": "?",
      "encoding": "base58"
    },
    "amount": {
      "value": "?"
    }
  },
  "public": {
    "threshold": {
      "value": "100"
    }
  },
  "circuit": ["amount > threshold"]
}
EOF

# Generate proof
zkplex-cli --json program.json \
  --secret secret:MySecret:base58 \
  --secret amount:150 \
  --prove \
  --proof output.json
```

#### Proof File Structure

Proofs are saved as JSON with **all necessary verification data**:

```json
{
  "proof": "base85-encoded-proof-bytes",
  "verification_context": "base85-encoded-context",
  "public_signals": {
    "B": {
      "value": "60"
    }
  }
}
```

**Key Features**:
- ✅ **No separate VK file needed** - verification context embedded
- ✅ **Self-contained** - includes circuit definition in verification_context
- ✅ **Public signals** - automatically extracted with encoding info

**Note:** The proof only generates if the circuit evaluates to true (1). If the circuit constraints are not satisfied, proof generation will fail with `ConstraintSystemFailure`.

#### Verify Proof

**Simple verification** (requires only proof file):

```bash
zkplex-cli --verify --proof output.json
```

**JSON output format**:

```bash
zkplex-cli --verify \
  --proof output.json \
  --into-json
```

**Output** (success):
```json
{
  "valid": true
}
```

**Output** (failure):
```json
{
  "valid": false,
  "error": "Verification failed: ..."
}
```

**Note:** The CLI may display additional information about the circuit evaluation during proof generation, but this is not part of the proof structure itself. The proof only verifies that the circuit constraints were satisfied.

#### Placeholder Support

Use `?` as placeholder for signal values that will be provided at proof generation time:

**Zircon format**:
```bash
# Create program with placeholders
echo '1/A:?,B:?/C:10/A+B==C' > check.zrc

# Provide values at proof time
zkplex-cli --zircon check.zrc \
  --secret A:5 \
  --secret B:5 \
  --prove \
  --proof output.json
```

**JSON format**:
```json
{
  "version": 1,
  "secret": {
    "password": {
      "value": "?",
      "encoding": "base58"
    }
  },
  "circuit": ["password == expected"]
}
```

**Placeholder with encoding**:
```bash
echo '1/wallet:?:base58/expected:abc:base58/wallet==expected' > verify.zrc
zkplex-cli --zircon verify.zrc \
  --secret wallet:xyz:base58 \
  --prove
```

**Error handling**:
```bash
# Missing placeholder value
zkplex-cli --zircon "1/A:?,B:10/-/A+B" --prove
# Error: Signal 'A' has placeholder '?' but no value provided via --secret
```

#### Preprocessing Support

Add preprocessing operations with `--preprocess`:

```bash
zkplex-cli --circuit "hash > 100" \
  --preprocess "hash<==sha256(secret{%x})" \
  --secret secret:255 \
  --prove \
  --proof output.json
```

### Examples

#### Example 1: Convert Zircon to JSON

```bash
zkplex-cli -z "1/age:25/-/age>=18" --into-json
```

**Output**:
```json
{
  "version": 1,
  "secret": {
    "age": {"value": "25", "encoding": "decimal"}
  },
  "public": {},
  "circuit": "age>=18"
}
```

#### Example 2: Estimate Constraints

```bash
zkplex-cli -z "1/A:10,B:20/-/sum<==A+B;sum>25" --estimate
```

**Output**:
```
Estimated constraints: ~69
Breakdown:
  - sum<==A+B: 1 constraint (assignment)
  - sum>25: 68 constraints (64-bit range check)
Total: 69 constraints
```

#### Example 3: Complex Preprocessing

```bash
zkplex-cli -z "1/password:secret/-/hash<==sha256(password{%s})/hash==expectedHash" --circuit
```

**Output**: Circuit with SHA-256 gadget (1000+ constraints)

#### Example 4: File Input/Output

```bash
# Read from file
zkplex-cli -z examples/balance_check.zircon --into-json > output.json
```

### File Format Detection

CLI automatically detects if input is a file path:

```bash
# File (has extension or exists)
zkplex-cli -z examples/age.zircon

# String (no extension, doesn't exist as file)
zkplex-cli -z "1/A:10/-/A>5"
```

### Error Handling

**Invalid format**:
```bash
zkplex-cli -z "invalid format"
# Error: Invalid Zircon format: expected 4 or 5 parts
```

**Undefined variable**:
```bash
zkplex-cli -z "1/A:10/-/B>5"
# Error: Undefined variable: B
```

**Value too large**:
```bash
zkplex-cli -z "1/huge:999999999999999999999/-/huge>100"
# Error: Value exceeds 2^64 for ordering comparison
```

### Help

```bash
zkplex-cli --help
```

**Output**:
```
ZKPlex CLI - Zero-Knowledge Proof Format Converter

USAGE:
    zkplex-cli [OPTIONS]

FORMAT CONVERSION OPTIONS:
    -z, --zircon <TEXT|FILE>     Input in Zircon format
    -j, --json <TEXT|FILE>       Input in JSON format

OUTPUT FORMAT OPTIONS:
    --into-json                  Convert to JSON format
    --into-zircon                Convert to Zircon format

CIRCUIT OPTIONS:
    --circuit                    Generate R1CS circuit
    --estimate                   Estimate constraint count

PROOF OPTIONS:
    --prove                      Generate zero-knowledge proof
    --verify <PROOF> <CIRCUIT>   Verify a proof

OTHER OPTIONS:
    --help                       Show this help message
    --version                    Show version information
```

## WASM API

### Installation

```bash
npm install zkplex-core
```

Or in browser:
```html
<script type="module">
  import init, { zircon_to_json } from './zkplex_core.js';
  await init();
</script>
```

### API Functions

#### zircon_to_json

Convert Zircon format to JSON.

**Signature**:
```typescript
function zircon_to_json(zircon: string): string
```

**Example**:
```javascript
import { zircon_to_json } from 'zkplex-core';

const zircon = "1/age:25/-/age>=18";
const json = zircon_to_json(zircon);
console.log(json);
// {"version":1,"secret":{"age":{"value":"25"}},"circuit":"age>=18"}
```

**Error handling**:
```javascript
try {
  const json = zircon_to_json("invalid");
} catch (error) {
  console.error("Parse error:", error);
}
```

#### json_to_zircon

Convert JSON to Zircon format.

**Signature**:
```typescript
function json_to_zircon(json: string): string
```

**Example**:
```javascript
import { json_to_zircon } from 'zkplex-core';

const json = JSON.stringify({
  version: 1,
  secret: { age: { value: "25" } },
  circuit: "age>=18"
});

const zircon = json_to_zircon(json);
console.log(zircon);
// 1/age:25/-/age>=18
```

#### generate_circuit

Generate R1CS circuit from Zircon.

**Signature**:
```typescript
function generate_circuit(zircon: string): string
```

**Example**:
```javascript
import { generate_circuit } from 'zkplex-core';

const zircon = "1/A:10,B:20/-/A>B";
const circuit = generate_circuit(zircon);
const circuitData = JSON.parse(circuit);

console.log("Constraints:", circuitData.num_constraints);
// Constraints: 68
```

#### estimate_constraints

Estimate constraint count for Zircon program.

**Signature**:
```typescript
function estimate_constraints(zircon: string): number
```

**Example**:
```javascript
import { estimate_constraints } from 'zkplex-core';

const zircon = "1/A:10,B:20/-/sum<==A+B;sum>25";
const count = estimate_constraints(zircon);
console.log("Estimated constraints:", count);
// Estimated constraints: 69
```

### Browser Example

**Complete example**:
```html
<!DOCTYPE html>
<html>
<head>
  <title>ZKPlex Zircon Converter</title>
</head>
<body>
  <h1>Zircon to JSON Converter</h1>

  <textarea id="zircon" rows="5" cols="60">
1/age:25/-/age>=18
  </textarea>

  <button onclick="convert()">Convert</button>

  <pre id="output"></pre>

  <script type="module">
    import init, { zircon_to_json, estimate_constraints } from './zkplex_core.js';

    await init();

    window.convert = function() {
      const zircon = document.getElementById('zircon').value.trim();

      try {
        const json = zircon_to_json(zircon);
        const estimate = estimate_constraints(zircon);

        const output = {
          json: JSON.parse(json),
          estimated_constraints: estimate
        };

        document.getElementById('output').textContent =
          JSON.stringify(output, null, 2);
      } catch (error) {
        document.getElementById('output').textContent =
          "Error: " + error;
      }
    };
  </script>
</body>
</html>
```

### Node.js Example

```javascript
// node_example.js
const { zircon_to_json, estimate_constraints } = require('zkplex-core');

const zircon = process.argv[2] || "1/age:25/-/age>=18";

try {
  const json = zircon_to_json(zircon);
  const estimate = estimate_constraints(zircon);

  console.log("JSON:", json);
  console.log("Estimated constraints:", estimate);
} catch (error) {
  console.error("Error:", error);
  process.exit(1);
}
```

**Run**:
```bash
node node_example.js "1/balance:1000/-/balance>100"
```

### TypeScript Definitions

```typescript
// zkplex-core.d.ts

export function zircon_to_json(zircon: string): string;
export function json_to_zircon(json: string): string;
export function generate_circuit(zircon: string): string;
export function estimate_constraints(zircon: string): number;
export function generate_proof(zircon: string): string;
export function verify_proof(proof: string, circuit: string): boolean;

export interface ZirconSignal {
  value: string;
  encoding?: 'decimal' | 'hex' | 'base58' | 'base64';
}

export interface ZirconProgram {
  version: number;
  secret?: Record<string, ZirconSignal>;
  public?: Record<string, ZirconSignal>;
  preprocess?: string;
  circuit: string;
}

export default function init(): Promise<void>;
```

## Rust Library

### Add Dependency

```toml
[dependencies]
zkplex-core = "0.1"
```

### Basic Usage

```rust
use zkplex_core::{
    parse_zircon,
    zircon_to_json,
    estimate_constraints,
};

fn main() {
    let zircon = "1/age:25/-/age>=18";

    // Parse Zircon
    match parse_zircon(zircon) {
        Ok(program) => {
            println!("Version: {}", program.version);
            println!("Secret signals: {:?}", program.secret);
            println!("Circuit: {}", program.circuit);
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }

    // Convert to JSON
    match zircon_to_json(zircon) {
        Ok(json) => println!("JSON: {}", json),
        Err(e) => eprintln!("Conversion error: {}", e),
    }

    // Estimate constraints
    match estimate_constraints(zircon) {
        Ok(count) => println!("Estimated: {} constraints", count),
        Err(e) => eprintln!("Estimation error: {}", e),
    }
}
```

### Advanced Usage

```rust
use zkplex_core::{
    ZirconProgram,
    generate_circuit,
    generate_proof,
    verify_proof,
};

fn full_workflow(zircon: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Parse program
    let program: ZirconProgram = parse_zircon(zircon)?;

    // Generate circuit
    let circuit = generate_circuit(&program)?;
    println!("Circuit has {} constraints", circuit.num_constraints);

    // Generate proof
    let proof = generate_proof(&program, circuit)?;
    println!("Proof generated: {} bytes", proof.len());

    // Verify proof
    let valid = verify_proof(&proof, &circuit)?;
    println!("Proof valid: {}", valid);

    Ok(())
}
```

## Integration Examples

### React App

```jsx
import React, { useState, useEffect } from 'react';
import init, { zircon_to_json, estimate_constraints } from 'zkplex-core';

function ZirconConverter() {
  const [initialized, setInitialized] = useState(false);
  const [zircon, setZircon] = useState("1/age:25/-/age>=18");
  const [output, setOutput] = useState(null);

  useEffect(() => {
    init().then(() => setInitialized(true));
  }, []);

  const convert = () => {
    try {
      const json = JSON.parse(zircon_to_json(zircon));
      const estimate = estimate_constraints(zircon);
      setOutput({ json, estimate });
    } catch (error) {
      setOutput({ error: error.toString() });
    }
  };

  if (!initialized) return <div>Loading WASM...</div>;

  return (
    <div>
      <textarea
        value={zircon}
        onChange={e => setZircon(e.target.value)}
        rows={5}
        cols={60}
      />
      <button onClick={convert}>Convert</button>
      {output && (
        <pre>{JSON.stringify(output, null, 2)}</pre>
      )}
    </div>
  );
}
```

### Express.js API

```javascript
const express = require('express');
const { zircon_to_json, estimate_constraints } = require('zkplex-core');

const app = express();
app.use(express.json());

app.post('/api/convert', (req, res) => {
  const { zircon } = req.body;

  try {
    const json = JSON.parse(zircon_to_json(zircon));
    const estimate = estimate_constraints(zircon);

    res.json({ json, estimate });
  } catch (error) {
    res.status(400).json({ error: error.toString() });
  }
});

app.listen(3000, () => {
  console.log('API running on port 3000');
});
```

**Usage**:
```bash
curl -X POST http://localhost:3000/api/convert \
  -H "Content-Type: application/json" \
  -d '{"zircon":"1/age:25/-/age>=18"}'
```

## Performance

### Constraint Estimation

| Operation | Constraints | Time (ms) |
|-----------|-------------|-----------|
| `A>B` | ~68 | <1 |
| `A==B` | ~3 | <1 |
| `sum<==A+B` | ~1 | <1 |
| `sha256(x)` | ~1000 | ~5 |
| `A>B;C>D;E>F` | ~204 | <1 |

### Parsing Performance

| Input Size | Parse Time | Memory |
|------------|------------|--------|
| Small (100 chars) | <1ms | <1KB |
| Medium (1KB) | ~2ms | ~5KB |
| Large (10KB) | ~20ms | ~50KB |

### WASM Bundle Size

| Component | Size (gzipped) |
|-----------|----------------|
| Core WASM | ~150KB |
| JS Glue | ~10KB |
| **Total** | **~160KB** |

## Troubleshooting

### Issue: WASM not loading in browser

**Solution**: Ensure proper MIME type:
```javascript
// webpack.config.js
module.exports = {
  experiments: {
    asyncWebAssembly: true
  }
};
```

### Issue: Module not found in Node.js

**Solution**: Use proper import:
```javascript
// CommonJS
const zkplex = require('zkplex-core');

// ES Modules
import * as zkplex from 'zkplex-core';
```

### Issue: Large constraint count

**Solution**: Optimize circuit:
- Use `==` instead of `>=` + `<=` for exact values
- Minimize range checks
- Combine operations where possible

## See Also

- **[Syntax](SYNTAX.md)** - Zircon format syntax
- **[Examples](EXAMPLES.md)** - Usage examples
- **[Best Practices](BEST_PRACTICES.md)** - Optimization tips
- **[Operators](OPERATORS.md)** - Operator reference