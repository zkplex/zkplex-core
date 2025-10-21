# ZKPlex Core Architecture

This document describes the internal architecture of `zkplex-core`, its data processing flow, and the interaction between its key modules.

## Overview

The primary goal of `zkplex-core` is to transform a high-level text-based circuit into a cryptographic Zero-Knowledge Proof. This process is broken down into several sequential stages, each handled by a dedicated module.

## High-Level Data Flow

The entire process can be visualized as a pipeline where the output of one module serves as the input for the next.

```
┌─────────────────────────────────────────────┐
│        User Input (Circuit String)          │
│  Example: "age >= 18" or "A + B == result"  │
└─────────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────┐
│   1. PREPROCESSING  (src/preprocess)        │
│  ┌────────────────────────────────────────┐ │
│  │  • Formatting & Normalization          │ │
│  │  • Hash function application           │ │
│  │  • Value encoding (hex, base58, etc.)  │ │
│  └────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
                      │
                      │ Cleaned & Preprocessed String
                      ▼
┌─────────────────────────────────────────────┐
│   2. PARSER  (src/parser)                   │
│  ┌────────────────────────────────────────┐ │
│  │  • Lexical analysis                    │ │
│  │  • Syntax parsing (circuit.pest)       │ │
│  │  • AST construction                    │ │
│  └────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
                      │
                      │ Abstract Syntax Tree (AST)
                      ▼
┌─────────────────────────────────────────────┐
│   3. CIRCUIT BUILDER  (src/circuit)         │
│  ┌────────────────────────────────────────┐ │
│  │  • AST → Arithmetic Constraints        │ │
│  │  • Strategy selection (auto/lookup/..) │ │
│  │  • Witness assignment                  │ │
│  │  • Public/Private input separation     │ │
│  └────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
                      │
                      │ Arithmetic Circuit + Witnesses
                      ▼
┌─────────────────────────────────────────────┐
│   4.  HALO2 PROVING SYSTEM                  │
│  ┌────────────────────────────────────────┐ │
│  │  • Parameter generation (Params::new)  │ │
│  │  • Key generation (VK/PK)              │ │
│  │  • Proof creation (IPA commitment)     │ │
│  │  • Proof encoding (ASCII85)            │ │
│  └────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
                      │
                      │ ZK Proof + Public Signals
                      ▼
┌─────────────────────────────────────────────┐
│             Final Proof (JSON)              │
│  {                                          │
│    "proof": "<base85-encoded>",             │
│    "verification_context": "...",           │
│    "public_signals": {...},                 │
│    "debug": {...}                           │
│  }                                          │
└─────────────────────────────────────────────┘
```

## CLI & WASM: Shared Core Architecture

Both CLI and WASM are **thin wrappers** around the same core modules. They use **identical logic** for proof generation and verification.

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
│              SHARED CORE MODULES (zkplex-core)              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  circuit/ - Circuit building and constraint logic     │  │
│  │  ├─ builder.rs        Circuit::from_program()         │  │
│  │  ├─ strategy.rs       validate_strategy_compat...()   │  │
│  │  ├─ estimator.rs      estimate_circuit_require...()   │  │
│  │  ├─ mod.rs            CircuitAuto, CircuitBoolean,    │  │
│  │  │                    CircuitBitD, CircuitLookup      │  │
│  │  └─ evaluate.rs       evaluate_expression()           │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  parser/ - Expression parsing                         │  │
│  │  ├─ circuit.rs        parse_circuit()                 │  │
│  │  └─ mod.rs            AST types                       │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  encoding/ - Value encoding/decoding                  │  │
│  │  └─ value.rs          parse_value(), bytes_to_*()     │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  api/ - Data types and formats                        │  │
│  │  └─ types.rs          Signal, Program, DebugInfo,     │  │
│  │                       VerificationContext, etc.       │  │
│  └───────────────────────────────────────────────────────┘  │
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

**Key Points:**
- ✅ CLI and WASM call **identical core methods**
- ✅ Same circuit building logic (`Circuit::from_program`)
- ✅ Same strategy validation (`validate_strategy_compatibility`)
- ✅ Same circuit estimation (`estimate_circuit_requirements_with_strategy`)
- ✅ Same Halo2 proof generation/verification calls
- ⚠️ **Only difference:** Interface layer (file I/O vs JavaScript/JsValue)

**Guarantee:** Proofs generated by CLI and WASM are **bitwise identical** for the same inputs because they use the **exact same core code**.

For detailed comparison, see [ARCHITECTURE_SHARED.md](ARCHITECTURE_SHARED.md).

---

## Module Breakdown

### 1. Preprocessing Module (`src/preprocess`)

-   **Input:** A raw string containing the circuit.
-   **Output:** A formatted and partially hashed string.

This module acts as the system's entry point. Its responsibilities include:
-   **`formatter.rs`**: Standardizing the circuit by removing extra whitespace and normalizing syntax. This simplifies the subsequent parsing stage.
-   **`hasher.rs`**: Identifying private data (secrets) within the circuit and replacing them with their hashes. This conceals the original values that should not be visible in the circuit's public inputs.

### 2. Parser Module (`src/parser`)

-   **Input:** The cleaned string from the preprocessing module.
-   **Output:** An Abstract Syntax Tree (AST).

This module is responsible for understanding the circuit's semantics.
-   Using the grammar defined in **`circuit.pest`**, it converts the flat string into a tree-like data structure—the AST.
-   **`ast.rs`** defines the nodes of this tree (e.g., `variable`, `add operator`, `equality condition`).
-   The AST is a universal representation that is much easier to work with programmatically than the original string.

### 3. Circuit Builder Module (`src/circuit`)

-   **Input:** The Abstract Syntax Tree (AST).
-   **Output:** An arithmetic circuit ready for the proving system.

This is the core of the ZKP logic.
-   **`builder.rs`** recursively traverses the AST and translates each node into one or more mathematical constraints understandable by the ZKP backend (e.g., Halo2).
-   For example, an `a + b == c` node will be converted into a set of constraints that enforce this equality.
-   This stage also defines the public and private inputs/outputs of the circuit.

### 4. WASM Interface (`src/wasm`)

-   **Input:** Functions called from JavaScript/TypeScript.
-   **Output:** Outputs returned to JS/TS.

This module is not part of the main data processing pipeline but acts as a "bridge" between the compiled Rust code and the external environment (a browser or Node.js).
-   **`bindings.rs`** defines the public functions (e.g., `prove(circuit: string)`) that can be invoked from JavaScript.
-   It handles data serialization and deserialization, converting JS strings and objects into Rust types and vice versa.
-   It is this module that allows the complex Rust-based logic to be easily integrated into web applications.

---

## Verification Without Secret Values

One of the key features of zero-knowledge proofs is that verification does not require access to secret witness values. ZKPlex implements this through a carefully designed verification process that reconstructs the circuit structure without evaluating secret signals.

### How Verification Works

**During Proof Generation (`prove()`):**

1. **Full Circuit Building**: All signals (both secret and public) are evaluated with their actual values
2. **Metadata Capture**: Critical circuit metadata is saved in the `VerifyContext`:
   - `cached_max_bits`: Maximum bit width needed for range check tables
   - `secret_signals`: Names of secret signals (without values)
   - `output_signal`: Name of the output signal
   - `k`: Circuit size parameter
   - `strategy`: Proof strategy used (Boolean, BitD, Lookup, Auto)

3. **VerifyContext Serialization**: The context is encoded with Base85 and included in the proof

**During Verification (`verify()`):**

1. **Context Reconstruction**: The `VerifyContext` is decoded from the proof
2. **Signal Setup**:
   - Secret signals are added with `value: None` (no actual values)
   - Public signals are added with their actual values from the proof
   - Output signal is excluded from circuit building (added separately)

3. **Circuit Rebuilding**: The circuit is reconstructed from the program
   - Assignment expressions like `computed_value <== (A+B)*C` are evaluated optimistically
   - If evaluation fails (due to missing secret values), the assignment is skipped
   - This maintains circuit structure without requiring secret witness values

4. **Metadata Restoration**:
   - `circuit.cached_max_bits` is restored from `verify_context.cached_max_bits`
   - This ensures the same range check table size is used during verification
   - Without this, the verifier would default to 64-bit tables (too large)

5. **Constraint Verification**: Halo2 verifies the proof using only public inputs and the reconstructed circuit structure

### Key Implementation Details

**src/api/core.rs:219-221** - Secret signals without values:
```rust
// Add secret signals with NO values (verifier doesn't have access to secrets)
for name in &verify_context.secret_signals {
    secret_sigs.insert(name.clone(), Signal {
        value: None,  // No value - will be skipped during circuit building
        encoding: None,
    });
}
```

**src/circuit/builder.rs:815-821** - Optional assignment evaluation:
```rust
// Evaluate the expression to get the intermediate signal value
// This may fail during verification when secret signals are not available
if let Ok(value) = evaluate_expression(&expression, &signal_values) {
    // Store the intermediate signal value for use in subsequent statements
    signal_values.insert(name.clone(), value);
}
```

**src/api/core.rs:239** - Restore cached_max_bits:
```rust
// Restore cached_max_bits from verify context (needed for range check table size)
// This is essential because circuit.signals may be empty during verification
circuit.cached_max_bits = verify_context.cached_max_bits;
```

### Why cached_max_bits is Critical

The `cached_max_bits` field stores the maximum bit width of all values in the circuit. This is used to:

1. **Determine Range Check Table Size**: Circuits using comparison operators (`>`, `<`, `>=`, `<=`) need lookup tables for range checks
2. **Optimize Circuit Size**: Smaller bit widths = smaller tables = fewer rows needed
3. **Ensure Consistency**: Prover and verifier must use identical table sizes

**Problem Without cached_max_bits:**
- During proving: Circuit evaluates all signals, computes max bits (e.g., 16 bits for value 60000)
- During verification: Circuit has empty `signals` map (secret values unknown)
- `compute_max_range_check_bits()` returns `None`, defaults to 64-bit table
- Result: `NotEnoughRowsAvailable` error due to table size mismatch

**Solution:**
- Save `cached_max_bits` during proof generation
- Restore it during verification from `VerifyContext`
- Both prover and verifier use identical range check table size

### Security Considerations

This approach maintains zero-knowledge properties because:

1. **No Secret Leakage**: Secret signal values are never included in the proof or verification context
2. **Structure Preservation**: Only circuit structure metadata is shared (signal names, not values)
3. **Cryptographic Security**: Halo2's verification checks ensure constraints are satisfied without revealing witnesses
4. **Metadata is Safe**: Information like `cached_max_bits` reveals only the maximum value size, not specific values

---