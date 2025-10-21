# Zircon Format - Best Practices

## Overview

This guide covers best practices, optimization strategies, and common pitfalls when working with Zircon format.

## Table of Contents

1. [Circuit Optimization](#circuit-optimization)
2. [Signal Management](#signal-management)
3. [Encoding Choices](#encoding-choices)
4. [Preprocessing Strategies](#preprocessing-strategies)
5. [Security Considerations](#security-considerations)
6. [Performance Tips](#performance-tips)
7. [Testing](#testing)
8. [Common Mistakes](#common-mistakes)
9. [Code Organization](#code-organization)
10. [Production Readiness](#production-readiness)

## Circuit Optimization

### 1. Prefer Equality Over Ordering

**❌ Inefficient** (130 constraints):
```
1/value:100/-/value>=100;value<=100
```

**✅ Efficient** (3 constraints):
```
1/value:100/expected:100/value==expected
```

**Reason**: Range checks require 64-bit range proofs (~65 constraints each). Equality only needs is_zero gadget (~3 constraints).

**Savings**: 127 constraints (43x faster)

### 2. Minimize Range Checks

**❌ Inefficient** (137 constraints):
```
1/sum:50/-/sum>49;sum<51
```

**✅ Efficient** (3 constraints):
```
1/sum:50/expected:50/sum==expected
```

**When you must use ranges**, combine conditions:

**❌ Multiple checks** (204 constraints):
```
1/age:25/-/age>18;age<65
```

**✅ Combined check** (136 constraints):
```
1/age:25,minAge:18,maxAge:65/-/age>=minAge;age<=maxAge
```

### 3. Combine Arithmetic Operations

**❌ Inefficient** (more intermediate variables):
```
1/A:10,B:20,C:30/-/temp1<==A+B;temp2<==temp1+C;temp2>50
```

**✅ Efficient** (fewer variables):
```
1/A:10,B:20,C:30/-/sum<==A+B+C;sum>50
```

**Reason**: Fewer intermediate variables = smaller witness, faster proving.

### 4. Reuse Computed Values

**❌ Inefficient** (redundant computation):
```
1/A:10,B:20/-/sum1<==A+B;sum2<==A+B;sum1==sum2
```

**✅ Efficient** (compute once):
```
1/A:10,B:20/-/sum<==A+B;sum==30
```

### 5. Avoid Unnecessary Constraints

**❌ Redundant**:
```
1/A:100/-/A>50;A!=0;A>0
```
- `A > 50` already implies `A != 0` and `A > 0`

**✅ Minimal**:
```
1/A:100/-/A>50
```

### 6. Order Constraints Logically

**❌ Confusing order**:
```
1/A:10,B:20/-/output>100;sum<==A+B;output<==sum*5
```

**✅ Logical order**:
```
1/A:10,B:20/-/sum<==A+B;output<==sum*5;output>100
```

**Reason**: Sequential order improves readability and debugging.

## Signal Management

### 1. Minimize Public Signals

**❌ Too many public signals**:
```
1/secretValue:100/threshold1:50,threshold2:75,threshold3:90,limit:200/secretValue>threshold1;secretValue<limit
```

**✅ Minimal public signals**:
```
1/secretValue:100/minThreshold:50,maxLimit:200/secretValue>=minThreshold;secretValue<=maxLimit
```

**Reason**:
- Public signals increase proof size
- Higher verification cost on-chain
- More data to transmit

**Rule of thumb**: Keep public signals < 5 when possible.

### 2. Use Descriptive Names

**❌ Cryptic names**:
```
1/a:25,b:18/-/a>=b
```

**✅ Descriptive names**:
```
1/userAge:25/minimumAge:18/userAge>=minimumAge
```

**Benefits**:
- Self-documenting code
- Easier debugging
- Better maintainability

### 3. Logical Privacy Boundaries

**❌ Mixed privacy**:
```
1/passwordHash:0xabc:hex,username:alice/expectedHash:0xdef:hex/...
```
- Username could be public if it's meant to be known

**✅ Clear boundaries**:
```
1/password:secret123/-/hash<==sha256(password{%s})/hash==expectedHash
```

**Guidelines**:
- **Secret**: Secrets, credentials, personal data
- **Public**: Thresholds, addresses to verify against, system parameters

### 4. Avoid Signal Name Collisions

**❌ Collision risk**:
```
1/amount:100/amount:200/...
```
- ERROR: Signal 'amount' defined twice

**✅ Unique names**:
```
1/inputAmount:100/expectedAmount:200/inputAmount==expectedAmount
```

## Encoding Choices

### 1. Use Appropriate Encoding

| Data Type | Encoding | Example |
|-----------|----------|---------|
| Numbers | `decimal` | `amount:1000` |
| Ethereum address | `hex` | `addr:0x123:hex` |
| Solana address | `base58` | `pubkey:9aE476...:base58` |
| Binary data | `base64` | `data:SGVs...:base64` |
| Hash output | `hex` | `hash:0xabc:hex` |

**❌ Wrong encoding**:
```
1/solanaAddr:0x123:hex/-/...
```
- Solana addresses use base58, not hex

**✅ Correct encoding**:
```
1/solanaAddr:9aE476sH92Vc7DMC8bZNpe1xNNNy1fNjFpCGvfMuZMwM:base58/-/...
```

### 2. Explicit Encoding for Non-Decimal

**⚠️ Ambiguous**:
```
1/hash:0xabc123/-/...
```
- Auto-detected as hex, but explicit is better

**✅ Explicit**:
```
1/hash:0xabc123:hex/-/...
```

### 3. Size Constraints

**Remember**:
- **Ordering comparisons** (`>`, `<`, `>=`, `<=`): Values must be < 2^64
- **Equality comparisons** (`==`, `!=`): Any size up to field limit (~255 bits)

**❌ Too large for ordering**:
```
1/solanaAddr:9aE476...:base58/-/solanaAddr>0
```
- Solana address = 32 bytes > 2^64 limit

**✅ Use equality**:
```
1/solanaAddr:9aE476...:base58/expectedAddr:9aE476...:base58/solanaAddr==expectedAddr
```

## Preprocessing Strategies

### 1. Hash Concatenation for Authentication

**❌ Separate hashes**:
```
1/user:alice,pass:secret/-/userHash<==sha256(user{%s});passHash<==sha256(pass{%s})/...
```

**✅ Combined hash**:
```
1/user:alice,pass:secret/-/hash<==sha256(user{%s}|pass{%s})/hash==storedHash
```

**Benefits**:
- Single hash comparison
- Stronger binding between values
- Fewer constraints

### 2. Preprocessing vs Circuit Logic

**Preprocessing** (before circuit):
- Hash functions
- Format conversions
- Derived values

**Circuit** (constraint system):
- Comparisons
- Range checks
- Boolean conditions

**❌ Wrong placement**:
```
1/data:hello/-/data>0
```
- Can't compare string directly

**✅ Correct preprocessing**:
```
1/data:hello/-/hash<==sha256(data{%s})/hash!=0
```

### 3. Format Specifiers

Choose correct format for hashing:

| Format | Use Case | Example |
|--------|----------|---------|
| `{%x}` | Hash numbers as hex | `sha256(amount{%x})` |
| `{%d}` | Hash numbers as decimal | `sha256(id{%d})` |
| `{%s}` | Hash as string | `sha256(password{%s})` |

**Example - User auth**:
```
1/userId:12345,password:secret/-/hash<==sha256(userId{%d}|password{%s})/hash==storedHash
```

## Security Considerations

### 1. Prevent Division by Zero

**❌ Unsafe**:
```
1/A:100,B:5/-/output<==A/B
```

**✅ Safe**:
```
1/A:100,B:5/-/B!=0;output<==A/B;output>0
```

**Always check**: `divisor != 0` before division.

### 2. Prevent Underflow

**❌ Unsafe**:
```
1/balance:1000,withdrawal:500/-/remaining<==balance-withdrawal
```

**✅ Safe**:
```
1/balance:1000,withdrawal:500/-/remaining<==balance-withdrawal;remaining>=0
```

**Always check**: Output ≥ 0 after subtraction in financial contexts.

### 3. Validate Input Ranges

**❌ Unchecked inputs**:
```
1/age:25/-/...
```

**✅ Range validated**:
```
1/age:25/-/age>=0;age<=150;...
```

**Always validate**:
- Age: 0 ≤ age ≤ 150
- Amounts: ≥ 0
- Percentages: 0 ≤ x ≤ 100

### 4. Secret Data Leakage

**❌ Leaks information**:
```
1/secretAmount:1000/expectedAmount:1000/secretAmount==expectedAmount
```
- Public signal reveals secret

**✅ No leakage**:
```
1/secretAmount:1000/minAmount:100/secretAmount>=minAmount
```
- Only reveals constraint, not exact value

### 5. Secure Hash Usage

**❌ Weak hash**:
```
1/password:secret/-/hash<==crc32(password{%s})/...
```
- CRC32 is not cryptographically secure

**✅ Strong hash**:
```
1/password:secret/-/hash<==sha256(password{%s})/hash==storedHash
```

**Recommended hashes**:
- SHA-256 (standard)
- SHA3-256 (modern)
- BLAKE2 (fast)

**Avoid for security**:
- MD5 (broken)
- SHA-1 (deprecated)
- CRC32 (checksum only)

## Performance Tips

### 1. Constraint Budget

Target constraint ranges for different proof systems:

| Proof System | Target Constraints | Max Practical |
|--------------|-------------------|---------------|
| Groth16 | < 1M | ~10M |
| PLONK | < 100K | ~1M |
| Halo2 | < 100K | ~1M |

**Monitor your circuit**:
```bash
zkplex-cli -z "your_circuit" --estimate
```

### 2. Hash Function Costs

| Hash Function | Constraints | Use Case |
|---------------|-------------|----------|
| SHA-256 | ~1000 | Standard, secure |
| SHA3-256 | ~1500 | Modern, secure |
| BLAKE2 | ~800 | Fast, secure |
| SHA-512 | ~2000 | Extra security |
| MD5 | ~500 | Non-security (checksums) |

**Choose wisely**:
- **Security critical**: SHA-256, SHA3-256
- **Performance critical**: BLAKE2
- **Non-security**: CRC32 (~50 constraints)

### 3. Batching

**❌ Multiple separate proofs**:
```
Proof 1: 1/balance1:1000/-/balance1>100
Proof 2: 1/balance2:2000/-/balance2>100
Proof 3: 1/balance3:3000/-/balance3>100
```

**✅ Single batched proof**:
```
1/balance1:1000,balance2:2000,balance3:3000/threshold:100/balance1>threshold;balance2>threshold;balance3>threshold
```

**Benefits**:
- Single proof generation
- Lower verification cost
- Better amortization

### 4. Precompute When Possible

**❌ Recompute**:
```
1/A:10,B:20/-/A+B>25;A+B<35
```

**✅ Compute once**:
```
1/A:10,B:20/-/sum<==A+B;sum>25;sum<35
```

## Testing

### 1. Test Valid and Invalid Cases

**Valid case**:
```
1/age:25/-/age>=18
```
✓ Should succeed

**Invalid case**:
```
1/age:17/-/age>=18
```
✗ Should fail

**Always test**:
- ✅ Valid inputs (should pass)
- ❌ Invalid inputs (should fail)
- 🔵 Boundary cases (edge of valid range)

### 2. Boundary Testing

**Test boundaries**:
```
age: 17 → Fail
age: 18 → Pass (exact boundary)
age: 19 → Pass
```

**Example tests**:
```
1/age:17/-/age>=18   # Fail
1/age:18/-/age>=18   # Pass
1/age:19/-/age>=18   # Pass
```

### 3. Constraint Estimation

**Before production**:
```bash
zkplex-cli -z "your_circuit" --estimate
```

**Check**:
- Total constraint count
- Within target range?
- Any expensive operations?

### 4. Security Testing

**Test attack vectors**:
- Division by zero
- Integer underflow
- Invalid inputs
- Boundary manipulation

**Example - Test division by zero**:
```
1/A:100,B:0/-/B!=0;output<==A/B
```
Should detect B=0 and fail.

## Common Mistakes

### Mistake 1: Using Ordering on Large Values

**❌ Error**:
```
1/ethAddr:0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb:hex/target:0xabc:hex/ethAddr>target
```
ERROR: Ethereum addresses (20 bytes) exceed 2^64 limit

**✅ Fix**:
```
1/ethAddr:0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb:hex/target:0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb:hex/ethAddr==target
```

### Mistake 2: Missing Semicolons

**❌ Error**:
```
1/A:10,B:20/-/A>5 B>10
```
ERROR: Expected ';' between statements

**✅ Fix**:
```
1/A:10,B:20/-/A>5;B>10
```

### Mistake 3: Using `=` Instead of `<==`

**❌ Error**:
```
1/A:10,B:20/-/sum=A+B
```
ERROR: Use `<==` for assignment

**✅ Fix**:
```
1/A:10,B:20/-/sum<==A+B;sum==30
```

### Mistake 4: Undefined Variables

**❌ Error**:
```
1/A:10/-/B>5
```
ERROR: Variable 'B' not defined

**✅ Fix**:
```
1/A:10,B:20/-/B>5
```

### Mistake 5: Wrong Encoding Format

**❌ Error**:
```
1/solanaKey:0x123:hex/-/...
```
ERROR: Solana keys use base58

**✅ Fix**:
```
1/solanaKey:9aE476sH92Vc7DMC8bZNpe1xNNNy1fNjFpCGvfMuZMwM:base58/-/...
```

### Mistake 6: Empty Signal Names

**❌ Error**:
```
1/:10/-/...
```
ERROR: Empty signal name

**✅ Fix**:
```
1/value:10/-/...
```

### Mistake 7: Circular Dependencies

**❌ Error**:
```
1/A:10/-/B<==A+C;C<==B+1
```
ERROR: Circular dependency

**✅ Fix**:
```
1/A:10,C:5/-/B<==A+C;D<==B+1
```

## Code Organization

### 1. Modular Circuits

**❌ Monolithic**:
```
1/user:alice,pass:secret,balance:1000,age:25/-/hash<==sha256(user{%s}|pass{%s});age>=18;balance>100/hash==storedHash
```

**✅ Modular** (separate concerns):

**Auth circuit**:
```
1/user:alice,pass:secret/-/hash<==sha256(user{%s}|pass{%s})/hash==storedHash
```

**Eligibility circuit**:
```
1/age:25,balance:1000/-/age>=18;balance>100
```

### 2. Reusable Patterns

Create standard patterns for common use cases:

**Age verification pattern**:
```
1/age:25/minAge:18/age>=minAge
```

**Balance check pattern**:
```
1/balance:1000,amount:100/-/remaining<==balance-amount;remaining>=0
```

**Auth pattern**:
```
1/user:alice,pass:secret/-/hash<==sha256(user{%s}|pass{%s})/hash==storedHash
```

### 3. Documentation

**Document each circuit**:
```
# Age Verification Circuit
# Proves: User age >= 18 without revealing exact age
# Secret: age (user's actual age)
# Public: minimumAge (18)
# Constraint: age >= minimumAge

1/age:25/minimumAge:18/age>=minimumAge
```

## Production Readiness

### 1. Checklist Before Deployment

- [ ] All constraints validated
- [ ] Constraint count within target
- [ ] Security review completed
- [ ] Boundary cases tested
- [ ] Valid/invalid input tests pass
- [ ] Performance benchmarked
- [ ] Documentation complete
- [ ] Error handling implemented

### 2. Monitoring

**Track metrics**:
- Proof generation time
- Verification time
- Constraint count
- Proof size
- Gas costs (on-chain)

### 3. Versioning

**Use version field**:
```
1/...   # Version 1
```

**Plan for upgrades**:
- Document breaking changes
- Maintain backward compatibility when possible
- Version circuit specifications

### 4. Error Handling

**Graceful failures**:
```rust
match parse_zircon(input) {
    Ok(program) => { /* process */ },
    Err(e) => {
        log::error!("Parse error: {}", e);
        // Handle gracefully
    }
}
```

### 5. Auditing

**Before production**:
1. **Internal review**: Team reviews circuit logic
2. **Security audit**: External security review
3. **Formal verification**: If high-stakes
4. **Penetration testing**: Try to break it

## Optimization Patterns

### Pattern 1: Equality Over Range

**Before** (130 constraints):
```
value>=100;value<=100
```

**After** (3 constraints):
```
value==100
```

**Savings**: 127 constraints

### Pattern 2: Combined Conditions

**Before** (204 constraints):
```
A>10;A<20;B>30;B<40
```

**After** (136 constraints):
```
A>=11;A<=19;B>=31;B<=39
```

**Savings**: 68 constraints

### Pattern 3: Precompute Shared Values

**Before**:
```
output1<==A+B+C;output2<==A+B+D
```

**After**:
```
partial<==A+B;output1<==partial+C;output2<==partial+D
```

**Savings**: 1 constraint (fewer additions)

## Summary: Quick Reference

### ✅ DO

- Use `==` instead of `>=` + `<=` for exact values
- Minimize public signals
- Use descriptive signal names
- Validate inputs (division by zero, underflow)
- Test valid, invalid, and boundary cases
- Explicit encoding for non-decimal values
- Combine operations where possible
- Document your circuits

### ❌ DON'T

- Use ordering (`>`, `<`) on large values (>2^64)
- Have redundant constraints
- Use cryptographically weak hashes for security
- Forget to validate inputs
- Mix privacy boundaries unnecessarily
- Use cryptic variable names
- Create circular dependencies

### 🎯 Optimization Priority

1. **High Impact**: Replace range checks with equality
2. **Medium Impact**: Combine arithmetic operations
3. **Low Impact**: Reorder constraints for readability

## See Also

- **[Operators](OPERATORS.md)** - Operator costs and usage
- **[Circuit](CIRCUIT.md)** - Circuit design patterns
- **[Examples](EXAMPLES.md)** - Real-world examples
- **[Tools](TOOLS.md)** - CLI and API reference
- **[Security](../SECURITY.md)** - Security guidelines