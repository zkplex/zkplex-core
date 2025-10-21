//! Circuit complexity estimation
//!
//! Provides hardware-independent metrics for circuit requirements.

use crate::api::EstimateResponse;
use crate::circuit::Circuit;
use crate::circuit::strategy::Strategy;
use crate::parser::Expression;

/// Estimate circuit requirements with optional strategy
///
/// Analyzes the circuit and returns hardware-independent metrics
/// about the required resources (k parameter, sizes, complexity).
///
/// # Arguments
///
/// * `circuit` - The circuit to estimate
/// * `strategy` - Optional range check strategy (auto, boolean, lookup, or bitd)
///
/// If strategy is not provided, defaults to "auto" behavior.
pub fn estimate_circuit_requirements_with_strategy(
    circuit: &Circuit,
    strategy: Option<Strategy>,
) -> EstimateResponse {
    let mut operation_count = 0;
    let mut cheap_comparison_count = 0;  // ==, != (use is_zero gadget)
    let mut expensive_comparison_count = 0;  // >, <, >=, <= (use range checks)

    // Count operations in the main expression
    if let Some(expr) = &circuit.expression {
        let (ops, cheap_comps, expensive_comps) = count_operations(expr);
        operation_count += ops;
        cheap_comparison_count += cheap_comps;
        expensive_comparison_count += expensive_comps;
    }

    let preprocess_count = circuit.statements.len() as u32;
    let total_comparisons = cheap_comparison_count + expensive_comparison_count;

    let max_bits = circuit.max_range_check_bits();
    let strategy_provided = strategy.unwrap_or(Strategy::Auto);

    // For auto strategy, determine the optimal strategy based on operations
    let actual_strategy = if matches!(strategy_provided, Strategy::Auto) {
        // Check what operations the circuit uses
        let uses_ordering = circuit.uses_range_check_comparisons();
        // let _uses_boolean = circuit.uses_boolean_operations();
        // let _uses_equality = circuit.uses_equality_comparisons();

        if uses_ordering {
            // Has ordering comparisons (>, <, >=, <=)
            // Choose between bitd and lookup based on bit size
            if let Some(bits) = max_bits {
                if bits <= 16 {
                    Strategy::Lookup  // Fast proving with reasonable table size
                } else {
                    Strategy::BitD    // Avoid huge lookup tables for large values
                }
            } else {
                Strategy::BitD  // Default if can't determine bit size
            }
        } else {
            // Has boolean operations, equality checks, or only arithmetic
            Strategy::Boolean
        }
    } else {
        strategy_provided
    };

    // Determine k_min and base_overhead based on BOTH max_bits AND actual strategy
    let (k_min, base_overhead) = match actual_strategy {
        Strategy::Boolean => {
            // Boolean: Has is_zero gadget for boolean/equality ops
            // Base strategy with no range check tables
            (8, 48u32)  // Small overhead for is_zero gadget
        }
        Strategy::BitD => {
            // Bit decomposition: NO lookup tables loaded, but needs more rows for gates
            // BitD uses constraint gates for each bit, which is more expensive than lookup
            match max_bits {
                None => (8, 64u32),       // No comparisons: minimal
                Some(8) => (9, 100u32),   // BitD for 8-bit: ~100 rows base (no tables!)
                Some(16) => (10, 150u32), // BitD for 16-bit: ~150 rows base
                Some(32) => (11, 200u32), // BitD for 32-bit: ~200 rows base
                Some(64) => (12, 250u32), // BitD for 64-bit: ~250 rows base
                Some(128) => (14, 350u32), // BitD for 128-bit (MD5): ~350 rows base
                Some(256) => (17, 600u32), // BitD for 256-bit (SHA-256, Keccak): ~600 rows base
                Some(512) => (20, 1000u32), // BitD for 512-bit (SHA-512, BLAKE2b): ~1000 rows base
                Some(_) => (20, 1000u32),  // Fallback for values > 512 bits
            }
        }
        Strategy::Lookup => {
            // Lookup: Always loads tables (fast proving, larger circuit)
            // For values > 64 bits, falls back to mixed approach (tables + bit decomp)
            match max_bits {
                None => (8, 64u32),       // No comparisons
                Some(8) => (8, 256u32),   // 8-bit table: 256 rows
                Some(16) => (17, 65536u32), // 16-bit table: 65536 rows
                Some(32) => (17, 65538u32), // 8 + 16-bit tables + bit decomp for rest
                Some(64) => (17, 65540u32), // All tables + bit decomp for upper 48 bits
                Some(128) => (17, 65550u32), // Tables + bit decomp for 128-bit
                Some(256) => (17, 65600u32), // Tables + bit decomp for 256-bit (SHA-256)
                Some(512) => (17, 65700u32), // Tables + bit decomp for 512-bit (SHA-512)
                Some(_) => (17, 65700u32),  // Fallback for > 512 bits
            }
        }
        Strategy::Auto => {
            // Auto: Adaptive (threshold=16, uses lookup for small, bitd for large)
            // Similar to lookup for small values, bitd for large
            match max_bits {
                None => (8, 64u32),
                Some(8) => (8, 256u32),   // Uses 8-bit lookup table
                Some(16) => (17, 65536u32), // Uses 16-bit lookup table
                Some(32) => (17, 65538u32), // Mixed: tables + bitd
                Some(64) => (17, 65540u32), // Mixed: tables + bitd
                Some(_) => (17, 65536u32),  // Fallback
            }
        }
    };

    let op_rows = operation_count * 4;  // ~4 rows per arithmetic operation

    // Cheap comparisons (==, !=): Use is_zero gadget which is efficient
    // is_zero gadget: Only needs to prove a * inv(a) = 1 OR a = 0
    // Costs ~8 rows per comparison
    let cheap_comparison_rows = cheap_comparison_count * 8;

    // Expensive comparisons (>, <, >=, <=): Use range checks
    // Cost depends on strategy:
    // - Boolean: 0 (doesn't support ordering comparisons)
    // - Lookup: ~10-15 rows per comparison (fast table lookup)
    // - BitD: ~50-100 rows per comparison (bit decomposition gates)
    let expensive_comparison_rows = match actual_strategy {
        Strategy::Boolean => 0,  // Boolean strategy doesn't support ordering comparisons
        Strategy::BitD => expensive_comparison_count * 80,  // BitD is more expensive per comparison
        Strategy::Lookup => expensive_comparison_count * 15, // Lookup is cheaper
        Strategy::Auto => expensive_comparison_count * 25,  // Auto: use balanced estimate
    };

    // Add 25% safety margin to estimated rows
    let estimated_rows_raw = base_overhead + op_rows + cheap_comparison_rows + expensive_comparison_rows;
    let estimated_rows = (estimated_rows_raw * 5) / 4;  // +25% safety margin

    // Find minimum k where 2^k >= estimated_rows
    let mut k_estimated = 8u32;
    while (1u32 << k_estimated) < estimated_rows && k_estimated < 30 {
        k_estimated += 1;
    }

    // Final k is the maximum of estimated k and minimum k for range checks
    let final_k = k_estimated.max(k_min);

    let total_rows = 1u64 << final_k;

    // Calculate sizes
    let params_size_bytes = total_rows * 32;  // 32 bytes per curve point

    // Halo2 proof size is much larger than initially estimated
    // Base proof overhead includes:
    // - Instance commitments (public inputs)
    // - 4 advice column commitments (4 × 32 = 128 bytes)
    // - Permutation argument commitments (several curve points)
    // - Lookup commitments (if strategy uses lookups)
    // - Vanishing argument (h(X) polynomial commitment + evaluations)
    // - Multiple polynomial evaluations at challenge points
    // - IPA proof: k rounds × 2 points × 32 bytes = k × 64 bytes
    //
    // Empirical measurements show: ~10 KB base + ~3 KB per k
    let proof_size_bytes = 10240 + (final_k as u64 * 3072);  // ~10KB base + 3KB per k

    // VK size depends on circuit structure
    // Fixed columns + permutation commitments
    let fixed_columns = 4u64;  // Typical circuit has ~4 fixed columns
    let vk_size_bytes = 1024 + (fixed_columns * 32);

    // Determine complexity level
    // With optimized table loading, k can now be as low as 8 for simple circuits
    let complexity = if final_k <= 10 {
        "Very Simple".to_string()  // k=8-10: tiny circuits (256-1024 rows)
    } else if final_k <= 14 {
        "Simple".to_string()  // k=11-14: small circuits (2K-16K rows)
    } else if final_k <= 18 {
        "Medium".to_string()  // k=15-18: medium circuits (32K-256K rows)
    } else if final_k <= 22 {
        "Complex".to_string()  // k=19-22: large circuits (512K-4M rows)
    } else if final_k <= 26 {
        "Very Complex".to_string()  // k=23-26: very large (8M-64M rows)
    } else {
        "Extremely Complex".to_string()  // k=27+: huge circuits (128M+ rows)
    };

    EstimateResponse {
        k: final_k,
        total_rows,
        estimated_rows,
        operation_count,
        comparison_count: total_comparisons,
        preprocess_count,
        params_size_bytes,
        proof_size_bytes,
        vk_size_bytes,
        complexity,
    }
}

/// Count operations in an expression tree
///
/// Returns (total_operations, cheap_comparisons, expensive_comparisons)
///
/// Cheap comparisons: ==, != (use is_zero gadget, ~8 rows)
/// Expensive comparisons: >, <, >=, <= (use range checks, ~25 rows)
fn count_operations(expr: &Expression) -> (u32, u32, u32) {
    use crate::parser::ComparisonOperator;

    match expr {
        Expression::Constant(_) | Expression::Variable(_) | Expression::Boolean(_) => (1, 0, 0),

        Expression::BinaryOp { left, right, .. } => {
            let (left_ops, left_cheap, left_expensive) = count_operations(left);
            let (right_ops, right_cheap, right_expensive) = count_operations(right);
            (
                2 + left_ops + right_ops,
                left_cheap + right_cheap,
                left_expensive + right_expensive
            )
        }

        Expression::Comparison { op, left, right } => {
            let (left_ops, left_cheap, left_expensive) = count_operations(left);
            let (right_ops, right_cheap, right_expensive) = count_operations(right);

            // Determine if this comparison is cheap or expensive based on operator
            let (new_cheap, new_expensive) = match op {
                // Cheap: is_zero gadget based (equality checks)
                ComparisonOperator::Equal | ComparisonOperator::NotEqual => (1, 0),

                // Expensive: range check based (ordering checks)
                ComparisonOperator::Greater | ComparisonOperator::Less |
                ComparisonOperator::GreaterEqual | ComparisonOperator::LessEqual => (0, 1),
            };

            (
                2 + left_ops + right_ops,
                left_cheap + right_cheap + new_cheap,
                left_expensive + right_expensive + new_expensive
            )
        }

        Expression::BooleanOp { left, right, .. } => {
            let (left_ops, left_cheap, left_expensive) = count_operations(left);
            let (right_ops, right_cheap, right_expensive) = count_operations(right);
            (
                2 + left_ops + right_ops,
                left_cheap + right_cheap,
                left_expensive + right_expensive
            )
        }

        Expression::UnaryOp { operand, .. } => {
            let (ops, cheap, expensive) = count_operations(operand);
            (1 + ops, cheap, expensive)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_simple_circuit_estimate() {
        use crate::parser::parse_circuit;

        let expr = parse_circuit("A + B").unwrap();
        let circuit = Circuit::new(expr, HashMap::new(), vec![]);

        let estimate = estimate_circuit_requirements_with_strategy(&circuit, None);

        // Simple arithmetic without comparisons should use minimal k (8)
        assert_eq!(estimate.k, 8);
        assert_eq!(estimate.total_rows, 1 << estimate.k);
        assert!(estimate.operation_count > 0);
        assert_eq!(estimate.comparison_count, 0);
        assert_eq!(estimate.complexity, "Very Simple");
    }

    #[test]
    fn test_comparison_circuit_estimate() {
        use crate::parser::parse_circuit;

        let expr = parse_circuit("A > B").unwrap();
        let circuit = Circuit::new(expr, HashMap::new(), vec![]);

        let estimate = estimate_circuit_requirements_with_strategy(&circuit, None);

        assert!(estimate.comparison_count >= 1);
        // With small values and 8-bit table (256 rows) + operations, k=8 or 9 is expected
        assert!(estimate.k <= 9);
        assert_eq!(estimate.complexity, "Very Simple");
    }

    #[test]
    fn test_complex_circuit_estimate() {
        use crate::parser::parse_circuit;

        let expr = parse_circuit("(A + B) * C > D && E < F").unwrap();
        let circuit = Circuit::new(expr, HashMap::new(), vec![]);

        let estimate = estimate_circuit_requirements_with_strategy(&circuit, None);

        assert!(estimate.operation_count > 5);
        assert!(estimate.comparison_count >= 2);
        // Complex circuit with multiple operations should use k >= 8 (optimized from 11 to 10 columns)
        assert!(estimate.k >= 8);
        assert!(estimate.k <= 10);
    }

    #[test]
    fn test_size_calculations() {
        use crate::parser::parse_circuit;

        let expr = parse_circuit("A + B").unwrap();
        let circuit = Circuit::new(expr, HashMap::new(), vec![]);

        let estimate = estimate_circuit_requirements_with_strategy(&circuit, None);

        // Params size should be 2^k * 32 bytes
        assert_eq!(estimate.params_size_bytes, estimate.total_rows * 32);

        // Proof size for k=8 should be ~10KB + 8*3KB = ~34KB
        // Halo2 proofs are larger than other systems due to flexibility
        assert!(estimate.proof_size_bytes >= 10240);
        assert!(estimate.proof_size_bytes < 50000);  // Upper bound ~50KB

        // VK size should be small
        assert!(estimate.vk_size_bytes >= 1024);
        assert!(estimate.vk_size_bytes < 10240);
    }

    #[test]
    fn test_complexity_levels() {
        use crate::parser::parse_circuit;

        // Very simple circuit
        let expr1 = parse_circuit("A").unwrap();
        let circuit1 = Circuit::new(expr1, HashMap::new(), vec![]);
        let est1 = estimate_circuit_requirements_with_strategy(&circuit1, None);
        assert!(est1.complexity.contains("Simple"));

        // More complex circuit
        let expr2 = parse_circuit("(A + B) * (C + D) > (E * F)").unwrap();
        let circuit2 = Circuit::new(expr2, HashMap::new(), vec![]);
        let est2 = estimate_circuit_requirements_with_strategy(&circuit2, None);
        // After optimization (10 columns), both might be k=8, so just check complexity difference
        assert!(est2.k >= est1.k);
        assert!(est2.operation_count > est1.operation_count);
    }
}