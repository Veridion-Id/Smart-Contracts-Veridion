#![cfg(test)]

extern crate alloc;

use alloc::format;
use soroban_sdk::{testutils::Address as _, symbol_short, Address, Env, String, Symbol};

use crate::{StellarPassport, StellarPassportClient};
use crate::types::VerificationType;

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
