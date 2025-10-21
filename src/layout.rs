//! Circuit layout visualization
//!
//! This module provides ASCII visualization of circuit layouts, showing:
//! - Row distribution (range check tables, circuit gates, unused rows)
//! - Resource requirements (proving key, proof size, verification key)
//! - Signal information (secret and public signals)
//! - Operation breakdown (arithmetic, comparison, preprocessing)
//! - Column configuration (advice, instance, selector, fixed)
//! - Gate type breakdown (arithmetic, comparison, preprocessing gates)
//! - Lookup table information (table sizes, overhead)
//! - Memory usage estimates (prover and verifier)
//! - Complexity analysis (timing estimates, optimization suggestions)

use crate::api::{
    Program,
    layout::{
        CircuitLayout, CircuitParameters, RowLayout, ResourceRequirements,
        SignalInformation, OperationBreakdown, ColumnConfiguration,
        GateBreakdown, ArithmeticGates, ComparisonGates, PreprocessingGates,
        LookupTableInfo, MemoryUsage, ProverMemory, VerifierMemory,
        ComplexityAnalysis,
    },
};
use crate::circuit::{Circuit, estimate_circuit_requirements_with_strategy, Strategy};
use std::process;

/// Helper function to format a line with fixed width, padded with spaces
fn format_table_line(text: &str, width: usize) -> String {
    let len = text.chars().count();
    if len >= width {
        // Truncate if too long
        text.chars().take(width).collect()
    } else {
        // Pad with spaces if too short
        format!("{}{}", text, " ".repeat(width - len))
    }
}

/// Print circuit layout visualization in ASCII
pub fn print_circuit_layout(program: &Program, strategy: Option<Strategy>) {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║          ZKPlex Circuit Layout Visualization               ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    // Build circuit from program
    let circuit = match Circuit::from_program(program) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error building circuit: {}", e);
            process::exit(1);
        }
    };

    // Get estimation
    let estimate = estimate_circuit_requirements_with_strategy(&circuit, strategy);
    let strategy_used = strategy.unwrap_or(Strategy::Auto);

    let circuit_str = program.circuit.join("; ");
    println!("Circuit: {}", circuit_str);
    if !program.preprocess.is_empty() {
        println!("Preprocess: {}", program.preprocess.join("; "));
    }
    println!("Strategy: {} - {}", strategy_used.as_str(), strategy_used.description());
    println!();

    let k = estimate.k;
    let total_rows = estimate.total_rows as u64;
    let used_rows = estimate.estimated_rows as u64;
    let max_bits = circuit.cached_max_bits.unwrap_or(8);

    println!("Parameters:");
    println!("  k = {} (2^{} = {} total rows)", k, k, total_rows);
    println!("  Range check bits: {}", max_bits);
    println!();

    // Calculate layout sections
    let range_table_rows = if estimate.comparison_count > 0 {
        1u64 << max_bits  // 2^max_bits rows for range table
    } else {
        0u64
    };

    let circuit_rows = used_rows - range_table_rows;
    let unused_rows = total_rows - used_rows;

    // ASCII visualization
    println!("Row Layout:");
    println!();

    let bar_width = 60;
    let range_bar_size = if total_rows > 0 { (range_table_rows as f64 / total_rows as f64 * bar_width as f64) as usize } else { 0 };
    let circuit_bar_size = if total_rows > 0 { (circuit_rows as f64 / total_rows as f64 * bar_width as f64) as usize } else { 0 };
    let unused_bar_size = bar_width - range_bar_size - circuit_bar_size;

    // Range check table section
    if range_table_rows > 0 {
        println!("┌{}┐", "─".repeat(bar_width));
        println!("│{:^width$}│", "RANGE CHECK TABLE", width = bar_width);
        println!("│{:^width$}│", format!("{}-bit lookup (rows 0-{})", max_bits, range_table_rows - 1), width = bar_width);
        println!("│{:^width$}│", format!("{} rows ({:.1}%)", range_table_rows, range_table_rows as f64 / total_rows as f64 * 100.0), width = bar_width);
        println!("├{}┤", "─".repeat(bar_width));
    } else {
        println!("┌{}┐", "─".repeat(bar_width));
    }

    // Circuit gates section
    if circuit_rows > 0 {
        println!("│{:^width$}│", "CIRCUIT GATES", width = bar_width);
        println!("│{:^width$}│", format!("rows {}-{}", range_table_rows, range_table_rows + circuit_rows - 1), width = bar_width);
        println!("│{:^width$}│", format!("{} rows ({:.1}%)", circuit_rows, circuit_rows as f64 / total_rows as f64 * 100.0), width = bar_width);

        // Show operation breakdown
        if estimate.operation_count > 0 {
            println!("│{:^width$}│", format!("Arithmetic: {}", estimate.operation_count), width = bar_width);
        }
        if estimate.comparison_count > 0 {
            println!("│{:^width$}│", format!("Comparisons: {}", estimate.comparison_count), width = bar_width);
        }
        if estimate.preprocess_count > 0 {
            println!("│{:^width$}│", format!("Preprocessing: {}", estimate.preprocess_count), width = bar_width);
        }

        if unused_rows > 0 {
            println!("├{}┤", "─".repeat(bar_width));
        }
    }

    // Unused section
    if unused_rows > 0 {
        println!("│{:^width$}│", "UNUSED (padding)", width = bar_width);
        println!("│{:^width$}│", format!("rows {}-{}", used_rows, total_rows - 1), width = bar_width);
        println!("│{:^width$}│", format!("{} rows ({:.1}%)", unused_rows, unused_rows as f64 / total_rows as f64 * 100.0), width = bar_width);
    }

    println!("└{}┘", "─".repeat(bar_width));
    println!();

    // Summary bar chart
    println!("Utilization:");
    print!("[");
    if range_bar_size > 0 {
        print!("{}", "█".repeat(range_bar_size));
    }
    if circuit_bar_size > 0 {
        print!("{}", "▓".repeat(circuit_bar_size));
    }
    if unused_bar_size > 0 {
        print!("{}", "░".repeat(unused_bar_size));
    }
    println!("]");
    println!(" █ Range Table  ▓ Circuit Gates  ░ Unused");
    println!();
    println!("Total utilization: {:.1}% ({}/{} rows)",
        used_rows as f64 / total_rows as f64 * 100.0, used_rows, total_rows);

    // Resource Requirements section
    println!();
    println!("┌{}┐", "─".repeat(60));
    println!("│{:^60}│", "RESOURCE REQUIREMENTS");
    println!("├{}┤", "─".repeat(60));
    println!("│ {} │", format_table_line(&format!("Proving Key (params): {} KB", estimate.params_size_bytes / 1024), 58));
    println!("│ {} │", format_table_line(&format!("Proof size: {:.1} KB", estimate.proof_size_bytes as f64 / 1024.0), 58));
    println!("│ {} │", format_table_line(&format!("Verification Key: {:.1} KB", estimate.vk_size_bytes as f64 / 1024.0), 58));
    println!("└{}┘", "─".repeat(60));

    // Signal Information section
    println!();
    println!("┌{}┐", "─".repeat(60));
    println!("│{:^60}│", "SIGNAL INFORMATION");
    println!("├{}┤", "─".repeat(60));

    let secret_count = program.secret.len();
    let public_count = program.public.len();
    let total_signals = secret_count + public_count;

    println!("│ {} │", format_table_line(&format!("Total signals: {}", total_signals), 58));
    println!("│ {} │", format_table_line(&format!("  Secret signals: {}", secret_count), 58));
    if secret_count > 0 {
        for (name, _) in program.secret.iter().take(5) {
            println!("│ {} │", format_table_line(&format!("    - {}", name), 58));
        }
        if secret_count > 5 {
            println!("│ {} │", format_table_line(&format!("    ... and {} more", secret_count - 5), 58));
        }
    }
    println!("│ {} │", format_table_line(&format!("  Public signals: {}", public_count), 58));
    if public_count > 0 {
        for (name, _) in program.public.iter().take(5) {
            println!("│ {} │", format_table_line(&format!("    - {}", name), 58));
        }
        if public_count > 5 {
            println!("│ {} │", format_table_line(&format!("    ... and {} more", public_count - 5), 58));
        }
    }
    println!("└{}┘", "─".repeat(60));

    // Operation Breakdown section
    println!();
    println!("┌{}┐", "─".repeat(60));
    println!("│{:^60}│", "OPERATION BREAKDOWN");
    println!("├{}┤", "─".repeat(60));

    let total_ops = estimate.operation_count + estimate.comparison_count + estimate.preprocess_count;
    println!("│ {} │", format_table_line(&format!("Total operations: {}", total_ops), 58));

    if estimate.operation_count > 0 {
        let op_pct = (estimate.operation_count as f64 / total_ops as f64) * 100.0;
        println!("│ {} │", format_table_line(&format!("  Arithmetic: {} ({:.1}%)", estimate.operation_count, op_pct), 58));
    }
    if estimate.comparison_count > 0 {
        let cmp_pct = (estimate.comparison_count as f64 / total_ops as f64) * 100.0;
        println!("│ {} │", format_table_line(&format!("  Comparisons: {} ({:.1}%)", estimate.comparison_count, cmp_pct), 58));
    }
    if estimate.preprocess_count > 0 {
        let prep_pct = (estimate.preprocess_count as f64 / total_ops as f64) * 100.0;
        println!("│ {} │", format_table_line(&format!("  Preprocessing: {} ({:.1}%)", estimate.preprocess_count, prep_pct), 58));
    }
    println!("└{}┘", "─".repeat(60));

    // Column Configuration section
    println!();
    println!("┌{}┐", "─".repeat(60));
    println!("│{:^60}│", "COLUMN CONFIGURATION");
    println!("├{}┤", "─".repeat(60));

    // Calculate column usage based on strategy
    let (advice_cols, fixed_cols, selector_cols, lookup_tables) = match strategy_used {
        Strategy::Boolean => (3, 0, 2, 0), // 3 advice [a,b,out] + 2 selectors (add,mul)
        Strategy::BitD => (3, 0, 2, 0),    // Same as Boolean (no lookup tables)
        Strategy::Lookup => {
            // Lookup uses more fixed columns for tables
            let tables = if max_bits <= 8 { 1 } else if max_bits <= 16 { 2 } else { 3 };
            (3, tables, 2, tables)
        }
        Strategy::Auto => {
            // Auto chooses based on max_bits
            if estimate.comparison_count > 0 {
                if max_bits <= 16 { (3, 2, 2, 2) } else { (3, 0, 2, 0) }
            } else {
                (3, 0, 2, 2)
            }
        }
    };

    let instance_cols = 1; // Always 1 for public inputs/outputs
    let total_cols = advice_cols + fixed_cols + selector_cols + instance_cols;

    println!("│ {} │", format_table_line(&format!("Total columns: {}", total_cols), 58));
    println!("│ {} │", format_table_line(&format!("  Advice columns: {} (intermediate values)", advice_cols), 58));
    println!("│ {} │", format_table_line(&format!("  Instance columns: {} (public I/O)", instance_cols), 58));
    println!("│ {} │", format_table_line(&format!("  Selector columns: {} (gate activation)", selector_cols), 58));
    if fixed_cols > 0 {
        println!("│ {} │", format_table_line(&format!("  Fixed columns: {} (lookup tables)", fixed_cols), 58));
    }
    println!("└{}┘", "─".repeat(60));

    // Gate Type Breakdown section
    println!();
    println!("┌{}┐", "─".repeat(60));
    println!("│{:^60}│", "GATE TYPE BREAKDOWN");
    println!("├{}┤", "─".repeat(60));

    println!("│ {} │", format_table_line("Arithmetic Gates:", 58));
    if estimate.operation_count > 0 {
        println!("│ {} │", format_table_line(&format!("  Addition/Subtraction: ~{}", estimate.operation_count / 2), 58));
        println!("│ {} │", format_table_line(&format!("  Multiplication/Division: ~{}", estimate.operation_count / 2), 58));
    } else {
        println!("│ {} │", format_table_line("  None (constant circuit)", 58));
    }

    if estimate.comparison_count > 0 {
        println!("│ {} │", format_table_line("", 58));
        println!("│ {} │", format_table_line("Comparison Gates:", 58));
        println!("│ {} │", format_table_line(&format!("  Ordering (>, <, >=, <=): {}", estimate.comparison_count), 58));
        println!("│ {} │", format_table_line(&format!("    Uses {} range checks", if max_bits == 0 { "no" } else { "costly" }), 58));
    }

    if estimate.preprocess_count > 0 {
        println!("│ {} │", format_table_line("", 58));
        println!("│ {} │", format_table_line("Preprocessing Gates:", 58));
        println!("│ {} │", format_table_line(&format!("  Hash operations: {}", estimate.preprocess_count), 58));
    }
    println!("└{}┘", "─".repeat(60));

    // Lookup Table Information section (only if using lookup strategy)
    if lookup_tables > 0 {
        println!();
        println!("┌{}┐", "─".repeat(60));
        println!("│{:^60}│", "LOOKUP TABLE INFORMATION");
        println!("├{}┤", "─".repeat(60));

        if max_bits <= 8 {
            println!("│ {} │", format_table_line("8-bit table: 256 rows", 58));
            println!("│ {} │", format_table_line(&format!("Table overhead: {:.1}% of circuit", 256.0 / total_rows as f64 * 100.0), 58));
        } else if max_bits <= 16 {
            println!("│ {} │", format_table_line("16-bit table: 65,536 rows", 58));
            println!("│ {} │", format_table_line(&format!("Table overhead: {:.1}% of circuit", 65536.0 / total_rows as f64 * 100.0), 58));
        } else {
            println!("│ {} │", format_table_line("Mixed: 8+16-bit tables with bit decomposition", 58));
            println!("│ {} │", format_table_line(&format!("Table overhead: {:.1}% of circuit", 65792.0 / total_rows as f64 * 100.0), 58));
        }

        println!("│ {} │", format_table_line("", 58));
        println!("│ {} │", format_table_line(&format!("Total lookups: {}", estimate.comparison_count), 58));
        println!("│ {} │", format_table_line("Benefit: Fast proving with pre-computed tables", 58));
        println!("└{}┘", "─".repeat(60));
    }

    // Memory Usage Estimation section
    println!();
    println!("┌{}┐", "─".repeat(60));
    println!("│{:^60}│", "MEMORY USAGE ESTIMATE");
    println!("├{}┤", "─".repeat(60));

    // Calculate memory per component
    let point_size = 32; // bytes per curve point
    let params_memory = (total_rows * point_size) as f64 / (1024.0 * 1024.0);
    let proof_memory = estimate.proof_size_bytes as f64 / 1024.0;
    let vk_memory = estimate.vk_size_bytes as f64 / 1024.0;

    // Estimate witness memory (all signals + intermediate values)
    let signal_count = program.secret.len() + program.public.len();
    let witness_rows = used_rows as f64;
    let witness_memory = (witness_rows * advice_cols as f64 * point_size as f64) / (1024.0 * 1024.0);

    println!("│ {} │", format_table_line("Prover Memory Requirements:", 58));
    println!("│ {} │", format_table_line(&format!("  Proving params: {:.1} MB", params_memory), 58));
    println!("│ {} │", format_table_line(&format!("  Witness data: {:.2} MB ({} signals)", witness_memory, signal_count), 58));
    println!("│ {} │", format_table_line(&format!("  Working memory: ~{:.1} MB", params_memory * 1.5), 58));
    println!("│ {} │", format_table_line(&format!("  Total (peak): ~{:.1} MB", params_memory * 2.5 + witness_memory), 58));
    println!("│ {} │", format_table_line("", 58));
    println!("│ {} │", format_table_line("Verifier Memory Requirements:", 58));
    println!("│ {} │", format_table_line(&format!("  Verification key: {:.1} KB", vk_memory), 58));
    println!("│ {} │", format_table_line(&format!("  Proof data: {:.1} KB", proof_memory), 58));
    println!("│ {} │", format_table_line(&format!("  Working memory: ~{:.1} KB", vk_memory + proof_memory * 2.0), 58));
    println!("│ {} │", format_table_line(&format!("  Total (peak): ~{:.1} KB", (vk_memory + proof_memory) * 3.0), 58));
    println!("└{}┘", "─".repeat(60));

    // Complexity summary with more details
    println!();
    println!("┌{}┐", "─".repeat(60));
    println!("│{:^60}│", "COMPLEXITY ANALYSIS");
    println!("├{}┤", "─".repeat(60));
    println!("│ {} │", format_table_line(&format!("Overall: {}", estimate.complexity), 58));
    println!("│ {} │", format_table_line("", 58));

    // Prover cost analysis
    let prover_cost = if k < 10 {
        "Very Fast (< 1 second)"
    } else if k < 14 {
        "Fast (1-5 seconds)"
    } else if k < 18 {
        "Medium (5-30 seconds)"
    } else {
        "Slow (30+ seconds)"
    };

    println!("│ {} │", format_table_line(&format!("Prover time (estimated): {}", prover_cost), 58));
    println!("│ {} │", format_table_line("Verifier time: Very Fast (< 50ms)", 58));
    println!("│ {} │", format_table_line("", 58));
    println!("│ {} │", format_table_line("Optimization suggestions:", 58));

    if unused_rows as f64 / total_rows as f64 > 0.5 {
        println!("│ {} │", format_table_line("  • Circuit is over-provisioned (much unused space)", 58));
        println!("│ {} │", format_table_line("    Consider using a different approach", 58));
    }

    if max_bits > 16 && matches!(strategy_used, Strategy::Lookup) {
        println!("│ {} │", format_table_line("  • Large lookup tables detected", 58));
        println!("│ {} │", format_table_line("    Try --proof-strategy bitd for smaller proofs", 58));
    }

    if estimate.comparison_count > 10 {
        println!("│ {} │", format_table_line("  • Many comparisons detected", 58));
        println!("│ {} │", format_table_line("    Consider reducing circuit complexity", 58));
    }

    println!("└{}┘", "─".repeat(60));
}

/// Build circuit layout data structure (for JSON API and ASCII visualization)
pub fn build_circuit_layout(program: &Program, strategy: Option<Strategy>) -> Result<CircuitLayout, String> {
    // Build circuit from program
    let circuit = Circuit::from_program(program)
        .map_err(|e| format!("Error building circuit: {}", e))?;

    // Get estimation
    let estimate = estimate_circuit_requirements_with_strategy(&circuit, strategy);
    let strategy_used = strategy.unwrap_or(Strategy::Auto);

    let k = estimate.k;
    let total_rows = estimate.total_rows as u64;
    let used_rows = estimate.estimated_rows as u64;
    let max_bits = circuit.cached_max_bits.unwrap_or(8);

    // Calculate layout sections
    let range_table_rows = if estimate.comparison_count > 0 {
        1u64 << max_bits
    } else {
        0u64
    };

    let circuit_rows = used_rows - range_table_rows;
    let unused_rows = total_rows - used_rows;

    // Row layout
    let row_layout = RowLayout {
        range_table_rows,
        range_table_percent: (range_table_rows as f64 / total_rows as f64) * 100.0,
        circuit_rows,
        circuit_percent: (circuit_rows as f64 / total_rows as f64) * 100.0,
        unused_rows,
        unused_percent: (unused_rows as f64 / total_rows as f64) * 100.0,
        used_rows,
        utilization_percent: (used_rows as f64 / total_rows as f64) * 100.0,
    };

    // Resource requirements
    let resources = ResourceRequirements {
        params_size_bytes: estimate.params_size_bytes as usize,
        params_size_kb: (estimate.params_size_bytes / 1024) as usize,
        proof_size_bytes: estimate.proof_size_bytes as usize,
        proof_size_kb: estimate.proof_size_bytes as f64 / 1024.0,
        vk_size_bytes: estimate.vk_size_bytes as usize,
        vk_size_kb: estimate.vk_size_bytes as f64 / 1024.0,
    };

    // Signal information
    let secret_count = program.secret.len();
    let public_count = program.public.len();
    let secret_names: Vec<String> = program.secret.keys().take(5).cloned().collect();
    let secret_more = if secret_count > 5 { secret_count - 5 } else { 0 };
    let public_names: Vec<String> = program.public.keys().take(5).cloned().collect();
    let public_more = if public_count > 5 { public_count - 5 } else { 0 };

    let signals = SignalInformation {
        total: secret_count + public_count,
        secret_count,
        secret_names,
        secret_more,
        public_count,
        public_names,
        public_more,
    };

    // Operation breakdown
    let total_ops = estimate.operation_count + estimate.comparison_count + estimate.preprocess_count;
    let operations = OperationBreakdown {
        total: total_ops as usize,
        arithmetic_count: estimate.operation_count as usize,
        arithmetic_percent: if total_ops > 0 {
            (estimate.operation_count as f64 / total_ops as f64) * 100.0
        } else {
            0.0
        },
        comparison_count: estimate.comparison_count as usize,
        comparison_percent: if total_ops > 0 {
            (estimate.comparison_count as f64 / total_ops as f64) * 100.0
        } else {
            0.0
        },
        preprocess_count: estimate.preprocess_count as usize,
        preprocess_percent: if total_ops > 0 {
            (estimate.preprocess_count as f64 / total_ops as f64) * 100.0
        } else {
            0.0
        },
    };

    // Column configuration
    let (advice_cols, fixed_cols, selector_cols, lookup_tables) = match strategy_used {
        Strategy::Boolean => (3, 0, 2, 0),
        Strategy::BitD => (3, 0, 2, 0),
        Strategy::Lookup => {
            let tables = if max_bits <= 8 { 1 } else if max_bits <= 16 { 2 } else { 3 };
            (3, tables, 2, tables)
        }
        Strategy::Auto => {
            if estimate.comparison_count > 0 {
                if max_bits <= 16 { (3, 2, 2, 2) } else { (3, 0, 2, 0) }
            } else {
                (3, 0, 2, 2)
            }
        }
    };

    let instance_cols = 1;
    let columns = ColumnConfiguration {
        total: advice_cols + fixed_cols + selector_cols + instance_cols,
        advice: advice_cols,
        instance: instance_cols,
        selector: selector_cols,
        fixed: fixed_cols,
    };

    // Gate breakdown
    let arithmetic = ArithmeticGates {
        addition_subtraction: (estimate.operation_count / 2) as usize,
        multiplication_division: (estimate.operation_count / 2) as usize,
    };

    let comparison = if estimate.comparison_count > 0 {
        Some(ComparisonGates {
            ordering_count: estimate.comparison_count as usize,
            uses_range_checks: max_bits > 0,
        })
    } else {
        None
    };

    let preprocessing = if estimate.preprocess_count > 0 {
        Some(PreprocessingGates {
            hash_operations: estimate.preprocess_count as usize,
        })
    } else {
        None
    };

    let gates = GateBreakdown {
        arithmetic,
        comparison,
        preprocessing,
    };

    // Lookup table information
    let lookup_tables_info = if lookup_tables > 0 {
        let (bit_size, table_rows, overhead_percent) = if max_bits <= 8 {
            ("8-bit".to_string(), 256u64, (256.0 / total_rows as f64) * 100.0)
        } else if max_bits <= 16 {
            ("16-bit".to_string(), 65536u64, (65536.0 / total_rows as f64) * 100.0)
        } else {
            ("Mixed (8+16-bit)".to_string(), 65792u64, (65792.0 / total_rows as f64) * 100.0)
        };

        Some(LookupTableInfo {
            bit_size,
            table_rows,
            overhead_percent,
            total_lookups: estimate.comparison_count as usize,
        })
    } else {
        None
    };

    // Memory usage
    let point_size = 32;
    let params_memory = (total_rows * point_size) as f64 / (1024.0 * 1024.0);
    let proof_memory = estimate.proof_size_bytes as f64 / 1024.0;
    let vk_memory = estimate.vk_size_bytes as f64 / 1024.0;
    let signal_count = program.secret.len() + program.public.len();
    let witness_rows = used_rows as f64;
    let witness_memory = (witness_rows * advice_cols as f64 * point_size as f64) / (1024.0 * 1024.0);

    let memory = MemoryUsage {
        prover: ProverMemory {
            params_mb: params_memory,
            witness_mb: witness_memory,
            signal_count,
            working_mb: params_memory * 1.5,
            total_mb: params_memory * 2.5 + witness_memory,
        },
        verifier: VerifierMemory {
            vk_kb: vk_memory,
            proof_kb: proof_memory,
            working_kb: vk_memory + proof_memory * 2.0,
            total_kb: (vk_memory + proof_memory) * 3.0,
        },
    };

    // Complexity analysis
    let prover_time = if k < 10 {
        "Very Fast (< 1 second)"
    } else if k < 14 {
        "Fast (1-5 seconds)"
    } else if k < 18 {
        "Medium (5-30 seconds)"
    } else {
        "Slow (30+ seconds)"
    }.to_string();

    let mut optimization_suggestions = Vec::new();
    if unused_rows as f64 / total_rows as f64 > 0.5 {
        optimization_suggestions.push("Circuit is over-provisioned (much unused space). Consider using a different approach.".to_string());
    }
    if max_bits > 16 && matches!(strategy_used, Strategy::Lookup) {
        optimization_suggestions.push("Large lookup tables detected. Try bitd strategy for smaller proofs.".to_string());
    }
    if estimate.comparison_count > 10 {
        optimization_suggestions.push("Many comparisons detected. Consider reducing circuit complexity.".to_string());
    }

    let complexity = ComplexityAnalysis {
        overall: estimate.complexity.clone(),
        prover_time,
        verifier_time: "Very Fast (< 50ms)".to_string(),
        optimization_suggestions,
    };

    // Build final layout
    let circuit_str = program.circuit.join("; ");
    let preprocess_str = if program.preprocess.is_empty() {
        None
    } else {
        Some(program.preprocess.join("; "))
    };

    Ok(CircuitLayout {
        circuit: circuit_str,
        preprocess: preprocess_str,
        strategy: strategy_used.as_str().to_string(),
        strategy_description: strategy_used.description().to_string(),
        parameters: CircuitParameters {
            k,
            total_rows,
            max_bits,
        },
        row_layout,
        resources,
        signals,
        operations,
        columns,
        gates,
        lookup_tables: lookup_tables_info,
        memory,
        complexity,
    })
}


/// Render circuit layout as ASCII string (for WASM API)
pub fn render_circuit_layout_ascii(layout: &CircuitLayout) -> String {
    let mut output = String::new();
    
    // Header
    output.push_str("╔════════════════════════════════════════════════════════════╗\n");
    output.push_str("║          ZKPlex Circuit Layout Visualization               ║\n");
    output.push_str("╚════════════════════════════════════════════════════════════╝\n");
    output.push_str("\n");

    // Circuit and strategy info
    output.push_str(&format!("Circuit: {}\n", layout.circuit));
    if let Some(ref preprocess) = layout.preprocess {
        output.push_str(&format!("Preprocess: {}\n", preprocess));
    }
    output.push_str(&format!("Strategy: {} - {}\n", layout.strategy, layout.strategy_description));
    output.push_str("\n");

    // Parameters
    output.push_str("Parameters:\n");
    output.push_str(&format!("  k = {} (2^{} = {} total rows)\n", 
        layout.parameters.k, layout.parameters.k, layout.parameters.total_rows));
    output.push_str(&format!("  Range check bits: {}\n", layout.parameters.max_bits));
    output.push_str("\n");

    // Row Layout diagram
    output.push_str("Row Layout:\n\n");
    
    let bar_width = 60;
    let total_rows = layout.parameters.total_rows as f64;
    let range_bar_size = ((layout.row_layout.range_table_rows as f64 / total_rows) * bar_width as f64) as usize;
    let circuit_bar_size = ((layout.row_layout.circuit_rows as f64 / total_rows) * bar_width as f64) as usize;
    let unused_bar_size = bar_width - range_bar_size - circuit_bar_size;
    
    if layout.row_layout.range_table_rows > 0 {
        output.push_str(&format!("┌{}┐\n", "─".repeat(bar_width)));
        output.push_str(&format!("│{:^width$}│\n", "RANGE CHECK TABLE", width = bar_width));
        output.push_str(&format!("│{:^width$}│\n", 
            format!("{}-bit lookup (rows 0-{})", layout.parameters.max_bits, layout.row_layout.range_table_rows - 1), 
            width = bar_width));
        output.push_str(&format!("│{:^width$}│\n", 
            format!("{} rows ({:.1}%)", layout.row_layout.range_table_rows, layout.row_layout.range_table_percent), 
            width = bar_width));
        output.push_str(&format!("├{}┤\n", "─".repeat(bar_width)));
    } else {
        output.push_str(&format!("┌{}┐\n", "─".repeat(bar_width)));
    }
    
    if layout.row_layout.circuit_rows > 0 {
        output.push_str(&format!("│{:^width$}│\n", "CIRCUIT GATES", width = bar_width));
        output.push_str(&format!("│{:^width$}│\n", 
            format!("rows {}-{}", layout.row_layout.range_table_rows, 
                layout.row_layout.range_table_rows + layout.row_layout.circuit_rows - 1), 
            width = bar_width));
        output.push_str(&format!("│{:^width$}│\n", 
            format!("{} rows ({:.1}%)", layout.row_layout.circuit_rows, layout.row_layout.circuit_percent), 
            width = bar_width));
        
        if layout.row_layout.unused_rows > 0 {
            output.push_str(&format!("├{}┤\n", "─".repeat(bar_width)));
        }
    }
    
    if layout.row_layout.unused_rows > 0 {
        output.push_str(&format!("│{:^width$}│\n", "UNUSED (padding)", width = bar_width));
        output.push_str(&format!("│{:^width$}│\n", 
            format!("rows {}-{}", layout.row_layout.used_rows, layout.parameters.total_rows - 1), 
            width = bar_width));
        output.push_str(&format!("│{:^width$}│\n", 
            format!("{} rows ({:.1}%)", layout.row_layout.unused_rows, layout.row_layout.unused_percent), 
            width = bar_width));
    }
    
    output.push_str(&format!("└{}┘\n", "─".repeat(bar_width)));
    output.push_str("\n");

    // Utilization bar
    output.push_str("Utilization:\n[");
    if range_bar_size > 0 {
        output.push_str(&"█".repeat(range_bar_size));
    }
    if circuit_bar_size > 0 {
        output.push_str(&"▓".repeat(circuit_bar_size));
    }
    if unused_bar_size > 0 {
        output.push_str(&"░".repeat(unused_bar_size));
    }
    output.push_str("]\n");
    output.push_str(" █ Range Table  ▓ Circuit Gates  ░ Unused\n\n");
    output.push_str(&format!("Total utilization: {:.1}% ({}/{} rows)\n",
        layout.row_layout.utilization_percent, layout.row_layout.used_rows, layout.parameters.total_rows));

    // Resources
    output.push_str("\n┌────────────────────────────────────────────────────────────┐\n");
    output.push_str("│                   RESOURCE REQUIREMENTS                    │\n");
    output.push_str("├────────────────────────────────────────────────────────────┤\n");
    output.push_str(&format!("│ {} │\n", format_table_line(&format!("Proving Key (params): {} KB", layout.resources.params_size_kb), 58)));
    output.push_str(&format!("│ {} │\n", format_table_line(&format!("Proof size: {:.1} KB", layout.resources.proof_size_kb), 58)));
    output.push_str(&format!("│ {} │\n", format_table_line(&format!("Verification Key: {:.1} KB", layout.resources.vk_size_kb), 58)));
    output.push_str("└────────────────────────────────────────────────────────────┘\n");

    // Signals
    output.push_str("\n┌────────────────────────────────────────────────────────────┐\n");
    output.push_str("│                     SIGNAL INFORMATION                     │\n");
    output.push_str("├────────────────────────────────────────────────────────────┤\n");
    output.push_str(&format!("│ {} │\n", format_table_line(&format!("Total signals: {}", layout.signals.total), 58)));
    output.push_str(&format!("│ {} │\n", format_table_line(&format!("  Secret signals: {}", layout.signals.secret_count), 58)));
    for name in &layout.signals.secret_names {
        output.push_str(&format!("│ {} │\n", format_table_line(&format!("    - {}", name), 58)));
    }
    if layout.signals.secret_more > 0 {
        output.push_str(&format!("│ {} │\n", format_table_line(&format!("    ... and {} more", layout.signals.secret_more), 58)));
    }
    output.push_str(&format!("│ {} │\n", format_table_line(&format!("  Public signals: {}", layout.signals.public_count), 58)));
    for name in &layout.signals.public_names {
        output.push_str(&format!("│ {} │\n", format_table_line(&format!("    - {}", name), 58)));
    }
    if layout.signals.public_more > 0 {
        output.push_str(&format!("│ {} │\n", format_table_line(&format!("    ... and {} more", layout.signals.public_more), 58)));
    }
    output.push_str("└────────────────────────────────────────────────────────────┘\n");

    // Operations
    output.push_str("\n┌────────────────────────────────────────────────────────────┐\n");
    output.push_str("│                    OPERATION BREAKDOWN                     │\n");
    output.push_str("├────────────────────────────────────────────────────────────┤\n");
    output.push_str(&format!("│ {} │\n", format_table_line(&format!("Total operations: {}", layout.operations.total), 58)));
    if layout.operations.arithmetic_count > 0 {
        output.push_str(&format!("│ {} │\n", format_table_line(&format!("  Arithmetic: {} ({:.1}%)", 
            layout.operations.arithmetic_count, layout.operations.arithmetic_percent), 58)));
    }
    if layout.operations.comparison_count > 0 {
        output.push_str(&format!("│ {} │\n", format_table_line(&format!("  Comparisons: {} ({:.1}%)", 
            layout.operations.comparison_count, layout.operations.comparison_percent), 58)));
    }
    if layout.operations.preprocess_count > 0 {
        output.push_str(&format!("│ {} │\n", format_table_line(&format!("  Preprocessing: {} ({:.1}%)", 
            layout.operations.preprocess_count, layout.operations.preprocess_percent), 58)));
    }
    output.push_str("└────────────────────────────────────────────────────────────┘\n");

    // Columns
    output.push_str("\n┌────────────────────────────────────────────────────────────┐\n");
    output.push_str("│                    COLUMN CONFIGURATION                    │\n");
    output.push_str("├────────────────────────────────────────────────────────────┤\n");
    output.push_str(&format!("│ {} │\n", format_table_line(&format!("Total columns: {}", layout.columns.total), 58)));
    output.push_str(&format!("│ {} │\n", format_table_line(&format!("  Advice columns: {} (intermediate values)", layout.columns.advice), 58)));
    output.push_str(&format!("│ {} │\n", format_table_line(&format!("  Instance columns: {} (public I/O)", layout.columns.instance), 58)));
    output.push_str(&format!("│ {} │\n", format_table_line(&format!("  Selector columns: {} (gate activation)", layout.columns.selector), 58)));
    if layout.columns.fixed > 0 {
        output.push_str(&format!("│ {} │\n", format_table_line(&format!("  Fixed columns: {} (lookup tables)", layout.columns.fixed), 58)));
    }
    output.push_str("└────────────────────────────────────────────────────────────┘\n");

    // Gates
    output.push_str("\n┌────────────────────────────────────────────────────────────┐\n");
    output.push_str("│                    GATE TYPE BREAKDOWN                     │\n");
    output.push_str("├────────────────────────────────────────────────────────────┤\n");
    output.push_str(&format!("│ {} │\n", format_table_line("Arithmetic Gates:", 58)));
    if layout.gates.arithmetic.addition_subtraction > 0 || layout.gates.arithmetic.multiplication_division > 0 {
        output.push_str(&format!("│ {} │\n", format_table_line(&format!("  Addition/Subtraction: ~{}", layout.gates.arithmetic.addition_subtraction), 58)));
        output.push_str(&format!("│ {} │\n", format_table_line(&format!("  Multiplication/Division: ~{}", layout.gates.arithmetic.multiplication_division), 58)));
    } else {
        output.push_str(&format!("│ {} │\n", format_table_line("  None (constant circuit)", 58)));
    }
    if let Some(ref comparison) = layout.gates.comparison {
        output.push_str(&format!("│ {} │\n", format_table_line("", 58)));
        output.push_str(&format!("│ {} │\n", format_table_line("Comparison Gates:", 58)));
        output.push_str(&format!("│ {} │\n", format_table_line(&format!("  Ordering (>, <, >=, <=): {}", comparison.ordering_count), 58)));
        if comparison.uses_range_checks {
            output.push_str(&format!("│ {} │\n", format_table_line("    Uses costly range checks", 58)));
        }
    }
    if let Some(ref preprocessing) = layout.gates.preprocessing {
        output.push_str(&format!("│ {} │\n", format_table_line("", 58)));
        output.push_str(&format!("│ {} │\n", format_table_line("Preprocessing Gates:", 58)));
        output.push_str(&format!("│ {} │\n", format_table_line(&format!("  Hash operations: {}", preprocessing.hash_operations), 58)));
    }
    output.push_str("└────────────────────────────────────────────────────────────┘\n");

    // Lookup tables
    if let Some(ref lookup) = layout.lookup_tables {
        output.push_str("\n┌────────────────────────────────────────────────────────────┐\n");
        output.push_str("│                  LOOKUP TABLE INFORMATION                  │\n");
        output.push_str("├────────────────────────────────────────────────────────────┤\n");
        output.push_str(&format!("│ {} │\n", format_table_line(&format!("{} table: {} rows", lookup.bit_size, lookup.table_rows), 58)));
        output.push_str(&format!("│ {} │\n", format_table_line(&format!("Table overhead: {:.1}% of circuit", lookup.overhead_percent), 58)));
        output.push_str(&format!("│ {} │\n", format_table_line("", 58)));
        output.push_str(&format!("│ {} │\n", format_table_line(&format!("Total lookups: {}", lookup.total_lookups), 58)));
        output.push_str(&format!("│ {} │\n", format_table_line("Benefit: Fast proving with pre-computed tables", 58)));
        output.push_str("└────────────────────────────────────────────────────────────┘\n");
    }

    // Memory
    output.push_str("\n┌────────────────────────────────────────────────────────────┐\n");
    output.push_str("│                   MEMORY USAGE ESTIMATE                    │\n");
    output.push_str("├────────────────────────────────────────────────────────────┤\n");
    output.push_str(&format!("│ {} │\n", format_table_line("Prover Memory Requirements:", 58)));
    output.push_str(&format!("│ {} │\n", format_table_line(&format!("  Proving params: {:.1} MB", layout.memory.prover.params_mb), 58)));
    output.push_str(&format!("│ {} │\n", format_table_line(&format!("  Witness data: {:.2} MB ({} signals)", 
        layout.memory.prover.witness_mb, layout.memory.prover.signal_count), 58)));
    output.push_str(&format!("│ {} │\n", format_table_line(&format!("  Working memory: ~{:.1} MB", layout.memory.prover.working_mb), 58)));
    output.push_str(&format!("│ {} │\n", format_table_line(&format!("  Total (peak): ~{:.1} MB", layout.memory.prover.total_mb), 58)));
    output.push_str(&format!("│ {} │\n", format_table_line("", 58)));
    output.push_str(&format!("│ {} │\n", format_table_line("Verifier Memory Requirements:", 58)));
    output.push_str(&format!("│ {} │\n", format_table_line(&format!("  Verification key: {:.1} KB", layout.memory.verifier.vk_kb), 58)));
    output.push_str(&format!("│ {} │\n", format_table_line(&format!("  Proof data: {:.1} KB", layout.memory.verifier.proof_kb), 58)));
    output.push_str(&format!("│ {} │\n", format_table_line(&format!("  Working memory: ~{:.1} KB", layout.memory.verifier.working_kb), 58)));
    output.push_str(&format!("│ {} │\n", format_table_line(&format!("  Total (peak): ~{:.1} KB", layout.memory.verifier.total_kb), 58)));
    output.push_str("└────────────────────────────────────────────────────────────┘\n");

    // Complexity
    output.push_str("\n┌────────────────────────────────────────────────────────────┐\n");
    output.push_str("│                    COMPLEXITY ANALYSIS                     │\n");
    output.push_str("├────────────────────────────────────────────────────────────┤\n");
    output.push_str(&format!("│ {} │\n", format_table_line(&format!("Overall: {}", layout.complexity.overall), 58)));
    output.push_str(&format!("│ {} │\n", format_table_line("", 58)));
    output.push_str(&format!("│ {} │\n", format_table_line(&format!("Prover time (estimated): {}", layout.complexity.prover_time), 58)));
    output.push_str(&format!("│ {} │\n", format_table_line(&format!("Verifier time: {}", layout.complexity.verifier_time), 58)));
    output.push_str(&format!("│ {} │\n", format_table_line("", 58)));
    output.push_str(&format!("│ {} │\n", format_table_line("Optimization suggestions:", 58)));
    if layout.complexity.optimization_suggestions.is_empty() {
        output.push_str(&format!("│ {} │\n", format_table_line("  None - circuit is well optimized", 58)));
    } else {
        for suggestion in &layout.complexity.optimization_suggestions {
            for line in suggestion.lines() {
                output.push_str(&format!("│ {} │\n", format_table_line(&format!("  • {}", line), 58)));
            }
        }
    }
    output.push_str("└────────────────────────────────────────────────────────────┘\n");

    output
}
