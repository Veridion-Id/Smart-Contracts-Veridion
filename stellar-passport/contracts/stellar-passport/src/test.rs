#![cfg(test)]

extern crate alloc;

use alloc::format;
use soroban_sdk::{testutils::Address as _, symbol_short, Address, Env, String, Symbol};

use crate::{StellarPassport, StellarPassportClient};
use crate::types::{VerificationType, Status};

#[test]
fn end_to_end_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StellarPassport);
    let client = StellarPassportClient::new(&env, &contract_id);

    let alice = Address::generate(&env);

    // Registro
    client.register(
        &alice,
        &String::from_str(&env, "Alice"),
        &String::from_str(&env, "Doe"),
    );
    assert_eq!(client.get_score(&alice), 0);

    // Primera verificación
    let s1 = client.upsert_verification(&alice, &VerificationType::Over18, &10);
    assert_eq!(s1, 10);

    // Actualizar misma verificación
    let s2 = client.upsert_verification(&alice, &VerificationType::Over18, &25);
    assert_eq!(s2, 25);

    // Otra verificación
    let s3 =
        client.upsert_verification(&alice, &VerificationType::Custom(symbol_short!("twitter")), &15);
    assert_eq!(s3, 40);

    let verifs = client.get_verifications(&alice);
    assert_eq!(verifs.len(), 2);
}

#[test]
#[should_panic]
fn register_and_duplicate_register_error() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StellarPassport);
    let client = StellarPassportClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    client.register(
        &alice,
        &String::from_str(&env, "Alice"),
        &String::from_str(&env, "Doe"),
    );

    // Second register should panic with AlreadyRegistered
    client.register(
        &alice,
        &String::from_str(&env, "Alice"),
        &String::from_str(&env, "Doe"),
    );
}

#[test]
fn upsert_flow_and_score_math() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StellarPassport);
    let client = StellarPassportClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    client.register(&alice, &String::from_str(&env, "A"), &String::from_str(&env, "B"));

    assert_eq!(client.get_score(&alice), 0);

    // first upsert
    assert_eq!(client.upsert_verification(&alice, &VerificationType::Over18, &5), 5);
    assert_eq!(client.get_score(&alice), 5);

    // update same type
    assert_eq!(client.upsert_verification(&alice, &VerificationType::Over18, &7), 7);
    assert_eq!(client.get_score(&alice), 7);

    // add another type
    assert_eq!(
        client.upsert_verification(&alice, &VerificationType::Custom(symbol_short!("x")), &3),
        10
    );
    assert_eq!(client.get_score(&alice), 10);
}

#[test]
#[should_panic]
fn get_score_not_registered_should_panic() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarPassport);
    let client = StellarPassportClient::new(&env, &contract_id);

    let bob = Address::generate(&env);

    // get_score for not registered should panic
    let _ = client.get_score(&bob);
}

#[test]
#[should_panic]
fn upsert_not_registered_should_panic() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarPassport);
    let client = StellarPassportClient::new(&env, &contract_id);

    let bob = Address::generate(&env);
    let _ = client.upsert_verification(&bob, &VerificationType::Over18, &1);
}

#[test]
fn max_verifications_limit() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarPassport);
    let client = StellarPassportClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    client.register(&alice, &String::from_str(&env, "A"), &String::from_str(&env, "B"));

    // fill up to the limit (assuming 50)
    for i in 0..50 {
        let sym = Symbol::new(&env, &format!("t{}", i));
        let _ = client.upsert_verification(&alice, &VerificationType::Custom(sym), &1);
    }
}

#[test]
#[should_panic]
fn max_verifications_limit_overflow_should_panic() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarPassport);
    let client = StellarPassportClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    client.register(&alice, &String::from_str(&env, "A"), &String::from_str(&env, "B"));

    for i in 0..50 {
        let sym = Symbol::new(&env, &format!("t{}", i));
        let _ = client.upsert_verification(&alice, &VerificationType::Custom(sym), &1);
    }

    // one more must panic
    let sym = Symbol::new(&env, "overflow");
    let _ = client.upsert_verification(&alice, &VerificationType::Custom(sym), &1);
}

// ===== NEW SECURITY TESTS =====

#[test]
fn test_self_issued_verification_flaw() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarPassport);
    let client = StellarPassportClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    client.register(&alice, &String::from_str(&env, "Alice"), &String::from_str(&env, "Doe"));

    // This test demonstrates the critical flaw: users can self-issue verifications
    // with arbitrary points, completely undermining the trust system
    let verifs = client.get_verifications(&alice);
    assert_eq!(verifs.len(), 0);

    // Alice can give herself 1000 points for any verification type
    let score = client.upsert_verification(&alice, &VerificationType::Over18, &1000);
    assert_eq!(score, 1000);

    let verifs = client.get_verifications(&alice);
    assert_eq!(verifs.len(), 1);
    
    // The verification shows Alice as the issuer (the flaw)
    let verif = &verifs.get(0).unwrap();
    assert_eq!(verif.issuer, alice);
    assert_eq!(verif.points, 1000);
    assert_eq!(verif.status, Status::Pending);
}

#[test]
#[should_panic]
fn test_negative_points_should_panic() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarPassport);
    let client = StellarPassportClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    client.register(&alice, &String::from_str(&env, "Alice"), &String::from_str(&env, "Doe"));

    // Negative points should panic
    let _ = client.upsert_verification(&alice, &VerificationType::Over18, &-10);
}

#[test]
#[should_panic]
fn test_zero_points_should_panic() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarPassport);
    let client = StellarPassportClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    client.register(&alice, &String::from_str(&env, "Alice"), &String::from_str(&env, "Doe"));

    // Zero points should panic
    let _ = client.upsert_verification(&alice, &VerificationType::Over18, &0);
}

#[test]
fn test_empty_string_registration_allowed() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarPassport);
    let client = StellarPassportClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    
    // This test shows that empty strings are allowed (potential data integrity issue)
    client.register(&alice, &String::from_str(&env, ""), &String::from_str(&env, ""));
    
    // User is still registered despite empty data
    assert_eq!(client.get_score(&alice), 0);
}

#[test]
fn test_empty_string_profile_update_allowed() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarPassport);
    let client = StellarPassportClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    client.register(&alice, &String::from_str(&env, "Alice"), &String::from_str(&env, "Doe"));
    
    // Update with empty strings (potential data integrity issue)
    client.update_profile(&alice, &String::from_str(&env, ""), &String::from_str(&env, ""));
    
    // Profile update succeeds despite empty data
    assert_eq!(client.get_score(&alice), 0);
}

#[test]
fn test_unauthorized_score_access() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarPassport);
    let client = StellarPassportClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    let _bob = Address::generate(&env);
    
    client.register(&alice, &String::from_str(&env, "Alice"), &String::from_str(&env, "Doe"));
    client.upsert_verification(&alice, &VerificationType::Over18, &100);
    
    // Bob can access Alice's score without authorization (privacy concern)
    assert_eq!(client.get_score(&alice), 100);
    
    // Bob can also access Alice's verifications without authorization
    let verifs = client.get_verifications(&alice);
    assert_eq!(verifs.len(), 1);
}

#[test]
fn test_verification_status_preservation_flaw() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarPassport);
    let client = StellarPassportClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    client.register(&alice, &String::from_str(&env, "Alice"), &String::from_str(&env, "Doe"));

    // Add initial verification
    client.upsert_verification(&alice, &VerificationType::Over18, &10);
    
    let verifs = client.get_verifications(&alice);
    let initial_verif = &verifs.get(0).unwrap();
    assert_eq!(initial_verif.status, Status::Pending);
    
    // Update the same verification - status should be preserved
    client.upsert_verification(&alice, &VerificationType::Over18, &20);
    
    let verifs = client.get_verifications(&alice);
    let updated_verif = &verifs.get(0).unwrap();
    assert_eq!(updated_verif.points, 20);
    assert_eq!(updated_verif.status, Status::Pending); // Status preserved
    
    // This demonstrates the flaw: there's no way to change verification status
    // once it's set, even if the verification is updated
}

#[test]
fn test_score_overflow_protection() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarPassport);
    let client = StellarPassportClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    client.register(&alice, &String::from_str(&env, "Alice"), &String::from_str(&env, "Doe"));

    // Test with large but safe numbers
    let large_score = i32::MAX - 1000;
    client.upsert_verification(&alice, &VerificationType::Over18, &large_score);
    assert_eq!(client.get_score(&alice), large_score);
    
    // Add a small amount that should still be safe
    client.upsert_verification(&alice, &VerificationType::Twitter, &500);
    assert_eq!(client.get_score(&alice), large_score + 500);
}

#[test]
#[should_panic]
fn test_score_overflow_should_panic() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarPassport);
    let client = StellarPassportClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    client.register(&alice, &String::from_str(&env, "Alice"), &String::from_str(&env, "Doe"));

    // Set score close to max
    let near_max = i32::MAX - 100;
    client.upsert_verification(&alice, &VerificationType::Over18, &near_max);
    
    // This should cause overflow
    let _ = client.upsert_verification(&alice, &VerificationType::Twitter, &200);
}

#[test]
fn test_verification_count_tracking() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarPassport);
    let client = StellarPassportClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    client.register(&alice, &String::from_str(&env, "Alice"), &String::from_str(&env, "Doe"));

    // Add multiple different verification types
    for i in 0..5 {
        let sym = Symbol::new(&env, &format!("test{}", i));
        client.upsert_verification(&alice, &VerificationType::Custom(sym), &10);
    }
    
    let verifs = client.get_verifications(&alice);
    assert_eq!(verifs.len(), 5);
    
    // Update an existing verification - count should not increase
    client.upsert_verification(&alice, &VerificationType::Custom(Symbol::new(&env, "test0")), &20);
    
    let verifs = client.get_verifications(&alice);
    assert_eq!(verifs.len(), 5); // Count should remain the same
}

#[test]
fn test_profile_update_preserves_score() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarPassport);
    let client = StellarPassportClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    client.register(&alice, &String::from_str(&env, "Alice"), &String::from_str(&env, "Doe"));
    client.upsert_verification(&alice, &VerificationType::Over18, &50);
    
    assert_eq!(client.get_score(&alice), 50);
    
    // Update profile
    client.update_profile(&alice, &String::from_str(&env, "Alice Updated"), &String::from_str(&env, "Doe Updated"));
    
    // Score should be preserved
    assert_eq!(client.get_score(&alice), 50);
}

#[test]
fn test_verification_timestamp_accuracy() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarPassport);
    let client = StellarPassportClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    client.register(&alice, &String::from_str(&env, "Alice"), &String::from_str(&env, "Doe"));
    
    let before = env.ledger().timestamp();
    client.upsert_verification(&alice, &VerificationType::Over18, &10);
    let after = env.ledger().timestamp();
    
    let verifs = client.get_verifications(&alice);
    let verif = &verifs.get(0).unwrap();
    
    // Timestamp should be within the expected range
    assert!(verif.timestamp >= before);
    assert!(verif.timestamp <= after);
}
