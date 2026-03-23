use soroban_sdk::{symbol_short, Address, Env, String};
use crate::types::Attestation;

pub struct Events;

impl Events {
    /// Emit event when an attestation is created
    ///
    /// Schema:
    ///   topics: ("created", subject: Address)
    ///   data:   (attestation_id: String, issuer: Address, claim_type: String, timestamp: u64)
    pub fn attestation_created(env: &Env, attestation: &Attestation) {
        env.events().publish(
            (symbol_short!("created"), attestation.subject.clone()),
            (
                attestation.id.clone(),
                attestation.issuer.clone(),
                attestation.claim_type.clone(),
                attestation.timestamp,
            ),
        );
    }

    /// Emit event when an attestation is revoked
    ///
    /// Schema:
    ///   topics: ("revoked", issuer: Address)
    ///   data:   attestation_id: String
    pub fn attestation_revoked(env: &Env, attestation_id: &String, issuer: &Address) {
        env.events().publish(
            (symbol_short!("revoked"), issuer.clone()),
            attestation_id.clone(),
        );
    }

    /// Emit event when an expired attestation is encountered during a check.
    /// Not emitted for revoked attestations.
    ///
    /// Schema:
    ///   topics: ("expired", subject: Address)
    ///   data:   attestation_id: String
    pub fn attestation_expired(env: &Env, attestation_id: &String, subject: &Address) {
        env.events().publish(
            (symbol_short!("expired"), subject.clone()),
            attestation_id.clone(),
        );
    }
}
