//! Event emission for TrustLink.
//!
//! Every state-changing operation in the contract publishes a structured event
//! so that off-chain indexers, dApps, and other contracts can react to changes
//! without polling storage.
//!
//! All helpers are collected on the zero-size [`Events`] struct. Each method
//! takes `&Env` plus the data relevant to that event and calls
//! `env.events().publish(topics, data)`.
//!
//! ## Event catalogue
//!
//! | Method                  | Symbol       | Topics                    | Data                                      |
//! |-------------------------|--------------|---------------------------|-------------------------------------------|
//! | `admin_initialized`     | `admin_init` | `(symbol,)`               | `(admin: Address, timestamp: u64)`        |
//! | `attestation_created`   | `created`    | `(symbol, subject)`       | `(id, issuer, claim_type, timestamp)`     |
//! | `attestation_revoked`   | `revoked`    | `(symbol, issuer)`        | `attestation_id`                          |
//! | `attestation_renewed`   | `renewed`    | `(symbol, issuer)`        | `(attestation_id, new_expiration)`        |
//! | `attestation_updated`   | `updated`    | `(symbol, issuer)`        | `(attestation_id, new_expiration)`        |
//! | `attestation_expired`   | `expired`    | `(symbol, subject)`       | `attestation_id`                          |
//! | `issuer_registered`     | `iss_reg`    | `(symbol, issuer)`        | `admin`                                   |
//! | `issuer_removed`        | `iss_rem`    | `(symbol, issuer)`        | `admin`                                   |
//! | `claim_type_registered` | `clmtype`    | `(symbol,)`               | `(claim_type, description)`               |

use soroban_sdk::{symbol_short, Address, Env, String};
use crate::types::Attestation;

pub struct Events;

impl Events {
    /// Emit an event when a new attestation is created.
    ///
    /// # Event schema
    /// ```text
    /// topics: ("created", subject: Address)
    /// data:   (attestation_id: String, issuer: Address, claim_type: String, timestamp: u64, metadata: Option<String>)
    /// ```
    ///
    /// # Parameters
    /// - `attestation` — the newly created attestation.


    pub fn attestation_created(env: &Env, attestation: &Attestation) {
        env.events().publish(
            (symbol_short!("created"), attestation.subject.clone()),
            (
                attestation.id.clone(),
                attestation.issuer.clone(),
                attestation.claim_type.clone(),
                attestation.timestamp,
                attestation.metadata.clone(),
            ),
        );
    }

    pub fn attestation_revoked(env: &Env, attestation_id: &String, issuer: &Address) {
        env.events().publish(
            (symbol_short!("revoked"), issuer.clone()),
            attestation_id.clone(),
        );
    }

    /// Emit event when an attestation is renewed.

    pub fn attestation_renewed(env: &Env, attestation_id: &String, issuer: &Address, new_expiration: Option<u64>) {
        env.events().publish(
            (symbol_short!("renewed"), issuer.clone()),
            (attestation_id.clone(), new_expiration),
        );
    }


    /// Emit an event when an expired attestation is encountered during a check.
    ///
    /// This event is **not** emitted for revoked attestations; revocation takes
    /// precedence over expiration in [`crate::types::Attestation::get_status`].
    ///
    /// # Event schema
    /// ```text
    /// topics: ("expired", subject: Address)
    /// data:   attestation_id: String
    /// ```
    ///
    /// # Parameters
    /// - `attestation_id` — ID of the expired attestation.
    /// - `subject` — address the attestation was issued about.

    pub fn attestation_expired(env: &Env, attestation_id: &String, subject: &Address) {
        env.events().publish(
            (symbol_short!("expired"), subject.clone()),
            attestation_id.clone(),
        );
    }

    pub fn attestation_updated(env: &Env, attestation_id: &String, issuer: &Address, new_expiration: Option<u64>) {
        env.events().publish(
            (symbol_short!("updated"), issuer.clone()),
            (attestation_id.clone(), new_expiration),
        );
    }

    pub fn admin_initialized(env: &Env, admin: &Address, timestamp: u64) {
        env.events().publish(
            (symbol_short!("adm_init"),),
            (admin.clone(), timestamp),
        );
    }

    pub fn issuer_registered(env: &Env, issuer: &Address, admin: &Address) {
        env.events().publish(
            (symbol_short!("iss_reg"), issuer.clone()),
            admin.clone(),
        );
    }

    pub fn issuer_removed(env: &Env, issuer: &Address, admin: &Address) {
        env.events().publish(
            (symbol_short!("iss_rem"), issuer.clone()),
            admin.clone(),
        );
    }

    pub fn claim_type_registered(env: &Env, claim_type: &String, description: &String) {
        env.events().publish(
            (symbol_short!("clmtype"),),
            (claim_type.clone(), description.clone()),
        );
    }

    pub fn contract_upgraded(env: &Env, admin: &Address) {
        env.events().publish(
            (symbol_short!("upgraded"),),
            admin.clone(),
        );
    }
}
