//! CLI tool for converting between ZKPlex program formats
//!
//! This tool converts between Zircon and JSON formats, validates programs,
//! and displays program information.
//!
//! # Examples
//!
//! Convert Zircon to JSON:
//! ```bash
//! zkplex-cli --zircon "1/A:10,B:20/-/A+B" --into-json
//! ```
//!
//! Convert JSON to Zircon:
//! ```bash
//! zkplex-cli --json program.json --into-zircon
//! ```
//!
//! Show program info:
//! ```bash
//! zkplex-cli --zircon "1/A:10/B:20/hash<==sha256(A{%x})/A>B" --info
//! ```

use std::process;
use std::fs;
use std::path::Path;
use indexmap::IndexMap;
use zkplex_core::api::{Program, Signal, ProveResponse, VerifyRequest};
use zkplex_core::api::program::Signal as ProgramSignal;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get build ID from environment variable (set during compilation)
/// If not set, returns None
fn get_build_id() -> Option<&'static str> {
    option_env!("BUILD_ID")
}

use zkplex_core::circuit::{Circuit, estimate_circuit_requirements_with_strategy, validate_strategy_compatibility, Strategy};
use zkplex_core::encoding::ValueEncoding;
use zkplex_core::layout;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    // Parse command line arguments
    let mut zircon_input: Option<String> = None;
    let mut json_input: Option<String> = None;
    let mut circuit_input: Option<String> = None;
    let mut preprocess_inputs: Vec<String> = Vec::new();
    let mut secret_signals: Vec<String> = Vec::new();
    let mut public_signals: Vec<String> = Vec::new();
    let mut proof_file: Option<String> = None;
    let mut into_json = false;
    let mut into_zircon = false;
    let mut show_info = false;
    let mut show_estimate = false;
    let mut show_layout = false;
    let mut do_prove = false;
    let mut do_verify = false;
    let mut proof_strategy: Option<Strategy> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--zircon" | "-z" => {
                if i + 1 < args.len() {
                    zircon_input = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --zircon requires a value");
                    process::exit(1);
                }
            }
            "--json" | "-j" => {
                if i + 1 < args.len() {
                    json_input = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --json requires a value");
                    process::exit(1);
                }
            }
            "--circuit" => {
                if i + 1 < args.len() {
                    circuit_input = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --circuit requires a value");
                    process::exit(1);
                }
            }
            "--preprocess" => {
                if i + 1 < args.len() {
                    preprocess_inputs.push(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --preprocess requires a value");
                    process::exit(1);
                }
            }
            "--secret" | "-s" => {
                if i + 1 < args.len() {
                    secret_signals.push(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --secret requires a value");
                    process::exit(1);
                }
            }
            "--public" | "-p" => {
                if i + 1 < args.len() {
                    public_signals.push(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --public requires a value");
                    process::exit(1);
                }
            }
            "--prove" => {
                do_prove = true;
                i += 1;
            }
            "--verify" => {
                do_verify = true;
                i += 1;
            }
            "--proof" => {
                if i + 1 < args.len() {
                    proof_file = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --proof requires a value");
                    process::exit(1);
                }
            }
            "--into-json" => {
                into_json = true;
                i += 1;
            }
            "--into-zircon" => {
                into_zircon = true;
                i += 1;
            }
            "--info" | "-i" => {
                show_info = true;
                i += 1;
            }
            "--estimate" | "-e" => {
                show_estimate = true;
                i += 1;
            }
            "--layout" | "-l" => {
                show_layout = true;
                i += 1;
            }
            "--proof-strategy" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<Strategy>() {
                        Ok(strategy) => proof_strategy = Some(strategy),
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            process::exit(1);
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("Error: --proof-strategy requires a value");
                    process::exit(1);
                }
            }
            "--help" | "-h" => {
                print_usage();
                process::exit(0);
            }
            "--version" | "-v" => {
                match get_build_id() {
                    Some(build_id) if !build_id.is_empty() => {
                        println!("zkplex-cli {} ({})", VERSION, build_id);
                    }
                    _ => {
                        println!("zkplex-cli {}", VERSION);
                    }
                }
                process::exit(0);
            }
            _ => {
                eprintln!("Error: Unknown option '{}'", args[i]);
                print_usage();
                process::exit(1);
            }
        }
    }

    // Handle prove command
    if do_prove {
        // Support --circuit, --zircon, or --json for proof generation
        if circuit_input.is_none() && zircon_input.is_none() && json_input.is_none() {
            eprintln!("Error: --circuit, --zircon, or --json is required for proof generation");
            process::exit(1);
        }

        // Create Program from input format
        let program = if let Some(circuit) = circuit_input {
            // Direct circuit mode - convert to Program
            let signals_map = parse_signals_from_cli(&secret_signals, &public_signals);

            // Convert signals to Program format
            let mut secret_sigs = IndexMap::new();
            let mut public_sigs = IndexMap::new();

            for (name, sig) in &signals_map {
                let prog_sig = ProgramSignal {
                    value: sig.value.clone(),
                    encoding: sig.encoding,
                };
                if sig.public {
                    public_sigs.insert(name.clone(), prog_sig);
                } else {
                    secret_sigs.insert(name.clone(), prog_sig);
                }
            }

            // Parse circuit and preprocess statements (split on semicolons)
            let circuit_statements = match Program::parse_statements(&circuit) {
                Ok(statements) => statements,
                Err(e) => {
                    eprintln!("Error parsing circuit statements: {}", e);
                    process::exit(1);
                }
            };

            // Join multiple --preprocess arguments
            let preprocess_combined = preprocess_inputs.join(";");
            let preprocess_statements = if !preprocess_combined.is_empty() {
                match Program::parse_statements(&preprocess_combined) {
                    Ok(statements) => statements,
                    Err(e) => {
                        eprintln!("Error parsing preprocess statements: {}", e);
                        process::exit(1);
                    }
                }
            } else {
                Vec::new()
            };

            Program {
                version: zkplex_core::api::PROOF_VERSION,
                secret: secret_sigs,
                public: public_sigs,
                preprocess: preprocess_statements,
                circuit: circuit_statements,
            }
        } else {
            // File format mode (zircon or json)
            let (input, format) = if let Some(zircon) = zircon_input.as_ref() {
                (zircon, "zircon")
            } else if let Some(json) = json_input.as_ref() {
                (json, "json")
            } else {
                unreachable!()
            };

            load_program_from_format(input, format, &secret_signals, &public_signals)
        };

        generate_proof(&program, proof_file.as_deref(), proof_strategy);
        return;
    }

    // Handle verify command
    if do_verify {
        if proof_file.is_none() {
            eprintln!("Error: --proof is required for verification");
            process::exit(1);
        }

        verify_proof(&proof_file.unwrap(), into_json);
        return;
    }

    // Handle estimate command
    if show_estimate {
        // Support --circuit, --zircon, or --json for estimation
        if circuit_input.is_none() && zircon_input.is_none() && json_input.is_none() {
            eprintln!("Error: --circuit, --zircon, or --json is required for estimation");
            process::exit(1);
        }

        // Create Program from input format (same logic as prove)
        let program = if let Some(circuit) = circuit_input {
            // Direct circuit mode - convert to Program
            let signals_map = parse_signals_from_cli(&secret_signals, &public_signals);

            // Convert signals to Program format
            let mut secret_sigs = IndexMap::new();
            let mut public_sigs = IndexMap::new();

            for (name, sig) in &signals_map {
                let prog_sig = ProgramSignal {
                    value: sig.value.clone(),
                    encoding: sig.encoding,
                };
                if sig.public {
                    public_sigs.insert(name.clone(), prog_sig);
                } else {
                    secret_sigs.insert(name.clone(), prog_sig);
                }
            }

            // Parse circuit and preprocess statements (split on semicolons)
            let circuit_statements = match Program::parse_statements(&circuit) {
                Ok(statements) => statements,
                Err(e) => {
                    eprintln!("Error parsing circuit statements: {}", e);
                    process::exit(1);
                }
            };

            // Join multiple --preprocess arguments
            let preprocess_combined = preprocess_inputs.join(";");
            let preprocess_statements = if !preprocess_combined.is_empty() {
                match Program::parse_statements(&preprocess_combined) {
                    Ok(statements) => statements,
                    Err(e) => {
                        eprintln!("Error parsing preprocess statements: {}", e);
                        process::exit(1);
                    }
                }
            } else {
                Vec::new()
            };

            Program {
                version: zkplex_core::api::PROOF_VERSION,
                secret: secret_sigs,
                public: public_sigs,
                preprocess: preprocess_statements,
                circuit: circuit_statements,
            }
        } else {
            // File format mode (zircon or json)
            let (input, format) = if let Some(zircon) = zircon_input.as_ref() {
                (zircon, "zircon")
            } else if let Some(json) = json_input.as_ref() {
                (json, "json")
            } else {
                unreachable!()
            };

            load_program_from_format(input, format, &secret_signals, &public_signals)
        };

        // Build circuit for estimation using from_program
        let circuit_obj = match Circuit::from_program(&program) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error building circuit: {}", e);
                process::exit(1);
            }
        };

        // Validate strategy compatibility with circuit operations
        if let Some(strategy_val) = proof_strategy {
            if let Err(e) = validate_strategy_compatibility(&circuit_obj, strategy_val) {
                eprintln!("Error: {}", e);
                eprintln!("Your circuit uses operations that are not compatible with the '{}' strategy.", strategy_val);
                eprintln!();
                eprintln!("Example:");
                eprintln!("  zkplex-cli --circuit \"<your circuit>\" --proof-strategy auto --estimate");
                process::exit(1);
            }
        }

        // Get estimation (use auto strategy for --estimate without --proof-strategy)
        let estimate = estimate_circuit_requirements_with_strategy(&circuit_obj, proof_strategy);

        if into_json {
            // Output in JSON format
            let json_output = serde_json::json!({
                "complexity": estimate.complexity,
                "k": estimate.k,
                "total_rows": estimate.total_rows,
                "estimated_rows": estimate.estimated_rows,
                "operation_count": estimate.operation_count,
                "comparison_count": estimate.comparison_count,
                "preprocess_count": estimate.preprocess_count,
                "params_size_bytes": estimate.params_size_bytes,
                "proof_size_bytes": estimate.proof_size_bytes,
                "vk_size_bytes": estimate.vk_size_bytes
            });
            println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
        } else {
            // Output in text format
            let circuit_str = program.circuit.join("; ");
            println!("ZKPlex Circuit Estimation");
            println!("=========================");
            println!();
            println!("Circuit: {}", circuit_str);
            if !program.preprocess.is_empty() {
                println!("Preprocess: {}", program.preprocess.join("; "));
            }
            println!();
            println!("Complexity: {}", estimate.complexity);
            println!();
            println!("Circuit Parameters:");
            println!("  Required k:        {}", estimate.k);
            println!("  Total rows (2^k):  {}", estimate.total_rows);
            println!("  Estimated rows:    {}", estimate.estimated_rows);
            println!("  Row utilization:   {:.1}%",
                (estimate.estimated_rows as f64 / estimate.total_rows as f64) * 100.0);
            println!();
            println!("Operations:");
            println!("  Arithmetic ops:    {}", estimate.operation_count);
            println!("  Comparisons:       {}", estimate.comparison_count);
            println!("  Preprocessing:     {}", estimate.preprocess_count);
            println!();
            println!("Resource Requirements (Hardware-Independent):");
            println!("  Params size:       {} bytes ({} KB)",
                estimate.params_size_bytes,
                estimate.params_size_bytes / 1024);
            println!("  Proof size:        {} bytes ({:.1} KB)",
                estimate.proof_size_bytes,
                estimate.proof_size_bytes as f64 / 1024.0);
            println!("  VK size:           {} bytes ({:.1} KB)",
                estimate.vk_size_bytes,
                estimate.vk_size_bytes as f64 / 1024.0);
            println!();
            println!("Note: These estimates are hardware-independent and show the");
            println!("      minimum requirements for proof generation and verification.");
        }
        return;
    }

    // Parse input program from provided format (file or string)
    let program = if let Some(zircon) = zircon_input {
        let content = read_input_or_file(&zircon);
        match Program::from_zircon(&content) {
            Ok(p) => Some(p),
            Err(e) => {
                eprintln!("Error parsing zircon format: {}", e);
                process::exit(1);
            }
        }
    } else if let Some(json) = json_input {
        let content = read_input_or_file(&json);
        match Program::from_json(&content) {
            Ok(p) => Some(p),
            Err(e) => {
                eprintln!("Error parsing JSON: {}", e);
                process::exit(1);
            }
        }
    } else {
        None
    };

    // Handle conversion commands
    if let Some(prog) = program {
        if into_json {
            match prog.to_json() {
                Ok(json) => println!("{}", json),
                Err(e) => {
                    eprintln!("Error serializing to JSON: {}", e);
                    process::exit(1);
                }
            }
        } else if into_zircon {
            println!("{}", prog.to_zircon());
        } else if show_info {
            print_program_info(&prog);
        } else if show_estimate {
            print_estimate(&prog);
        } else if show_layout {
            layout::print_circuit_layout(&prog, proof_strategy);
        } else {
            // No conversion requested, just validate
            println!("✓ Valid program format");
        }
    } else {
        eprintln!("Error: No input provided");
        print_usage();
        process::exit(1);
    }
}

/// Read input from file or return the string itself
///
/// If the input looks like a file path and the file exists, read its contents.
/// Otherwise, return the input string as-is.
fn read_input_or_file(input: &str) -> String {
    let path = Path::new(input);

    // If path exists as a file, read it
    if path.exists() && path.is_file() {
        match fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading file '{}': {}", input, e);
                process::exit(1);
            }
        }
    } else {
        // Not a file, use input as-is
        input.to_string()
    }
}

fn print_usage() {
    println!("zkplex-cli - ZKPlex Program Format Converter and Prover");
    println!();
    println!("USAGE:");
    println!("    zkplex-cli [OPTIONS]");
    println!();
    println!("FORMAT CONVERSION OPTIONS:");
    println!("    -z, --zircon <TEXT|FILE> Input in Zircon format (text or file path)");
    println!("    -j, --json <TEXT|FILE>   Input in JSON format (text or file path)");
    println!("    --into-json             Convert to JSON format");
    println!("    --into-zircon           Convert to Zircon format");
    println!("    -i, --info              Show program information");
    println!("    -e, --estimate          Estimate circuit requirements");
    println!("    -l, --layout            Show circuit layout visualization (ASCII)");
    println!();
    println!("PROOF GENERATION/VERIFICATION OPTIONS:");
    println!("    --circuit <TEXT>              Circuit expression (e.g., \"A + B > 100\")");
    println!("    --preprocess <TEXT>           Preprocessing expression (e.g., \"hash<==sha256(secret)\")");
    println!("                                  Can be used multiple times for multiple preprocessing steps");
    println!("    -s, --secret <name:value[:enc]>   Secret signal (can be used multiple times)");
    println!("    -p, --public <name:value[:enc]>   Public signal (can be used multiple times)");
    println!("                                  At least one public signal is REQUIRED for proofs");
    println!("                                  Use '?' as value for output signal (computed from circuit)");
    println!("                                  Encodings: base58/b58, base64/b64, base85/b85, hex, decimal");
    println!("    --prove                       Generate a proof");
    println!("    --verify                      Verify a proof");
    println!("    --proof <FILE>                Proof file (for output or input)");
    println!("    --proof-strategy <STRATEGY>   Circuit strategy (auto|boolean|lookup|bitd)");
    println!("                                  auto:    {} - Ops: {}", Strategy::Auto.description(), Strategy::Auto.operations());
    println!("                                  boolean: {} - Ops: {}", Strategy::Boolean.description(), Strategy::Boolean.operations());
    println!("                                  lookup:  {} - Ops: {}", Strategy::Lookup.description(), Strategy::Lookup.operations());
    println!("                                  bitd:    {} - Ops: {}", Strategy::BitD.description(), Strategy::BitD.operations());
    println!();
    println!("ENCODING FORMATS:");
    println!("    decimal  - Decimal numbers (e.g., \"12345\")");
    println!("    hex      - Hexadecimal with or without 0x prefix (e.g., \"0x1a2b\" or \"1a2b\")");
    println!("    base58   - Base58 encoding (Bitcoin/Solana addresses)");
    println!("    base64   - Base64 encoding (standard)");
    println!("    base85   - ASCII85 encoding (Adobe standard, compatible with online decoders)");
    println!();
    println!("GENERAL OPTIONS:");
    println!("    -h, --help                    Print help information");
    println!("    -v, --version                 Print version information");
    println!();
    println!("EXAMPLES:");
    println!();
    println!("  1. Age Verification (full workflow):");
    println!("    # Step 1: Estimate circuit requirements");
    println!("    zkplex-cli --circuit \"age >= 18\" --secret age:25 --estimate");
    println!();
    println!("    # Step 2: Generate proof (age is secret, result is public)");
    println!("    zkplex-cli --circuit \"age >= 18\" \\");
    println!("               --secret age:25 \\");
    println!("               --prove > proof.json");
    println!();
    println!("    # Step 3: Verify proof (without knowing the secret age)");
    println!("    zkplex-cli --verify --proof proof.json");
    println!();
    println!("  2. Threshold Check with Output Signal:");
    println!("    # Prove (A + B) * C > threshold and output the computed result");
    println!("    zkplex-cli --circuit \"(A + B) * C > threshold\" \\");
    println!("               --secret A:10 --secret B:20 --secret C:2 \\");
    println!("               --public threshold:50 \\");
    println!("               --public result:? \\");
    println!("               --prove");
    println!("    # Output: result = 60 (computed automatically, secrets remain hidden)");
    println!();
    println!("  3. Zircon Format (compact blockchain format):");
    println!("    # Estimate from Zircon file");
    println!("    zkplex-cli --zircon \"1/age:25/-/age>=18\" --estimate");
    println!();
    println!("    # Generate proof from Zircon file");
    println!("    zkplex-cli --zircon proof.zrc --prove");
    println!();
    println!("    # With output signal in Zircon format");
    println!("    zkplex-cli --zircon \"1/A:10,B:20/result:?/-A+B\" --prove");
    println!();
    println!("  4. Base58 Encoding (Solana/Bitcoin addresses):");
    println!("    zkplex-cli --circuit \"myAddress == targetAddress\" \\");
    println!("               --secret myAddress:9aE476sH92Vc7DMC8bZNpe1xNNNy1fNjFpCGvfMuZMwM:base58 \\");
    println!("               --public targetAddress:9aE476sH92Vc7DMC8bZNpe1xNNNy1fNjFpCGvfMuZMwM:base58 \\");
    println!("               --prove");
    println!();
    println!("  5. Hash Preprocessing:");
    println!("    zkplex-cli --circuit \"sha256(secret) == target\" \\");
    println!("               --secret secret:hello \\");
    println!("               --public target:0x2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824:hex \\");
    println!("               --prove");
    println!();
    println!("  6. Format Conversion:");
    println!("    # Convert Zircon to JSON");
    println!("    zkplex-cli --zircon \"1/A:10,B:20/-/A+B\" --into-json");
    println!();
    println!("    # Show program info");
    println!("    zkplex-cli --zircon proof.zrc --info");
    println!();
    println!("OUTPUT SIGNALS:");
    println!("    - Output signal receives the computed circuit result");
    println!("    - Mark with '?' as value: --public result:?");
    println!("    - Exactly ONE output signal required per proof");
    println!("    - Output signal cannot be used in circuit expression");
    println!("    - Example: Circuit 'A + B' with output signal 'result:?' will compute result = A + B");
    println!();
    println!("NOTES:");
    println!("    - Public signals are included in the proof and can be verified");
    println!("    - Secret signals are NOT saved in proof.json (only used during generation)");
    println!("    - Use '?' as placeholder in Zircon files, then provide values via CLI");
    println!("    - Proof encoding uses ASCII85 (Adobe standard, compatible with online decoders)");
}

fn print_estimate(program: &Program) {
    println!("ZKPlex Circuit Estimation");
    println!("=========================");
    println!();

    // Build circuit from program using Circuit::from_program
    let circuit = match Circuit::from_program(program) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error building circuit: {}", e);
            process::exit(1);
        }
    };

    let estimate = estimate_circuit_requirements_with_strategy(&circuit, None);

    println!("Complexity: {}", estimate.complexity);
    println!();
    println!("Circuit Parameters:");
    println!("  Required k:        {}", estimate.k);
    println!("  Total rows (2^k):  {}", estimate.total_rows);
    println!("  Estimated rows:    {}", estimate.estimated_rows);
    println!("  Row utilization:   {:.1}%",
        (estimate.estimated_rows as f64 / estimate.total_rows as f64) * 100.0);
    println!();
    println!("Operations:");
    println!("  Arithmetic ops:    {}", estimate.operation_count);
    println!("  Comparisons:       {}", estimate.comparison_count);
    println!("  Preprocessing:     {}", estimate.preprocess_count);
    println!();
    println!("Resource Requirements (Hardware-Independent):");
    println!("  Params size:       {} bytes ({} KB)",
        estimate.params_size_bytes,
        estimate.params_size_bytes / 1024);
    println!("  Proof size:        {} bytes ({:.1} KB)",
        estimate.proof_size_bytes,
        estimate.proof_size_bytes as f64 / 1024.0);
    println!("  VK size:           {} bytes ({:.1} KB)",
        estimate.vk_size_bytes,
        estimate.vk_size_bytes as f64 / 1024.0);
    println!();
    println!("Note: These estimates are hardware-independent and show the");
    println!("      minimum requirements for proof generation and verification.");
}

fn print_program_info(program: &Program) {
    println!("ZKPlex Program Information");
    println!("==========================");
    println!();
    println!("Version: {}", program.version);
    println!();

    println!("Secret Signals: {}", program.secret.len());
    for (name, signal) in &program.secret {
        let value_str = signal.value.as_deref().unwrap_or("");
        println!("  - {}: {} (encoding: {:?})", name, value_str, signal.encoding);
    }
    println!();

    println!("Public Signals: {}", program.public.len());
    for (name, signal) in &program.public {
        let value_str = signal.value.as_deref().unwrap_or("");
        println!("  - {}: {} (encoding: {:?})", name, value_str, signal.encoding);
    }
    println!();

    if !program.preprocess.is_empty() {
        println!("Preprocessing: {} statements", program.preprocess.len());
        for (i, stmt) in program.preprocess.iter().enumerate() {
            println!("  {}. {}", i + 1, stmt);
        }
        println!();
    }

    println!("Circuit: {} statements", program.circuit.len());
    for (i, stmt) in program.circuit.iter().enumerate() {
        println!("  {}. {}", i + 1, stmt);
    }
    println!();

    // Calculate zircon format size
    let zircon = program.to_zircon();
    println!("Zircon format size: {} bytes", zircon.len());
    println!("Zircon format: {}", zircon);
}

/// Convert encoding string to ValueEncoding enum
fn string_to_value_encoding(s: &str) -> Result<ValueEncoding, String> {
    match s.to_lowercase().as_str() {
        "decimal" => Ok(ValueEncoding::Decimal),
        "hex" => Ok(ValueEncoding::Hex),
        "base58" | "b58" => Ok(ValueEncoding::Base58),
        "base64" | "b64" => Ok(ValueEncoding::Base64),
        "base85" | "b85" => Ok(ValueEncoding::Base85),
        "text" | "txt" | "string" | "str" => Ok(ValueEncoding::Text),
        _ => Err(format!("Unknown encoding: {}. Supported: decimal, hex, base58/b58, base64/b64, base85/b85, text/txt/string/str", s)),
    }
}

/// Parse signal in format "name", "name:value" or "name:value:encoding"
///
/// Supported encodings: base58/b58, base64/b64, hex, decimal
///
/// Examples:
/// - "output" - output signal with no value (empty string)
/// - "A:10" - decimal value 10 (default)
/// - "A:3J98t1WpEZ73CNmYviecrnyiWrnqRhWNLy:base58" - base58 encoded value
/// - "A:3J98t1WpEZ73CNmYviecrnyiWrnqRhWNLy:b58" - same as above (short form)
/// - "pubkey:AbCdEf:hex" - hex encoded value
fn parse_signal(signal_str: &str) -> Result<(String, Option<String>, Option<ValueEncoding>), String> {
    let parts: Vec<&str> = signal_str.split(':').collect();

    match parts.len() {
        1 => {
            // Format: name (output signal with no value)
            Ok((parts[0].to_string(), None, None))
        }
        2 => {
            // Format: name:value (no encoding)
            // Special case: "?" means output signal (empty value)
            let value = if parts[1] == "?" {
                None
            } else {
                Some(parts[1].to_string())
            };
            Ok((parts[0].to_string(), value, None))
        }
        3 => {
            // Format: name:value:encoding
            let encoding_str = parts[2];

            // Validate and convert encoding
            let encoding = string_to_value_encoding(encoding_str)?;

            // Special case: "?" means output signal (empty value)
            let value = if parts[1] == "?" {
                None
            } else {
                Some(parts[1].to_string())
            };

            Ok((parts[0].to_string(), value, Some(encoding)))
        }
        _ => {
            Err(format!(
                "Invalid signal format '{}', expected 'name', 'name:value' or 'name:value:encoding'",
                signal_str
            ))
        }
    }
}

/// Check if program has secret signals with concrete values (not placeholders)
/// Returns a warning message if found
fn check_program_privacy_warning(program: &Program) -> Option<String> {
    let has_secret_concrete_values = program.secret.iter()
        .any(|(_, sig)| sig.value.as_deref() != Some("?"));

    if has_secret_concrete_values {
        Some(
            "⚠ WARNING: Program contains secret signals with concrete values.\n\
             These values will NOT be saved in proof.json (only public signals are saved).\n\
             However, the circuit IS saved. Ensure your circuit doesn't contain\n\
             literal secret values (use variable names instead).".to_string()
        )
    } else {
        None
    }
}

/// Helper function to parse signals from command line arguments
fn parse_signals_from_cli(
    secret_signals: &[String],
    public_signals: &[String],
) -> IndexMap<String, Signal> {
    let mut signals = IndexMap::new();

    // Parse secret signals
    for sig_str in secret_signals {
        match parse_signal(sig_str) {
            Ok((name, value, encoding)) => {
                signals.insert(name, Signal {
                    value,
                    encoding,
                    public: false,
                });
            }
            Err(e) => {
                eprintln!("Error parsing secret signal: {}", e);
                process::exit(1);
            }
        }
    }

    // Parse public signals
    for sig_str in public_signals {
        match parse_signal(sig_str) {
            Ok((name, value, encoding)) => {
                signals.insert(name, Signal {
                    value,
                    encoding,
                    public: true,
                });
            }
            Err(e) => {
                eprintln!("Error parsing public signal: {}", e);
                process::exit(1);
            }
        }
    }

    signals
}

/// Apply signal overrides from command line to program
/// Handles '?' placeholders by replacing them with values from --secret/--public
fn apply_signal_overrides_cli(
    program: &mut Program,
    secret_signals_cli: &[String],
    public_signals_cli: &[String],
) {
    use zkplex_core::api::Signal as TypesSignal;

    // Parse command line signals into TypesSignal format
    let mut overrides = IndexMap::new();

    for sig_str in secret_signals_cli {
        if let Ok((name, value, encoding)) = parse_signal(sig_str) {
            overrides.insert(name, TypesSignal {
                value,
                encoding,
                public: false,
            });
        }
    }

    for sig_str in public_signals_cli {
        if let Ok((name, value, encoding)) = parse_signal(sig_str) {
            overrides.insert(name, TypesSignal {
                value,
                encoding,
                public: true,
            });
        }
    }

    // Use shared helper function
    if let Err(e) = zkplex_core::api::apply_signal_overrides(program, &overrides) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

/// Generate a proof from a Program
fn generate_proof(
    program: &Program,
    output_file: Option<&str>,
    strategy: Option<Strategy>,
) {
    use std::fs;

    // Validate and display strategy
    let strategy_value = strategy.unwrap_or(Strategy::Auto);
    eprintln!("Circuit strategy: {} - {}", strategy_value.as_str(), strategy_value.description());

    // Join circuit statements for display
    let circuit_str = program.circuit.join("; ");
    eprintln!("Circuit: {}", circuit_str);

    // Display signals
    eprintln!("Signals:");
    for (name, sig) in &program.secret {
        let value_str = sig.value.as_ref().map(|v| v.as_str()).unwrap_or("");
        eprintln!("  {} = {} (secret)", name, value_str);
    }
    for (name, sig) in &program.public {
        let value_str = sig.value.as_ref().map(|v| v.as_str()).unwrap_or("");
        eprintln!("  {} = {} (public)", name, value_str);
    }

    // Convert Program to ProveRequest using shared helper
    let prove_request = zkplex_core::api::program_to_prove_request(program, strategy_value);

    // Call core prove function
    eprintln!("Generating proving key...");
    eprintln!("Creating proof...");
    let response = match zkplex_core::api::core::prove(prove_request) {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Error generating proof: {}", e);
            process::exit(1);
        }
    };

    // Output some info about the proof
    let proof_bytes_len = response.proof.len() * 3 / 4;  // Approximate size (Base85 overhead)
    eprintln!("Proof size: ~{} bytes (~{:.1} KB)", proof_bytes_len, proof_bytes_len as f64 / 1024.0);
    eprintln!("Proof encoding: Base85 {} bytes", response.proof.len());

    // Show warnings if any
    if let Some(ref debug) = response.debug {
        if let Some(ref warnings) = debug.warnings {
            for warning in warnings {
                eprintln!("\n⚠ WARNING: {}", warning);
            }
        }
    }

    // Serialize response to JSON
    let json = match serde_json::to_string_pretty(&response) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Failed to serialize response: {}", e);
            process::exit(1);
        }
    };

    // Output proof
    if let Some(file) = output_file {
        if let Err(e) = fs::write(file, &json) {
            eprintln!("Failed to write proof to file: {}", e);
            process::exit(1);
        }
        eprintln!("✓ Proof saved to {}", file);
    } else {
        // Output JSON to stdout (no prefix message to keep it clean for piping)
        println!("{}", json);
        eprintln!("\n✓ Proof generated successfully");
    }
}

// /// Helper function to generate proof with a given circuit type
// fn generate_proof_with_circuit<C>(
//     circuit: C,
//     public_inputs: Vec<Fp>,
//     k: u32,
// ) -> Result<Vec<u8>, String>
// where
//     C: PlonkCircuit<Fp> + Clone,
// {
//     // Generate universal parameters for the circuit size
//     let params: Params<EqAffine> = Params::new(k);
//
//     // Generate verifying key
//     let empty_circuit = circuit.clone().without_witnesses();
//     let vk = keygen_vk(&params, &empty_circuit)
//         .map_err(|e| format!("Failed to generate VK: {:?}", e))?;
//
//     // Generate proving key
//     eprintln!("Generating proving key...");
//     let pk = keygen_pk(&params, vk.clone(), &empty_circuit)
//         .map_err(|e| format!("Failed to generate PK: {:?}", e))?;
//
//     // Create proof
//     eprintln!("Creating proof...");
//     let mut transcript = Blake2bWrite::<_, EqAffine, Challenge255<_>>::init(vec![]);
//
//     // Prepare public inputs
//     let public_inputs_slice: &[Fp] = &public_inputs;
//     let public_inputs_for_circuit: &[&[Fp]] = &[public_inputs_slice];
//
//     create_proof(&params, &pk, &[circuit], &[public_inputs_for_circuit], OsRng, &mut transcript)
//         .map_err(|e| format!("Failed to create proof: {:?}", e))?;
//
//     Ok(transcript.finalize())
// }

/// Verify a proof
fn verify_proof(proof_file: &str, into_json: bool) {
    use std::fs;

    if !into_json {
        println!("Verifying proof from {}...", proof_file);
    }

    // Read proof file
    let json = match fs::read_to_string(proof_file) {
        Ok(content) => content,
        Err(e) => {
            if into_json {
                let error_json = serde_json::json!({
                    "valid": false,
                    "error": format!("Failed to read proof file: {}", e)
                });
                println!("{}", serde_json::to_string_pretty(&error_json).unwrap());
                process::exit(1);
            } else {
                eprintln!("Failed to read proof file: {}", e);
                process::exit(1);
            }
        }
    };

    // Parse proof response (ProveResponse format)
    let prove_response: ProveResponse = match serde_json::from_str(&json) {
        Ok(resp) => resp,
        Err(e) => {
            if into_json {
                let error_json = serde_json::json!({
                    "valid": false,
                    "error": format!("Failed to parse proof JSON: {}", e)
                });
                println!("{}", serde_json::to_string_pretty(&error_json).unwrap());
                process::exit(1);
            } else {
                eprintln!("Failed to parse proof JSON: {}", e);
                process::exit(1);
            }
        }
    };

    // Create VerifyRequest
    let verify_request = VerifyRequest {
        version: prove_response.version,
        proof: prove_response.proof,
        verify_context: prove_response.verify_context,
        public_signals: prove_response.public_signals,
    };

    // Call core verify function
    let verify_response = match zkplex_core::api::core::verify(verify_request) {
        Ok(resp) => resp,
        Err(e) => {
            if into_json {
                let error_json = serde_json::json!({
                    "valid": false,
                    "error": e
                });
                println!("{}", serde_json::to_string_pretty(&error_json).unwrap());
                process::exit(1);
            } else {
                eprintln!("Verification error: {}", e);
                process::exit(1);
            }
        }
    };

    // Output result
    if into_json {
        println!("{}", serde_json::to_string_pretty(&verify_response).unwrap());
        if !verify_response.valid {
            process::exit(1);
        }
    } else {
        if verify_response.valid {
            println!("✓ Proof is VALID");
        } else {
            eprintln!("✗ Proof is INVALID{}",
                verify_response.error.map(|e| format!(": {}", e)).unwrap_or_default());
            process::exit(1);
        }
    }
}

/// Helper function to load program from different formats with error handling
fn load_program_from_format(
    input: &str,
    format: &str,
    secret_signals: &[String],
    public_signals: &[String],
) -> Program {
    // Read content from file or use as string
    let content = read_input_or_file(input);

    // Parse program based on format
    let mut program = match format {
        "zircon" => Program::from_zircon(&content)
            .unwrap_or_else(|e| {
                eprintln!("Error parsing zircon format: {}", e);
                process::exit(1);
            }),
        "json" => Program::from_json(&content)
            .unwrap_or_else(|e| {
                eprintln!("Error parsing JSON format: {}", e);
                process::exit(1);
            }),
        _ => {
            eprintln!("Error: Unknown format '{}'", format);
            process::exit(1);
        }
    };

    // Check for privacy warnings BEFORE applying overrides
    if let Some(warning) = check_program_privacy_warning(&program) {
        eprintln!("{}", warning);
        eprintln!();
    }

    // Handle '?' placeholders and override with command line signals
    apply_signal_overrides_cli(&mut program, secret_signals, public_signals);

    program
}
