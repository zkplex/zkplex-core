# ZKPlex Design Philosophy

This document explains ZKPlex's design philosophy and technical approach to zero-knowledge proofs, with detailed comparisons to Circom to illustrate key architectural decisions.

## Table of Contents

- [Key Philosophy](#key-philosophy)
- [1. Logical Operators: Manual vs Built-in](#1-logical-operators-manual-vs-built-in)
- [2. Constraint Systems: R1CS vs Custom Gates](#2-constraint-systems-r1cs-vs-custom-gates)
- [3. Proof Systems: Groth16 vs Halo2 IPA](#3-proof-systems-groth16-vs-halo2-ipa)
- [4. Developer Experience](#4-developer-experience)
- [5. Constraint Efficiency](#5-constraint-efficiency)
- [6. How Logical Operations Work (ZKP Level)](#6-how-logical-operations-work-zkp-level)
- [Summary Table](#summary-table)
- [Which Should You Choose?](#which-should-you-choose)

---

## Key Philosophy

**Circom** = "Assembly language for ZK"
- Full control over circuit construction
- Requires deep understanding of constraint systems
- Optimal proof sizes with Groth16 backend
- Component-based architecture

**ZKPlex** = "High-level language for ZK"
- Automation and abstraction
- Developer-friendly natural syntax
- Transparent setup (no trusted ceremony)
- Expression-based architecture

---

## 1. Logical Operators: Manual vs Built-in

### Circom - Manual (Low-Level)

Logical operators are **NOT** built into the compiler for signals. You must manually import templates from circomlib:

```circom
// ❌ This does NOT work for signals:
signal input a;
signal input b;
signal output c;
c <== a && b;  // COMPILATION ERROR!

// ✅ Must manually import templates:
include "circomlib/gates.circom";

template MyCircuit() {
    signal input a;
    signal input b;
    signal output c;

    // Create component manually
    component andGate = AND();
    andGate.a <== a;
    andGate.b <== b;
    c <== andGate.out;

    // YOU are responsible for binary constraints:
    a * (a - 1) === 0;  // must write yourself!
    b * (b - 1) === 0;
}
```

**Developer responsibility:**
- Import correct templates from circomlib
- Instantiate components manually
- Ensure binary constraints for boolean operations
- Understand component internals

**Constraint cost for AND:** ~3 constraints
```
a * a === a        // binary constraint
b * b === b        // binary constraint
a * b === c        // AND logic
```

### ZKPlex - Built-in (High-Level)

Logical operators are **built into the compiler**. Parser automatically recognizes them and generates constraints:

```rust
// ✅ Works immediately:
"a AND b"
// or
"a && b"

// Compiler AUTOMATICALLY:
// 1. Parses logical operator
// 2. Generates is_zero gadgets
// 3. Adds binary constraints
// 4. Creates all necessary constraints
```

**Implementation in `src/circuit/builder.rs`:**

```rust
/// Logical AND: both values non-zero -> 1, else 0
fn logical_and(
    &self,
    mut layouter: impl Layouter<Fp>,
    a: &AssignedCell<Fp, Fp>,
    b: &AssignedCell<Fp, Fp>,
) -> Result<AssignedCell<Fp, Fp>, Error> {
    let chip = ComparisonChip::new(comparison_config);

    // Convert a to boolean using is_zero gadget (3 constraints)
    let a_is_zero = chip.is_zero(layouter.namespace(|| "a_is_zero"), a)?;
    let a_bool = chip.is_zero(layouter.namespace(|| "a_to_bool"), &a_is_zero)?;

    // Convert b to boolean (3 constraints)
    let b_is_zero = chip.is_zero(layouter.namespace(|| "b_is_zero"), b)?;
    let b_bool = chip.is_zero(layouter.namespace(|| "b_to_bool"), &b_is_zero)?;

    // Multiply: bool_a * bool_b = output
    self.mul(layouter.namespace(|| "and_mul"), &a_bool, &b_bool)
}
```

**Constraint cost for AND:** ~6 constraints
- More constraints than Circom but fully automated
- No manual template management required
- Binary constraints handled automatically

---

## 2. Constraint Systems: R1CS vs Custom Gates

### Circom - R1CS (Rank-1 Constraint System)

All constraints must have the form: **(A) × (B) = (C)**

This is a **quadratic constraint system** (degree 2 polynomials only).

**Example: AND operation**
```
a * a = a        // binary constraint (a is 0 or 1)
b * b = b        // binary constraint (b is 0 or 1)
a * b = c        // AND logic
```

**Limitations:**
- Only quadratic operations (degree 2)
- Complex operations require multiple constraints
- Addition requires workarounds: `a + b = c` becomes `1 * (a + b) = c`

**Example: Addition in R1CS**
```circom
// For a + b = c, you need:
signal input a;
signal input b;
signal output c;

// Constraint: 1 * (a + b) = c
// Must express as multiplication
c <== a + b;  // compiler converts to R1CS form
```

### ZKPlex - Halo2 Custom Gates

Can create **arbitrary polynomial constraints** (not limited to degree 2):

```rust
// Can write constraints like:
a + b + c * d - output = 0

// Example: Custom gate for addition in src/circuit/config.rs
meta.create_gate("add", |meta| {
    let a = meta.query_advice(advice[0], Rotation::cur());
    let b = meta.query_advice(advice[1], Rotation::cur());
    let c = meta.query_advice(advice[2], Rotation::cur());

    // Constraint: a + b = c (single gate!)
    vec![a + b - c]
});
```

**Advantages:**
- More flexible constraint expressions
- Can combine multiple operations in one gate
- Native support for addition, subtraction without workarounds
- Fewer gates for certain operations

**Example: Complex constraint**
```rust
// Single gate can express:
// (a + b) * c - d + e = output
meta.create_gate("complex", |meta| {
    let a = meta.query_advice(advice[0], Rotation::cur());
    let b = meta.query_advice(advice[1], Rotation::cur());
    let c = meta.query_advice(advice[2], Rotation::cur());
    let d = meta.query_advice(advice[3], Rotation::cur());
    let e = meta.query_advice(advice[4], Rotation::cur());
    let output = meta.query_advice(advice[5], Rotation::cur());

    vec[(a + b) * c - d + e - output]
});
```

---

## 3. Proof Systems: Groth16 vs Halo2 IPA

### Circom (typically with Groth16 backend)

**Characteristics:**
```
✅ Proof size: ~128-256 bytes (very small!)
✅ Verification: O(1) - extremely fast (~5ms)
✅ Prover time: Fast for small circuits
❌ Trusted setup: Requires multi-party ceremony (MPC)
❌ Circuit-specific: New setup needed for each circuit
❌ Setup complexity: Ceremony must be secure
```

**Trusted Setup Ceremony:**
- Requires "powers of tau" ceremony
- Multiple participants needed for security
- If ceremony compromised, can create fake proofs
- Separate ceremony for each circuit modification

**Use cases:**
- Production systems with stable circuits
- Blockchain verification (small proofs critical)
- Systems where verification speed is paramount
- Applications with established trust

### ZKPlex (Halo2 with IPA polynomial commitment)

**Characteristics:**
```
❌ Proof size: ~30-40 KB (due to IPA commitments)
✅ No trusted setup: Transparent (uses hash functions only)
✅ Universal: One setup works for all circuits
✅ Recursive proofs: Easier to compose
✅ Circuit-agnostic: Change circuit without new setup
✅ Development-friendly: No ceremony needed
```

**Why larger proofs?**
- IPA (Inner Product Argument) includes polynomial commitments
- Each commitment adds ~32 bytes
- Proofs contain multiple commitments for soundness
- Trade-off: transparency for size

**Use cases:**
- Development and prototyping
- Systems where trust is critical
- Circuits that change frequently
- Applications requiring proof composition
- Research and experimentation

---

## 4. Developer Experience

### Circom - Component-Based Architecture

Requires understanding of circomlib structure and manual component management:

```circom
// Example: Age verification (age >= 18)
include "circomlib/comparators.circom";
include "circomlib/gates.circom";
include "circomlib/bitify.circom";

template AgeVerification() {
    signal input age;
    signal output valid;

    // Need to specify bit width
    component lt = LessThan(64);
    lt.in[0] <== age;
    lt.in[1] <== 18;

    // Manually invert
    component not = NOT();
    not.in <== lt.out;
    valid <== not.out;
}
```

**Developer must:**
- Know available templates in circomlib
- Import correct dependencies
- Understand bit width requirements
- Manually instantiate and wire components
- Manage signal routing between components

**Learning curve:**
- Steep - requires understanding R1CS
- Must study circomlib templates
- Need to understand constraint system internals
- Debugging can be challenging

### ZKPlex - Expression-Based Architecture

Natural syntax similar to traditional programming:

```rust
// Same age verification:
"age >= 18"

// Complex expressions work naturally:
"(income > 50000 AND age >= 21) OR verified"

// With preprocessing:
"hash(secret) == publicHash"
```

**Developer gets:**
- Intuitive expression syntax
- Automatic constraint generation
- Built-in operator support
- Clear error messages
- No manual component management

**Learning curve:**
- Gentle - similar to any programming language
- Start coding immediately
- Focus on logic, not constraints
- Natural debugging experience

---

## 5. Constraint Efficiency

### Comparison: `A > B` (Greater Than)

**Circom (LessThan template from circomlib):**

```circom
template LessThan(n) {
    // n = bit width (typically 64 for most numbers)
    signal input in[2];
    signal output out;

    // Bit decomposition for both inputs
    component n2b1 = Num2Bits(n);  // 64 constraints
    n2b1.in <== in[0];

    component n2b2 = Num2Bits(n);  // 64 constraints
    n2b2.in <== in[1];

    // Range checks on bits
    // 64 constraints for in[0]
    // 64 constraints for in[1]

    // Comparison logic
    // Additional constraints for bit comparison

    // Total: ~250+ constraints
}
```

**Constraint breakdown:**
- Bit decomposition: 128 constraints (64 per input)
- Range checks: 128 constraints
- Comparison logic: ~20 constraints
- **Total: ~250-300 constraints**

**ZKPlex (range proof implementation in `src/circuit/builder.rs`):**

```rust
fn greater_than(
    &self,
    mut layouter: impl Layouter<Fp>,
    a: &AssignedCell<Fp, Fp>,
    b: &AssignedCell<Fp, Fp>,
) -> Result<AssignedCell<Fp, Fp>, Error> {
    let chip = ComparisonChip::new(comparison_config);

    // 1. Compute difference: diff = a - b (1 constraint)
    let diff = self.sub(layouter.namespace(|| "a_minus_b"), a, b)?;

    // 2. Range check: diff in [1, 2^64) (64 constraints)
    let in_range = chip.range_check(
        layouter.namespace(|| "range_check"),
        &diff,
        64
    )?;

    // 3. is_zero check (3 constraints)
    let not_zero = chip.is_zero(layouter.namespace(|| "check"), &in_range)?;

    Ok(not_zero)
}
```

**Constraint breakdown:**
- Subtraction: 1 constraint
- Range check: 64 constraints (using lookup table)
- is_zero gadget: 3 constraints
- **Total: ~68 constraints**

**Result:** ZKPlex is **~3-4x more efficient** for comparisons due to native range proofs in Halo2 and lookup tables.

### Comparison: Addition (`A + B = C`)

**Circom (R1CS):**
```circom
c <== a + b;
// Compiles to: 1 * (a + b) = c
// Constraints: 1
```

**ZKPlex (Custom Gate):**
```rust
self.add(layouter, a, b)?;
// Custom gate: a + b - c = 0
// Constraints: 1
```

**Result:** Equal efficiency (1 constraint each)

### Comparison: Logical AND

**Circom:**
```circom
component and = AND();
and.a <== a;
and.b <== b;
c <== and.out;

// Constraints:
// a * a = a       (1)
// b * b = b       (1)
// a * b = c       (1)
// Total: 3 constraints
```

**ZKPlex:**
```rust
self.logical_and(layouter, a, b)?;

// Constraints:
// is_zero(a): 3 constraints
// is_zero(NOT a): 3 constraints (bool conversion)
// Same for b
// multiply: 1 constraint
// Total: ~6 constraints
```

**Result:** Circom is 2x more efficient for AND, but ZKPlex provides automation

---

## 6. How Logical Operations Work (ZKP Level)

Both Circom and ZKPlex execute logical operations **at the ZKP constraint level**, NOT as pre-computation.

### What does "at ZKP level" mean?

The logical operations become **cryptographic constraints** that are proven in the circuit.

**❌ NOT this (pre-execution):**
```javascript
// WRONG - this would break zero-knowledge:
let result = (secretA && secretB) ? 1 : 0;
prove({ result: result }); // Only result in proof
// Problem: Prover could lie about computation
```

**✅ Actually this (constraint-based):**
```javascript
// CORRECT - logic becomes constraints:
prove({
    circuit: "secretA AND secretB",
    // Entire logical operation is proven cryptographically
    // Verifier can verify AND was computed correctly
    // without knowing secretA or secretB
});
```

### Example: Age Verification

**Zircon code:**
```
age >= 18 AND age < 120
```

**This does NOT compile to:**
```javascript
let check1 = (25 >= 18);    // true
let check2 = (25 < 120);    // true
let result = check1 && check2; // true
prove(result);
```

**It compiles to constraints:**
```
constraint 1: range_check(age - 18)      // age >= 18
constraint 2: range_check(120 - age)     // age < 120
constraint 3: is_zero(NOT constraint1)   // bool conversion
constraint 4: is_zero(NOT constraint2)   // bool conversion
constraint 5: multiply(bool1, bool2)     // AND operation
constraint 6: output === result          // final check
```

The verifier checks **all 6 constraints** mathematically without knowing `age = 25`.

### Why is this critical for zero-knowledge?

1. **Pre-execution would break trustlessness:**
   - If logic executes before proof, must trust the prover
   - Prover could fake the computation result
   - No cryptographic guarantee

2. **Constraint-based execution ensures ZK:**
   - Every operation is mathematically proven
   - Verifier cryptographically verifies constraints
   - Impossible to fake results without witness knowledge
   - True zero-knowledge property maintained

### In Circom:

```circom
// Logical operations are templates that generate R1CS constraints
component and = AND();
and.a <== secretA;
and.b <== secretB;

// This creates constraints:
// secretA * secretA == secretA  (binary check)
// secretB * secretB == secretB  (binary check)
// result == secretA * secretB   (AND logic)

// These constraints are PROVEN in the circuit
// Verifier checks them cryptographically
```

### In ZKPlex:

```rust
// Logical operations use is_zero gadget which generates Halo2 constraints
logical_and(a, b)

// This creates constraints through is_zero gadget:
// For is_zero(x):
//   - if x == 0: witness w = 0, constraint: x * w == 0, output = 1
//   - if x != 0: witness w = 1/x, constraint: x * w == 1, output = 0

// The AND operation uses multiple is_zero calls:
// 1. is_zero(a) -> check if a is zero
// 2. is_zero(NOT a) -> convert to boolean
// 3-4. Same for b
// 5. multiply(bool_a, bool_b) -> AND result

// All constraints are PROVEN cryptographically
```

---

## Summary Table

| Aspect | Circom | ZKPlex |
|--------|--------|--------|
| **Philosophy** | Low-level assembly | High-level language |
| **Logical Operators** | Manual (templates) | Built-in (compiler) |
| **Constraint System** | R1CS (A×B=C) | Custom gates (polynomials) |
| **Proof System** | Groth16 (typical) | Halo2 IPA |
| **Proof Size** | ~200 bytes ✅ | ~35 KB ❌ |
| **Trusted Setup** | Required ❌ | Not required ✅ |
| **Setup Type** | Circuit-specific ❌ | Universal ✅ |
| **Verification Time** | ~5ms ✅ | ~50ms ⚠️ |
| **Developer UX** | Steep learning curve | Intuitive syntax |
| **Constraint Efficiency** | Depends on circomlib | Optimized for operations |
| **AND Operation** | 3 constraints ✅ | 6 constraints ⚠️ |
| **Comparison (>)** | ~250 constraints ❌ | ~68 constraints ✅ |
| **Circuit Changes** | Needs new setup ❌ | No new setup ✅ |
| **Production Ready** | Yes (established) | Yes (newer) |

---

## Which Should You Choose?

### Choose Circom if:
- ✅ Proof size is critical (blockchain verification)
- ✅ Circuit is stable and won't change frequently
- ✅ Team has ZK expertise
- ✅ Verification speed is paramount
- ✅ Willing to manage trusted setup
- ✅ Need battle-tested production system

### Choose ZKPlex if:
- ✅ Rapid prototyping and development
- ✅ Circuit changes frequently
- ✅ Trust/transparency is critical (no setup ceremony)
- ✅ Team prefers high-level abstractions
- ✅ Need intuitive developer experience
- ✅ Proof size ~35KB is acceptable
- ✅ Want universal setup for all circuits

---

## Future Improvements

### Potential ZKPlex Optimizations:

1. **Optimize logical operators** when inputs are known binary:
   - Current: 6 constraints for AND
   - Possible: 3 constraints (matching Circom)
   - If inputs guaranteed binary, skip is_zero conversion

2. **Add lookup table optimizations:**
   - Use Halo2 lookup arguments for common operations
   - Reduce range check constraints
   - Trade setup size for proving efficiency

3. **Support alternative backends:**
   - Add Groth16 backend option for smaller proofs
   - Keep IPA as default for transparency
   - Let developers choose trade-offs

### Potential Circom Improvements (community-driven):

1. **Higher-level abstractions** in circomlib:
   - More intuitive templates
   - Better composition patterns
   - Improved error messages

2. **Alternative backends** without trusted setup:
   - PLONK or Halo2 backend support
   - Trade proof size for transparency

---

## References

- [Circom Documentation](https://docs.circom.io/)
- [circomlib Templates](https://github.com/iden3/circomlib)
- [Halo2 Book](https://zcash.github.io/halo2/)
- [Groth16 Paper](https://eprint.iacr.org/2016/260.pdf)
- [IPA Polynomial Commitment](https://zcash.github.io/halo2/design/proving-system/inner-product-argument.html)
- [R1CS Explanation](https://medium.com/@VitalikButerin/quadratic-arithmetic-programs-from-zero-to-hero-f6d558cea649)

---

## Contributing

Found inaccuracies or have suggestions? Please open an issue or PR on our [GitHub repository](https://github.com/zkplex/zkplex-core).