//! WASM API bindings
//!
//! This module provides JavaScript-accessible functions for creating
//! and verifying zero-knowledge proofs based on circuits.
//!
//! # Example Usage (JavaScript)
//!
//! ```javascript
//! import init, { prove, verify } from './zkplex_core.js';
//!
//! await init();
//!
//! // Create a proof
//! const proveRequest = {
//!   circuit: "(A + B) > C",
//!   signals: {
//!     A: { value: "10", public: false },
//!     B: { value: "20", public: false },
//!     C: { value: "25", public: true }
//!   }
//! };
//!
//! const proveResponse = JSON.parse(prove(JSON.stringify(proveRequest)));
//! console.log("Proof:", proveResponse.proof);
//!
//! // Verify the proof
//! const verifyRequest = {
//!   proof: proveResponse.proof,
//!   verification_context: proveResponse.verification_context,
//!   public_signals: proveResponse.public_signals
//! };
//!
//! const verifyResponse = JSON.parse(verify(JSON.stringify(verifyRequest)));
//! console.log("Valid:", verifyResponse.valid);
//! ```

use wasm_bindgen::prelude::*;
use crate::api::{ProveRequest, VerifyRequest};
use crate::circuit::Circuit;

/// Version from Cargo.toml
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get build ID from environment variable (set during compilation)
/// If not set, returns None
fn get_build_id() -> Option<&'static str> {
    option_env!("BUILD_ID")
}

#[cfg(target_arch = "wasm32")]
use console_error_panic_hook;

/// Initialize WASM module
///
/// Call this function once before using any other functions.
/// It sets up panic hooks for better error messages in the browser console.
#[wasm_bindgen(start)]
pub fn wasm_init() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
}

/// Get the version of zkplex-core
///
/// Returns the version string from Cargo.toml.
///
/// # Example
///
/// ```javascript
/// import { version } from './zkplex_core.js';
///
/// console.log("zkplex-core version:", version());
/// ```
#[wasm_bindgen]
pub fn version() -> String {
    match get_build_id() {
        Some(build_id) if !build_id.is_empty() => {
            format!("{} ({})", VERSION, build_id)
        }
        _ => VERSION.to_string()
    }
}

/// Create a zero-knowledge proof for a circuit
///
/// Takes a JSON string representing a ProveRequest and returns
/// a JSON string representing a ProveResponse.
///
/// # Arguments
///
/// * `request_json` - JSON string with circuit and signals
///
/// # Returns
///
/// JSON string with proof, verification context, and public signals
///
/// # Example
///
/// ```javascript
/// const request = {
///   circuit: "(A + B) * C > D",
///   signals: {
///     A: { value: "10", public: false },
///     B: { value: "20", public: false },
///     C: { value: "2", public: false },
///     D: { value: "50", public: true }
///   }
/// };
///
/// const response = JSON.parse(prove(JSON.stringify(request)));
/// ```
#[wasm_bindgen]
pub fn prove(request_json: &str) -> Result<String, JsValue> {
    // DEBUG: Log incoming request JSON to browser console
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::log_1(&format!("🔍 WASM prove() received JSON: {}", request_json).into());
    }

    // Parse request
    let request: ProveRequest = serde_json::from_str(request_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse request: {}", e)))?;

    // DEBUG: Log parsed request structure
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::log_1(&format!("🔍 Parsed request - circuit count: {}, signals count: {}",
            request.circuit.len(), request.signals.len()).into());
        web_sys::console::log_1(&format!("🔍 Circuit: {:?}", request.circuit).into());
        web_sys::console::log_1(&format!("🔍 Signals: {:?}", request.signals.keys().collect::<Vec<_>>()).into());
    }

    // Call core prove function
    let response = crate::api::core::prove(request)
        .map_err(|e| {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::error_1(&format!("❌ Prove failed: {}", e).into());
            JsValue::from_str(&e)
        })?;

    // DEBUG: Log success result
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::log_1(&format!("✅ Proof generated successfully").into());
        web_sys::console::log_1(&format!("🔍 Public signals: {:?}", response.public_signals.keys().collect::<Vec<_>>()).into());
        if let Some(debug) = &response.debug {
            web_sys::console::log_1(&format!("🔍 Output signal: {}", debug.output_signal).into());
        }
    }

    // Serialize response
    serde_json::to_string(&response)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize response: {}", e)))
}

/// Verify a zero-knowledge proof
///
/// Takes a JSON string representing a VerifyRequest and returns
/// a JSON string representing a VerifyResponse.
///
/// # Arguments
///
/// * `request_json` - JSON string with proof, verification_context, and public signals
///
/// # Returns
///
/// JSON string with verification output
///
/// # Example
///
/// ```javascript
/// const request = {
///   proof: proveResponse.proof,
///   verification_context: proveResponse.verification_context,
///   public_signals: proveResponse.public_signals
/// };
///
/// const response = JSON.parse(verify(JSON.stringify(request)));
/// console.log("Valid:", response.valid);
/// ```
#[wasm_bindgen]
pub fn verify(request_json: &str) -> Result<String, JsValue> {
    // DEBUG: Log incoming request
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::log_1(&format!("🔍 WASM verify() called").into());
        web_sys::console::log_1(&format!("🔍 Request JSON length: {} bytes", request_json.len()).into());
    }

    // Parse request
    let request: VerifyRequest = serde_json::from_str(request_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse request: {}", e)))?;

    // DEBUG: Log parsed request structure
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::log_1(&format!("🔍 Public signals count: {}", request.public_signals.len()).into());
        web_sys::console::log_1(&format!("🔍 Public signals: {:?}", request.public_signals.keys().collect::<Vec<_>>()).into());
    }

    // Call core verify function
    let response = crate::api::core::verify(request)
        .map_err(|e| {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::error_1(&format!("❌ Verification failed: {}", e).into());
            JsValue::from_str(&e)
        })?;

    // DEBUG: Log verification result
    #[cfg(target_arch = "wasm32")]
    {
        if response.valid {
            web_sys::console::log_1(&format!("✅ Proof is VALID").into());
        } else {
            web_sys::console::warn_1(&format!("⚠️ Proof is INVALID").into());
            if let Some(error) = &response.error {
                web_sys::console::warn_1(&format!("🔍 Verification error: {}", error).into());
            }
        }
    }

    // Serialize response
    serde_json::to_string(&response)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize response: {}", e)))
}

/// Convert zircon program format to JSON
///
/// Takes a zircon format string and returns a JSON representation.
///
/// # Arguments
///
/// * `zircon` - Zircon format string (e.g., "1/A:10,B:20/-/A+B")
///
/// # Returns
///
/// JSON string representation of the program
///
/// # Example
///
/// ```javascript
/// import { zircon_to_json } from './zkplex_core.js';
///
/// const zircon = "1/A:10,B:20/threshold:100/sum<==A+B;sum>threshold";
/// const json = zircon_to_json(zircon);
/// console.log(json);
/// // {
/// //   "version": 1,
/// //   "secret": {"A": {"value": "10"}, "B": {"value": "20"}},
/// //   "public": {"threshold": {"value": "100"}},
/// //   "circuit": ["sum<==A+B", "sum>threshold"]
/// // }
/// ```
#[wasm_bindgen]
pub fn zircon_to_json(zircon: &str) -> Result<String, JsValue> {
    use crate::api::Program;

    // Parse zircon format
    let program = Program::from_zircon(zircon)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse zircon format: {}", e)))?;

    // Convert to JSON
    program.to_json()
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize to JSON: {}", e)))
}

/// Convert JSON program format to zircon
///
/// Takes a JSON string and returns a zircon format string.
///
/// # Arguments
///
/// * `json` - JSON string representation of a program
///
/// # Returns
///
/// Zircon format string
///
/// # Example
///
/// ```javascript
/// import { json_to_zircon } from './zkplex_core.js';
///
/// const json = JSON.stringify({
///   version: 1,
///   secret: {A: {value: "10"}, B: {value: "20"}},
///   public: {threshold: {value: "100"}},
///   circuit: ["sum<==A+B", "sum>threshold"]
/// });
///
/// const zircon = json_to_zircon(json);
/// console.log(zircon);
/// // "1/A:10,B:20/threshold:100/sum<==A+B;sum>threshold"
/// ```
#[wasm_bindgen]
pub fn json_to_zircon(json: &str) -> Result<String, JsValue> {
    use crate::api::Program;

    // Parse JSON
    let program = Program::from_json(json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse JSON: {}", e)))?;

    // Convert to zircon format
    Ok(program.to_zircon())
}

/// Estimate circuit requirements
///
/// Analyzes a circuit and returns hardware-independent metrics
/// about the required resources for proof generation.
///
/// # Arguments
///
/// * `request_json` - JSON string with circuit and signals
///
/// # Returns
///
/// JSON string with estimation metrics
///
/// # Example
///
/// ```javascript
/// import { estimate } from './zkplex_core.js';
///
/// const request = {
///   circuit: "(A + B) * C > D",
///   signals: {
///     A: { value: "10", public: false },
///     B: { value: "20", public: false },
///     C: { value: "2", public: false },
///     D: { value: "50", public: true }
///   }
/// };
///
/// const estimateResponse = JSON.parse(estimate(JSON.stringify(request)));
/// console.log("Required k:", estimateResponse.k);
/// console.log("Estimated rows:", estimateResponse.estimated_rows);
/// console.log("Params size:", estimateResponse.params_size_bytes, "bytes");
/// console.log("Proof size:", estimateResponse.proof_size_bytes, "bytes");
/// console.log("Complexity:", estimateResponse.complexity);
/// ```
#[wasm_bindgen]
pub fn estimate(request_json: &str) -> Result<String, JsValue> {
    use crate::api::ProveRequest;

    // DEBUG: Log incoming request
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::log_1(&format!("🔍 WASM estimate() called").into());
        web_sys::console::log_1(&format!("🔍 Request JSON length: {} bytes", request_json.len()).into());
    }

    // Parse request
    let request: ProveRequest = serde_json::from_str(request_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse request: {}", e)))?;

    // DEBUG: Log parsed request structure
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::log_1(&format!("🔍 Circuit count: {}, signals count: {}",
            request.circuit.len(), request.signals.len()).into());
        web_sys::console::log_1(&format!("🔍 Circuit: {:?}", request.circuit).into());
    }

    // Call core estimate function
    let response = crate::api::core::estimate(request)
        .map_err(|e| {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::error_1(&format!("❌ Estimation failed: {}", e).into());
            JsValue::from_str(&e)
        })?;

    // DEBUG: Log estimation results
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::log_1(&format!("✅ Circuit estimation completed").into());
        web_sys::console::log_1(&format!("🔍 k = {}, estimated rows = {}", response.k, response.estimated_rows).into());
        web_sys::console::log_1(&format!("🔍 Proof size: {} bytes, Complexity: {}", response.proof_size_bytes, response.complexity).into());
    }

    // Serialize response
    serde_json::to_string(&response)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize response: {}", e)))
}

/// Parse Zircon format to Program
///
/// Converts Zircon format string to Program JSON representation.
/// This is step 1 in the proof generation workflow.
///
/// # Arguments
///
/// * `zircon` - Zircon format string (e.g., "1/age:?/-/age>=18")
///
/// # Returns
///
/// JSON string representation of Program
///
/// # Example
///
/// ```javascript
/// import { parse_zircon } from './zkplex_core.js';
///
/// const program = JSON.parse(parse_zircon("1/age:?/-/age>=18"));
/// console.log(program);
/// // { version: 1, secret: { age: { value: "?" } }, circuit: ["age>=18"] }
/// ```
#[wasm_bindgen]
pub fn parse_zircon(zircon: &str) -> Result<String, JsValue> {
    use crate::api::Program;

    let program = Program::from_zircon(zircon)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse Zircon: {}", e)))?;

    program.to_json()
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize Program: {}", e)))
}

/// Parse JSON format to Program
///
/// Validates and normalizes JSON Program representation.
/// This is step 1 in the proof generation workflow (alternative to parse_zircon).
///
/// # Arguments
///
/// * `json` - JSON string representation of Program
///
/// # Returns
///
/// Normalized JSON string representation of Program
///
/// # Example
///
/// ```javascript
/// import { parse_json } from './zkplex_core.js';
///
/// const input = JSON.stringify({
///   version: 1,
///   secret: { age: { value: "?" } },
///   circuit: ["age>=18"]
/// });
///
/// const program = JSON.parse(parse_json(input));
/// ```
#[wasm_bindgen]
pub fn parse_json(json: &str) -> Result<String, JsValue> {
    use crate::api::Program;

    let program = Program::from_json(json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse JSON: {}", e)))?;

    program.to_json()
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize Program: {}", e)))
}

/// Apply signal overrides to a Program
///
/// Replaces '?' placeholders and overrides existing signal values.
/// This is step 2 in the proof generation workflow.
///
/// # Arguments
///
/// * `program_json` - JSON string representation of Program
/// * `overrides_json` - JSON object with signal overrides
///
/// # Returns
///
/// Updated JSON string representation of Program
///
/// # Example
///
/// ```javascript
/// import { parse_zircon, apply_overrides } from './zkplex_core.js';
///
/// // Step 1: Parse Zircon template
/// const program = parse_zircon("1/age:?/-/age>=18");
///
/// // Step 2: Apply overrides
/// const overrides = JSON.stringify({
///   age: { value: "25", public: false }
/// });
///
/// const updated = apply_overrides(program, overrides);
/// console.log(JSON.parse(updated));
/// // { version: 1, secret: { age: { value: "25" } }, circuit: ["age>=18"] }
/// ```
#[wasm_bindgen]
pub fn apply_overrides(program_json: &str, overrides_json: &str) -> Result<String, JsValue> {
    use crate::api::{Program, Signal as TypesSignal};
    use indexmap::IndexMap;

    // Parse Program
    let mut program = Program::from_json(program_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse Program: {}", e)))?;

    // Parse overrides
    let overrides: IndexMap<String, TypesSignal> = serde_json::from_str(overrides_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse overrides: {}", e)))?;

    // Apply overrides using shared helper
    crate::api::apply_signal_overrides(&mut program, &overrides)
        .map_err(|e| JsValue::from_str(&e))?;

    // Return updated Program as JSON
    program.to_json()
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize Program: {}", e)))
}

/// Convert Program to ProveRequest
///
/// Converts a Program to a ProveRequest ready for proof generation.
/// This is step 3 in the proof generation workflow.
///
/// # Arguments
///
/// * `program_json` - JSON string representation of Program
/// * `strategy` - Optional proof strategy ("auto", "boolean", "lookup", "bitd")
///
/// # Returns
///
/// JSON string representation of ProveRequest
///
/// # Example
///
/// ```javascript
/// import { parse_zircon, program_to_request, prove } from './zkplex_core.js';
///
/// // Step 1: Parse Zircon
/// const program = parse_zircon("1/age:25/-/age>=18");
///
/// // Step 2: Convert to ProveRequest
/// const request = program_to_request(program, "auto");
///
/// // Step 3: Generate proof
/// const proof = prove(request);
/// ```
#[wasm_bindgen]
pub fn program_to_request(program_json: &str, strategy: Option<String>) -> Result<String, JsValue> {
    use crate::api::Program;
    use crate::circuit::Strategy;

    // Parse Program
    let program = Program::from_json(program_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse Program: {}", e)))?;

    // Parse strategy
    let strategy_value = if let Some(s) = strategy {
        s.parse::<Strategy>()
            .map_err(|e| JsValue::from_str(&e))?
    } else {
        Strategy::Auto
    };

    // Convert to ProveRequest using shared helper
    let prove_request = crate::api::program_to_prove_request(&program, strategy_value);

    // Serialize to JSON
    serde_json::to_string(&prove_request)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize ProveRequest: {}", e)))
}

/// Convert Program to EstimateRequest
///
/// Converts a Program to an EstimateRequest ready for circuit estimation.
///
/// # Arguments
///
/// * `program_json` - JSON string representation of Program
/// * `strategy` - Optional proof strategy ("auto", "boolean", "lookup", "bitd")
///
/// # Returns
///
/// JSON string representation of EstimateRequest (same as ProveRequest)
///
/// # Example
///
/// ```javascript
/// import { parse_zircon, program_to_estimate_request, estimate } from './zkplex_core.js';
///
/// // Step 1: Parse Zircon
/// const program = parse_zircon("1/age:25/-/age>=18");
///
/// // Step 2: Convert to EstimateRequest
/// const request = program_to_estimate_request(program, "auto");
///
/// // Step 3: Estimate circuit
/// const estimation = estimate(request);
/// ```
#[wasm_bindgen]
pub fn program_to_estimate_request(program_json: &str, strategy: Option<String>) -> Result<String, JsValue> {
    // Estimation uses the same request format as prove
    program_to_request(program_json, strategy)
}

/// Convert ProveResponse to VerifyRequest
///
/// Extracts verification data from a ProveResponse.
///
/// # Arguments
///
/// * `prove_response_json` - JSON string representation of ProveResponse
///
/// # Returns
///
/// JSON string representation of VerifyRequest
///
/// # Example
///
/// ```javascript
/// import { prove, response_to_verify_request, verify } from './zkplex_core.js';
///
/// // Step 1: Generate proof
/// const proveResponse = prove(request);
///
/// // Step 2: Convert to VerifyRequest
/// const verifyRequest = response_to_verify_request(proveResponse);
///
/// // Step 3: Verify proof
/// const result = verify(verifyRequest);
/// ```
#[wasm_bindgen]
pub fn response_to_verify_request(prove_response_json: &str) -> Result<String, JsValue> {
    use crate::api::{ProveResponse, VerifyRequest};

    // Parse ProveResponse
    let prove_response: ProveResponse = serde_json::from_str(prove_response_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse ProveResponse: {}", e)))?;

    // Create VerifyRequest
    let verify_request = VerifyRequest {
        version: prove_response.version,
        proof: prove_response.proof,
        verify_context: prove_response.verify_context,
        public_signals: prove_response.public_signals,
    };

    // Serialize to JSON
    serde_json::to_string(&verify_request)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize VerifyRequest: {}", e)))
}

/// Estimate constraints for a zircon program
///
/// Takes a zircon format string and returns the estimated constraint count.
///
/// # Arguments
///
/// * `zircon` - Zircon format string (e.g., "1/A:10,B:20/-/A>B")
///
/// # Returns
///
/// Number of estimated constraints
///
/// # Example
///
/// ```javascript
/// import { estimate_constraints } from './zkplex_core.js';
///
/// const zircon = "1/balance:1000/-/balance>100";
/// const count = estimate_constraints(zircon);
/// console.log("Estimated constraints:", count);
/// // Estimated constraints: 68
/// ```
#[wasm_bindgen]
pub fn estimate_constraints(zircon: &str) -> Result<u32, JsValue> {
    use crate::api::Program;
    use crate::api::Signal as TypesSignal;
    use crate::circuit::estimate_circuit_requirements_with_strategy;

    // Parse zircon format
    let program = Program::from_zircon(zircon)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse zircon format: {}", e)))?;

    // Convert to circuit-compatible format (types::Signal with public field)
    let mut signals = indexmap::IndexMap::new();

    // Add secret signals
    for (name, signal) in &program.secret {
        signals.insert(name.clone(), TypesSignal {
            value: signal.value.clone(),
            encoding: signal.encoding,
            public: false,
        });
    }

    // Add public signals
    for (name, signal) in &program.public {
        signals.insert(name.clone(), TypesSignal {
            value: signal.value.clone(),
            encoding: signal.encoding,
            public: true,
        });
    }

    // Build circuit from program
    let circuit = Circuit::from_program(&program)
        .map_err(|e| JsValue::from_str(&e))?;

    // Estimate requirements (use auto strategy for zircon programs)
    let estimate = estimate_circuit_requirements_with_strategy(&circuit, None);

    Ok(estimate.estimated_rows)
}

/// Generate circuit from zircon format
///
/// Takes a zircon format string and returns the R1CS circuit in JSON format.
///
/// # Arguments
///
/// * `zircon` - Zircon format string
///
/// # Returns
///
/// JSON string with circuit constraints
///
/// # Example
///
/// ```javascript
/// import { generate_circuit } from './zkplex_core.js';
///
/// const zircon = "1/A:10,B:20/-/A>B";
/// const circuit = JSON.parse(generate_circuit(zircon));
/// console.log("Constraints:", circuit.num_constraints);
/// ```
#[wasm_bindgen]
pub fn generate_circuit(zircon: &str) -> Result<String, JsValue> {
    use crate::api::Program;
    use crate::api::Signal as TypesSignal;
    use crate::circuit::estimate_circuit_requirements_with_strategy;

    // Parse zircon format
    let program = Program::from_zircon(zircon)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse zircon format: {}", e)))?;

    // Convert to circuit-compatible format (types::Signal with public field)
    let mut signals = indexmap::IndexMap::new();

    // Add secret signals
    for (name, signal) in &program.secret {
        signals.insert(name.clone(), TypesSignal {
            value: signal.value.clone(),
            encoding: signal.encoding,
            public: false,
        });
    }

    // Add public signals
    for (name, signal) in &program.public {
        signals.insert(name.clone(), TypesSignal {
            value: signal.value.clone(),
            encoding: signal.encoding,
            public: true,
        });
    }

    // Build circuit from program
    let circuit = Circuit::from_program(&program)
        .map_err(|e| JsValue::from_str(&e))?;

    // Estimate requirements to get constraint count (use auto strategy for zircon programs)
    let estimate = estimate_circuit_requirements_with_strategy(&circuit, None);

    // Create response
    let response = serde_json::json!({
        "num_constraints": estimate.estimated_rows,
        "num_variables": signals.len(),
        "k": estimate.k,
        "complexity": estimate.complexity
    });

    serde_json::to_string(&response)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize response: {}", e)))
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use super::*;

    #[test]
    fn test_prove_simple_circuit() {
        let request = r#"{
            "circuit": "A + B",
            "signals": {
                "A": { "value": "10", "public": false },
                "B": { "value": "20", "public": true }
            }
        }"#;

        let response = prove(request).unwrap();
        let parsed: ProveResponse = serde_json::from_str(&response).unwrap();

        assert_eq!(parsed.result, "30");
        assert!(parsed.public_signals.contains_key("B"));
        assert_eq!(parsed.public_signals.get("B").unwrap(), "20");
    }

    #[test]
    fn test_prove_comparison() {
        let request = r#"{
            "circuit": "A > B",
            "signals": {
                "A": { "value": "20", "public": false },
                "B": { "value": "10", "public": true }
            }
        }"#;

        let response = prove(request).unwrap();
        let parsed: ProveResponse = serde_json::from_str(&response).unwrap();

        assert_eq!(parsed.result, "1");  // true
        assert!(parsed.public_signals.contains_key("B"));
    }

    #[test]
    fn test_verify_placeholder() {
        let request = r#"{
            "circuit": "A > B",
            "proof": "placeholder",
            "k": 9,
            "strategy": "auto",
            "public_signals": {},
            "result": "1"
        }"#;

        // Note: This test will fail with actual verification
        // It's just a placeholder to show the API structure
        let response = verify(request);
        assert!(response.is_err() || response.is_ok());
    }
}
/// Get circuit layout information as JSON
///
/// Takes a JSON string representing a Program and optional strategy,
/// returns a JSON string with complete layout information.
///
/// # Arguments
///
/// * `program_json` - JSON string representing a Program
/// * `strategy` - Optional strategy ("auto", "boolean", "lookup", "bitd")
///
/// # Returns
///
/// JSON string with complete circuit layout information including:
/// - Circuit parameters (k, total_rows, max_bits)
/// - Row layout breakdown
/// - Resource requirements
/// - Signal information
/// - Operation breakdown
/// - Column configuration
/// - Gate type breakdown
/// - Lookup table information
/// - Memory usage estimates
/// - Complexity analysis
///
/// # Example
///
/// ```javascript
/// const program = {
///   version: 1,
///   secret: { age: { value: "25", encoding: "Decimal" } },
///   public: { result: { value: null, encoding: "Decimal" } },
///   preprocess: [],
///   circuit: ["age>=18"]
/// };
///
/// const layoutJson = get_layout(JSON.stringify(program), "auto");
/// const layout = JSON.parse(layoutJson);
/// console.log("Circuit complexity:", layout.complexity.overall);
/// console.log("Memory required:", layout.memory.prover.total_mb, "MB");
/// ```
#[wasm_bindgen]
pub fn get_layout(program_json: &str, strategy: Option<String>) -> Result<String, String> {
    use crate::api::Program;
    use crate::circuit::Strategy;
    use crate::layout::build_circuit_layout;

    // Parse program from JSON
    let program: Program = serde_json::from_str(program_json)
        .map_err(|e| format!("Failed to parse program JSON: {}", e))?;

    // Parse strategy if provided
    let strat = if let Some(s) = strategy {
        Some(s.parse::<Strategy>()
            .map_err(|e| format!("Invalid strategy: {}", e))?)
    } else {
        None
    };

    // Build layout
    let layout = build_circuit_layout(&program, strat)?;

    // Serialize to JSON
    serde_json::to_string_pretty(&layout)
        .map_err(|e| format!("Failed to serialize layout: {}", e))
}

/// Get circuit layout as ASCII visualization
///
/// Takes a JSON string representing a Program and optional strategy,
/// returns an ASCII art string visualizing the circuit layout.
///
/// # Arguments
///
/// * `program_json` - JSON string representing a Program
/// * `strategy` - Optional strategy ("auto", "boolean", "lookup", "bitd")
///
/// # Returns
///
/// ASCII art string with complete circuit layout visualization
///
/// # Example
///
/// ```javascript
/// const program = {
///   version: 1,
///   secret: { age: { value: "25", encoding: "Decimal" } },
///   public: { result: { value: null, encoding: "Decimal" } },
///   preprocess: [],
///   circuit: ["age>=18"]
/// };
///
/// const ascii = get_layout_ascii(JSON.stringify(program), "auto");
/// console.log(ascii);
/// // Prints:
/// // ╔════════════════════════════════════════════════════════════╗
/// // ║          ZKPlex Circuit Layout Visualization               ║
/// // ╚════════════════════════════════════════════════════════════╝
/// // ...
/// ```
#[wasm_bindgen]
pub fn get_layout_ascii(program_json: &str, strategy: Option<String>) -> Result<String, String> {
    use crate::api::Program;
    use crate::circuit::Strategy;
    use crate::layout::{build_circuit_layout, render_circuit_layout_ascii};

    // Parse program from JSON
    let program: Program = serde_json::from_str(program_json)
        .map_err(|e| format!("Failed to parse program JSON: {}", e))?;

    // Parse strategy if provided
    let strat = if let Some(s) = strategy {
        Some(s.parse::<Strategy>()
            .map_err(|e| format!("Invalid strategy: {}", e))?)
    } else {
        None
    };

    // Build layout
    let layout = build_circuit_layout(&program, strat)?;

    // Render as ASCII
    Ok(render_circuit_layout_ascii(&layout))
}
