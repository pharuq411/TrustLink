#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env, String};

fn create_test_contract(env: &Env) -> (Address, TrustLinkContractClient) {
    let contract_id = env.register_contract(None, TrustLinkContract);
    let client = TrustLinkContractClient::new(env, &contract_id);
    (contract_id, client)
}

#[test]
fn test_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    
    client.initialize(&admin);
    
    let stored_admin = client.get_admin();
    assert_eq!(stored_admin, admin);
}

#[test]
#[should_panic(expected = "AlreadyInitialized")]
fn test_double_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    
    client.initialize(&admin);
    client.initialize(&admin); // Should panic
}

#[test]
fn test_register_and_check_issuer() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    
    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);
    
    assert!(client.is_issuer(&issuer));
}

#[test]
fn test_remove_issuer() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    
    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);
    assert!(client.is_issuer(&issuer));
    
    client.remove_issuer(&admin, &issuer);
    assert!(!client.is_issuer(&issuer));
}

#[test]
fn test_create_attestation() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    
    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);
    
    let claim_type = String::from_str(&env, "KYC_PASSED");
    let attestation_id = client.create_attestation(&issuer, &subject, &claim_type, &None);
    
    let attestation = client.get_attestation(&attestation_id);
    assert_eq!(attestation.issuer, issuer);
    assert_eq!(attestation.subject, subject);
    assert_eq!(attestation.claim_type, claim_type);
    assert!(!attestation.revoked);
}

#[test]
fn test_has_valid_claim() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    
    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);
    
    let claim_type = String::from_str(&env, "KYC_PASSED");
    client.create_attestation(&issuer, &subject, &claim_type, &None);
    
    assert!(client.has_valid_claim(&subject, &claim_type));
    
    let other_claim = String::from_str(&env, "ACCREDITED");
    assert!(!client.has_valid_claim(&subject, &other_claim));
}

#[test]
fn test_revoke_attestation() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    
    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);
    
    let claim_type = String::from_str(&env, "KYC_PASSED");
    let attestation_id = client.create_attestation(&issuer, &subject, &claim_type, &None);
    
    assert!(client.has_valid_claim(&subject, &claim_type));
    
    client.revoke_attestation(&issuer, &attestation_id);
    
    assert!(!client.has_valid_claim(&subject, &claim_type));
    
    let attestation = client.get_attestation(&attestation_id);
    assert!(attestation.revoked);
}

#[test]
fn test_expired_attestation() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    
    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);
    
    let claim_type = String::from_str(&env, "KYC_PASSED");
    let current_time = env.ledger().timestamp();
    let expiration = Some(current_time + 100);
    
    let attestation_id = client.create_attestation(&issuer, &subject, &claim_type, &expiration);
    
    // Should be valid initially
    assert!(client.has_valid_claim(&subject, &claim_type));
    
    // Fast forward time past expiration
    env.ledger().with_mut(|li| {
        li.timestamp = current_time + 200;
    });
    
    // Should now be invalid
    assert!(!client.has_valid_claim(&subject, &claim_type));
    
    let status = client.get_attestation_status(&attestation_id);
    assert_eq!(status, types::AttestationStatus::Expired);
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
    client.create_attestation(&issuer, &subject, &claim_type, &Some(current_time + 100));

    env.ledger().with_mut(|li| li.timestamp = current_time + 200);
    assert!(!client.has_valid_claim(&subject, &claim_type));

    // Verify at least one "expired" event was emitted by this contract
    let expired_sym = soroban_sdk::symbol_short!("expired");
    let found = env.events().all().iter().any(|(id, topics, _)| {
        id == contract_id && topics.get(0) == Some(expired_sym.into())
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
    let attestation_id = client.create_attestation(
        &issuer, &subject, &claim_type, &Some(current_time + 100),
    );

    env.ledger().with_mut(|li| li.timestamp = current_time + 200);

    let status = client.get_attestation_status(&attestation_id);
    assert_eq!(status, types::AttestationStatus::Expired);

    let expired_sym = soroban_sdk::symbol_short!("expired");
    let found = env.events().all().iter().any(|(id, topics, _)| {
        id == contract_id && topics.get(0) == Some(expired_sym.into())
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
    let attestation_id = client.create_attestation(
        &issuer, &subject, &claim_type, &Some(current_time + 100),
    );
    client.revoke_attestation(&issuer, &attestation_id);

    env.ledger().with_mut(|li| li.timestamp = current_time + 200);

    // Revoked takes precedence — status is Revoked, not Expired
    let status = client.get_attestation_status(&attestation_id);
    assert_eq!(status, types::AttestationStatus::Revoked);

    // No expired event should have been emitted
    let expired_sym = soroban_sdk::symbol_short!("expired");
    let found = env.events().all().iter().any(|(id, topics, _)| {
        id == contract_id && topics.get(0) == Some(expired_sym.into())
    });
    assert!(!found, "expired event must not be emitted for revoked attestation");
}
        })
        .collect();
    assert!(expired_events.is_empty());
}

#[test]
#[should_panic(expected = "DuplicateAttestation")]
fn test_duplicate_attestation() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    
    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);
    
    let claim_type = String::from_str(&env, "KYC_PASSED");
    
    // Mock the timestamp to be consistent
    env.ledger().with_mut(|li| {
        li.timestamp = 1000;
    });
    
    client.create_attestation(&issuer, &subject, &claim_type, &None);
    client.create_attestation(&issuer, &subject, &claim_type, &None); // Should panic
}

#[test]
fn test_pagination() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    
    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);
    
    // Create multiple attestations
    let claims = ["CLAIM_0", "CLAIM_1", "CLAIM_2", "CLAIM_3", "CLAIM_4"];
    for claim_str in claims.iter() {
        let claim = String::from_str(&env, claim_str);
        client.create_attestation(&issuer, &subject, &claim, &None);
    }
    
    let page1 = client.get_subject_attestations(&subject, &0, &2);
    assert_eq!(page1.len(), 2);
    
    let page2 = client.get_subject_attestations(&subject, &2, &2);
    assert_eq!(page2.len(), 2);
    
    let page3 = client.get_subject_attestations(&subject, &4, &2);
    assert_eq!(page3.len(), 1);
}
