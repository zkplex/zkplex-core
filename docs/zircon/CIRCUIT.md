# Zircon Format - Circuit

## Overview

The **circuit** section is the heart of your zero-knowledge proof. It defines the constraints that must be satisfied for the proof to be valid.

```
version/secret/public/circuit
                     └──────┘
                      Circuit constraints
```

With preprocessing:
```
version/secret/public/preprocess/circuit
                                └──────┘
                                 Circuit constraints
```

## Circuit Output

**The result of the last expression becomes the circuit output.**

- For simple expressions: `(A + B) > C` → output is `1` (true) or `0` (false)
- For assignments: `result<==A+B` → output is the value of `result`
- For multiple statements: `sum<==A+B;sum>100` → output is `1` or `0` from the last expression

**Note**: The variable name `output` has no special meaning. You can use it like any other variable, or not use it at all. The circuit output is always determined by the last expression.

## Purpose

The circuit section:
- Defines **what you want to prove**
- Specifies **constraints** that must be satisfied
- Uses signals from secret/public sections
- Can use values from preprocessing
- **All statements must be true** for proof to be valid

## Basic Syntax

### Single Constraint

```
expression OP expression
```

Where `OP` is a comparison operator: `>`, `<`, `>=`, `<=`, `==`, `!=`

**Example**:
```
1/age:25/result:?/age>=18
```

### Multiple Constraints

Separated by semicolons `;`:

```
constraint1;constraint2;constraint3
```

**All must be satisfied**:
```
1/A:10,B:20,C:15/result:?/A>5;B>15;C<20
```

Proves: A > 5 **AND** B > 15 **AND** C < 20

## Constraint Types

### 1. Comparison Constraints

Direct comparisons between expressions:

```
A > B
A >= 100
(A+B) < C
```

**Examples**:
```
1/balance:1000/threshold:500/balance>=threshold
1/age:25,min:18,max:65/age>=min;age<=max
1/A:10,B:20,C:5/A+B>C
```

### 2. Assignment Constraints

Assign and constrain simultaneously:

```
variable<==expression
```

**Examples**:
```
1/A:10,B:20/result:?/sum<==A+B;sum>25
1/price:100,qty:5/result:?/total<==price*qty;total<=1000
```

### 3. Boolean Constraints

Combine conditions with AND, OR, NOT:

```
(A>10) AND (B<20)
(flag==1) OR (balance>1000)
NOT(age<18)
```

**Examples**:
```
1/age:25,balance:1000/result:?/(age>=18)AND(balance>100)
1/A:10,B:5/result:?/(A>B)OR(A==0)
1/status:1/result:?/NOT(status==0)
```

### 4. Equality Constraints

Check if values are equal:

```
hash==expected
A==B
```

**Examples**:
```
1/password:secret/result:?/hash<==sha256(password{%s})/hash==storedHash
1/A:10,B:10/result:?/A==B
```

## Multiple Constraints

### All Must Be True

When you have multiple constraints, **ALL** must be satisfied:

```
1/A:10,B:20,C:30/result:?/A>5;B>15;C>25;A<B;B<C
```

This proves:
1. A > 5 ✓
2. B > 15 ✓
3. C > 25 ✓
4. A < B ✓
5. B < C ✓

**If ANY constraint fails, the entire proof fails.**

### Sequential Evaluation

Constraints with assignments are evaluated in order:

```
1/A:10,B:20/result:?/sum<==A+B;doubled<==sum*2;doubled>50
```

**Execution**:
1. `sum = A + B = 30`
2. `doubled = sum * 2 = 60`
3. Check: `doubled > 50` ✓

### Inter-dependent Constraints

Later constraints can use variables from earlier ones:

```
1/principal:1000,rate:5,time:2/result:?/interest<==principal*rate*time/100;total<==principal+interest;total>1000
```

**Execution**:
1. `interest = (1000 * 5 * 2) / 100 = 100`
2. `total = 1000 + 100 = 1100`
3. Check: `total > 1000` ✓

## Complex Expressions

### Arithmetic in Constraints

```
1/A:10,B:20,C:5/result:?/(A+B)*C>100
```

Proves: `(10 + 20) * 5 = 150 > 100` ✓

### Nested Expressions

```
1/A:10,B:20,C:30,D:40/result:?/((A+B)*(C-D))>-500
```

Calculation: `(10 + 20) * (30 - 40) = 30 * (-10) = -300 > -500` ✓

### Mixed Operations

```
1/balance:1000,price:100,qty:5,fee:50/result:?/total<==price*qty+fee;remaining<==balance-total;remaining>=0
```

## Using Preprocessing

Circuit can use values computed in preprocessing:

```
1/secret:hello/result:?/hash<==sha256(secret{%s})/hash==expectedHash
                       └──────────┬─────────┘ └──────┬───────┘
                          Preprocessing        Circuit uses 'hash'
```

**Another example**:
```
1/A:10,B:20/result:?/sum<==A+B;hash<==sha256(sum{%x})/hash>0x1000;sum<100
           Preprocess ────┘                    └──── Circuit uses both
```

## Constraint Patterns

### Pattern 1: Range Check

Prove value in range [min, max]:

```
1/value:150/min:100,max:200/value>=min;value<=max
```

### Pattern 2: Non-Zero Check

Prove value is not zero:

```
1/amount:100/result:?/amount!=0
```

### Pattern 3: Multi-Step Validation

```
1/balance:1000,amount:100,fee:10/minBalance:50/cost<==amount+fee;remaining<==balance-cost;remaining>=minBalance
```

Validates:
1. Calculate total cost
2. Calculate remaining balance
3. Ensure remaining ≥ minimum

### Pattern 4: Conditional Logic

```
1/A:10,B:20,flag:1/result:?/output<==flag*A+(1-flag)*B;output>5
```

Output = A if flag==1, else B

### Pattern 5: Multiple Comparisons

```
1/A:10,B:20,C:15/result:?/A<B;B>C;A<C
```

Proves ordering: A < C < B

### Pattern 6: Hash Verification

```
1/data:secret/result:?/hash<==sha256(data{%s})/hash==expected
```

Proves knowledge of data that hashes to expected value.

## Operators in Circuit

### Arithmetic Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `+` | Addition | `A+B>100` |
| `-` | Subtraction | `A-B<50` |
| `*` | Multiplication | `A*B>=200` |
| `/` | Division | `A/B==5` |

### Comparison Operators

| Operator | Description | Size Limit |
|----------|-------------|------------|
| `>` | Greater than | < 2^64 |
| `<` | Less than | < 2^64 |
| `>=` | Greater or equal | < 2^64 |
| `<=` | Less or equal | < 2^64 |
| `==` | Equal | Any size |
| `!=` | Not equal | Any size |

**Important**: Ordering comparisons (`>`, `<`, `>=`, `<=`) require values < 2^64

### Boolean Operators

| Operator | Syntax | Description |
|----------|--------|-------------|
| AND | `AND`, `&&` | Both must be true |
| OR | `OR`, `\|\|` | At least one true |
| NOT | `NOT`, `!` | Negation |

### Parentheses

```
(expression)
```

Control evaluation order:
```
(A+B)*C != A+B*C
```

## Operator Precedence

From highest to lowest:

1. `()` Parentheses
2. `!`, `NOT` Boolean NOT
3. `*`, `/` Multiplication, Division
4. `+`, `-` Addition, Subtraction
5. `>`, `<`, `>=`, `<=`, `==`, `!=` Comparisons
6. `AND`, `&&` Boolean AND
7. `OR`, `||` Boolean OR

**Example**:
```
A + B * C         →  A + (B * C)
A > B AND C > D   →  (A > B) AND (C > D)
```

**Use parentheses for clarity**:
```
✅ (A+B) > (C+D)
⚠️  A+B > C+D        # Same, but less clear
```

## Common Patterns

### 1. Simple Threshold

```
1/value:150/threshold:100/value>=threshold
```

### 2. Range Proof

```
1/value:150/result:?/value>=100;value<=200
```

### 3. Equality Check

```
1/A:100,B:100/result:?/A==B
```

### 4. Inequality Check

```
1/A:100/forbidden:50/A!=forbidden
```

### 5. Complex Computation

```
1/A:10,B:20,C:5/result:?/output<==A*B+C;output>100
```

### 6. Multi-Value Validation

```
1/v1:10,v2:20,v3:30/result:?/sum<==v1+v2+v3;sum>50;v1>0;v2>0;v3>0
```

All values positive and sum > 50.

### 7. Conditional Output

```
1/A:10,B:20,useA:1/result:?/output<==useA*A+(1-useA)*B;output>15
```

### 8. Balance After Transaction

```
1/balance:1000,amount:100,fee:10/result:?/total<==amount+fee;remaining<==balance-total;remaining>=0;amount>0
```

## Constraint Complexity

### Simple (Low cost)

Single comparison:
```
A > B
```

**Constraints**: ~68 (for 64-bit range check)

### Medium (Moderate cost)

Multiple comparisons:
```
A>10;B>20;A<B
```

**Constraints**: ~204 (3 range checks)

### Complex (Higher cost)

Computations + comparisons:
```
sum<==A+B;product<==C*D;sum>100;product>1000;sum<product
```

**Constraints**: ~340+ (arithmetic + range checks)

### Very Complex (Highest cost)

Hash + multiple constraints:
```
hash<==sha256(data{%s});partial<==hash/1000;partial>100;hash!=0
```

**Constraints**: 1000+ (SHA256 circuit is expensive)

## Best Practices

### 1. Order Constraints Logically

```
✅ sum<==A+B;product<==A*B;sum>product
❌ sum>product;sum<==A+B;product<==A*B
```

### 2. Avoid Redundant Constraints

```
✅ A>10;A<20
❌ A>10;A<20;A!=0      # Redundant: A>10 implies A!=0
```

### 3. Use Meaningful Variable Names

```
✅ remaining<==balance-amount;remaining>=0
❌ r<==b-a;r>=0
```

### 4. Break Complex Logic

```
✅ step1<==A+B;step2<==step1*C;output<==step2-D;output>100
❌ (((A+B)*C)-D)>100
```

### 5. Validate Inputs

```
✅ amount>0;fee>=0;balance>=amount+fee
❌ balance>=amount+fee                      # Missing input validation
```

### 6. Use Parentheses for Clarity

```
✅ (A>10)AND(B<20)
⚠️  A>10 AND B<20         # Works, but less clear
```

## Common Errors

### Error 1: Undefined Variable

```
❌ 1/A:10/-/B>5
ERROR: Variable 'B' not defined

✅ 1/A:10,B:20/-/B>5
```

### Error 2: Value Too Large for Comparison

```
❌ 1/huge:999999999999999999999999/-/huge>100
ERROR: Value exceeds 2^64 for ordering comparison

✅ 1/huge:999999999999999999999999/expected:999999999999999999999999/huge==expected
   (Use equality instead)
```

### Error 3: Missing Semicolon

```
❌ 1/A:10,B:20/-/A>5 B>10
ERROR: Expected ';' between statements

✅ 1/A:10,B:20/-/A>5;B>10
```

### Error 4: Using Assignment in Comparison

```
❌ 1/A:10,B:20/-/(sum=A+B)>25
ERROR: Use <== for assignment

✅ 1/A:10,B:20/-/sum<==A+B;sum>25
```

### Error 5: Circular Dependency

```
❌ 1/A:10/-/B<==A+C;C<==B+1
ERROR: Circular dependency

✅ 1/A:10,C:5/-/B<==A+C;D<==B+1
```

## Advanced Topics

### Constraint Counting

Each constraint type has different costs:

- **Assignment**: ~1 constraint
- **Arithmetic**: ~1 constraint per operation
- **Equality**: ~3 constraints (is_zero gadget)
- **Greater/Less**: ~68 constraints (64-bit range check)
- **GreaterEqual/LessEqual**: ~65 constraints

**Example**:
```
A>B                # ~68 constraints
A==B               # ~3 constraints
sum<==A+B;sum>100  # ~1 + ~68 = ~69 constraints
```

### Optimization Tips

**Minimize range checks**:
```
✅ sum<==A+B;sum==expected           # 1 + 3 = 4 constraints
⚠️  sum<==A+B;sum>expected-1;sum<expected+1  # 1 + 68 + 68 = 137 constraints
```

**Use equality when possible**:
```
✅ A==100                            # 3 constraints
⚠️  A>=100;A<=100                    # 65 + 65 = 130 constraints
```

**Combine constraints**:
```
✅ output<==A+B+C;output>100         # Better
⚠️  partial1<==A+B;partial2<==partial1+C;partial2>100  # More constraints
```

## See Also

- **[Operators](OPERATORS.md)** - Detailed operator reference
- **[Preprocessing](PREPROCESSING.md)** - Computing values before circuit
- **[Signals](SIGNALS.md)** - Defining input signals
- **[Examples](EXAMPLES.md)** - Real-world circuit examples
- **[Best Practices](BEST_PRACTICES.md)** - Circuit optimization