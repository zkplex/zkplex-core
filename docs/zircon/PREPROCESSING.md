# Zircon Format - Preprocessing

## Overview

Preprocessing is the **fourth component** of the extended Zircon format (5-part structure). It allows you to perform computations and transformations **before** the main circuit constraints are evaluated.

```
version/secret/public/preprocess/circuit 
                     └─────────┘
                      Preprocessing section
```

## When to Use Preprocessing

### ✅ Use Preprocessing For

- **Cryptographic hashing** of input values
- **Data transformation** before constraints
- **Derived values** computation
- **Value concatenation** for multi-input operations
- **Format conversions** (hex, string, etc.)

### ⏩ Execute in Circuit Instead

- **Simple arithmetic** that's part of the proof
- **Direct comparisons** between signals
- **Final validation logic**

## Basic Syntax

### Single Statement
```
hash<==sha256(secret{%x})
```

### Multiple Statements
Separate with semicolons `;`:
```
hash1<==sha256(A{%x});hash2<==sha256(B{%x});combined<==hash1+hash2
```

### Complete Example
```
1/secret:hello/-/hash<==sha256(secret{%x})/hash==expected
```

## Hash Functions

All supported cryptographic hash functions:

### SHA-256
```
hash<==sha256(data{%x})
```
- **Output**: 32 bytes (256 bits)
- **Use case**: General-purpose hashing, Bitcoin, Ethereum

### SHA-1
```
hash<==sha1(data{%x})
```
- **Output**: 20 bytes (160 bits)
- **Use case**: Legacy systems, Git commits

### SHA-512
```
hash<==sha512(data{%x})
```
- **Output**: 64 bytes (512 bits)
- **Use case**: High-security applications

### SHA3-256
```
hash<==sha3_256(data{%x})
```
- **Output**: 32 bytes (256 bits)
- **Use case**: Modern cryptographic systems

### SHA3-512
```
hash<==sha3_512(data{%x})
```
- **Output**: 64 bytes (512 bits)
- **Use case**: High-security SHA3 variant

### MD5
```
hash<==md5(data{%x})
```
- **Output**: 16 bytes (128 bits)
- **Use case**: Checksums (not cryptographically secure)

### BLAKE2
```
hash<==blake2(data{%x})
```
- **Output**: 32 bytes (256 bits)
- **Use case**: Fast cryptographic hashing

### CRC32
```
checksum<==crc32(data{%x})
```
- **Output**: 4 bytes (32 bits)
- **Use case**: Data integrity checks, not cryptographic

## Format Specifiers

**⚠️ REQUIRED**: Format specifiers are **mandatory** for all hash function arguments. Omitting them will output in an error.

Control how values are formatted before hashing:

### Hexadecimal: `{%x}`
```
hash<==sha256(address{%x})
```
Converts value to hexadecimal string before hashing.

**Example**:
- Input: `255`
- Format: `ff`
- Hash: `sha256("ff")`

### Decimal: `{%d}`
```
hash<==sha256(amount{%d})
```
Converts value to decimal string.

**Example**:
- Input: `255`
- Format: `255`
- Hash: `sha256("255")`

### String: `{%s}`
```
hash<==sha256(message{%s})
```
Uses value as-is (string format).

**Example**:
- Input: `hello`
- Format: `hello`
- Hash: `sha256("hello")`

### Why Format Specifiers Are Required

Format specifiers determine how values are represented before hashing, and **different formats produce different hashes**:

```
value = 255

sha256(value{%x})  →  sha256("ff")   →  hash_1
sha256(value{%d})  →  sha256("255")  →  hash_2  (different!)
sha256(value{%s})  →  sha256("255")  →  hash_2
```

**Without explicit format specification**:
- Outputs become unpredictable
- Code intent is unclear
- Bugs are harder to detect

**Therefore, format specifiers are mandatory** to ensure:
- ✅ Deterministic behavior
- ✅ Clear code intent
- ✅ Consistent outputs across implementations

## Value Concatenation

**Key Feature**: Combine multiple values inside hash functions using `|` (pipe) operator.

### Basic Concatenation

```
hash<==sha256(val1{%x}|val2{%x})
```

Concatenates `val1` and `val2` before hashing.

### Multiple Values

```
hash<==sha256(A{%x}|B{%x}|C{%x})
```

Combines three values: `A`, `B`, and `C`.

### Mixed Formats

```
hash<==sha256(id{%d}|address{%x}|name{%s})
```

Each value can have its own format specifier.

### Example: User Authentication

```
1/userId:12345,password:secret123/-/hash<==sha256(userId{%d}|password{%s})/hash==storedHash
```

**Process**:
1. Format `userId` as decimal: `"12345"`
2. Format `password` as string: `"secret123"`
3. Concatenate: `"12345secret123"`
4. Hash: `sha256("12345secret123")`
5. Compare with `storedHash`

### Example: Multi-part Data Integrity

```
1/part1:0xabc,part2:0xdef,part3:0x123/-/combined<==sha256(part1{%x}|part2{%x}|part3{%x})/combined==expected
```

**Process**:
1. `part1{%x}` → `"abc"`
2. `part2{%x}` → `"def"`
3. `part3{%x}` → `"123"`
4. Concatenate: `"abcdef123"`
5. Hash: `sha256("abcdef123")`

## Arithmetic Operations

Perform calculations in preprocessing:

### Basic Arithmetic

```
sum<==A+B
```

### Chained Operations

```
total<==A+B;doubled<==total*2;final<==doubled-10
```

### With Hash Outputs

```
hash1<==sha256(A{%x});hash2<==sha256(B{%x});combined<==hash1+hash2
```

## Multiple Preprocessing Steps

Execute operations sequentially:

### Example: Hash Chain

```
1/secret:hello/-/hash1<==sha256(secret{%x});hash2<==sha256(hash1{%x});hash3<==sha256(hash2{%x})/hash3==final
```

**Execution order**:
1. `hash1 = sha256(secret)`
2. `hash2 = sha256(hash1)`
3. `hash3 = sha256(hash2)`
4. Circuit validates: `hash3 == final`

### Example: Complex Transformation

```
1/A:10,B:20,C:30/-/sum<==A+B;product<==B*C;hashSum<==sha256(sum{%x});hashProd<==sha256(product{%x});combined<==sha256(hashSum{%x}|hashProd{%x})/combined<0x1000000
```

**Steps**:
1. `sum = 10 + 20 = 30`
2. `product = 20 * 30 = 600`
3. `hashSum = sha256("1e")` (30 in hex)
4. `hashProd = sha256("258")` (600 in hex)
5. `combined = sha256(hashSum + hashProd)`
6. Validate: `combined < 0x1000000`

## Advanced Patterns

### Pattern 1: Salted Hash

```
1/password:secret123,salt:randomsalt456/-/hash<==sha256(password{%s}|salt{%s})/hash==stored
```

Combines password with salt before hashing.

### Pattern 2: Merkle Tree Leaf

```
1/data:mydata,index:5/-/leaf<==sha256(index{%d}|data{%s})/leaf==expectedLeaf
```

Creates Merkle tree leaf from index and data.

### Pattern 3: Multi-field Hash

```
1/field1:value1,field2:value2,field3:value3/-/record<==sha256(field1{%s}|field2{%s}|field3{%s})/record==checksum
```

Hashes structured data with multiple fields.

### Pattern 4: Nested Transformations

```
1/A:100,B:200/-/sum<==A+B;hashA<==sha256(A{%x});hashB<==sha256(B{%x});combined<==sha256(hashA{%x}|hashB{%x}|sum{%x})/combined!=0
```

Combines arithmetic and cryptographic operations.

## Best Practices

### 1. Use Appropriate Format Specifiers

```
✅ hash<==sha256(address{%x})    # Hex for addresses
✅ hash<==sha256(amount{%d})     # Decimal for numbers
✅ hash<==sha256(name{%s})       # String for text

❌ hash<==sha256(address)        # Missing format
```

### 2. Order Operations Logically

```
✅ a<==A+B;b<==a*2;c<==b-10       # Clear dependency chain
❌ c<==b-10;a<==A+B;b<==a*2       # Confusing order
```

### 3. Use Descriptive Names

```
✅ userHash<==sha256(userId{%d}|userName{%s})
❌ h<==sha256(a{%d}|b{%s})
```

### 4. Keep Preprocessing Simple

```
✅ hash<==sha256(data{%x})                          # Simple, clear
⚠️  result<==sha256(sha256(sha256(data{%x}){%x}){%x})  # Too complex
```

For complex logic, break into multiple statements:
```
✅ hash1<==sha256(data{%x});hash2<==sha256(hash1{%x});hash3<==sha256(hash2{%x})
```

## Limitations

### No Control Flow

Preprocessing does **not** support:
- `if/else` conditionals
- Loops
- Function definitions

All operations are **sequential** and **deterministic**.

### Value Size Constraints

- Hash outputs are field elements
- Arithmetic uses field arithmetic (mod p)
- Large values may wrap around

### No External Data

Preprocessing cannot:
- Read from blockchain
- Access external APIs
- Use random values

All inputs must be provided as signals.

## Common Errors

### Error 1: Missing Format Specifier

```
❌ hash<==sha256(data)
✅ hash<==sha256(data{%x})
```

### Error 2: Wrong Separator

```
❌ hash<==sha256(A{%x},B{%x})    # Wrong: comma
✅ hash<==sha256(A{%x}|B{%x})    # Correct: pipe
```

### Error 3: Undefined Variable

```
❌ result<==sha256(unknown{%x})  # 'unknown' not defined
✅ result<==sha256(A{%x})         # 'A' defined in signals
```

### Error 4: Circular Dependency

```
❌ a<==b+1;b<==a+1               # Circular reference
✅ a<==A+1;b<==a+1               # Linear dependency
```

## Performance Considerations

### Hash Function Speed

**Fastest**: CRC32, MD5
**Fast**: BLAKE2, SHA-256
**Slower**: SHA-512, SHA3-512

For production, use **SHA-256** or **BLAKE2** (good balance of speed and security).

### Number of Operations

Each preprocessing step adds to proof generation time:
- **1-3 steps**: Negligible impact
- **5-10 steps**: Noticeable but acceptable
- **20+ steps**: Consider optimization

### Concatenation Cost

Multiple concatenations are cheap:
```
hash<==sha256(A{%x}|B{%x}|C{%x}|D{%x}|E{%x})  # Fine
```

The hash itself dominates the cost, not concatenation.

## Examples Collection

### Example 1: Simple Hash

```
1/password:mypass/-/hash<==sha256(password{%s})/hash==stored
```

### Example 2: Salted Hash

```
1/password:mypass,salt:abc123/-/hash<==sha256(password{%s}|salt{%s})/hash==stored
```

### Example 3: Timestamp Verification

```
1/data:important,timestamp:1234567890/-/hash<==sha256(data{%s}|timestamp{%d})/hash==signature
```

### Example 4: Multi-field Record

```
1/name:alice,age:30,email:alice@example.com/-/record<==sha256(name{%s}|age{%d}|email{%s})/record==checksum
```

### Example 5: Hash Chain (3 levels)

```
1/secret:xyz/-/h1<==sha256(secret{%s});h2<==sha256(h1{%x});h3<==sha256(h2{%x})/h3==final
```

### Example 6: Combined Operations

```
1/A:100,B:200,C:300/-/sum<==A+B+C;hash<==sha256(sum{%x})/hash>0x1000;sum<1000
```

## See Also

- **[Circuit](CIRCUIT.md)** - Using preprocessed values in circuit
- **[Operators](OPERATORS.md)** - Available operators
- **[Examples](EXAMPLES.md)** - Real-world use cases
- **[Encoding](ENCODING.md)** - Value encoding formats