# Zircon Format - Examples

## Overview

This document provides real-world examples of Zircon format programs, from simple to complex use cases.

## Example Categories

1. **Basic Arithmetic** - Simple calculations and comparisons
2. **Range Proofs** - Proving values in ranges
3. **Authentication** - Password and credential verification
4. **Blockchain** - Address and signature verification
5. **Financial** - Balance and transaction proofs
6. **Complex Logic** - Multi-step computations
7. **Real-World Applications** - Complete use cases

## Basic Arithmetic

### Example 1: Simple Comparison

**Goal**: Prove A > B

```
1/A:10,B:5/result:?/A>B
```

**Breakdown**:
- Version: `1`
- Secret: `A=10, B=5`
- Public: None
- Circuit: `A > B`

**Output**: Proof that A is greater than B, without revealing values.

### Example 2: Addition

**Goal**: Prove sum of two numbers

```
1/A:10,B:20/result:?/sum<==A+B;sum==30
```

**Breakdown**:
- Computes: `sum = A + B = 30`
- Proves: Sum equals 30

### Example 3: Multiplication

**Goal**: Calculate product

```
1/price:100,quantity:5/result:?/total<==price*quantity;total==500
```

**Breakdown**:
- Computes: `total = 100 × 5 = 500`
- Proves: Product is 500

### Example 4: Division

**Goal**: Integer division

```
1/total:100,count:5/result:?/average<==total/count;average==20
```

**Breakdown**:
- Computes: `average = 100 ÷ 5 = 20`
- Proves: Average is 20

## Range Proofs

### Example 5: Age Verification

**Goal**: Prove age ≥ 18 without revealing age

```
1/age:25/result:?/age>=18
```

**Use case**: Age-restricted services, KYC compliance

**Privacy**: Actual age remains secret, only proves minimum requirement.

### Example 6: Range Check

**Goal**: Prove value in range [100, 200]

```
1/value:150/result:?/value>=100;value<=200
```

**Use case**: Value bounds verification, sensor data validation

### Example 7: Public Threshold

**Goal**: Prove secret value exceeds public threshold

```
1/balance:1000/minimumBalance:500/balance>=minimumBalance
```

**Breakdown**:
- Secret: `balance = 1000`
- Public: `minimumBalance = 500`
- Proves: Balance meets minimum requirement

### Example 8: Multi-Range

**Goal**: Prove value in specific ranges

```
1/score:85/result:?/(score>=80;score<90)OR(score>=95;score<=100)
```

**Use case**: Grade validation (B or A+ range)

## Authentication

### Example 9: Password Verification

**Goal**: Prove password knowledge without revealing it

```
1/password:secret123/result:?/hash<==sha256(password{%s})/hash==0x2bb80d537b1da3e38bd30361aa855686bde0eacd7162fef6a25fe97bf527a25b:hex
```

**Breakdown**:
- Secret: `password = "secret123"`
- Preprocessing: `hash = SHA256("secret123")`
- Circuit: `hash == expected_hash`

**Use case**: Zero-knowledge authentication, password proofs

### Example 10: User ID + Password

**Goal**: Authenticate with user ID and password

```
1/userId:12345,password:mypass/result:?/hash<==sha256(userId{%d}|password{%s})/hash==0xabc123:hex
```

**Breakdown**:
- Concatenates: `"12345" | "mypass"`
- Hashes combined value
- Proves: User knows both credentials

### Example 11: Multi-Factor Authentication

**Goal**: Prove knowledge of password + OTP

```
1/password:secret,otp:123456/result:?/passHash<==sha256(password{%s});otpHash<==sha256(otp{%d});combined<==sha256(passHash{%x}|otpHash{%x})/combined==expectedHash
```

**Breakdown**:
1. Hash password separately
2. Hash OTP separately
3. Combine and hash again
4. Verify against expected hash

## Blockchain

### Example 12: Ethereum Address Verification

**Goal**: Prove address ownership without revealing it

```
1/myAddress:0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb:hex/targetAddress:0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb:hex/myAddress==targetAddress
```

**Use case**: Secret address matching, airdrop eligibility

### Example 13: Solana Public Key

**Goal**: Verify Solana address

```
1/myPubkey:9aE476sH92Vc7DMC8bZNpe1xNNNy1fNjFpCGvfMuZMwM:base58/expectedPubkey:9aE476sH92Vc7DMC8bZNpe1xNNNy1fNjFpCGvfMuZMwM:base58/myPubkey==expectedPubkey
```

**Use case**: Solana wallet verification, governance participation

### Example 14: Transaction Hash

**Goal**: Verify transaction ID

```
1/txData:0x1a2b3c4d:hex/result:?/txHash<==sha256(txData{%x})/txHash==0xexpectedHash:hex
```

**Use case**: Transaction proof, payment verification

### Example 15: Multiple Addresses

**Goal**: Prove ownership of multiple addresses

```
1/addr1:0xabc:hex,addr2:0xdef:hex/target1:0xabc:hex,target2:0xdef:hex/addr1==target1;addr2==target2
```

**Use case**: Multi-sig verification, portfolio proof

## Financial

### Example 16: Balance Proof

**Goal**: Prove sufficient balance for transaction

```
1/balance:1000,amount:300,fee:50/result:?/totalCost<==amount+fee;remaining<==balance-totalCost;remaining>=0;amount>0
```

**Breakdown**:
1. Calculate total cost: `amount + fee = 350`
2. Calculate remaining: `1000 - 350 = 650`
3. Prove: Remaining ≥ 0 (sufficient funds)
4. Prove: Amount > 0 (valid transaction)

**Use case**: Secret payment proofs, account validation

### Example 17: Income Range

**Goal**: Prove income in range without revealing amount

```
1/income:75000/result:?/income>=50000;income<=100000
```

**Use case**: Loan applications, eligibility checks

### Example 18: Tax Bracket

**Goal**: Prove tax calculation correct

```
1/income:100000,rate:25/expectedTax:25000/tax<==income*rate/100;tax==expectedTax
```

**Breakdown**:
- Income: Secret
- Tax rate: Public
- Expected tax: Public
- Proves: Correct tax calculation

### Example 19: Portfolio Value

**Goal**: Prove total portfolio value exceeds threshold

```
1/asset1:10000,asset2:20000,asset3:15000/minimumValue:40000/total<==asset1+asset2+asset3;total>=minimumValue
```

**Use case**: Collateral verification, net worth proof

## Complex Logic

### Example 20: Conditional Output

**Goal**: Select value based on flag

```
1/A:10,B:20,flag:1/result:?/output<==flag*A+(1-flag)*B;output>5
```

**Breakdown**:
- If `flag == 1`: `output = A = 10`
- If `flag == 0`: `output = B = 20`
- Proves: Output > 5

**Use case**: Conditional payments, feature flags

### Example 21: Weighted Average

**Goal**: Calculate weighted average

```
1/v1:100,w1:3,v2:80,w2:2/expectedAvg:92/weightedSum<==v1*w1+v2*w2;totalWeight<==w1+w2;avg<==weightedSum/totalWeight;avg==expectedAvg
```

**Breakdown**:
1. `weightedSum = 100×3 + 80×2 = 460`
2. `totalWeight = 3 + 2 = 5`
3. `avg = 460 ÷ 5 = 92`

**Use case**: Grade calculation, scoring systems

### Example 22: Multi-Step Computation

**Goal**: Complex calculation with multiple steps

```
1/A:10,B:20,C:5/result:?/step1<==A+B;step2<==step1*C;step3<==step2-A;output<==step3/B;output>5
```

**Breakdown**:
1. `step1 = 10 + 20 = 30`
2. `step2 = 30 × 5 = 150`
3. `step3 = 150 - 10 = 140`
4. `output = 140 ÷ 20 = 7`
5. Prove: `output > 5` ✓

### Example 23: Boolean Combinations

**Goal**: Complex boolean conditions

```
1/age:25,balance:1000,status:1/result:?/(age>=18)AND((balance>500)OR(status==1))
```

**Breakdown**:
- Proves: Age ≥ 18 AND (Balance > 500 OR Status == 1)
- All conditions must be satisfied

**Use case**: Multi-criteria verification, access control

## Real-World Applications

### Example 24: KYC Compliance

**Goal**: Prove age, income, and residency

```
1/age:30,income:80000,countryCode:840/result:?/age>=18;income>=50000;countryCode==840
```

**Breakdown**:
- Age ≥ 18 (adult)
- Income ≥ $50,000 (minimum requirement)
- Country code = 840 (USA)

**Use case**: Financial services onboarding

### Example 25: Credit Score Proof

**Goal**: Prove credit score in acceptable range

```
1/creditScore:750/result:?/creditScore>=700;creditScore<=850
```

**Use case**: Loan applications, rental applications

### Example 26: Subscription Eligibility

**Goal**: Verify subscription tier and payment

```
1/userId:12345,tier:3,lastPayment:20231001/result:?/tier>=2;lastPayment>=20231001
```

**Breakdown**:
- Tier ≥ 2 (premium or higher)
- Last payment ≥ 2023-10-01 (recent)

**Use case**: Access control, feature gates

### Example 27: Voting Eligibility

**Goal**: Prove eligible to vote

```
1/age:25,citizenship:1,felon:0/result:?/age>=18;citizenship==1;felon==0
```

**Breakdown**:
- Age ≥ 18
- Citizenship = 1 (citizen)
- Felon = 0 (not a felon)

**Use case**: Anonymous voting, governance

### Example 28: Insurance Premium

**Goal**: Verify insurance calculation

```
1/age:30,riskScore:75/baseRate:100/premium<==baseRate+(age/10)+(riskScore/10);premium<=200
```

**Breakdown**:
- `premium = 100 + 3 + 7.5 = 110.5`
- Prove: Premium ≤ $200 (reasonable)

### Example 29: Airdrop Eligibility

**Goal**: Prove eligible for token airdrop

```
1/myAddress:0xabc:hex,balance:1000,txCount:50/minBalance:100,minTx:10/balance>=minBalance;txCount>=minTx
```

**Breakdown**:
- Balance ≥ 100 tokens
- Transaction count ≥ 10
- Address verification

**Use case**: Token airdrops, rewards distribution

### Example 30: Decentralized Identity

**Goal**: Prove identity attributes

```
1/birthYear:1990,nationality:840,educationLevel:4/result:?/age<==2024-birthYear;age>=18;nationality==840;educationLevel>=3
```

**Breakdown**:
- Calculate age: `2024 - 1990 = 34`
- Prove: Age ≥ 18
- Prove: Nationality = 840 (USA)
- Prove: Education level ≥ 3 (Bachelor's or higher)

**Use case**: Job applications, credential verification

## Advanced Examples

### Example 31: Merkle Tree Membership

**Goal**: Prove value is in Merkle tree

```
1/leaf:0xabc:hex,sibling1:0xdef:hex,sibling2:0x123:hex/root:0x789:hex/hash1<==sha256(leaf{%x}|sibling1{%x});hash2<==sha256(hash1{%x}|sibling2{%x});hash2==root
```

**Breakdown**:
1. Hash leaf with sibling1
2. Hash output with sibling2
3. Verify equals root

**Use case**: Anonymous set membership, whitelist proofs

### Example 32: Double Spending Prevention

**Goal**: Prove UTXO validity

```
1/utxoAmount:1000,spendAmount:700/result:?/change<==utxoAmount-spendAmount;change>=0;spendAmount>0;utxoAmount>0
```

**Breakdown**:
- UTXO amount = 1000
- Spend amount = 700
- Change = 300
- All values positive

**Use case**: Blockchain UTXO validation

### Example 33: Privacy-Preserving Auction

**Goal**: Prove bid is valid without revealing amount

```
1/bidAmount:5000/minimumBid:1000,maximumBid:10000/bidAmount>=minimumBid;bidAmount<=maximumBid
```

**Use case**: Sealed-bid auctions, secret bidding

### Example 34: Supply Chain Verification

**Goal**: Verify product authenticity

```
1/productId:12345,batchNumber:67890,manufactureDate:20231001/result:?/productHash<==sha256(productId{%d}|batchNumber{%d}|manufactureDate{%d})/productHash==expectedHash
```

**Use case**: Anti-counterfeiting, supply chain tracking

### Example 35: Cross-Chain Bridge

**Goal**: Prove token balance on source chain

```
1/sourceBalance:1000,bridgeAmount:500/minBridge:100,maxBridge:1000/remaining<==sourceBalance-bridgeAmount;remaining>=0;bridgeAmount>=minBridge;bridgeAmount<=maxBridge
```

**Breakdown**:
- Source balance: 1000
- Bridge amount: 500
- Remaining: 500 (valid)
- Within bridge limits

**Use case**: Cross-chain token transfers

## Performance Considerations

### Example 36: Optimized vs Unoptimized

**Unoptimized** (137 constraints):
```
1/value:100/result:?/value>99;value<101
```

**Optimized** (3 constraints):
```
1/value:100/expected:100/value==expected
```

**Lesson**: Use equality when possible instead of range checks.

### Example 37: Constraint Minimization

**Unoptimized**:
```
1/A:10,B:20,C:30/result:?/temp1<==A+B;temp2<==temp1+C;temp2>50
```

**Optimized**:
```
1/A:10,B:20,C:30/result:?/sum<==A+B+C;sum>50
```

**Lesson**: Combine operations to reduce intermediate variables.

## Edge Cases

### Example 38: Zero Division Prevention

```
1/amount:100,divisor:5/result:?/divisor!=0;output<==amount/divisor;output>0
```

**Key**: Always check divisor ≠ 0 before division.

### Example 39: Underflow Prevention

```
1/balance:1000,withdrawal:500/result:?/remaining<==balance-withdrawal;remaining>=0
```

**Key**: Verify output ≥ 0 to prevent underflow.

### Example 40: Large Value Equality

```
1/solanaAddr:9aE476sH92Vc7DMC8bZNpe1xNNNy1fNjFpCGvfMuZMwM:base58/expectedAddr:9aE476sH92Vc7DMC8bZNpe1xNNNy1fNjFpCGvfMuZMwM:base58/solanaAddr==expectedAddr
```

**Key**: Use equality (not ordering) for large values like addresses.

## Testing Examples

### Example 41: Simple Unit Test

```
1/A:10,B:5/result:?/A>B
```

**Expected**: Proof succeeds

```
1/A:5,B:10/result:?/A>B
```

**Expected**: Proof fails (A not > B)

### Example 42: Boundary Testing

```
1/age:18/result:?/age>=18
```

**Expected**: Succeeds (exact boundary)

```
1/age:17/result:?/age>=18
```

**Expected**: Fails (below boundary)

## Common Patterns Summary

| Pattern | Example | Use Case |
|---------|---------|----------|
| **Range check** | `value>=min;value<=max` | Age, amount bounds |
| **Hash verification** | `hash<==sha256(data{%s})/hash==expected` | Password, credentials |
| **Balance check** | `remaining<==balance-amount;remaining>=0` | Payments, transfers |
| **Conditional** | `output<==flag*A+(1-flag)*B` | Feature flags, options |
| **Aggregation** | `sum<==v1+v2+v3;avg<==sum/3` | Averages, totals |
| **Concatenation** | `hash<==sha256(id{%d}\|pass{%s})` | Multi-value auth |
| **Multi-step** | `a<==f(x);b<==g(a);c<==h(b)` | Complex calculations |

## See Also

- **[Operators](OPERATORS.md)** - Operator reference
- **[Preprocessing](PREPROCESSING.md)** - Hash functions and transformations
- **[Circuit](CIRCUIT.md)** - Circuit constraints
- **[Best Practices](BEST_PRACTICES.md)** - Optimization tips
- **[Tools](TOOLS.md)** - CLI and API usage