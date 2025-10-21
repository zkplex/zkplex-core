# Zircon Format - Syntax Reference

## Format Structure

Zircon programs use **slash-separated** components:

### 4-Part Format (Basic)

```
version/secret/public/circuit
```

**When to use**: Programs without preprocessing operations.

### 5-Part Format (Extended)

```
version/secret/public/preprocess/circuit
```

**When to use**: Programs with hash functions, transformations, or derived values.

## Delimiters

| Delimiter | Purpose | Example |
|-----------|---------|---------|
| `/` | Separates major sections | `1/A:10/B:20/A>B` |
| `,` | Separates signals within section | `A:10,B:20,C:30` |
| `:` | Separates signal name:value:encoding | `address:0x123:hex` |
| `;` | Separates statements | `A<==B+C;D<==E*F` |
| `\|` | Concatenates values in hash | `sha256(A{\%x}\|B{\%x})` |

## Component Breakdown

### 1. Version

**Syntax**: Single integer

```
1
```

**Rules**:
- Must be a positive integer
- Currently only version `1` is supported
- Future versions will be backward compatible

**Examples**:
```
✅ 1
❌ 1.0    # No decimals
❌ v1     # No prefix
```

### 2. Secret Signals

**Syntax**: Comma-separated list of signal definitions

```
name:value[:encoding],name:value[:encoding],...
```

**Rules**:
- Each signal: `name:value` or `name:value:encoding`
- Names: `[A-Za-z_][A-Za-z0-9_]*`
- Values: Any string (depends on encoding)
- Use `-` for empty section

**Examples**:
```
✅ A:10
✅ A:10,B:20,C:30
✅ secret:0x1a2b3c:hex
✅ address:9aE476sH...:base58
✅ -                      # Empty
❌ A:10;B:20              # Wrong separator
❌ 123:10                 # Invalid name
```

### 3. Public Signals

**Syntax**: Same as secret signals

```
name:value[:encoding],name:value[:encoding],...
```

**Rules**:
- Identical format to secret signals
- Signals here are **visible to verifier**
- Use `-` for empty section

**Examples**:
```
✅ threshold:100
✅ minAge:18,maxAge:65
✅ -                      # Empty
```

### 4. Preprocessing (Optional)

**Syntax**: Semicolon-separated statements

```
statement;statement;...
```

**Statement types**:

**Hash assignment**:
```
variable<==hashfunc(input{format})
```

**Concatenated hash**:
```
variable<==hashfunc(input1{format}|input2{format}|...)
```

**Arithmetic assignment**:
```
variable<==expression
```

**Rules**:
- Statements execute **sequentially**
- Later statements can use earlier outputs
- Omit this section if no preprocessing needed

**Examples**:
```
✅ hash<==sha256(data{%x})
✅ sum<==A+B;hash<==sha256(sum{%x})
✅ h<==sha256(A{%x}|B{%x}|C{%x})
❌ hash<==sha256(undefined{%x})    # Undefined variable
```

### 5. Circuit

**Syntax**: Semicolon-separated constraints

```
constraint;constraint;...
```

**Constraint types**:

**Assignment**:
```
variable<==expression
```

**Comparison**:
```
expression OP expression
```
Where `OP` is: `>`, `<`, `>=`, `<=`, `==`, `!=`

**Boolean**:
```
expr AND expr
expr OR expr
NOT expr
```

**Rules**:
- All constraints must be satisfied
- Can reference signals and preprocessed values
- Multiple statements separated by `;`

**Examples**:
```
✅ A>B
✅ sum<==A+B;sum>100
✅ (A>10)AND(B<20)
✅ A>=18;A<=65;A!=forbidden
```

## Signal Naming

### Valid Names

**Rules**:
- Start with letter or underscore: `[A-Za-z_]`
- Followed by letters, digits, or underscores: `[A-Za-z0-9_]*`
- Case-sensitive

**Examples**:
```
✅ A
✅ myVariable
✅ user_age
✅ Balance123
✅ _secretKey
❌ 123abc        # Starts with digit
❌ my-variable   # Hyphen not allowed
❌ user.name     # Dot not allowed
```

### Reserved Names

None currently reserved, but best practices:
- Avoid single letters in production: `a`, `b`, `c`
- Use descriptive names: `userBalance`, `threshold`

## Value Encoding

### Decimal (Default)

```
amount:1000
```

**Range**:
- Arbitrary precision (BigUint)
- Ordering comparisons: [0, 2^64)
- Equality comparisons: any size

### Hexadecimal

```
hash:0x1a2b3c4d:hex
```

**Rules**:
- Optional `0x` prefix in value
- Must specify `:hex` encoding
- Case-insensitive

### Base58

```
address:9aE476sH92Vc7DMC8bZNpe1xNNNy1fNjFpCGvfMuZMwM:base58
```

**Use for**:
- Solana addresses
- Bitcoin addresses
- IPFS hashes

### Base64

```
data:SGVsbG8gV29ybGQ=:base64
```

**Rules**:
- Standard Base64 encoding
- Padding (`=`) required

## Operators

### Arithmetic

| Operator | Description | Example |
|----------|-------------|---------|
| `+` | Addition | `A+B` |
| `-` | Subtraction | `A-B` |
| `*` | Multiplication | `A*B` |
| `/` | Integer division | `A/B` |

### Comparison

| Operator | Description | Size Limit |
|----------|-------------|------------|
| `>` | Greater than | < 2^64 |
| `<` | Less than | < 2^64 |
| `>=` | Greater or equal | < 2^64 |
| `<=` | Less or equal | < 2^64 |
| `==` | Equal | Any size |
| `!=` | Not equal | Any size |

### Boolean

| Operator | Syntax | Description |
|----------|--------|-------------|
| AND | `AND`, `&&` | Boolean AND |
| OR | `OR`, `\|\|` | Boolean OR |
| NOT | `NOT`, `!` | Boolean NOT |

### Grouping

```
(expression)
```

**Examples**:
```
(A+B)*C
(A>10)AND(B<20)
NOT(A==B)
```

## Operator Precedence

**Highest to Lowest**:

1. `()` - Parentheses
2. `!`, `NOT` - Boolean NOT
3. `*`, `/` - Multiplication, Division
4. `+`, `-` - Addition, Subtraction
5. `>`, `<`, `>=`, `<=`, `==`, `!=` - Comparisons
6. `AND`, `&&` - Boolean AND
7. `OR`, `||` - Boolean OR

**Examples**:
```
A+B*C       →  A+(B*C)
A>B AND C>D →  (A>B) AND (C>D)
NOT A>B     →  (NOT A)>B       # Use: NOT(A>B)
```

## Assignment Operator

### Constraint Assignment: `<==`

```
variable<==expression
```

**Behavior**:
1. Assigns `expression` to `variable`
2. Creates constraint that assignment is correct
3. Proves computation was done correctly

**Examples**:
```
sum<==A+B             # Assigns and proves sum
hash<==sha256(A{%x})  # Assigns and proves hash
```

**Not supported**:
```
❌ variable=expression   # Plain assignment not supported
❌ variable==expression  # This is comparison, not assignment
```

## Format Specifiers (Preprocessing)

Used in hash functions to control formatting:

| Specifier | Format | Example Input | Formatted |
|-----------|--------|---------------|-----------|
| `{%x}` | Hexadecimal | `255` | `"ff"` |
| `{%d}` | Decimal | `255` | `"255"` |
| `{%s}` | String | `"hello"` | `"hello"` |

**Usage**:
```
sha256(value{%x})     # Hex format
sha256(amount{%d})    # Decimal format
sha256(name{%s})      # String format
```

## Whitespace

**Not allowed** in Zircon format.

```
✅ 1/A:10,B:20/-/A+B
❌ 1 / A:10, B:20 / - / A + B
```

Whitespace would increase size and parsing complexity.

## Comments

**Not supported** in Zircon format.

For documentation, use separate files or JSON format.

## Grammar (BNF)

```bnf
program       ::= version "/" secret "/" public "/" circuit
                | version "/" secret "/" public "/" preprocess "/" circuit

version       ::= integer

secret        ::= signals | "-"
public        ::= signals | "-"

signals       ::= signal ("," signal)*
signal        ::= name ":" value (":" encoding)?

preprocess    ::= statements
circuit       ::= statements

statements    ::= statement (";" statement)*
statement     ::= assignment | constraint | expression

assignment    ::= name "<=" "=" expression
constraint    ::= expression comparison expression
              | expression boolean expression
              | "NOT" expression
              | "!" expression

expression    ::= term (("+"|"-") term)*
term          ::= factor (("*"|"/") factor)*
factor        ::= number
              | name
              | hashfunc "(" hashargs ")"
              | "(" expression ")"

hashfunc      ::= "sha256" | "sha1" | "sha512" | "sha3_256"
              | "sha3_512" | "md5" | "blake2" | "crc32"

hashargs      ::= hasharg ("|" hasharg)*
hasharg       ::= name "{" format "}"

format        ::= "%x" | "%d" | "%s"

comparison    ::= ">" | "<" | ">=" | "<=" | "==" | "!="
boolean       ::= "AND" | "&&" | "OR" | "||"

name          ::= [A-Za-z_][A-Za-z0-9_]*
value         ::= [^\s,:;/]+
encoding      ::= "decimal" | "hex" | "base58" | "base64"
integer       ::= [0-9]+
number        ::= [0-9]+
```

## Edge Cases

### Empty Sections

**Secret signals empty**:
```
1/-/threshold:100/A>threshold
```

**Public signals empty**:
```
1/secret:123/result:?/secret>100
```

**Both empty**:
```
1/-/result:?/1>0    # Constant constraint (always true)
```

### Single Signal

```
1/A:10/result:?/A>5
```

No trailing comma needed.

### No Preprocessing

Use 4-part format:
```
1/A:10/B:20/A>B
```

**Not**:
```
❌ 1/A:10/B:20/-/A>B    # Don't use empty preprocess
```

### Multiple Constraints

```
1/A:10,B:20/result:?/A>5;B>10;A<B
```

All must be satisfied.

## Validation Rules

### At Parse Time

1. **Structure**: Must have 4 or 5 parts
2. **Version**: Must be valid integer
3. **Signal names**: Must match `[A-Za-z_][A-Za-z0-9_]*`
4. **Encoding**: Must be valid (decimal, hex, base58, base64)
5. **Operators**: Must be recognized
6. **Syntax**: Must follow grammar

### At Execution Time

1. **Variables**: All referenced variables must be defined
2. **Types**: Operations must be type-compatible
3. **Values**: Must be valid for encoding
4. **Constraints**: All must be satisfiable

## Error Messages

Common parsing errors:

```
❌ "Invalid format: expected 4 or 5 parts, got 3"
   → Missing section

❌ "Unknown encoding: base62"
   → Invalid encoding name

❌ "Invalid signal format 'A:10:20'"
   → Too many colons

❌ "Undefined variable: x"
   → Variable not in signals or preprocess

❌ "Expected ';' between statements"
   → Missing separator
```

## Best Practices

### 1. Use 4-part format when possible

```
✅ 1/A:10/B:20/A>B              # Simple, clean
⚠️  1/A:10/B:20/-/A>B           # Unnecessary empty section
```

### 2. Explicit encoding for non-decimal

```
✅ addr:0x123:hex
❌ addr:0x123                    # Ambiguous
```

### 3. Descriptive signal names

```
✅ userAge:25,minimumAge:18
❌ a:25,b:18
```

### 4. Logical statement order

```
✅ sum<==A+B;doubled<==sum*2
❌ doubled<==sum*2;sum<==A+B     # Confusing
```

## See Also

- **[Signals](SIGNALS.md)** - Signal definition details
- **[Encoding](ENCODING.md)** - Value encoding formats
- **[Preprocessing](PREPROCESSING.md)** - Preprocessing operations
- **[Operators](OPERATORS.md)** - Operator reference
- **[Examples](EXAMPLES.md)** - Real-world examples