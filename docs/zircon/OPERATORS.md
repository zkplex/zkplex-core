# Zircon Format - Operators

## Overview

Zircon supports three categories of operators for building zero-knowledge proof constraints:

1. **Arithmetic operators** - Mathematical calculations
2. **Comparison operators** - Value comparisons
3. **Boolean operators** - Boolean logic

## Arithmetic Operators

### Addition: `+`

**Syntax**: `A + B`

**Description**: Adds two values.

**Examples**:
```
1/A:10,B:20/-/sum<==A+B;sum==30
1/balance:1000,amount:100/-/balance+amount>1000
1/x:5,y:7,z:3/-/(x+y)+z==15
```

**Properties**:
- Commutative: `A+B == B+A`
- Associative: `(A+B)+C == A+(B+C)`
- Identity: `A+0 == A`

**Field arithmetic**: Outputs wrap at field modulus (~255 bits).

### Subtraction: `-`

**Syntax**: `A - B`

**Description**: Subtracts B from A.

**Examples**:
```
1/balance:1000,spent:300/-/remaining<==balance-spent;remaining>0
1/total:100,fee:10/-/net<==total-fee;net>=90
1/A:20,B:5,C:3/-/A-B-C==12
```

**Properties**:
- NOT commutative: `A-B != B-A`
- NOT associative: `(A-B)-C != A-(B-C)`
- Identity: `A-0 == A`
- Inverse: `A-A == 0`

**Underflow**: In field arithmetic, `0-1` wraps to field modulus - 1.

### Multiplication: `*`

**Syntax**: `A * B`

**Description**: Multiplies two values.

**Examples**:
```
1/price:100,quantity:5/-/total<==price*quantity;total==500
1/A:10,B:20,C:2/-/output<==A*B*C;output==400
1/x:3,y:4/-/(x*y)>10
```

**Properties**:
- Commutative: `A*B == B*A`
- Associative: `(A*B)*C == A*(B*C)`
- Identity: `A*1 == A`
- Zero: `A*0 == 0`
- Distributive: `A*(B+C) == A*B + A*C`

**Constraint cost**: ~1 constraint per multiplication.

### Division: `/`

**Syntax**: `A / B`

**Description**: Integer division (no remainder).

**Examples**:
```
1/total:100,count:5/-/average<==total/count;average==20
1/A:20,B:3/-/quotient<==A/B;quotient==6
1/amount:1000,divisor:100/-/amount/divisor>5
```

**Properties**:
- NOT commutative: `A/B != B/A`
- NOT associative: `(A/B)/C != A/(B/C)`
- Identity: `A/1 == A`

**Integer division**: `20/3 == 6` (not 6.666...)

**Division by zero**: Undefined behavior. Always ensure divisor != 0:
```
✅ 1/A:100,B:5/-/B!=0;output<==A/B
❌ 1/A:100,B:0/-/output<==A/B           # ERROR
```

## Comparison Operators

### Greater Than: `>`

**Syntax**: `A > B`

**Description**: True if A is strictly greater than B.

**Size constraint**: Both values must be < 2^64 (18,446,744,073,709,551,616)

**Examples**:
```
1/age:25,minimum:18/-/age>minimum
1/balance:1000,threshold:500/-/balance>threshold
1/A:100,B:50,C:25/-/A>B;B>C
```

**Constraint cost**: ~68 constraints (64-bit range check + is_zero gadget)

**Returns**: `1` for true, `0` for false (as field element)

**Errors**:
```
❌ 1/huge:99999999999999999999/-/huge>100
   ERROR: Value exceeds 2^64

✅ 1/huge:99999999999999999999/expected:99999999999999999999/-/huge==expected
   Use equality for large values
```

### Less Than: `<`

**Syntax**: `A < B`

**Description**: True if A is strictly less than B.

**Size constraint**: Both values must be < 2^64

**Examples**:
```
1/age:16,maximum:18/-/age<maximum
1/amount:100,limit:1000/-/amount<limit
1/A:10,B:20,C:30/-/A<B;B<C
```

**Constraint cost**: ~68 constraints

**Equivalence**: `A < B` is equivalent to `B > A`

### Greater or Equal: `>=`

**Syntax**: `A >= B`

**Description**: True if A is greater than or equal to B.

**Size constraint**: Both values must be < 2^64

**Examples**:
```
1/age:18,minimum:18/-/age>=minimum
1/balance:1000,required:500/-/balance>=required
1/score:85,passing:60/-/score>=passing
```

**Constraint cost**: ~65 constraints (64-bit range check only)

**Efficiency**: Slightly cheaper than `>` (no is_zero gadget needed)

### Less or Equal: `<=`

**Syntax**: `A <= B`

**Description**: True if A is less than or equal to B.

**Size constraint**: Both values must be < 2^64

**Examples**:
```
1/age:65,maximum:65/-/age<=maximum
1/amount:500,limit:1000/-/amount<=limit
1/A:10,B:20/-/A<=B
```

**Constraint cost**: ~65 constraints

**Equivalence**: `A <= B` is equivalent to `B >= A`

### Equal: `==`

**Syntax**: `A == B`

**Description**: True if A equals B.

**Size constraint**: **NO LIMIT** - works with arbitrary-size values

**Examples**:
```
1/password:secret/-/hash<==sha256(password{%s})/hash==storedHash
1/A:100,B:100/-/A==B
1/addr:9aE476...:base58/target:9aE476...:base58/addr==target
```

**Constraint cost**: ~3 constraints (is_zero gadget)

**Use case**: Preferred for large values (addresses, hashes)

**Large values OK**:
```
✅ 1/solanaAddr:9aE476sH92Vc7DMC8bZNpe1xNNNy1fNjFpCGvfMuZMwM:base58/expected:9aE476...:base58/solanaAddr==expected
```

### Not Equal: `!=`

**Syntax**: `A != B`

**Description**: True if A does not equal B.

**Size constraint**: **NO LIMIT** - works with arbitrary-size values

**Examples**:
```
1/value:100,forbidden:50/-/value!=forbidden
1/A:10,B:20/-/A!=B
1/id:123,excluded:456/-/id!=excluded
```

**Constraint cost**: ~3 constraints

**Equivalence**: `A != B` is equivalent to `NOT(A == B)`

## Boolean Operators

### AND

**Syntax**: `A AND B` or `A && B`

**Description**: True if both A and B are true (non-zero).

**Examples**:
```
1/age:25,balance:1000/-/(age>=18)AND(balance>100)
1/A:10,B:20,C:30/-/(A>5)&&(B>15)&&(C>25)
1/x:1,y:1/-/(x==1)AND(y==1)
```

**Truth table**:
```
A | B | A AND B
0 | 0 | 0
0 | 1 | 0
1 | 0 | 0
1 | 1 | 1
```

**Constraint cost**: Low (boolean logic)

**Properties**:
- Commutative: `A AND B == B AND A`
- Associative: `(A AND B) AND C == A AND (B AND C)`
- Identity: `A AND 1 == A`
- Zero: `A AND 0 == 0`

**Multiple conditions**:
```
1/A:10,B:20,C:30/-/A>5;B>10;C>20
```
Equivalent to: `(A>5) AND (B>10) AND (C>20)`

### OR

**Syntax**: `A OR B` or `A || B`

**Description**: True if either A or B (or both) are true.

**Examples**:
```
1/age:16,balance:1000000/-/(age>=18)OR(balance>1000000)
1/A:5,B:25/-/(A>10)||(B>20)
1/admin:1,owner:0/-/(admin==1)OR(owner==1)
```

**Truth table**:
```
A | B | A OR B
0 | 0 | 0
0 | 1 | 1
1 | 0 | 1
1 | 1 | 1
```

**Properties**:
- Commutative: `A OR B == B OR A`
- Associative: `(A OR B) OR C == A OR (B OR C)`
- Identity: `A OR 0 == A`
- Saturation: `A OR 1 == 1`

### NOT

**Syntax**: `NOT A` or `!A`

**Description**: Negates the boolean value.

**Examples**:
```
1/banned:0/-/NOT(banned==1)
1/A:10/-/!(A==0)
1/valid:1/-/NOT(valid==0)
```

**Truth table**:
```
A | NOT A
0 | 1
1 | 0
```

**Properties**:
- Double negation: `NOT(NOT A) == A`
- De Morgan's laws:
  - `NOT(A AND B) == (NOT A) OR (NOT B)`
  - `NOT(A OR B) == (NOT A) AND (NOT B)`

## Assignment Operator

### Constraint Assignment: `<==`

**Syntax**: `variable <== expression`

**Description**: Assigns value AND creates constraint.

**Examples**:
```
1/A:10,B:20/-/sum<==A+B;sum==30
1/price:100,qty:5/-/total<==price*qty;total>400
1/data:hello/-/hash<==sha256(data{%s})/hash!=0
```

**Behavior**:
1. Evaluates `expression`
2. Assigns output to `variable`
3. Creates constraint proving assignment is correct

**NOT simple assignment**: This creates a ZKP constraint, not just an assignment.

**Contrast with comparison**:
```
sum<==A+B     # Assignment + constraint
sum==A+B      # ERROR: Can't use == for assignment (use <==)
```

## Operator Precedence

**From highest to lowest**:

1. **`()`** - Parentheses (grouping)
2. **`!`, `NOT`** - Boolean NOT
3. **`*`, `/`** - Multiplication, Division
4. **`+`, `-`** - Addition, Subtraction
5. **`>`, `<`, `>=`, `<=`, `==`, `!=`** - Comparisons
6. **`AND`, `&&`** - Boolean AND
7. **`OR`, `||`** - Boolean OR

### Precedence Examples

```
A + B * C           →  A + (B * C)
A > B AND C > D     →  (A > B) AND (C > D)
NOT A > B           →  (NOT A) > B       # Probably not intended!
NOT(A > B)          →  NOT (A > B)       # Correct
A + B > C + D       →  (A + B) > (C + D)
```

### Best Practice: Use Parentheses

```
✅ (A+B) > (C+D)           # Clear
✅ NOT(A>B)                # Clear
✅ (A>10) AND (B<20)       # Clear

⚠️  A+B > C+D              # Works, but less clear
⚠️  NOT A>B                # Confusing
⚠️  A>10 AND B<20          # Works, but less clear
```

## Operator Constraints Cost

| Operator | Constraints | Notes |
|----------|-------------|-------|
| `+` | ~1 | Custom gate |
| `-` | ~1 | Custom gate |
| `*` | ~1 | Custom gate |
| `/` | ~1 | Custom gate |
| `==` | ~3 | is_zero gadget |
| `!=` | ~3 | is_zero gadget |
| `>` | ~68 | 64-bit range check + is_zero |
| `<` | ~68 | 64-bit range check + is_zero |
| `>=` | ~65 | 64-bit range check only |
| `<=` | ~65 | 64-bit range check only |
| `AND` | Low | Boolean logic |
| `OR` | Low | Boolean logic |
| `NOT` | Low | Boolean logic |

### Optimization Tips

**Prefer equality over ordering**:
```
✅ A==100                  # 3 constraints
⚠️  A>=100;A<=100          # 130 constraints
```

**Minimize range checks**:
```
✅ sum<==A+B;sum==expected     # 1 + 3 = 4 constraints
⚠️  sum<==A+B;sum>99;sum<101   # 1 + 68 + 68 = 137 constraints
```

**Combine operations**:
```
✅ output<==A+B+C              # Better
⚠️  temp<==A+B;output<==temp+C # More constraints
```

## Supported Operators Summary

### ✅ Fully Supported

**Arithmetic**:
- `+` Addition
- `-` Subtraction
- `*` Multiplication
- `/` Integer division

**Comparison**:
- `>` Greater than
- `<` Less than
- `>=` Greater or equal
- `<=` Less or equal
- `==` Equal
- `!=` Not equal

**Boolean**:
- `AND`, `&&` Boolean AND
- `OR`, `||` Boolean OR
- `NOT`, `!` Boolean NOT

**Grouping**:
- `()` Parentheses

**Assignment**:
- `<==` Constraint assignment

### ❌ Not Yet Supported

Planned for future versions:

**Bitwise**:
- `&` Bitwise AND
- `|` Bitwise OR
- `^` Bitwise XOR
- `~` Bitwise NOT
- `>>` Right shift
- `<<` Left shift

**Power**:
- `**` Exponentiation

**Modulo**:
- `%` Remainder

**Integer Division**:
- `\` Integer division without remainder (different from `/`)

**Compound Assignment**:
- `+=`, `-=`, `*=`, `/=`, `%=`

**Increment/Decrement**:
- `++`, `--`

**Ternary**:
- `? :` Conditional expression

## Common Patterns

### Range Check

```
1/value:150/-/value>=100;value<=200
```

### Multiple Conditions (AND)

```
1/A:10,B:20/-/A>5;B>10
```
Equivalent to: `(A>5) AND (B>10)`

### Alternative Conditions (OR)

```
1/age:16,balance:2000000/-/(age>=18)OR(balance>1000000)
```

### Negation

```
1/status:1/-/NOT(status==0)
```

### Complex Expression

```
1/A:10,B:20,C:5/-/((A+B)*C)>100
```

### Conditional Output

```
1/A:10,B:20,flag:1/-/output<==flag*A+(1-flag)*B;output>0
```

Output = A if flag==1, else B

## Best Practices

### 1. Use Appropriate Operator

```
✅ Large values: A==B                  # Equality
❌ Large values: A>=B;A<=B             # Ordering fails

✅ Small values: A>B                   # Ordering OK
✅ Small values: A==B                  # Also OK
```

### 2. Minimize Constraint Count

```
✅ A==expected                         # 3 constraints
❌ A>=expected;A<=expected             # 130 constraints
```

### 3. Use Parentheses

```
✅ (A+B)*(C+D)
⚠️  A+B*C+D              # Different meaning!
```

### 4. Validate Inputs

```
✅ divisor!=0;output<==amount/divisor
❌ output<==amount/divisor              # May divide by zero
```

### 5. Boolean Grouping

```
✅ (age>=18)AND(balance>100)
⚠️  age>=18 AND balance>100            # Less clear
```

## See Also

- **[Circuit](CIRCUIT.md)** - Using operators in circuits
- **[Syntax](SYNTAX.md)** - Operator syntax rules
- **[Examples](EXAMPLES.md)** - Real-world operator usage
- **[Best Practices](BEST_PRACTICES.md)** - Optimization tips
