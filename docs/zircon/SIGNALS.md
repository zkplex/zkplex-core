# Zircon Format - Signals

## Overview

**Signals** are the input values for your zero-knowledge proof program. In Zircon format, signals are divided into two categories:

- **Secret signals** (witnesses): Secret values known only to the prover
- **Public signals**: Values visible to everyone, including the verifier

```
version/secret/public/circuit
       ├─────┘└──────┘
        Secret  Public
```

## Signal Structure

### Basic Format

```
name:value
```

### With Encoding

```
name:value:encoding
```

### Multiple Signals

Separated by commas:

```
name1:value1,name2:value2,name3:value3
```

## Secret Signals

### What are Secret Signals?

Secret signals are **witness values** - secret information that:
- Only the prover knows
- Never revealed in the proof
- Used to satisfy circuit constraints
- Cryptographically protected

### When to Use Secret

Mark a signal as secret when:
- ✅ Value must remain secret (passwords, keys, balances)
- ✅ Revealing value defeats the purpose (age, income)
- ✅ Privacy is required (user data, credentials)

### Syntax

```
1/secret1:value1,secret2:value2/public_section/circuit
```

### Examples

**Age verification (hide age)**:
```
1/age:25/-/age>=18
```
Proves age ≥ 18 without revealing actual age.

**Balance check (hide balance)**:
```
1/balance:1000000/-/balance>100000
```
Proves sufficient balance without revealing amount.

**Password verification**:
```
1/password:secret123/-/hash<==sha256(password{%s})/hash==stored
```
Proves password knowledge without revealing it.

**Multi-value secrets**:
```
1/secret1:abc,secret2:def,secret3:xyz/-/...
```

## Public Signals

### What are Public Signals?

Public signals are **known values** that:
- Everyone can see
- Included in the proof
- Used for verification
- Define the statement being proven

### When to Use Public

Mark a signal as public when:
- ✅ Value is already public (thresholds, limits)
- ✅ Verifier needs to know it (target addresses)
- ✅ Part of the verification statement (minimum age)

### Syntax

```
1/secret_section/public1:value1,public2:value2/circuit
```

### Examples

**Age threshold (public)**:
```
1/age:25/minimumAge:18/age>=minimumAge
```
Age is secret, minimum is public.

**Price verification**:
```
1/balance:1000,amount:5/price:100/balance>=(amount*price)
```
Balance and amount are secret, price is public.

**Address matching**:
```
1/myAddress:0x123.../targetAddress:0xabc.../myAddress==targetAddress
```
My address is secret, target is public.

## Empty Sections

### No Secret Signals

Use `-` (hyphen):

```
1/-/threshold:100/A>threshold
```

All inputs come from public signals or constants.

### No Public Signals

Use `-` (hyphen):

```
1/secret:123/-/secret>100
```

No public parameters needed.

### Both Empty

```
1/-/-/1==1
```

Proves a constant statement (usually not useful).

## Signal Naming

### Valid Names

**Rules**:
- Start with: Letter or underscore `[A-Za-z_]`
- Followed by: Letters, digits, underscores `[A-Za-z0-9_]*`
- Case-sensitive

**Valid examples**:
```
✅ age
✅ userBalance
✅ minimum_age
✅ Balance123
✅ _secretKey
✅ ETH_address
```

**Invalid examples**:
```
❌ 123age          # Starts with digit
❌ user-balance    # Contains hyphen
❌ user.name       # Contains dot
❌ my balance      # Contains space
❌ $amount         # Starts with $
```

### Naming Conventions

**Recommended**:
```
✅ camelCase      → userBalance, minAge
✅ snake_case     → user_balance, min_age
✅ PascalCase     → UserBalance, MinAge
```

**Avoid**:
```
⚠️  Single letters → a, b, c (except in examples)
⚠️  Abbreviations → usr, bal, amt (not self-documenting)
⚠️  Generic names → value1, data2, temp
```

**Best practice**:
```
✅ Descriptive: userAge, minimumAge, passwordHash
❌ Cryptic: a, b, x, tmp
```

## Value Formats

### Default (Decimal)

```
amount:1000000
```

No encoding specified = decimal.

### Explicit Encoding

```
address:0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb:hex
```

Encoding must be one of:
- `decimal`
- `hex`
- `base58`
- `base64`

See **[ENCODING.md](ENCODING.md)** for details.

## Signal Scope

### Visibility in Program

Signals are visible in:

1. **Preprocessing section** (if present)
```
1/A:10,B:20/-/sum<==A+B/sum>25
         Preprocess uses A, B ──┘
```

2. **Circuit section**
```
1/A:10,B:20/-/A>5;B>10;A<B
              └─────────┘ Circuit uses A, B
```

### Undefined Variables

**Error** if you reference undefined signals:

```
❌ 1/A:10/-/B>5
            └── ERROR: B not defined
```

```
✅ 1/A:10,B:20/-/B>5
              └── OK: B is defined
```

## Signal Assignment

### In Preprocessing

Preprocessing can create **new variables**:

```
1/A:10,B:20/-/sum<==A+B;doubled<==sum*2/doubled>50
```

Variables created:
- `sum` = 10 + 20 = 30
- `doubled` = 30 * 2 = 60

These can be used in circuit section.

### In Circuit

Circuit can also create variables:

```
1/A:10,B:20/-/product<==A*B;product>100
```

But typically circuit focuses on **constraints**, not assignments.

## Secret vs Public: Decision Guide

### Decision Tree

```
Is the value already public?
├─ Yes → Public signal
└─ No
   └─ Does revealing it compromise security/privacy?
      ├─ Yes → Secret signal
      └─ No → Either (prefer public for transparency)
```

### Examples by Category

**Always Secret**:
- Passwords, secret keys
- Personal data (SSN, etc.)
- Financial balances
- Secret keys, seeds

**Always Public**:
- Thresholds, limits
- Public addresses (when verifying against them)
- Smart contract addresses
- Public parameters

**Context-Dependent**:
- Amounts (secret for user, public for audit)
- Timestamps (depends on use case)
- IDs (depends on anonymity requirements)

## Common Patterns

### Pattern 1: Secret Data, Public Threshold

```
1/value:150/minimum:100,maximum:200/value>=minimum;value<=maximum
```

Prove value in range without revealing it.

### Pattern 2: Secret Computation, Public Output

```
1/A:10,B:20/expectedSum:30/sum<==A+B;sum==expectedSum
```

Prove computation output without revealing inputs.

### Pattern 3: All Secret

```
1/secret:xyz123/-/hash<==sha256(secret{%s})/hash==storedHash
```

All inputs secret, constraint against known hash.

### Pattern 4: Multiple Public Parameters

```
1/balance:1000/minBalance:100,fee:50,limit:500/remaining<==balance-fee;remaining>=minBalance;remaining<=limit
```

Balance secret, all constraints public.

### Pattern 5: Secret Match Against Public

```
1/myAddress:0xabc.../targetAddress:0xdef.../myAddress==targetAddress
```

Prove address match without revealing secret address.

## Value Size Considerations

### All Signals

- Stored as field elements (Pallas field)
- Maximum ~255 bits
- Arithmetic operations use field arithmetic

### Secret Signals

- Can be any size up to field limit
- No visibility to verifier

### Public Signals

- Included in proof
- Larger public signals = larger proof
- Consider gas costs on-chain

**Recommendation**: Keep public signals minimal for efficiency.

## Multiple Signals Management

### Boolean Grouping

Group related signals:

```
✅ 1/input1:10,input2:20,input3:30/param1:100,param2:200/...
✅ 1/userData:...,userAge:...,userBalance:.../systemThreshold:.../...

❌ 1/input1:10,param1:100,input2:20,param2:200/...
   └── Mixed grouping, harder to read
```

### Ordering

**No semantic difference**, but for readability:

1. Order by importance
2. Order alphabetically
3. Order by dependency

**Example**:
```
# By importance
1/userId:123,userName:alice,userEmail:alice@x.com/-/...

# Alphabetically
1/email:alice@x.com,id:123,name:alice/-/...

# By dependency
1/principal:1000,rate:5,time:2/interest<==principal*rate*time/100/...
```

## Signal Reuse

### Same Name in Secret and Public

**Not allowed**:

```
❌ 1/amount:100/amount:100/...
      └── ERROR: 'amount' defined twice
```

Each signal name must be **unique** across secret and public sections.

### Workaround

Use different names:

```
✅ 1/userAmount:100/expectedAmount:100/userAmount==expectedAmount
```

## Type Safety

### No Explicit Types

Zircon doesn't have type declarations. All values are field elements.

### Implicit Typing

Type determined by usage:

```
1/age:25/-/age>18              # Numeric
1/name:alice/-/hash<==sha256(name{%s})/...  # String (for hash)
1/addr:0xabc:hex/-/...         # Hex bytes
```

### Type Conversion

Handled by encoding and format specifiers:

```
1/value:255/-/hash<==sha256(value{%x})/...
```
Converts `255` to hex `"ff"` for hashing.

## Best Practices

### 1. Minimize Public Signals

```
✅ 1/balance:1000,amount:50/threshold:100/...
   └── Only one public signal

⚠️  1/balance:1000/threshold:100,limit:500,fee:10,min:50/...
   └── Many public signals increase proof size
```

### 2. Use Descriptive Names

```
✅ 1/userAge:25,minimumAge:18/-/userAge>=minimumAge
❌ 1/a:25,b:18/-/a>=b
```

### 3. Explicit Encoding for Non-Decimal

```
✅ 1/addr:0x123:hex/-/...
❌ 1/addr:0x123/-/...
```

### 4. Boolean Privacy Boundaries

```
✅ 1/secretKey:abc/-/hash<==sha256(secretKey{%s})/hash==publicHash
   └── Secret secret, hash public

❌ 1/secretKey:abc,publicHash:xyz/-/hash<==sha256(secretKey{%s})/hash==publicHash
   └── Hash could be public signal instead
```

### 5. Document Signal Purpose

In separate documentation, explain each signal:

```
# UserAuthProof
- userId (secret): User's unique identifier
- passwordHash (secret): Hashed password
- storedHash (public): Hash from database
```

## Common Errors

### Error 1: Missing Signal

```
❌ 1/A:10/-/B>5
ERROR: Undefined variable 'B'

✅ 1/A:10,B:20/-/B>5
```

### Error 2: Duplicate Signal Name

```
❌ 1/amount:100/amount:200/...
ERROR: Signal 'amount' already defined

✅ 1/inputAmount:100/expectedAmount:200/...
```

### Error 3: Invalid Name

```
❌ 1/123value:10/-/...
ERROR: Invalid signal name

✅ 1/value123:10/-/...
```

### Error 4: Wrong Encoding

```
❌ 1/addr:0xabc:base58/-/...
ERROR: Invalid base58 value

✅ 1/addr:0xabc:hex/-/...
```

### Error 5: Empty Name

```
❌ 1/:10/-/...
ERROR: Empty signal name

✅ 1/value:10/-/...
```

## Advanced Topics

### Signal Transformation in Preprocessing

```
1/rawData:hello/-/processed<==sha256(rawData{%s});doubled<==processed*2/doubled>1000
```

Creates derived signals from inputs.

### Conditional Logic (via Constraints)

```
1/A:10,B:20,flag:1/-/output<==flag*A+(1-flag)*B/output>5
```

`output = A if flag==1, else B`

### Signal Aggregation

```
1/v1:10,v2:20,v3:30/-/sum<==v1+v2+v3;avg<==sum/3/avg>15
```

## See Also

- **[Encoding](ENCODING.md)** - Value encoding formats
- **[Syntax](SYNTAX.md)** - Signal syntax rules
- **[Circuit](CIRCUIT.md)** - Using signals in circuit
- **[Preprocessing](PREPROCESSING.md)** - Signal transformation
- **[Examples](EXAMPLES.md)** - Real-world signal usage