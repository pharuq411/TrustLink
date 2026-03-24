#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::{Address as _, Events as _, Ledger}, Address, Env, String};

fn setup(env: &Env) -> (Address, Address, TrustLinkContractClient) {
    let contract_id = env.register_contract(None, TrustLinkContract);
    let client = TrustLinkContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let issuer = Address::generate(env);
    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);
    (admin, issuer, client)
}
fn create_test_contract(env: &Env) -> (Address, TrustLinkContractClient) {
    let contract_id = env.register_contract(None, TrustLinkContract);
    let client = TrustLinkContractClient::new(env, &contract_id);
    (contract_id, client)
}

fn setup_with_id(env: &Env) -> (Address, Address, Address, TrustLinkContractClient) {
    let contract_id = env.register_contract(None, TrustLinkContract);
    let client = TrustLinkContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let issuer = Address::generate(env);
    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);
    (contract_id, admin, issuer, client)
}

// ── Initialization ────────────────────────────────────────────────────────────

#[test]
fn test_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    client.initialize(&admin);
    assert_eq!(client.get_admin(), admin);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_double_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    client.initialize(&admin);
    client.initialize(&admin);
}

// ── Issuer registry ───────────────────────────────────────────────────────────

#[test]
fn test_register_and_check_issuer() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, issuer, client) = setup(&env);
    let _ = admin;
    assert!(client.is_issuer(&issuer));
}

#[test]
fn test_remove_issuer() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, issuer, client) = setup(&env);
    assert!(client.is_issuer(&issuer));
    client.remove_issuer(&admin, &issuer);
    assert!(!client.is_issuer(&issuer));
}

#[test]
fn test_register_issuer_emits_event() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let (contract_id, client) = create_test_contract(&env);
    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);

    let events = env.events().all();
    let (_, topics, data) = events.last().unwrap();
    let topic0: soroban_sdk::Symbol = soroban_sdk::TryFromVal::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    let topic1: Address = soroban_sdk::TryFromVal::try_from_val(&env, &topics.get(1).unwrap()).unwrap();
    let event_data: Address = soroban_sdk::TryFromVal::try_from_val(&env, &data).unwrap();
    assert_eq!(topic0, soroban_sdk::symbol_short!("iss_reg"));
    assert_eq!(topic1, issuer);
    assert_eq!(event_data, admin);
    let _ = contract_id;
}

#[test]
fn test_remove_issuer_emits_event() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let (contract_id, client) = create_test_contract(&env);
    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);
    client.remove_issuer(&admin, &issuer);

    let events = env.events().all();
    let (_, topics, data) = events.last().unwrap();
    let topic0: soroban_sdk::Symbol = soroban_sdk::TryFromVal::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    let topic1: Address = soroban_sdk::TryFromVal::try_from_val(&env, &topics.get(1).unwrap()).unwrap();
    let event_data: Address = soroban_sdk::TryFromVal::try_from_val(&env, &data).unwrap();
    assert_eq!(topic0, soroban_sdk::symbol_short!("iss_rem"));
    assert_eq!(topic1, issuer);
    assert_eq!(event_data, admin);
    let _ = contract_id;
}

#[test]
fn test_register_issuer_error_no_event() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let wrong_admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    client.initialize(&admin);
    let events_before = env.events().all().len();
    let _ = client.try_register_issuer(&wrong_admin, &issuer);
    assert_eq!(env.events().all().len(), events_before);
}

#[test]
fn test_remove_issuer_error_no_event() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, issuer, client) = setup(&env);
    let wrong_admin = Address::generate(&env);
    let events_before = env.events().all().len();
    let _ = client.try_remove_issuer(&wrong_admin, &issuer);
    assert_eq!(env.events().all().len(), events_before);
    let _ = admin;
}

// ── create_attestation ────────────────────────────────────────────────────────

#[test]
fn test_create_attestation() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let claim_type = String::from_str(&env, "KYC_PASSED");
    let id = client.create_attestation(&issuer, &subject, &claim_type, &None, &None);
    let att = client.get_attestation(&id);
    assert_eq!(att.issuer, issuer);
    assert_eq!(att.subject, subject);
    assert_eq!(att.claim_type, claim_type);
    assert!(!att.revoked);
    assert_eq!(att.metadata, None);
}

#[test]
fn test_create_attestation_with_metadata() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let claim_type = String::from_str(&env, "KYC_PASSED");
    let meta = Some(String::from_str(&env, "source=acme,level=2"));
    let id = client.create_attestation(&issuer, &subject, &claim_type, &None, &meta);
    let att = client.get_attestation(&id);
    assert_eq!(att.metadata, meta);
}

#[test]
fn test_create_attestation_metadata_exactly_256_chars() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let claim_type = String::from_str(&env, "KYC_PASSED");
    // Build a 256-byte array of 'a'
    let bytes = [b'a'; 256];
    let meta = Some(String::from_bytes(&env, &bytes));
    // Should succeed
    let id = client.create_attestation(&issuer, &subject, &claim_type, &None, &meta);
    let att = client.get_attestation(&id);
    assert_eq!(att.metadata, meta);
}

#[test]
fn test_create_attestation_metadata_257_chars_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let claim_type = String::from_str(&env, "KYC_PASSED");
    let bytes = [b'a'; 257];
    let meta = Some(String::from_bytes(&env, &bytes));
    let result = client.try_create_attestation(&issuer, &subject, &claim_type, &None, &meta);
    assert_eq!(result, Err(Ok(types::Error::MetadataTooLong)));
}

#[test]
fn test_create_attestation_event_includes_metadata() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (contract_id, client) = create_test_contract(&env);
    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);

    let claim_type = String::from_str(&env, "KYC_PASSED");
    let meta = Some(String::from_str(&env, "ref=123"));
    client.create_attestation(&issuer, &subject, &claim_type, &None, &meta);

    let created_sym = soroban_sdk::symbol_short!("created");
    let found = env.events().all().iter().any(|(id, topics, _)| {
        id == contract_id
            && topics.get(0).map(|v| v.shallow_eq(&created_sym.to_val())).unwrap_or(false)
    });
    assert!(found, "expected a created event to be emitted");
    let _ = contract_id;
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_duplicate_attestation() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let claim_type = String::from_str(&env, "KYC_PASSED");
    env.ledger().with_mut(|li| li.timestamp = 1000);
    client.create_attestation(&issuer, &subject, &claim_type, &None, &None);
    client.create_attestation(&issuer, &subject, &claim_type, &None, &None);
}

// ── has_valid_claim / revoke / expire ─────────────────────────────────────────

#[test]
fn test_has_valid_claim() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let claim_type = String::from_str(&env, "KYC_PASSED");
    client.create_attestation(&issuer, &subject, &claim_type, &None, &None);
    assert!(client.has_valid_claim(&subject, &claim_type));
    assert!(!client.has_valid_claim(&subject, &String::from_str(&env, "OTHER")));
}

#[test]
fn test_revoke_attestation() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let claim_type = String::from_str(&env, "KYC_PASSED");
    let id = client.create_attestation(&issuer, &subject, &claim_type, &None, &None);
    assert!(client.has_valid_claim(&subject, &claim_type));
    client.revoke_attestation(&issuer, &id);
    assert!(!client.has_valid_claim(&subject, &claim_type));
    assert!(client.get_attestation(&id).revoked);
}

#[test]
fn test_expired_attestation() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let claim_type = String::from_str(&env, "KYC_PASSED");
    let current_time = env.ledger().timestamp();
    let id = client.create_attestation(&issuer, &subject, &claim_type, &Some(current_time + 100), &None);
    assert!(client.has_valid_claim(&subject, &claim_type));
    env.ledger().with_mut(|li| li.timestamp = current_time + 200);
    assert!(!client.has_valid_claim(&subject, &claim_type));
    assert_eq!(client.get_attestation_status(&id), types::AttestationStatus::Expired);
}

#[test]
fn test_expired_event_emitted_on_has_valid_claim() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (contract_id, client) = create_test_contract(&env);
    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);
    let claim_type = String::from_str(&env, "KYC_PASSED");
    let current_time = env.ledger().timestamp();
    client.create_attestation(&issuer, &subject, &claim_type, &Some(current_time + 100), &None);
    env.ledger().with_mut(|li| li.timestamp = current_time + 200);
    assert!(!client.has_valid_claim(&subject, &claim_type));
    let expired_sym = soroban_sdk::symbol_short!("expired");
    let found = env.events().all().iter().any(|(id, topics, _)| {
        id == contract_id && topics.get(0).map(|v| v.shallow_eq(&expired_sym.to_val())).unwrap_or(false)
    });
    assert!(found, "expected an expired event to be emitted");
}

#[test]
fn test_expired_event_emitted_on_get_attestation_status() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (contract_id, client) = create_test_contract(&env);
    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);
    let claim_type = String::from_str(&env, "KYC_PASSED");
    let current_time = env.ledger().timestamp();
    let id = client.create_attestation(&issuer, &subject, &claim_type, &Some(current_time + 100), &None);
    let attestation_id = client.create_attestation(
        &issuer, &subject, &claim_type, &Some(current_time + 100), &None,
    );
    env.ledger().with_mut(|li| li.timestamp = current_time + 200);
    assert_eq!(client.get_attestation_status(&id), types::AttestationStatus::Expired);
    let expired_sym = soroban_sdk::symbol_short!("expired");
    let found = env.events().all().iter().any(|(id, topics, _)| {
        id == contract_id && topics.get(0).map(|v| v.shallow_eq(&expired_sym.to_val())).unwrap_or(false)
    });
    assert!(found, "expected an expired event to be emitted");
}

#[test]
fn test_no_expired_event_for_revoked_attestation() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (contract_id, client) = create_test_contract(&env);
    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);
    let claim_type = String::from_str(&env, "KYC_PASSED");
    let current_time = env.ledger().timestamp();
    let id = client.create_attestation(&issuer, &subject, &claim_type, &Some(current_time + 100), &None);
    client.revoke_attestation(&issuer, &id);
    let attestation_id = client.create_attestation(
        &issuer, &subject, &claim_type, &Some(current_time + 100), &None,
    );
    client.revoke_attestation(&issuer, &attestation_id);
    env.ledger().with_mut(|li| li.timestamp = current_time + 200);
    assert_eq!(client.get_attestation_status(&id), types::AttestationStatus::Revoked);
    let expired_sym = soroban_sdk::symbol_short!("expired");
    let found = env.events().all().iter().any(|(id, topics, _)| {
        id == contract_id && topics.get(0).map(|v| v.shallow_eq(&expired_sym.to_val())).unwrap_or(false)
    });
    assert!(!found, "expired event must not be emitted for revoked attestation");
}

// ── Pagination ────────────────────────────────────────────────────────────────

#[test]
fn test_pagination() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    for claim_str in ["CLAIM_0", "CLAIM_1", "CLAIM_2", "CLAIM_3", "CLAIM_4"].iter() {
        client.create_attestation(&issuer, &subject, &String::from_str(&env, claim_str), &None, &None);
    }
    assert_eq!(client.get_subject_attestations(&subject, &0, &2).len(), 2);
    assert_eq!(client.get_subject_attestations(&subject, &2, &2).len(), 2);
    assert_eq!(client.get_subject_attestations(&subject, &4, &2).len(), 1);
}

// ── Issuer / subject attestation counts ──────────────────────────────────────

#[test]
fn test_issuer_attestation_count_zero() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    assert_eq!(client.get_issuer_attestations(&issuer, &0, &100).len(), 0);
}

#[test]
fn test_issuer_attestation_count_after_create() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None);
    client.create_attestation(&issuer, &subject, &String::from_str(&env, "ACCREDITED"), &None, &None);
    assert_eq!(client.get_issuer_attestations(&issuer, &0, &100).len(), 2);
}

#[test]
fn test_issuer_attestation_count_includes_revoked() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let id = client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None);
    client.revoke_attestation(&issuer, &id);
    assert_eq!(client.get_issuer_attestations(&issuer, &0, &100).len(), 1);
}

#[test]
fn test_subject_attestation_count_zero() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, _, client) = setup(&env);
    let subject = Address::generate(&env);
    assert_eq!(client.get_subject_attestations(&subject, &0, &100).len(), 0);
}

#[test]
fn test_subject_attestation_count_after_create() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None);
    assert_eq!(client.get_subject_attestations(&subject, &0, &100).len(), 1);
}

#[test]
fn test_subject_attestation_count_includes_revoked() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let id = client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None);
    client.revoke_attestation(&issuer, &id);
    assert_eq!(client.get_subject_attestations(&subject, &0, &100).len(), 1);
    let (_, client) = create_test_contract(&env);

    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);

    let claim_type = String::from_str(&env, "KYC_PASSED");

    // Create with valid_from = None — backward-compatible path
    let attestation_id =
        client.create_attestation(&issuer, &subject, &claim_type, &None, &None);

    let attestation = client.get_attestation(&attestation_id);
    assert_eq!(attestation.valid_from, None);

    // Status must be Valid (not Pending)
    let status = client.get_attestation_status(&attestation_id);
    assert_eq!(status, types::AttestationStatus::Valid);

    assert!(client.has_valid_claim(&subject, &claim_type));
}

// ── Batch revocation tests ────────────────────────────────────────────────────

fn setup_batch_env(env: &Env) -> (Address, Address, TrustLinkContractClient) {
    let admin = Address::generate(env);
    let issuer = Address::generate(env);
    let (_, client) = create_test_contract(env);
    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);
    (admin, issuer, client)
}

#[test]
fn test_valid_claim_count_zero() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, _, client) = setup(&env);
    let subject = Address::generate(&env);
    assert_eq!(client.get_valid_claims(&subject).len(), 0);
    let id1 = client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None);
    let id2 = client.create_attestation(&issuer, &subject, &String::from_str(&env, "ACCREDITED_INVESTOR"), &None, &None);
    let id3 = client.create_attestation(&issuer, &subject, &String::from_str(&env, "MERCHANT_VERIFIED"), &None, &None);

    let mut ids = soroban_sdk::Vec::new(&env);
    ids.push_back(id1.clone());
    ids.push_back(id2.clone());
    ids.push_back(id3.clone());

    let count = client.revoke_attestations_batch(&issuer, &ids);
    assert_eq!(count, 3);

    assert!(client.get_attestation(&id1).revoked);
    assert!(client.get_attestation(&id2).revoked);
    assert!(client.get_attestation(&id3).revoked);
}

#[test]
fn test_valid_claim_count_after_create() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);

    client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None);
    assert_eq!(client.get_valid_claims(&subject).len(), 1);

    let id1 = client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None);
    let id2 = client.create_attestation(&issuer, &subject, &String::from_str(&env, "ACCREDITED_INVESTOR"), &None, &None);

    let mut ids = soroban_sdk::Vec::new(&env);
    ids.push_back(id1);
    ids.push_back(id2);

    let count = client.revoke_attestations_batch(&issuer, &ids);
    assert_eq!(count, 2);


#[test]
fn test_valid_claim_count_excludes_revoked() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let id = client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None);
    client.revoke_attestation(&issuer, &id);
    assert_eq!(client.get_valid_claims(&subject).len(), 0);


    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);

    let id1 = client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None);
    let id2 = client.create_attestation(&issuer, &subject, &String::from_str(&env, "ACCREDITED_INVESTOR"), &None, &None);

    let mut ids = soroban_sdk::Vec::new(&env);
    ids.push_back(id1);
    ids.push_back(id2);

    client.revoke_attestations_batch(&issuer, &ids);

    let revoked_sym = soroban_sdk::symbol_short!("revoked");
    let revoked_count = env.events().all().iter().filter(|(id, topics, _)| {
        *id == contract_id && topics.get(0).map(|v| v.shallow_eq(&revoked_sym.to_val())).unwrap_or(false)
    }).count();

    assert_eq!(revoked_count, 2, "expected one revoked event per attestation");

}

#[test]
fn test_valid_claim_count_excludes_expired() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let current_time = env.ledger().timestamp();
    client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &Some(current_time + 100), &None);
    env.ledger().with_mut(|li| li.timestamp = current_time + 200);
    assert_eq!(client.get_valid_claims(&subject).len(), 0);
}

// ── get_attestation_by_type ───────────────────────────────────────────────────

#[test]
fn test_get_attestation_by_type_found() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let claim_type = String::from_str(&env, "KYC_PASSED");
    client.create_attestation(&issuer, &subject, &claim_type, &None, &None);
    let att = client.get_attestation_by_type(&subject, &claim_type);
    assert_eq!(att.claim_type, claim_type);
}

#[test]
fn test_get_attestation_by_type_not_found() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, _, client) = setup(&env);
    let subject = Address::generate(&env);
    let result = client.try_get_attestation_by_type(&subject, &String::from_str(&env, "KYC_PASSED"));
    assert_eq!(result, Err(Ok(types::Error::NotFound)));
}

#[test]
fn test_get_attestation_by_type_ignores_revoked() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let claim_type = String::from_str(&env, "KYC_PASSED");
    let id = client.create_attestation(&issuer, &subject, &claim_type, &None, &None);
    client.revoke_attestation(&issuer, &id);
    let result = client.try_get_attestation_by_type(&subject, &claim_type);
    assert_eq!(result, Err(Ok(types::Error::NotFound)));
}

#[test]
fn test_get_attestation_by_type_ignores_expired() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let claim_type = String::from_str(&env, "KYC_PASSED");
    let current_time = env.ledger().timestamp();
    client.create_attestation(&issuer, &subject, &claim_type, &Some(current_time + 100), &None);
    env.ledger().with_mut(|li| li.timestamp = current_time + 200);
    let result = client.try_get_attestation_by_type(&subject, &claim_type);
    assert_eq!(result, Err(Ok(types::Error::NotFound)));
}

#[test]
fn test_get_attestation_by_type_returns_most_recent_valid() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let claim_type = String::from_str(&env, "KYC_PASSED");
    // Create two attestations at different timestamps
    env.ledger().with_mut(|li| li.timestamp = 1000);
    client.create_attestation(&issuer, &subject, &claim_type, &None, &None);
    env.ledger().with_mut(|li| li.timestamp = 2000);
    let id2 = client.create_attestation(&issuer, &subject, &claim_type, &None, &None);
    let att = client.get_attestation_by_type(&subject, &claim_type);
    assert_eq!(att.id, id2);
}

#[test]
fn test_get_attestation_by_type_multiple_claim_types() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let kyc = String::from_str(&env, "KYC_PASSED");
    let acc = String::from_str(&env, "ACCREDITED");
    client.create_attestation(&issuer, &subject, &kyc, &None, &None);
    client.create_attestation(&issuer, &subject, &acc, &None, &None);
    assert_eq!(client.get_attestation_by_type(&subject, &kyc).claim_type, kyc);
    assert_eq!(client.get_attestation_by_type(&subject, &acc).claim_type, acc);
}

// ── Batch revocation ──────────────────────────────────────────────────────────

fn setup_batch_env(env: &Env) -> (Address, Address, TrustLinkContractClient) {
    let (admin, issuer, client) = setup(env);
    (admin, issuer, client)
}

#[test]
fn test_batch_revoke_success() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup_batch_env(&env);
    let subject = Address::generate(&env);
    let id1 = client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None);
    let id2 = client.create_attestation(&issuer, &subject, &String::from_str(&env, "ACCREDITED"), &None, &None);
    let id3 = client.create_attestation(&issuer, &subject, &String::from_str(&env, "MERCHANT"), &None, &None);
    let mut ids = soroban_sdk::Vec::new(&env);
    ids.push_back(id1.clone());
    ids.push_back(id2.clone());
    ids.push_back(id3.clone());
    let count = client.revoke_attestations_batch(&issuer, &ids);
    assert_eq!(count, 3);
    assert!(client.get_attestation(&id1).revoked);
    assert!(client.get_attestation(&id2).revoked);
    assert!(client.get_attestation(&id3).revoked);
}

#[test]
fn test_batch_revoke_returns_count() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup_batch_env(&env);
    let subject = Address::generate(&env);
    let id1 = client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None);
    let id2 = client.create_attestation(&issuer, &subject, &String::from_str(&env, "ACCREDITED"), &None, &None);
    let mut ids = soroban_sdk::Vec::new(&env);
    ids.push_back(id1);
    ids.push_back(id2);
    assert_eq!(client.revoke_attestations_batch(&issuer, &ids), 2);
}

#[test]
fn test_batch_revoke_emits_events_for_each() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (contract_id, client) = create_test_contract(&env);
    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);
    let id1 = client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None);
    let id2 = client.create_attestation(&issuer, &subject, &String::from_str(&env, "ACCREDITED"), &None, &None);
    let mut ids = soroban_sdk::Vec::new(&env);
    ids.push_back(id1);
    ids.push_back(id2);
    client.revoke_attestations_batch(&issuer, &ids);
    let revoked_sym = soroban_sdk::symbol_short!("revoked");
    let count = env.events().all().iter().filter(|(id, topics, _)| {
        *id == contract_id && topics.get(0).map(|v| v.shallow_eq(&revoked_sym.to_val())).unwrap_or(false)
    }).count();
    assert_eq!(count, 2);
    let claim_type = String::from_str(&env, "KYC_PASSED");
    let attestation_id =
        client.create_attestation(&issuer, &subject, &claim_type, &None, &None);

    client.revoke_attestation(&issuer, &attestation_id);

    // Capture event count before the failing renewal
    let events_before = env.events().all().len();

    let new_expiration = Some(env.ledger().timestamp() + 1_000);
    let _ = client.try_renew_attestation(&issuer, &attestation_id, &new_expiration);

    // No new events should have been emitted
    let events_after = env.events().all().len();
    assert_eq!(events_before, events_after);

}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_batch_revoke_unauthorized_issuer_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, issuer, client) = setup_batch_env(&env);
    let other_issuer = Address::generate(&env);
    client.register_issuer(&admin, &other_issuer);
    let subject = Address::generate(&env);

    let id = client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None);

    // issuer creates an attestation
    let id = client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None);

    // other_issuer tries to revoke issuer's attestation — must panic Unauthorized

    let mut ids = soroban_sdk::Vec::new(&env);
    ids.push_back(id);
    client.revoke_attestations_batch(&other_issuer, &ids);
}

#[test]

#[should_panic(expected = "Error(Contract, #5)")]
fn test_batch_revoke_already_revoked_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup_batch_env(&env);
    let subject = Address::generate(&env);
    let id = client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None);
    client.revoke_attestation(&issuer, &id);
    let mut ids = soroban_sdk::Vec::new(&env);
    ids.push_back(id);
    client.revoke_attestations_batch(&issuer, &ids);

#[should_panic(expected = "Error(Contract, #6)")]
fn test_batch_revoke_already_revoked_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, issuer, client) = setup_batch_env(&env);
    let subject = Address::generate(&env);

    let id = client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None);
    client.revoke_attestation(&issuer, &id);

    // Attempting to batch-revoke an already-revoked attestation must panic AlreadyRevoked
    let mut ids = soroban_sdk::Vec::new(&env);
    ids.push_back(id);
    client.revoke_attestations_batch(&issuer, &ids);
}

#[test]
fn test_batch_revoke_single_auth_check() {
    // Verifies the function works end-to-end with mock_all_auths (single auth path).
    // If auth were checked per-attestation the mock would still pass, but this
    // confirms the happy-path with one auth invocation for the whole batch.
    let env = Env::default();
    env.mock_all_auths();

    let (_, issuer, client) = setup_batch_env(&env);
    let subject = Address::generate(&env);

    let mut ids = soroban_sdk::Vec::new(&env);
    for claim in ["C1", "C2", "C3", "C4", "C5"].iter() {
        let id = client.create_attestation(
            &issuer, &subject, &String::from_str(&env, claim), &None, &None,
        );
        ids.push_back(id);
    }

    let count = client.revoke_attestations_batch(&issuer, &ids);
    assert_eq!(count, 5);
}

#[test]
fn test_batch_revoke_empty_vec() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, issuer, client) = setup_batch_env(&env);

    let ids: soroban_sdk::Vec<String> = soroban_sdk::Vec::new(&env);
    let count = client.revoke_attestations_batch(&issuer, &ids);
    assert_eq!(count, 0);
}

// ── Claim type registry tests ─────────────────────────────────────────────────

#[test]
fn test_register_and_get_claim_type() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    client.initialize(&admin);

    let ct = String::from_str(&env, "KYC_PASSED");
    let desc = String::from_str(&env, "Subject has passed KYC verification");
    client.register_claim_type(&admin, &ct, &desc);

    let result = client.get_claim_type_description(&ct);
    assert_eq!(result, Some(desc));
}

#[test]
fn test_get_claim_type_description_unknown_returns_none() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    client.initialize(&admin);

    let result = client.get_claim_type_description(&String::from_str(&env, "UNKNOWN"));
    assert_eq!(result, None);
}

#[test]
fn test_register_claim_type_updates_description() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    client.initialize(&admin);

    let ct = String::from_str(&env, "KYC_PASSED");
    client.register_claim_type(&admin, &ct, &String::from_str(&env, "v1 description"));
    client.register_claim_type(&admin, &ct, &String::from_str(&env, "v2 description"));

    let result = client.get_claim_type_description(&ct);
    assert_eq!(result, Some(String::from_str(&env, "v2 description")));
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_register_claim_type_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let not_admin = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    client.initialize(&admin);

    client.register_claim_type(
        &not_admin,
        &String::from_str(&env, "KYC_PASSED"),
        &String::from_str(&env, "desc"),
    );
}

#[test]
fn test_list_claim_types_pagination() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    client.initialize(&admin);

    let types = [
        ("KYC_PASSED",          "Passed KYC"),
        ("ACCREDITED_INVESTOR", "Accredited investor status"),
        ("MERCHANT_VERIFIED",   "Verified merchant"),
        ("AML_CLEARED",         "AML screening passed"),
        ("SANCTIONS_CHECKED",   "Sanctions list checked"),
    ];

    for (ct, desc) in types.iter() {
        client.register_claim_type(
            &admin,
            &String::from_str(&env, ct),
            &String::from_str(&env, desc),
        );
    }

    let page1 = client.list_claim_types(&0, &2);
    assert_eq!(page1.len(), 2);
    assert_eq!(page1.get(0).unwrap(), String::from_str(&env, "KYC_PASSED"));

    let page2 = client.list_claim_types(&2, &2);
    assert_eq!(page2.len(), 2);

    let page3 = client.list_claim_types(&4, &2);
    assert_eq!(page3.len(), 1);
    assert_eq!(page3.get(0).unwrap(), String::from_str(&env, "SANCTIONS_CHECKED"));
}

#[test]
fn test_batch_revoke_single_auth_check() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup_batch_env(&env);
    let subject = Address::generate(&env);
    let mut ids = soroban_sdk::Vec::new(&env);
    for claim in ["C1", "C2", "C3", "C4", "C5"].iter() {
        let id = client.create_attestation(&issuer, &subject, &String::from_str(&env, claim), &None, &None);
        ids.push_back(id);
    }
    assert_eq!(client.revoke_attestations_batch(&issuer, &ids), 5);
}

#[test]
fn test_batch_revoke_empty_vec() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup_batch_env(&env);
    let ids: soroban_sdk::Vec<String> = soroban_sdk::Vec::new(&env);
    assert_eq!(client.revoke_attestations_batch(&issuer, &ids), 0);
}

// ── update_expiration ─────────────────────────────────────────────────────────

#[test]
fn test_update_expiration_extend() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup_batch_env(&env);
    let subject = Address::generate(&env);
    let current_time = env.ledger().timestamp();

    let id = client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &Some(current_time + 100), &None);

    let id = client.create_attestation(
        &issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &Some(current_time + 100), &None,
    );

    // Extend expiration

    client.update_expiration(&issuer, &id, &Some(current_time + 1000));
    assert_eq!(client.get_attestation(&id).expiration, Some(current_time + 1000));
}

#[test]
fn test_update_expiration_shorten() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup_batch_env(&env);
    let subject = Address::generate(&env);
    let current_time = env.ledger().timestamp();


    let id = client.create_attestation(
        &issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &Some(current_time + 1000), &None,
    );


    client.update_expiration(&issuer, &id, &Some(current_time + 50));
    assert_eq!(client.get_attestation(&id).expiration, Some(current_time + 50));
}

#[test]
fn test_update_expiration_remove() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup_batch_env(&env);
    let subject = Address::generate(&env);
    let current_time = env.ledger().timestamp();

    let id = client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &Some(current_time + 100), &None);

    let id = client.create_attestation(
        &issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &Some(current_time + 100), &None,
    );

    // Remove expiration entirely

    client.update_expiration(&issuer, &id, &None);
    assert_eq!(client.get_attestation(&id).expiration, None);
}

#[test]
fn test_update_expiration_status_reflects_immediately() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup_batch_env(&env);
    let subject = Address::generate(&env);
    let current_time = env.ledger().timestamp();


    let id = client.create_attestation(
        &issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &Some(current_time + 100), &None,
    );

    // Fast-forward past expiration — should be expired

    env.ledger().with_mut(|li| li.timestamp = current_time + 200);
    assert_eq!(client.get_attestation_status(&id), types::AttestationStatus::Expired);
    client.update_expiration(&issuer, &id, &Some(current_time + 500));
    assert_eq!(client.get_attestation_status(&id), types::AttestationStatus::Valid);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_update_expiration_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, issuer, client) = setup_batch_env(&env);
    let other_issuer = Address::generate(&env);
    client.register_issuer(&admin, &other_issuer);
    let subject = Address::generate(&env);

    let id = client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None);

    let id = client.create_attestation(
        &issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None,
    );

    // other_issuer cannot update issuer's attestation

    client.update_expiration(&other_issuer, &id, &Some(9999));
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_update_expiration_revoked_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup_batch_env(&env);
    let subject = Address::generate(&env);

  
    let id = client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None);


    let id = client.create_attestation(
        &issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None,
    );

    client.revoke_attestation(&issuer, &id);
    client.update_expiration(&issuer, &id, &Some(9999));
}

#[test]
fn test_update_expiration_emits_event() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (contract_id, client) = create_test_contract(&env);
    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);

    let id = client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None);


    let id = client.create_attestation(
        &issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None,
    );


    client.update_expiration(&issuer, &id, &Some(5000));
    let updated_sym = soroban_sdk::symbol_short!("updated");
    let found = env.events().all().iter().any(|(cid, topics, _)| {
        cid == contract_id && topics.get(0).map(|v| v.shallow_eq(&updated_sym.to_val())).unwrap_or(false)
    });
    assert!(found, "expected an updated event to be emitted");
}

// ── renew_attestation ─────────────────────────────────────────────────────────

#[test]
fn test_renew_valid_attestation() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let current_time: u64 = 1_000;
    env.ledger().with_mut(|l| l.timestamp = current_time);
    let id = client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &Some(current_time + 500), &None);
    let new_exp = Some(current_time + 2_000);
    client.renew_attestation(&issuer, &id, &new_exp);
    assert_eq!(client.get_attestation(&id).expiration, new_exp);
    assert_eq!(client.get_attestation_status(&id), types::AttestationStatus::Valid);
}

#[test]
fn test_renew_expired_attestation() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let current_time: u64 = 1_000;
    env.ledger().with_mut(|l| l.timestamp = current_time);
    let id = client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &Some(current_time + 100), &None);
    env.ledger().with_mut(|l| l.timestamp = current_time + 200);
    assert_eq!(client.get_attestation_status(&id), types::AttestationStatus::Expired);
    client.renew_attestation(&issuer, &id, &Some(current_time + 5_000));
    assert_eq!(client.get_attestation_status(&id), types::AttestationStatus::Valid);
    assert!(client.has_valid_claim(&subject, &String::from_str(&env, "KYC_PASSED")));
}

#[test]
fn test_renew_with_none_expiration() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let current_time: u64 = 1_000;
    env.ledger().with_mut(|l| l.timestamp = current_time);
    let id = client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &Some(current_time + 500), &None);
    client.renew_attestation(&issuer, &id, &None);
    assert_eq!(client.get_attestation(&id).expiration, None);
    assert_eq!(client.get_attestation_status(&id), types::AttestationStatus::Valid);
}

#[test]
fn test_renew_revoked_attestation_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let id = client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None);
    client.revoke_attestation(&issuer, &id);
    let result = client.try_renew_attestation(&issuer, &id, &Some(env.ledger().timestamp() + 1_000));
    assert_eq!(result, Err(Ok(types::Error::AlreadyRevoked)));
}

#[test]
fn test_renew_wrong_issuer_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, issuer_a, client) = setup(&env);
    let issuer_b = Address::generate(&env);
    client.register_issuer(&admin, &issuer_b);
    let subject = Address::generate(&env);
    let id = client.create_attestation(&issuer_a, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None);
    let result = client.try_renew_attestation(&issuer_b, &id, &Some(env.ledger().timestamp() + 1_000));
    assert_eq!(result, Err(Ok(types::Error::Unauthorized)));
}


#[test]
fn test_renew_unregistered_issuer_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let unregistered = Address::generate(&env);
    let subject = Address::generate(&env);
    let id = client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None);
    let result = client.try_renew_attestation(&unregistered, &id, &Some(env.ledger().timestamp() + 1_000));
    assert_eq!(result, Err(Ok(types::Error::Unauthorized)));
}

#[test]
fn test_renew_missing_attestation_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let result = client.try_renew_attestation(&issuer, &String::from_str(&env, "does-not-exist"), &Some(env.ledger().timestamp() + 1_000));
    assert_eq!(result, Err(Ok(types::Error::NotFound)));
}

#[test]
fn test_renew_past_expiration_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let current_time: u64 = 2_000;
    env.ledger().with_mut(|l| l.timestamp = current_time);
    let id = client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None);
    let result = client.try_renew_attestation(&issuer, &id, &Some(current_time - 1));
    assert_eq!(result, Err(Ok(types::Error::InvalidExpiration)));
}

#[test]
fn test_renew_expiration_equal_current_time_rejected() {

// ── has_any_claim Unit Tests (Task 2.1) ───────────────────────────────────────

#[test]
fn test_has_any_claim_empty_list_returns_false() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (_, client) = create_test_contract(&env);

    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);

    let claim_type = String::from_str(&env, "KYC_PASSED");
    client.create_attestation(&issuer, &subject, &claim_type, &None, &None);

    let empty: soroban_sdk::Vec<String> = soroban_sdk::Vec::new(&env);
    assert!(!client.has_any_claim(&subject, &empty));
}

#[test]
fn test_has_any_claim_single_valid_returns_true() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (_, client) = create_test_contract(&env);

    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);

    let claim_type = String::from_str(&env, "KYC_PASSED");
    client.create_attestation(&issuer, &subject, &claim_type, &None, &None);

    let mut list = soroban_sdk::Vec::new(&env);
    list.push_back(claim_type);
    assert!(client.has_any_claim(&subject, &list));
}

#[test]
fn test_has_any_claim_multiple_types_one_valid_returns_true() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (_, client) = create_test_contract(&env);

    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);

    let kyc = String::from_str(&env, "KYC_PASSED");
    client.create_attestation(&issuer, &subject, &kyc, &None, &None);

    let mut list = soroban_sdk::Vec::new(&env);
    list.push_back(String::from_str(&env, "ACCREDITED"));
    list.push_back(kyc);
    list.push_back(String::from_str(&env, "INVESTOR"));
    assert!(client.has_any_claim(&subject, &list));
}

#[test]
fn test_has_any_claim_multiple_types_none_valid_returns_false() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (_, client) = create_test_contract(&env);

    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);

    let kyc = String::from_str(&env, "KYC_PASSED");
    client.create_attestation(&issuer, &subject, &kyc, &None, &None);

    let mut list = soroban_sdk::Vec::new(&env);
    list.push_back(String::from_str(&env, "ACCREDITED"));
    list.push_back(String::from_str(&env, "INVESTOR"));
    assert!(!client.has_any_claim(&subject, &list));
}

#[test]
fn test_has_any_claim_revoked_returns_false() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (_, client) = create_test_contract(&env);

    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);

    let claim_type = String::from_str(&env, "KYC_PASSED");
    let attestation_id = client.create_attestation(&issuer, &subject, &claim_type, &None, &None);
    client.revoke_attestation(&issuer, &attestation_id);

    let mut list = soroban_sdk::Vec::new(&env);
    list.push_back(claim_type);
    assert!(!client.has_any_claim(&subject, &list));
}

#[test]
fn test_has_any_claim_expired_returns_false() {
   
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let current_time: u64 = 2_000;
    env.ledger().with_mut(|l| l.timestamp = current_time);
    let id = client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None);
    let result = client.try_renew_attestation(&issuer, &id, &Some(current_time));
    assert_eq!(result, Err(Ok(types::Error::InvalidExpiration)));
}


#[test]
fn test_no_event_on_renewal_error() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let id = client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None);
    client.revoke_attestation(&issuer, &id);
    let events_before = env.events().all().len();
    let _ = client.try_renew_attestation(&issuer, &id, &Some(env.ledger().timestamp() + 1_000));
    assert_eq!(env.events().all().len(), events_before);
}

// ── has_any_claim ─────────────────────────────────────────────────────────────

    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (_, client) = create_test_contract(&env);

    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);

    let current_time: u64 = 1_000;
    env.ledger().with_mut(|l| l.timestamp = current_time);

    let claim_type = String::from_str(&env, "KYC_PASSED");
    let expiration = Some(current_time + 100);
    client.create_attestation(&issuer, &subject, &claim_type, &expiration, &None);

    // Advance past expiration
    env.ledger().with_mut(|l| l.timestamp = current_time + 200);

    let mut list = soroban_sdk::Vec::new(&env);
    list.push_back(claim_type);
    assert!(!client.has_any_claim(&subject, &list));
}


#[test]
fn test_has_any_claim_pending_returns_false() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);

    client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None);
    let empty: soroban_sdk::Vec<String> = soroban_sdk::Vec::new(&env);
    assert!(!client.has_any_claim(&subject, &empty));

    let (_, client) = create_test_contract(&env);

    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);

    let current_time: u64 = 1_000;
    env.ledger().with_mut(|l| l.timestamp = current_time);

    let claim_type = String::from_str(&env, "KYC_PASSED");
    let valid_from = Some(current_time + 500);
    client.create_attestation(&issuer, &subject, &claim_type, &None, &valid_from);

    // Still before valid_from
    let mut list = soroban_sdk::Vec::new(&env);
    list.push_back(claim_type);
    assert!(!client.has_any_claim(&subject, &list));

}

#[test]
fn test_has_any_claim_no_attestations_returns_false() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    client.initialize(&admin);

    let subject = Address::generate(&env);
    let mut list = soroban_sdk::Vec::new(&env);
    list.push_back(String::from_str(&env, "KYC_PASSED"));
    assert!(!client.has_any_claim(&subject, &list));
}

#[test]
fn test_has_any_claim_single_element_equivalence_with_has_valid_claim() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let claim_type = String::from_str(&env, "KYC_PASSED");
    client.create_attestation(&issuer, &subject, &claim_type, &None, &None);
    let mut list = soroban_sdk::Vec::new(&env);
    list.push_back(claim_type.clone());

    assert_eq!(
        client.has_any_claim(&subject, &list),
        client.has_valid_claim(&subject, &claim_type)
    );
}

// ── has_all_claims Unit Tests ─────────────────────────────────────────────────

#[test]
fn test_has_all_claims_empty_list_returns_true() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    client.initialize(&admin);

    let subject = Address::generate(&env);
    let empty: soroban_sdk::Vec<String> = soroban_sdk::Vec::new(&env);
    // Vacuous truth: no claims required → always true
    assert!(client.has_all_claims(&subject, &empty));
}

#[test]
fn test_has_all_claims_all_valid_returns_true() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let kyc = String::from_str(&env, "KYC_PASSED");
    let acc = String::from_str(&env, "ACCREDITED_INVESTOR");
    client.create_attestation(&issuer, &subject, &kyc, &None, &None);


    client.create_attestation(&issuer, &subject, &acc, &None, &None);


    let mut list = soroban_sdk::Vec::new(&env);
    list.push_back(kyc);
    list.push_back(acc);

    assert!(client.has_all_claims(&subject, &list));
}

#[test]
fn test_has_all_claims_one_missing_returns_false() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);

    client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None);

    let (_, client) = create_test_contract(&env);

    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);

    let kyc = String::from_str(&env, "KYC_PASSED");
    client.create_attestation(&issuer, &subject, &kyc, &None, &None);
    // ACCREDITED_INVESTOR is never issued


    let mut list = soroban_sdk::Vec::new(&env);
    list.push_back(kyc);
    list.push_back(String::from_str(&env, "ACCREDITED_INVESTOR"));

    assert!(!client.has_all_claims(&subject, &list));
}

#[test]
fn test_has_all_claims_one_revoked_returns_false() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);

    let claim_type = String::from_str(&env, "KYC_PASSED");
    let id = client.create_attestation(&issuer, &subject, &claim_type, &None, &None);
    client.revoke_attestation(&issuer, &id);

    let (_, client) = create_test_contract(&env);

    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);

    let kyc = String::from_str(&env, "KYC_PASSED");
    let acc = String::from_str(&env, "ACCREDITED_INVESTOR");
    let kyc_id = client.create_attestation(&issuer, &subject, &kyc, &None, &None);
    client.create_attestation(&issuer, &subject, &acc, &None, &None);

    // Revoke KYC
    client.revoke_attestation(&issuer, &kyc_id);


    let mut list = soroban_sdk::Vec::new(&env);
    list.push_back(kyc);
    list.push_back(acc);

    assert!(!client.has_all_claims(&subject, &list));
}

#[test]
fn test_has_all_claims_one_expired_returns_false() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let current_time: u64 = 1_000;
    env.ledger().with_mut(|l| l.timestamp = current_time);

    let claim_type = String::from_str(&env, "KYC_PASSED");
    client.create_attestation(&issuer, &subject, &claim_type, &Some(current_time + 100), &None);

    let kyc = String::from_str(&env, "KYC_PASSED");
    let acc = String::from_str(&env, "ACCREDITED_INVESTOR");
    // KYC expires soon, ACCREDITED does not
    client.create_attestation(&issuer, &subject, &kyc, &Some(current_time + 100), &None);
    client.create_attestation(&issuer, &subject, &acc, &None, &None);

    // Advance past KYC expiration

    env.ledger().with_mut(|l| l.timestamp = current_time + 200);
    let mut list = soroban_sdk::Vec::new(&env);
    list.push_back(kyc);
    list.push_back(acc);

    assert!(!client.has_all_claims(&subject, &list));
}

#[test]

fn test_has_any_claim_no_attestations_returns_false() {

fn test_has_all_claims_one_pending_returns_false() {

    let env = Env::default();
    env.mock_all_auths();
    let (_, _, client) = setup(&env);
    let subject = Address::generate(&env);
    let mut list = soroban_sdk::Vec::new(&env);
    list.push_back(String::from_str(&env, "KYC_PASSED"));
    assert!(!client.has_any_claim(&subject, &list));
}

// ── has_all_claims ────────────────────────────────────────────────────────────

#[test]
fn test_has_all_claims_empty_returns_true() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, _, client) = setup(&env);
    let subject = Address::generate(&env);
    let empty: soroban_sdk::Vec<String> = soroban_sdk::Vec::new(&env);
    assert!(client.has_all_claims(&subject, &empty));
}

#[test]
fn test_has_all_claims_all_valid_returns_true() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let kyc = String::from_str(&env, "KYC_PASSED");
    let acc = String::from_str(&env, "ACCREDITED");
    client.create_attestation(&issuer, &subject, &kyc, &None, &None);
    client.create_attestation(&issuer, &subject, &acc, &None, &None);
    let mut list = soroban_sdk::Vec::new(&env);
    list.push_back(kyc);
    list.push_back(acc);
    assert!(client.has_all_claims(&subject, &list));
}

#[test]
fn test_has_all_claims_one_missing_returns_false() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    client.create_attestation(&issuer, &subject, &String::from_str(&env, "KYC_PASSED"), &None, &None);
    let mut list = soroban_sdk::Vec::new(&env);
    list.push_back(String::from_str(&env, "KYC_PASSED"));
    list.push_back(String::from_str(&env, "ACCREDITED"));
    assert!(!client.has_all_claims(&subject, &list));
}

#[test]
fn test_has_all_claims_one_revoked_returns_false() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let kyc = String::from_str(&env, "KYC_PASSED");
    let acc = String::from_str(&env, "ACCREDITED");
    client.create_attestation(&issuer, &subject, &kyc, &None, &None);
    let id2 = client.create_attestation(&issuer, &subject, &acc, &None, &None);
    client.revoke_attestation(&issuer, &id2);
    let mut list = soroban_sdk::Vec::new(&env);
    list.push_back(kyc);
    list.push_back(acc);
    assert!(!client.has_all_claims(&subject, &list));
}

#[test]
fn test_has_all_claims_one_expired_returns_false() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, issuer, client) = setup(&env);
    let subject = Address::generate(&env);
    let current_time: u64 = 1_000;
    env.ledger().with_mut(|l| l.timestamp = current_time);
    let kyc = String::from_str(&env, "KYC_PASSED");
    let acc = String::from_str(&env, "ACCREDITED");
    client.create_attestation(&issuer, &subject, &kyc, &None, &None);
    client.create_attestation(&issuer, &subject, &acc, &Some(current_time + 100), &None);
    env.ledger().with_mut(|l| l.timestamp = current_time + 200);
    let mut list = soroban_sdk::Vec::new(&env);
    list.push_back(kyc);
    list.push_back(acc);
    assert!(!client.has_all_claims(&subject, &list));
}

#[test]
fn test_has_all_claims_no_attestations_returns_false() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, _, client) = setup(&env);
    let subject = Address::generate(&env);
    let mut list = soroban_sdk::Vec::new(&env);
    list.push_back(String::from_str(&env, "KYC_PASSED"));

    let kyc = String::from_str(&env, "KYC_PASSED");
    let acc = String::from_str(&env, "ACCREDITED_INVESTOR");
    // KYC is pending (valid_from in the future)
    client.create_attestation(&issuer, &subject, &kyc, &None, &Some(current_time + 500));
    client.create_attestation(&issuer, &subject, &acc, &None, &None);

    let mut list = soroban_sdk::Vec::new(&env);
    list.push_back(kyc);
    list.push_back(acc);

    assert!(!client.has_all_claims(&subject, &list));
}

// ── Claim type registry ───────────────────────────────────────────────────────

#[test]

fn test_register_and_get_claim_type() {

fn test_has_all_claims_no_attestations_returns_false() {

    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    client.initialize(&admin);
    let ct = String::from_str(&env, "KYC_PASSED");
    let desc = String::from_str(&env, "Subject has passed KYC verification");
    client.register_claim_type(&admin, &ct, &desc);
    assert_eq!(client.get_claim_type_description(&ct), Some(desc));
}

#[test]
fn test_get_claim_type_description_unknown_returns_none() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    client.initialize(&admin);
    assert_eq!(client.get_claim_type_description(&String::from_str(&env, "UNKNOWN")), None);
}


#[test]
fn test_register_claim_type_updates_description() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    client.initialize(&admin);
    let ct = String::from_str(&env, "KYC_PASSED");
    client.register_claim_type(&admin, &ct, &String::from_str(&env, "v1 description"));
    client.register_claim_type(&admin, &ct, &String::from_str(&env, "v2 description"));
    assert_eq!(client.get_claim_type_description(&ct), Some(String::from_str(&env, "v2 description")));
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_register_claim_type_unauthorized() {

    let subject = Address::generate(&env);
    let mut list = soroban_sdk::Vec::new(&env);
    list.push_back(String::from_str(&env, "KYC_PASSED"));

    assert!(!client.has_all_claims(&subject, &list));
}

#[test]
fn test_has_all_claims_single_valid_returns_true() {

    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let not_admin = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    client.initialize(&admin);
    client.register_claim_type(&not_admin, &String::from_str(&env, "KYC_PASSED"), &String::from_str(&env, "desc"));
}

#[test]
fn test_list_claim_types_pagination() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    client.initialize(&admin);
    let claim_types = [
        ("KYC_PASSED", "Passed KYC"),
        ("ACCREDITED_INVESTOR", "Accredited investor status"),
        ("MERCHANT_VERIFIED", "Verified merchant"),
        ("AML_CLEARED", "AML screening passed"),
        ("SANCTIONS_CHECKED", "Sanctions list checked"),
    ];
    for (ct, desc) in claim_types.iter() {
        client.register_claim_type(&admin, &String::from_str(&env, ct), &String::from_str(&env, desc));
    }
    let page1 = client.list_claim_types(&0, &2);
    assert_eq!(page1.len(), 2);
    assert_eq!(page1.get(0).unwrap(), String::from_str(&env, "KYC_PASSED"));
    let page2 = client.list_claim_types(&2, &2);
    assert_eq!(page2.len(), 2);
    let page3 = client.list_claim_types(&4, &2);
    assert_eq!(page3.len(), 1);
    assert_eq!(page3.get(0).unwrap(), String::from_str(&env, "SANCTIONS_CHECKED"));
}

#[test]
fn test_list_claim_types_empty() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    client.initialize(&admin);
    assert_eq!(client.list_claim_types(&0, &10).len(), 0);
}

#[test]
fn test_register_claim_type_emits_event() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let (contract_id, client) = create_test_contract(&env);
    client.initialize(&admin);
    client.register_claim_type(&admin, &String::from_str(&env, "KYC_PASSED"), &String::from_str(&env, "KYC verified"));
    let clmtype_sym = soroban_sdk::symbol_short!("clmtype");
    let found = env.events().all().iter().any(|(id, topics, _)| {
        id == contract_id && topics.get(0).map(|v| v.shallow_eq(&clmtype_sym.to_val())).unwrap_or(false)
    });
    assert!(found, "expected a clmtype event to be emitted");
}

// ── Version / contract metadata ───────────────────────────────────────────────

#[test]
fn test_get_version_after_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    client.initialize(&admin);
    assert_eq!(client.get_version(), String::from_str(&env, "1.0.0"));
}

#[test]
fn test_get_contract_metadata_after_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    client.initialize(&admin);
    let meta = client.get_contract_metadata();
    assert_eq!(meta.name, String::from_str(&env, "TrustLink"));
    assert_eq!(meta.version, String::from_str(&env, "1.0.0"));
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_get_version_before_initialization_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = create_test_contract(&env);
    client.get_version();

    let kyc = String::from_str(&env, "KYC_PASSED");
    client.create_attestation(&issuer, &subject, &kyc, &None, &None);

    let mut list = soroban_sdk::Vec::new(&env);
    list.push_back(kyc.clone());

    // Single-element list must agree with has_valid_claim
    assert_eq!(
        client.has_all_claims(&subject, &list),
        client.has_valid_claim(&subject, &kyc)
    );
}

#[test]
fn test_has_all_claims_short_circuits_on_first_missing() {
    // Verifies short-circuit: put the missing claim first so the function
    // would have to scan further to find the valid ones if it didn't stop early.
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (_, client) = create_test_contract(&env);

    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);

    let kyc = String::from_str(&env, "KYC_PASSED");
    let acc = String::from_str(&env, "ACCREDITED_INVESTOR");
    let mer = String::from_str(&env, "MERCHANT_VERIFIED");
    // Only issue the last two; the first is missing
    client.create_attestation(&issuer, &subject, &acc, &None, &None);
    client.create_attestation(&issuer, &subject, &mer, &None, &None);

    let mut list = soroban_sdk::Vec::new(&env);
    list.push_back(kyc);   // missing — should short-circuit here
    list.push_back(acc);
    list.push_back(mer);

    assert!(!client.has_all_claims(&subject, &list));

// ── Batch attestation creation tests ─────────────────────────────────────────

#[test]
fn test_create_attestations_batch_success() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, issuer, client) = setup_batch_env(&env);
    let claim_type = String::from_str(&env, "KYC_PASSED");

    let mut subjects: soroban_sdk::Vec<Address> = soroban_sdk::Vec::new(&env);
    for _ in 0..10 {
        subjects.push_back(Address::generate(&env));
    }

    let ids = client.create_attestations_batch(&issuer, &subjects, &claim_type, &None);

    assert_eq!(ids.len(), 10);

    // Verify each attestation was stored correctly and in order
    for (i, id) in ids.iter().enumerate() {
        let attestation = client.get_attestation(&id);
        assert_eq!(attestation.issuer, issuer);
        assert_eq!(attestation.subject, subjects.get(i as u32).unwrap());
        assert_eq!(attestation.claim_type, claim_type);
        assert!(!attestation.revoked);
    }
}

#[test]
fn test_create_attestations_batch_single_auth_check() {
    // Confirms the happy-path works with a single mock_all_auths call,
    // meaning auth is not checked per-subject.
    let env = Env::default();
    env.mock_all_auths();

    let (_, issuer, client) = setup_batch_env(&env);
    let claim_type = String::from_str(&env, "KYC_PASSED");

    let mut subjects: soroban_sdk::Vec<Address> = soroban_sdk::Vec::new(&env);
    subjects.push_back(Address::generate(&env));
    subjects.push_back(Address::generate(&env));
    subjects.push_back(Address::generate(&env));

    let ids = client.create_attestations_batch(&issuer, &subjects, &claim_type, &None);
    assert_eq!(ids.len(), 3);
}

#[test]
fn test_create_attestations_batch_returns_ids_in_order() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, issuer, client) = setup_batch_env(&env);
    let claim_type = String::from_str(&env, "KYC_PASSED");

    let subject_a = Address::generate(&env);
    let subject_b = Address::generate(&env);
    let subject_c = Address::generate(&env);

    let mut subjects: soroban_sdk::Vec<Address> = soroban_sdk::Vec::new(&env);
    subjects.push_back(subject_a.clone());
    subjects.push_back(subject_b.clone());
    subjects.push_back(subject_c.clone());

    let ids = client.create_attestations_batch(&issuer, &subjects, &claim_type, &None);

    // Each ID must correspond to the subject at the same index
    assert_eq!(client.get_attestation(&ids.get(0).unwrap()).subject, subject_a);
    assert_eq!(client.get_attestation(&ids.get(1).unwrap()).subject, subject_b);
    assert_eq!(client.get_attestation(&ids.get(2).unwrap()).subject, subject_c);
}

#[test]
fn test_create_attestations_batch_emits_event_per_subject() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, client) = create_test_contract(&env);
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);

    let claim_type = String::from_str(&env, "KYC_PASSED");
    let mut subjects: soroban_sdk::Vec<Address> = soroban_sdk::Vec::new(&env);
    for _ in 0..5 {
        subjects.push_back(Address::generate(&env));
    }

    client.create_attestations_batch(&issuer, &subjects, &claim_type, &None);

    let created_sym = soroban_sdk::symbol_short!("created");
    let created_count = env.events().all().iter().filter(|(id, topics, _)| {
        *id == contract_id
            && topics.get(0).map(|v| v.shallow_eq(&created_sym.to_val())).unwrap_or(false)
    }).count();

    assert_eq!(created_count, 5, "expected one created event per subject");
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_create_attestations_batch_unauthorized_issuer_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, _, client) = setup_batch_env(&env);
    let unregistered = Address::generate(&env);
    let claim_type = String::from_str(&env, "KYC_PASSED");

    let mut subjects: soroban_sdk::Vec<Address> = soroban_sdk::Vec::new(&env);
    subjects.push_back(Address::generate(&env));

    // Unregistered issuer must panic with Unauthorized
    client.create_attestations_batch(&unregistered, &subjects, &claim_type, &None);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_create_attestations_batch_duplicate_causes_full_failure() {
    // A duplicate subject in the batch (same issuer/subject/claim_type/timestamp)
    // must cause the entire batch to fail with DuplicateAttestation.
    let env = Env::default();
    env.mock_all_auths();

    let (_, issuer, client) = setup_batch_env(&env);
    let claim_type = String::from_str(&env, "KYC_PASSED");

    // Pre-create an attestation for subject_a
    let subject_a = Address::generate(&env);
    client.create_attestation(&issuer, &subject_a, &claim_type, &None, &None);

    // Batch includes subject_a again — same timestamp means duplicate ID
    let subject_b = Address::generate(&env);
    let mut subjects: soroban_sdk::Vec<Address> = soroban_sdk::Vec::new(&env);
    subjects.push_back(subject_b);
    subjects.push_back(subject_a); // duplicate

    client.create_attestations_batch(&issuer, &subjects, &claim_type, &None);
}

#[test]
fn test_create_attestations_batch_empty_vec() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, issuer, client) = setup_batch_env(&env);
    let claim_type = String::from_str(&env, "KYC_PASSED");

    let subjects: soroban_sdk::Vec<Address> = soroban_sdk::Vec::new(&env);
    let ids = client.create_attestations_batch(&issuer, &subjects, &claim_type, &None);

    assert_eq!(ids.len(), 0);
}

#[test]
fn test_create_attestations_batch_with_expiration() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, issuer, client) = setup_batch_env(&env);
    let claim_type = String::from_str(&env, "KYC_PASSED");
    let expiration = Some(env.ledger().timestamp() + 1_000);

    let mut subjects: soroban_sdk::Vec<Address> = soroban_sdk::Vec::new(&env);
    subjects.push_back(Address::generate(&env));
    subjects.push_back(Address::generate(&env));

    let ids = client.create_attestations_batch(&issuer, &subjects, &claim_type, &expiration);

    for id in ids.iter() {
        let attestation = client.get_attestation(&id);
        assert_eq!(attestation.expiration, expiration);
        assert_eq!(client.get_attestation_status(&id), types::AttestationStatus::Valid);
    }
}

#[test]
fn test_create_attestations_batch_updates_subject_and_issuer_indexes() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, issuer, client) = setup_batch_env(&env);
    let claim_type = String::from_str(&env, "KYC_PASSED");

    let subject = Address::generate(&env);
    let mut subjects: soroban_sdk::Vec<Address> = soroban_sdk::Vec::new(&env);
    subjects.push_back(subject.clone());

    client.create_attestations_batch(&issuer, &subjects, &claim_type, &None);

    // Subject index should have one entry
    let subject_attestations = client.get_subject_attestations(&subject, &0, &10);
    assert_eq!(subject_attestations.len(), 1);

    // Issuer index should have one entry
    let issuer_attestations = client.get_issuer_attestations(&issuer, &0, &10);
    assert_eq!(issuer_attestations.len(), 1);

}
