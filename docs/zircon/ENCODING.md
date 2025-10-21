# Zircon Format - Value Encoding

## Overview

Zircon supports multiple encoding formats for signal values, allowing you to work with different data types while maintaining compact representation.

## Encoding Syntax

### Without Encoding (Default)

```
name:value
```

Defaults to **decimal** encoding.

### With Explicit Encoding

```
name:value:encoding
```

Where `encoding` is one of:
- `decimal`
- `hex`
- `base58`
- `base64`
- `base85`
- `text`

## Supported Encodings

### 1. Decimal

**Default encoding**. Numbers in base-10.

#### Syntax

```
amount:1000000
amount:1000000:decimal     # Explicit
```

#### When to Use

- ✅ Numeric values (ages, counts, amounts)
- ✅ Arithmetic operations
- ✅ Simple integers
- ✅ Default choice for most use cases

#### Examples

```
1/age:25/-/age>=18
1/balance:1000000,amount:500000/-/balance>=amount
1/count:42/threshold:10/count>threshold
```

#### Value Range

**Arbitrary precision** using BigUint internally.

**For arithmetic**: Any value up to field size (~255 bits)

**For ordering comparisons** (`>`, `<`, `>=`, `<=`): Values must be < 2^64

**For equality** (`==`, `!=`): Any size up to field limit

#### Examples with Sizes

```
✅ 1/small:100/-/small>50                    # Small value, all operations OK
✅ 1/large:999999999999999999999/-/large==expected  # Large value, equality OK
❌ 1/tooBig:999999999999999999999/-/tooBig>100      # ERROR: Too large for ordering
```

### 2. Hexadecimal

Binary data represented as hexadecimal strings.

#### Syntax

```
hash:0x1a2b3c4d:hex
address:742d35Cc6634C0532925a3b844Bc9e7595f0bEb:hex    # 0x prefix optional
```

#### When to Use

- ✅ Ethereum addresses
- ✅ Cryptographic hashes
- ✅ Binary data
- ✅ Smart contract addresses
- ✅ Transaction IDs

#### Examples

```
# Ethereum address
1/myAddr:0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb:hex/targetAddr:0x123...:hex/myAddr==targetAddr

# Hash value
1/data:hello/-/hash<==sha256(data{%s})/hash==0x2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824:hex

# Transaction ID
1/txId:0xabc123def456:hex/-/...
```

#### Format Rules

- Case-insensitive: `0xABC` same as `0xabc`
- `0x` prefix optional in value
- Must be valid hex characters: `[0-9A-Fa-f]`
- Even number of hex digits (byte-aligned)

#### Valid Examples

```
✅ 0x1a2b3c4d
✅ 1a2b3c4d
✅ 0x1A2B3C4D
✅ 0x00
```

#### Invalid Examples

```
❌ 0xGHIJ        # Invalid hex characters
❌ 0x123         # Odd number of digits (use 0x0123)
❌ 1a2b3c4d:decimal  # Wrong encoding specified
```

### 3. Base58

Used for Bitcoin and Solana addresses.

#### Syntax

```
address:9aE476sH92Vc7DMC8bZNpe1xNNNy1fNjFpCGvfMuZMwM:base58
```

#### When to Use

- ✅ Solana public keys (32 bytes)
- ✅ Bitcoin addresses
- ✅ IPFS content IDs
- ✅ Blockchain identifiers

#### Examples

```
# Solana address (32 bytes)
1/myKey:9aE476sH92Vc7DMC8bZNpe1xNNNy1fNjFpCGvfMuZMwM:base58/expectedKey:9aE476...:base58/myKey==expectedKey

# Bitcoin address
1/btcAddr:1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa:base58/-/...

# IPFS hash
1/cid:QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG:base58/-/...
```

#### Base58 Alphabet

Uses Bitcoin's Base58 alphabet (excludes `0`, `O`, `I`, `l` to avoid confusion):

```
123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz
```

#### Size Considerations

**Solana addresses**: 32 bytes → ~44 base58 characters

**Bitcoin addresses**: 25 bytes → ~34 base58 characters

**For ordering comparisons**: 32-byte values **cannot** use `>`, `<`, `>=`, `<=` (exceeds 2^64 limit)

**For equality**: ✅ Works fine

```
✅ 1/addr:9aE476...:base58/expected:9aE476...:base58/addr==expected
❌ 1/addr:9aE476...:base58/-/addr>0                               # ERROR
```

### 4. Base85 (ASCII85)

Adobe's ASCII85 encoding - more compact than Base64.

#### Syntax

```
proof:FD,5.F.....@r<~:base85
data:<~@;KZ?FD5W(~>:base85
```

#### When to Use

- ✅ Proof encoding (ZKPlex internal use)
- ✅ Compact binary data representation
- ✅ More efficient than Base64 (~25% smaller)
- ✅ Compatible with Adobe tools and online decoders

#### Examples

```
# Compact proof encoding
1/data:FD,5.F.....@r<~:base85/-/...

# With delimiters (optional)
1/data:<~FD,5.F.....@r<~~>:base85/-/...
```

#### Format Rules

- Standard ASCII85 alphabet (85 printable ASCII characters)
- Optional `<~` prefix and `~>` suffix delimiters
- More compact than Base64 (4→5 encoding vs 3→4)

#### Valid Examples

```
✅ FD,5.F.....@r<~
✅ <~FD,5.F.....@r<~~>
✅ @;KZ?FD5W(
```

### 5. Base64

Universal binary data encoding.

#### Syntax

```
data:SGVsbG8gV29ybGQ=:base64
signature:YWJjZGVmZ2hpamtsbW5vcA==:base64
```

#### When to Use

- ✅ Binary data
- ✅ Encrypted data
- ✅ Signatures
- ✅ General-purpose encoding
- ✅ When other formats don't fit

#### Examples

```
# Binary data
1/encrypted:SGVsbG8gV29ybGQ=:base64/-/hash<==sha256(encrypted{%x})/...

# Signature
1/sig:YWJjZGVmZ2hpamtsbW5vcA==:base64/expected:YWJj...:base64/sig==expected

# Arbitrary data
1/payload:AQIDBAU=:base64/-/...
```

#### Format Rules

- Standard Base64 alphabet: `A-Z`, `a-z`, `0-9`, `+`, `/`
- Padding with `=` required
- Must be valid Base64 string

#### Valid Examples

```
✅ SGVsbG8gV29ybGQ=
✅ AQIDBAU=
✅ YWJjZGVmZ2hpamtsbW5vcA==
```

#### Invalid Examples

```
❌ SGVsbG8gV29ybGQ      # Missing padding
❌ SGVs!bG8gV29ybGQ=    # Invalid character !
```

### 6. Text (UTF-8)

Plain UTF-8 text strings - useful for preprocessing inputs like passwords or names.

#### Syntax

```
password:hello:text
name:Alice:text
message:Hello World:text
```

#### When to Use

- ✅ **Preprocessing inputs** (hash functions, string operations)
- ✅ Plain text passwords
- ✅ Human-readable strings
- ✅ Names, messages, labels
- ⚠️  **Not for circuit values** (text is converted to bytes for hashing)

#### Examples

```
# Password hashing
1/password:hello:text/target:0x2cf24dba...:hex/hash<==sha256(password)/hash==target

# Name hashing
1/name:Alice:text/-/hash<==sha256(name)/...

# Multi-word text (no spaces in Zircon, use underscores or Base64)
1/msg:Hello_World:text/-/hash<==sha256(msg)/...
```

#### Auto-Detection

When encoding is not specified, plain text is auto-detected:

```
✅ password:hello              # Auto-detected as text
✅ password:hello:text         # Explicit (recommended)
```

#### Format Rules

- UTF-8 encoded strings
- Any printable characters
- For multi-word text, consider Base64 encoding to preserve spaces

#### Valid Examples

```
✅ hello
✅ Alice
✅ test123
✅ password_with_underscores
```

#### Best Practices

**Use `text` for preprocessing:**
```
✅ 1/secret:hello:text/-/hash<==sha256(secret)/hash==expected:hex
```

**Don't use for large data:**
```
❌ 1/book:TheEntireNovel...:text/-/...
✅ 1/bookHash:0xabc...:hex/-/...
```

**For multi-word text, use Base64:**
```
⚠️  1/msg:Hello World:text/-/...        # Spaces may cause parsing issues
✅ 1/msg:SGVsbG8gV29ybGQ=:base64/-/...  # Safer for complex text
```

## Encoding Auto-Detection

When encoding is **not specified**, Zircon tries to auto-detect:

### Auto-Detection Rules

1. **Starts with `0x`** → Hexadecimal
2. **Only digits** `[0-9]` → Decimal
3. **Contains `+`, `/`, or `=`** → Base64
4. **Base58 alphabet** (no 0, O, I, l) → Base58
5. **Everything else** → Text (UTF-8 string)

### Recommendation

**Always specify encoding explicitly** for non-decimal values:

```
✅ address:0x123:hex
✅ solAddr:9aE476...:base58
✅ data:SGVs...:base64
✅ password:hello:text

⚠️  address:0x123          # Auto-detected as hex, but explicit is better
⚠️  password:hello         # Auto-detected as text, but explicit is better
❌ solAddr:9aE476...       # Ambiguous: could be base58 or base64
```

## Encoding Comparison

| Encoding | Size (bytes→chars) | Use Case | Ordering Comparisons |
|----------|-------------------|----------|----------------------|
| **Decimal** | Variable | Numbers | < 2^64 only |
| **Hex** | 2× + `0x` | Ethereum, hashes | Depends on value |
| **Base58** | ~1.37× | Solana, Bitcoin | 32-byte = ❌ |
| **Base64** | ~1.33× + padding | Binary data | Depends on value |
| **Base85** | ~1.25× | Compact encoding | Depends on value |
| **Text** | 1× (UTF-8) | Strings, preprocessing | N/A (for hashing) |

## Conversion Examples

### Same Value, Different Encodings

**Decimal 255**:
```
decimal:  255
hex:      0xff
base64:   /w==
```

**Decimal 1000000**:
```
decimal:  1000000
hex:      0xf4240
```

### Ethereum Address

**Hex** (standard):
```
address:0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb:hex
```

**NOT decimal** (addresses are binary data):
```
❌ address:123456789012345678901234567890:decimal
```

### Solana Address

**Base58** (standard):
```
pubkey:9aE476sH92Vc7DMC8bZNpe1xNNNy1fNjFpCGvfMuZMwM:base58
```

**NOT hex** (loses leading zeros, not standard format):
```
❌ pubkey:0x7a3b...:hex
```

## Size Constraints Summary

| Encoding | Max Size (Arbitrary) | Max Size (Ordering) | Max Size (Equality) |
|----------|----------------------|---------------------|---------------------|
| Decimal | ~255 bits | 64 bits (2^64) | ~255 bits |
| Hex | ~255 bits | 64 bits | ~255 bits |
| Base58 | ~255 bits | 64 bits | ~255 bits |
| Base64 | ~255 bits | 64 bits | ~255 bits |

### Key Takeaway

**Ordering comparisons** (`>`, `<`, `>=`, `<=`) work only with values < 2^64, **regardless of encoding**.

**Equality comparisons** (`==`, `!=`) work with any size up to field limit (~255 bits).

## Encoding in Preprocessing

Format specifiers control how values are formatted for hash functions:

### Hexadecimal Format

```
hash<==sha256(value{%x})
```

**Example**:
```
1/amount:255/-/hash<==sha256(amount{%x})/...
```

Formats `255` as `"ff"` before hashing.

### Decimal Format

```
hash<==sha256(value{%d})
```

**Example**:
```
1/amount:255/-/hash<==sha256(amount{%d})/...
```

Formats `255` as `"255"` before hashing.

### String Format

```
hash<==sha256(value{%s})
```

**Example**:
```
1/name:alice/-/hash<==sha256(name{%s})/...
```

Uses value as-is.

See **[PREPROCESSING.md](PREPROCESSING.md)** for details.

## Best Practices

### 1. Use Appropriate Encoding

```
✅ Ethereum address → hex
✅ Solana address → base58
✅ Age, balance → decimal
✅ Binary data → base64
✅ Hash output → hex
✅ Passwords, names → text
✅ Compact proofs → base85

❌ Ethereum address → base58
❌ Solana address → decimal
❌ Binary data → decimal
❌ Passwords → decimal
```

### 2. Be Explicit

```
✅ address:0x123:hex
⚠️  address:0x123              # Auto-detected, but less clear
```

### 3. Match Blockchain Standards

```
✅ Solana → base58
✅ Ethereum → hex (with 0x)
✅ Bitcoin → base58
```

### 4. Consider Size Limits

```
✅ Small values → all comparisons work
✅ Large values → use equality only

❌ 32-byte address with > comparison
```

### 5. Validate Format

```
✅ Hex: even digits, valid hex chars
✅ Base58: Bitcoin alphabet
✅ Base64: valid Base64 with padding
```

## Common Errors

### Error 1: Wrong Encoding for Value

```
❌ 1/addr:9aE476...:hex/-/...
ERROR: Invalid hex value

✅ 1/addr:9aE476...:base58/-/...
```

### Error 2: Missing Encoding for Non-Decimal

```
⚠️  1/hash:0xabc123/-/...
WARN: Auto-detected as hex, specify explicitly

✅ 1/hash:0xabc123:hex/-/...
```

### Error 3: Value Too Large for Ordering

```
❌ 1/huge:9aE476sH92Vc7DMC8bZNpe1xNNNy1fNjFpCGvfMuZMwM:base58/-/huge>0
ERROR: Value exceeds 2^64

✅ 1/huge:9aE476...:base58/expected:9aE476...:base58/huge==expected
```

### Error 4: Odd Hex Digits

```
❌ 1/value:0x123:hex/-/...
ERROR: Odd number of hex digits

✅ 1/value:0x0123:hex/-/...
```

### Error 5: Invalid Base64 Padding

```
❌ 1/data:SGVsbG8:base64/-/...
ERROR: Invalid Base64 (missing padding)

✅ 1/data:SGVsbG8=:base64/-/...
```

### Error 6: Hash Comparison with Ordering Operators

```
❌ 1/password:hello:text/-/hash<==sha256(password)/hash>1000
ERROR: max_bits must be 8, 16, 32, or 64, got 256
Reason: SHA-256 produces 256-bit values, exceeding 64-bit limit for ordering comparisons

✅ 1/password:hello:text/expected:0x2cf24dba5fb0a30e...:hex/-/hash<==sha256(password)/hash==expected
✅ 1/password:hello:text/-/hash<==sha256(password)/hash!=0
```

**Why it fails:**
- Hash functions (SHA-256, Keccak-256, etc.) output 256-bit values
- Ordering comparisons (`>`, `<`, `>=`, `<=`) require range checks limited to 64 bits
- Equality comparisons (`==`, `!=`) work with any size (no range check needed)

**Best practices for hash comparisons:**

```
✅ Compare hash to expected value (equality)
1/secret:hello:text/target:0x2cf24dba...:hex/-/hash<==sha256(secret)/hash==target

✅ Check hash is non-zero
1/data:test:text/-/hash<==sha256(data)/hash!=0

✅ Compare two hashes for equality
1/data1:hello:text,data2:world:text/-/hash1<==sha256(data1);hash2<==sha256(data2)/hash1==hash2

❌ DO NOT use ordering comparisons with hashes
1/secret:hello:text/-/hash<==sha256(secret)/hash>100        # ERROR: 256 bits
1/data:test:text/-/hash<==sha256(data)/hash<999999999       # ERROR: 256 bits
```

**Summary:** For hash values, always use `==` or `!=`, never use `>`, `<`, `>=`, `<=`.

## Encoding Conversion Tools

### CLI

```bash
# View signal encoding
zkplex-cli --zircon "1/addr:0xabc:hex/-/..." --info
```

### Programmatic

```typescript
import { zircon_to_json } from './zkplex_core.js';

const json = zircon_to_json("1/addr:0xabc:hex/-/...");
// JSON shows encoding: { "addr": { "value": "0xabc", "encoding": "hex" } }
```

## See Also

- **[Signals](SIGNALS.md)** - Defining signals with encoding
- **[Preprocessing](PREPROCESSING.md)** - Format specifiers for hashing
- **[Syntax](SYNTAX.md)** - Encoding syntax rules
- **[Examples](EXAMPLES.md)** - Real-world encoding usage