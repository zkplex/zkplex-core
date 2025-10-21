# Shared Architecture: CLI & WASM

## Yes! CLI and WASM use IDENTICAL logic

Both CLI and WASM are **thin wrappers** around the same core API (`src/api/core.rs`).

---

## New Architecture (with `src/api/core.rs`)

```
┌─────────────────────────────────────────────────────────────┐
│                        User Interfaces                      │
├─────────────────────────┬───────────────────────────────────┤
│  CLI (zkplex-cli.rs)    │    WASM (bindings.rs)             │
│  - Argument parsing     │    - JSON parsing                 │
│  - File I/O             │    - JsValue conversion           │
│  - stdout/stderr        │    - Error handling for JS        │
└─────────────────────────┴───────────────────────────────────┘
                             ▼
┌─────────────────────────────────────────────────────────────┐
│         🔥 SINGLE SOURCE OF TRUTH: api/core.rs 🔥           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  pub fn prove(request: ProveRequest)                        │
│      -> Result<ProveResponse, String>                       │
│                                                             │
│  pub fn verify(request: VerifyRequest)                      │
│      -> Result<VerifyResponse, String>                      │
│                                                             │
│  pub fn estimate(request: ProveRequest)                     │
│      -> Result<EstimateResponse, String>                    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
                             ▼
┌─────────────────────────────────────────────────────────────┐
│              SHARED CORE MODULES (zkplex-core)              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  circuit/ - Circuit building and constraint logic   │  │
│  │  ├─ builder.rs        Circuit::from_program()       │  │
│  │  ├─ strategy.rs       validate_strategy_compat...() │  │
│  │  ├─ estimator.rs      estimate_circuit_require...() │  │
│  │  ├─ mod.rs            CircuitAuto, CircuitBoolean,  │  │
│  │  │                    CircuitBitD, CircuitLookup    │  │
│  │  └─ evaluate.rs       evaluate_expression()         │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  parser/ - Expression parsing                       │  │
│  │  ├─ circuit.rs        parse_circuit()               │  │
│  │  └─ mod.rs            AST types                     │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  encoding/ - Value encoding/decoding                │  │
│  │  └─ value.rs          parse_value(), bytes_to_*()   │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  api/ - Data types and core logic                   │  │
│  │  ├─ types.rs          Signal, ProveRequest/Response,│  │
│  │  │                    VerifyRequest/Response,       │  │
│  │  │                    VerificationContext, etc.     │  │
│  │  ├─ program.rs        Program, Signal types         │  │
│  │  └─ core.rs          🔥 CORE API: prove(), verify(),│  │
│  │                       estimate() - SHARED LOGIC     │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                    HALO2 (external crate)                   │
├─────────────────────────────────────────────────────────────┤
│  - Params::new(k)           - keygen_vk()                   │
│  - keygen_pk()              - create_proof()                │
│  - verify_proof()           - SingleVerifier                │
└─────────────────────────────────────────────────────────────┘
```

---

## Implementation Comparison

### ✅ **Everything is now identical:**

| Component | CLI | WASM | Source |
|-----------|-----|------|--------|
| **Proof generation** | `zkplex_core::api::core::prove()` | `crate::api::core::prove()` | `src/api/core.rs:30` |
| **Proof verification** | `zkplex_core::api::core::verify()` | `crate::api::core::verify()` | `src/api/core.rs:173` |
| **Circuit estimation** | `zkplex_core::api::core::estimate()` | `crate::api::core::estimate()` | `src/api/core.rs:290` |

### ⚠️ **Differences ONLY in wrappers:**

| Aspect | CLI | WASM | Reason |
|--------|-----|------|--------|
| **Input parsing** | Parses files, command-line arguments → creates `ProveRequest` | Parses JSON string → creates `ProveRequest` | Different data sources |
| **Error handling** | `String` → `eprintln!()` + `process::exit(1)` | `String` → `JsValue` | Different execution environments |
| **Output** | Pretty-printed JSON to file or stdout | JSON string → JsValue | Different I/O interfaces |

---

## Example: Proof Verification

### CLI (`src/bin/zkplex-cli.rs:1250`):
```rust
fn verify_proof(proof_file: &str, into_json: bool) {
    // 1. Read proof file
    let json = fs::read_to_string(proof_file)?;

    // 2. Parse to ProveResponse
    let prove_response: ProveResponse = serde_json::from_str(&json)?;

    // 3. Create VerifyRequest
    let verify_request = VerifyRequest {
        version: prove_response.version,
        proof: prove_response.proof,
        verification_context: prove_response.verification_context,
        public_signals: prove_response.public_signals,
    };

    // 4. 🔥 Call SHARED core function
    let verify_response = zkplex_core::api::core::verify(verify_request)?;

    // 5. Output result
    if verify_response.valid {
        println!("✓ Proof is VALID");
    } else {
        eprintln!("✗ Proof is INVALID");
    }
}
```

### WASM (`src/wasm/bindings.rs:156`):
```rust
#[wasm_bindgen]
pub fn verify(request_json: &str) -> Result<String, JsValue> {
    // 1. Parse request from JSON string
    let request: VerifyRequest = serde_json::from_str(request_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse request: {}", e)))?;

    // 2. 🔥 Call SHARED core function (SAME AS CLI!)
    let response = crate::api::core::verify(request)
        .map_err(|e| JsValue::from_str(&e))?;

    // 3. Serialize response to JSON
    serde_json::to_string(&response)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize response: {}", e)))
}
```

---

## Example: Proof Generation

### CLI:
```rust
// CLI simply creates ProveRequest, then:
let response = zkplex_core::api::core::prove(prove_request)?;

// All logic is in core.rs!
```

### WASM:
```rust
#[wasm_bindgen]
pub fn prove(request_json: &str) -> Result<String, JsValue> {
    let request: ProveRequest = serde_json::from_str(request_json)?;

    // 🔥 Same as CLI!
    let response = crate::api::core::prove(request)
        .map_err(|e| JsValue::from_str(&e))?;

    serde_json::to_string(&response)?
}
```

---

## Core API (`src/api/core.rs`)

### Functions:

```rust
pub fn prove(request: ProveRequest) -> Result<ProveResponse, String>
```
- Converts `ProveRequest` → `Program`
- Builds circuit via `Circuit::from_program()`
- Validates strategy
- Estimates requirements (k)
- Selects wrapper (Auto/Boolean/BitD/Lookup)
- Generates VK/PK via Halo2
- Creates proof via `create_proof()`
- Encodes proof in ASCII85
- Returns `ProveResponse`

```rust
pub fn verify(request: VerifyRequest) -> Result<VerifyResponse, String>
```
- Decodes `verification_context` from ASCII85
- Reconstructs circuit from context + public signals
- Generates VK (via `generate_vk_for_strategy()`)
- Decodes proof from ASCII85
- Verifies via `verify_proof()`
- Returns `VerifyResponse { valid: bool }`

```rust
pub fn estimate(request: ProveRequest) -> Result<EstimateResponse, String>
```
- Converts `ProveRequest` → `Program`
- Builds circuit
- Validates strategy
- Calls `estimate_circuit_requirements_with_strategy()`
- Returns estimate (k, rows, sizes, etc.)

---

## Conclusion

✅ **CLI and WASM use IDENTICAL logic:**
- Both call `crate::api::core::prove()`, `verify()`, `estimate()`
- All proof logic is in ONE place: `src/api/core.rs`
- Same circuit building methods
- Same validators
- Same requirement estimation
- Same constraint generators
- Same Halo2 calls

⚠️ **Differences only in wrappers:**
- CLI: files + arguments → `ProveRequest` → `core::prove()` → pretty JSON
- WASM: JSON string → `ProveRequest` → `core::prove()` → JSON string
- CLI: ~80 lines of wrapper in `verify_proof()`
- WASM: ~15 lines of wrapper in `verify()`

📊 **Consistency guarantee:**
A proof generated in CLI is **100% identical** to a proof generated in WASM with the same inputs, because they use **THE SAME function** `api::core::prove()`.

🎯 **Benefits of new architecture:**
- ✅ Single source of truth - easier to maintain
- ✅ Guaranteed identity - impossible to desynchronize CLI and WASM
- ✅ Less code duplication - CLI `verify_proof()` reduced from ~350 lines to ~80
- ✅ Easier testing - tests for `core::prove()` automatically cover both CLI and WASM
- ✅ Simple refactoring - changes in one place automatically propagate to both platforms