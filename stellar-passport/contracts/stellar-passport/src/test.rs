#![cfg(test)]

use soroban_sdk::{testutils::Address as _, symbol_short, Address, Env, String};

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
