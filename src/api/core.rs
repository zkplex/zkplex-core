//! Core API functions for proof generation and verification
//!
//! This module contains the platform-independent logic for:
//! - `prove()`    - Generate a zero-knowledge proof
//! - `verify()`   - Verify a proof
//! - `estimate()` - Estimate circuit requirements
//!
//! Both CLI and WASM bindings use these functions as their core implementation.

use crate::circuit::{
    Circuit, CircuitAuto, CircuitBoolean, CircuitBitD, CircuitLookup,
    estimate_circuit_requirements_with_strategy, validate_strategy_compatibility,
};
use crate::api::{ProveRequest, ProveResponse, VerifyRequest, VerifyResponse, DebugInfo, PublicSignal, VerifyContext};
use halo2_proofs::pasta::{Fp, EqAffine};
use halo2_proofs::poly::commitment::Params;
use halo2_proofs::plonk::{Circuit as PlonkCircuit, keygen_vk, keygen_pk, create_proof, verify_proof, SingleVerifier};
use halo2_proofs::transcript::{Blake2bWrite, Blake2bRead, Challenge255};
use rand_core::OsRng;
use indexmap::IndexMap;
use crate::api::program::Signal;

/// Generate a zero-knowledge proof
///
/// # Arguments
/// * `request` - Proof generation request containing circuit and signals
///
/// # Returns
/// * `Ok(ProveResponse)` - Proof and verification context
/// * `Err(String)` - Error message if proof generation fails
pub fn prove(request: ProveRequest) -> Result<ProveResponse, String> {
    // Convert request to Program, then build circuit
    let program = request.to_program();
    let circuit = Circuit::from_program(&program)
        .map_err(|e| format!("Failed to build circuit: {}", e))?;

    // Validate strategy compatibility with circuit operations
    validate_strategy_compatibility(&circuit, request.strategy)?;

    // Estimate circuit requirements to determine k automatically based on strategy
    let estimate = estimate_circuit_requirements_with_strategy(&circuit, Some(request.strategy));
    let k = estimate.k;

    // Generate universal parameters for the circuit size
    let params: Params<EqAffine> = Params::new(k);

    // Find all output signals (public signals with no value or empty value or "?")
    let output_signals: Vec<String> = request.signals.iter()
        .filter(|(_, sig)| sig.public && sig.value.as_ref().map(|v| v.is_empty() || v == "?").unwrap_or(true))
        .map(|(name, _)| name.clone())
        .collect();

    // Validate that exactly one output signal exists
    if output_signals.is_empty() {
        return Err("No output signal found. At least one public signal must have no value (or '?') to receive the circuit result.".to_string());
    }
    if output_signals.len() > 1 {
        return Err(format!(
            "Multiple output signals found: {}. Only one public signal can have no value (or '?') to receive the circuit result.",
            output_signals.join(", ")
        ));
    }

    let output_signal_name = output_signals[0].clone();

    // Collect public signal values (exclude output signal, it will be added separately)
    let mut public_inputs: Vec<Fp> = circuit.public_signal_names.iter()
        .filter(|name| *name != &output_signal_name)
        .filter_map(|name| circuit.signals.get(name).copied())
        .collect();

    // The output signal value comes from circuit_output, not from circuit.signals
    // circuit_output contains the evaluated result of the circuit expression
    let output_signal_value = circuit.circuit_output
        .ok_or_else(|| "Circuit did not produce an output value".to_string())?;

    // Append circuit_output as the last public input (required for constraint)
    public_inputs.push(output_signal_value);

    // Generate proof using the appropriate circuit wrapper based on strategy
    use crate::circuit::Strategy;
    let proof_bytes = match request.strategy {
        Strategy::Boolean => {
            let circuit_wrapped = CircuitBoolean(circuit.clone());
            generate_proof_for_circuit(circuit_wrapped, public_inputs.clone(), &params)?
        }
        Strategy::BitD => {
            let circuit_wrapped = CircuitBitD(circuit.clone());
            generate_proof_for_circuit(circuit_wrapped, public_inputs.clone(), &params)?
        }
        Strategy::Lookup => {
            let circuit_wrapped = CircuitLookup(circuit.clone());
            generate_proof_for_circuit(circuit_wrapped, public_inputs.clone(), &params)?
        }
        Strategy::Auto => {
            let circuit_wrapped = CircuitAuto(circuit.clone());
            generate_proof_for_circuit(circuit_wrapped, public_inputs.clone(), &params)?
        }
    };

    // Encode proof with ASCII85 (Adobe standard, compatible with online decoders)
    let proof_encoded = ascii85::encode(&proof_bytes);

    // Check for privacy warnings
    let mut warnings = Vec::new();
    let has_secret_concrete_values = request.signals.iter()
        .any(|(_, sig)| !sig.public && sig.value.is_some());

    if has_secret_concrete_values {
        warnings.push(
            "Program contains secret signals with concrete values. \
             These values will NOT be saved in proof (only public signals are saved). \
             However, the circuit IS saved. Ensure your circuit doesn't contain \
             literal secret values (use variable names instead).".to_string()
        );
    }

    // Prepare public signals output with encoding information
    let public_signals_output: IndexMap<String, PublicSignal> = request.signals.iter()
        .filter(|(_, sig)| sig.public)
        .map(|(name, sig)| {
            let value = if name == &output_signal_name {
                field_to_u64(&output_signal_value).to_string()
            } else {
                sig.value.clone().unwrap_or_default()
            };
            (name.clone(), PublicSignal {
                value,
                encoding: sig.encoding,
            })
        })
        .collect();

    // Collect secret signal names for circuit reconstruction during verification
    let secret_signals: Vec<String> = request.signals.iter()
        .filter(|(_, sig)| !sig.public)
        .map(|(name, _)| name.clone())
        .collect();

    // Create verification context
    let verify_context = VerifyContext {
        k,
        preprocess: request.preprocess.clone(),
        circuit: request.circuit.clone(),
        strategy: request.strategy.clone(),
        secret_signals: secret_signals.clone(),
        output_signal: output_signal_name.clone(),
        cached_max_bits: circuit.cached_max_bits,
    };

    // Serialize verification context to JSON
    let verify_context_json = serde_json::to_string(&verify_context)
        .map_err(|e| format!("Failed to serialize verification context: {}", e))?;

    // Encode verification context with Base85
    let verify_context_encoded = ascii85::encode(verify_context_json.as_bytes());

    // Create debug info
    let debug_info = DebugInfo {
        preprocess: request.preprocess.clone(),
        circuit: request.circuit.clone(),
        k,
        strategy: request.strategy.clone(),
        max_bits: circuit.cached_max_bits,
        secret_signals,
        output_signal: output_signal_name,
        warnings: if warnings.is_empty() { None } else { Some(warnings) },
    };

    // Create response
    Ok(ProveResponse {
        version: crate::api::PROOF_VERSION,
        proof: proof_encoded,
        verify_context: verify_context_encoded,
        public_signals: public_signals_output,
        debug: Some(debug_info),
    })
}

/// Verify a zero-knowledge proof
///
/// # Arguments
/// * `request` - Verification request containing proof, context and public signals
///
/// # Returns
/// * `Ok(VerifyResponse)` - Verification result (valid/invalid)
/// * `Err(String)` - Error message if verification fails
pub fn verify(request: VerifyRequest) -> Result<VerifyResponse, String> {
    // Decode verification context
    let verify_context_bytes = ascii85::decode(&request.verify_context)
        .map_err(|e| format!("Failed to decode verification context: {}", e))?;

    let verify_context_json = String::from_utf8(verify_context_bytes)
        .map_err(|e| format!("Failed to decode verification context as UTF-8: {}", e))?;

    let verify_context: VerifyContext = serde_json::from_str(&verify_context_json)
        .map_err(|e| format!("Failed to parse verification context: {}", e))?;

    // Convert to program and build circuit

    let mut secret_sigs = IndexMap::new();
    let mut public_sigs = IndexMap::new();

    // Add public signals (convert from PublicSignal to Signal)
    // IMPORTANT: Skip the output signal - it will be handled separately
    for (name, public_sig) in &request.public_signals {
        if name == &verify_context.output_signal {
            // Skip output signal - it should not be in program.public during circuit building
            // It will be added to public_inputs separately after circuit evaluation
            continue;
        }
        public_sigs.insert(name.clone(), Signal {
            value: Some(public_sig.value.clone()),
            encoding: public_sig.encoding,
        });
    }

    // Add secret signals with NO values (verifier doesn't have access to secrets)
    // These are just placeholders to maintain circuit structure
    for name in &verify_context.secret_signals {
        secret_sigs.insert(name.clone(), Signal {
            value: None,  // No value - will be skipped during circuit building
            encoding: None,
        });
    }

    let program = crate::api::Program {
        version: crate::api::PROOF_VERSION,
        secret: secret_sigs,
        public: public_sigs,
        preprocess: verify_context.preprocess.clone(),
        circuit: verify_context.circuit.clone(),
    };

    let mut circuit = Circuit::from_program(&program)
        .map_err(|e| format!("Failed to build circuit: {}", e))?;

    // Restore cached_max_bits from verify context (needed for range check table size)
    // This is essential because circuit.signals may be empty during verification
    circuit.cached_max_bits = verify_context.cached_max_bits;

    // Generate params with the same k used during proof generation
    let params: Params<EqAffine> = Params::new(verify_context.k);

    // Collect public signal values in the same order as circuit.public_signal_names
    // IMPORTANT: Exclude output signal from public_signal_names, as it will be added separately
    let mut public_inputs: Vec<Fp> = circuit.public_signal_names.iter()
        .filter(|name| *name != &verify_context.output_signal)
        .filter_map(|name| circuit.signals.get(name).copied())
        .collect();

    // Add output signal value from public signals
    let output_str = request.public_signals.get(&verify_context.output_signal)
        .map(|sig| &sig.value)
        .ok_or_else(|| format!("Missing output signal '{}' in public signals", verify_context.output_signal))?;

    let output_u64: u64 = output_str.parse()
        .map_err(|_| "Failed to parse output value from proof".to_string())?;
    let output_fp = Fp::from(output_u64);
    public_inputs.push(output_fp);

    // Generate VK for the same strategy as was used during proving
    let vk = generate_vk_for_strategy(&circuit, verify_context.strategy, &params)?;

    // Decode proof
    let proof_bytes = ascii85::decode(&request.proof)
        .map_err(|e| format!("Failed to decode proof: {}", e))?;

    // Verify the proof
    let strategy = SingleVerifier::new(&params);
    let mut transcript = Blake2bRead::<_, EqAffine, Challenge255<_>>::init(&proof_bytes[..]);

    let public_inputs_slice: &[Fp] = &public_inputs;
    let public_inputs_for_verification: &[&[Fp]] = &[public_inputs_slice];

    let verification_result = verify_proof(
        &params,
        &vk,
        strategy,
        &[public_inputs_for_verification],
        &mut transcript,
    );

    // Create response
    Ok(VerifyResponse {
        valid: verification_result.is_ok(),
        error: verification_result.err().map(|e| format!("{:?}", e)),
    })
}

/// Estimate circuit requirements
///
/// # Arguments
/// * `request` - Estimation request containing circuit
///
/// # Returns
/// * Estimation result with k, row counts, and resource requirements
pub fn estimate(request: ProveRequest) -> Result<crate::api::EstimateResponse, String> {
    // Convert request to Program, then build circuit
    let program = request.to_program();
    let circuit = Circuit::from_program(&program)
        .map_err(|e| format!("Failed to build circuit: {}", e))?;

    // Validate strategy compatibility
    validate_strategy_compatibility(&circuit, request.strategy)?;

    // Get estimation
    let estimate = estimate_circuit_requirements_with_strategy(&circuit, Some(request.strategy));

    Ok(crate::api::EstimateResponse {
        complexity: estimate.complexity.to_string(),
        k: estimate.k,
        total_rows: estimate.total_rows,
        estimated_rows: estimate.estimated_rows,
        operation_count: estimate.operation_count,
        comparison_count: estimate.comparison_count,
        preprocess_count: estimate.preprocess_count,
        params_size_bytes: estimate.params_size_bytes,
        proof_size_bytes: estimate.proof_size_bytes,
        vk_size_bytes: estimate.vk_size_bytes,
    })
}

// ============================================================================
// Helper functions
// ============================================================================

/// Generate proof for a specific circuit type
fn generate_proof_for_circuit<C>(
    circuit: C,
    public_inputs: Vec<Fp>,
    params: &Params<EqAffine>,
) -> Result<Vec<u8>, String>
where
    C: PlonkCircuit<Fp> + Clone,
{
    let empty_wrapped = circuit.clone().without_witnesses();

    // Generate VK
    let vk = keygen_vk(params, &empty_wrapped)
        .map_err(|e| format!("Failed to generate VK: {:?}", e))?;

    // Generate PK
    let pk = keygen_pk(params, vk.clone(), &empty_wrapped)
        .map_err(|e| format!("Failed to generate PK: {:?}", e))?;

    // Create proof
    let mut transcript = Blake2bWrite::<_, EqAffine, Challenge255<_>>::init(vec![]);

    let public_inputs_slice: &[Fp] = &public_inputs;
    let public_inputs_for_circuit: &[&[Fp]] = &[public_inputs_slice];

    create_proof(params, &pk, &[circuit], &[public_inputs_for_circuit], OsRng, &mut transcript)
        .map_err(|e| format!("Failed to create proof: {:?}", e))?;

    Ok(transcript.finalize())
}

/// Generate VK for a specific strategy
fn generate_vk_for_strategy(
    circuit: &Circuit,
    strategy: crate::circuit::Strategy,
    params: &Params<EqAffine>,
) -> Result<halo2_proofs::plonk::VerifyingKey<EqAffine>, String> {
    use crate::circuit::Strategy;
    let result = match strategy {
        Strategy::Boolean => {
            let circuit_wrapped = CircuitBoolean(circuit.clone());
            let empty_wrapped = circuit_wrapped.without_witnesses();
            keygen_vk(params, &empty_wrapped)
        }
        Strategy::BitD => {
            let circuit_wrapped = CircuitBitD(circuit.clone());
            let empty_wrapped = circuit_wrapped.without_witnesses();
            keygen_vk(params, &empty_wrapped)
        }
        Strategy::Lookup => {
            let circuit_wrapped = CircuitLookup(circuit.clone());
            let empty_wrapped = circuit_wrapped.without_witnesses();
            keygen_vk(params, &empty_wrapped)
        }
        Strategy::Auto => {
            let circuit_wrapped = CircuitAuto(circuit.clone());
            let empty_wrapped = circuit_wrapped.without_witnesses();
            keygen_vk(params, &empty_wrapped)
        }
    };

    result.map_err(|e| format!("Failed to generate VK: {:?}", e))
}

/// Convert field element to u64
fn field_to_u64(f: &Fp) -> u64 {
    use ff::PrimeField;
    let bytes = f.to_repr();
    let mut value = 0u64;
    for i in 0..8.min(bytes.as_ref().len()) {
        value |= (bytes.as_ref()[i] as u64) << (i * 8);
    }
    value
}