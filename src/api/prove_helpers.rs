//! Proof generation helpers
//!
//! This module contains shared logic for converting Programs to ProveRequests
//! and applying signal overrides. Used by both CLI and WASM API.

use crate::api::{Program, ProveRequest, Signal as TypesSignal};
use crate::circuit::Strategy;
use indexmap::IndexMap;

/// Apply signal overrides to a Program
///
/// Replaces '?' placeholders and overrides existing values with provided signals.
/// Used by CLI (with --secret/--public flags) and WASM API (with overrides parameter).
///
/// # Arguments
///
/// * `program` - The program to modify (will be modified in place)
/// * `overrides` - Map of signal names to Signal values with override data
///
/// # Errors
///
/// Returns error if a secret signal has '?' placeholder but no override is provided
pub fn apply_signal_overrides(
    program: &mut Program,
    overrides: &IndexMap<String, TypesSignal>,
) -> Result<(), String> {
    // Apply overrides to secret signals
    for (name, override_signal) in overrides {
        if !override_signal.public {
            // Secret signal override
            if let Some(secret_sig) = program.secret.get_mut(name) {
                // Replace placeholder or override existing value
                if let Some(value) = &override_signal.value {
                    secret_sig.value = Some(value.clone());
                }
                if let Some(encoding) = override_signal.encoding {
                    secret_sig.encoding = Some(encoding);
                }
            } else {
                // Add new secret signal
                program.secret.insert(name.clone(), crate::api::program::Signal {
                    value: override_signal.value.clone(),
                    encoding: override_signal.encoding,
                });
            }
        } else {
            // Public signal override
            if let Some(public_sig) = program.public.get_mut(name) {
                // Replace placeholder or override existing value
                if let Some(value) = &override_signal.value {
                    public_sig.value = Some(value.clone());
                }
                if let Some(encoding) = override_signal.encoding {
                    public_sig.encoding = Some(encoding);
                }
            } else {
                // Add new public signal
                program.public.insert(name.clone(), crate::api::program::Signal {
                    value: override_signal.value.clone(),
                    encoding: override_signal.encoding,
                });
            }
        }
    }

    // Validate that no '?' placeholders remain in secret signals
    for (name, signal) in &program.secret {
        if signal.value.as_deref() == Some("?") {
            return Err(format!(
                "Secret signal '{}' has placeholder '?' but no value provided in overrides",
                name
            ));
        }
    }

    Ok(())
}

/// Convert Program to ProveRequest
///
/// Merges secret and public signals into a single signals map with public flags.
///
/// # Arguments
///
/// * `program` - The program to convert
/// * `strategy` - The proof strategy to use
///
/// # Returns
///
/// ProveRequest ready to be passed to core::prove()
pub fn program_to_prove_request(
    program: &Program,
    strategy: Strategy,
) -> ProveRequest {
    let mut signals = IndexMap::new();

    // Add secret signals
    for (name, sig) in &program.secret {
        signals.insert(name.clone(), TypesSignal {
            value: sig.value.clone(),
            encoding: sig.encoding,
            public: false,
        });
    }

    // Add public signals
    for (name, sig) in &program.public {
        signals.insert(name.clone(), TypesSignal {
            value: sig.value.clone(),
            encoding: sig.encoding,
            public: true,
        });
    }

    ProveRequest {
        preprocess: program.preprocess.clone(),
        circuit: program.circuit.clone(),
        signals,
        strategy,
    }
}