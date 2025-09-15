#![no_std]

pub mod errors;
pub mod types;

use errors::PassportError;
use types::*;

use soroban_sdk::{
    contract, contractimpl, panic_with_error, Address, Env, String, Symbol, Vec,
};

// Constants
const MAX_VERIFICATIONS_PER_USER: u32 = 50;

// Helper functions
fn read_user(env: &Env, wallet: &Address) -> Option<User> {
    let key = DataKey::User(wallet.clone());
    env.storage().instance().get(&key)
}

fn write_user(env: &Env, user: &User) {
    let key = DataKey::User(user.wallet.clone());
    env.storage().instance().set(&key, user);
}

fn read_verifs(env: &Env, wallet: &Address) -> Vec<Verification> {
    let key = DataKey::Verifications(wallet.clone());
    env.storage().instance().get(&key).unwrap_or_else(|| Vec::new(&env))
}

fn write_verifs(env: &Env, wallet: &Address, verifs: &Vec<Verification>) {
    let key = DataKey::Verifications(wallet.clone());
    env.storage().instance().set(&key, verifs);
}

fn safe_add_i32(a: i32, b: i32) -> Result<i32, PassportError> {
    a.checked_add(b).ok_or(PassportError::Overflow)
}

fn safe_sub_i32(a: i32, b: i32) -> Result<i32, PassportError> {
    a.checked_sub(b).ok_or(PassportError::Overflow)
}

#[contract]
pub struct StellarPassport;

#[contractimpl]
impl StellarPassport {
    pub fn version(_env: Env) -> u32 {
        1
    }

    pub fn register(env: Env, wallet: Address, name: String, surnames: String) {
        wallet.require_auth();
        if read_user(&env, &wallet).is_some() {
            panic_with_error!(&env, PassportError::AlreadyRegistered);
        }

        let user = User {
            wallet: wallet.clone(),
            name,
            surnames,
            score: 0,
            ver_count: 0,
        };
        write_user(&env, &user);

        env.events().publish(
            (Symbol::new(&env, "passport"), Symbol::new(&env, "UserRegistered")),
            Event::UserRegistered(wallet),
        );
    }

    pub fn get_score(env: Env, wallet: Address) -> i32 {
        match read_user(&env, &wallet) {
            Some(u) => u.score,
            None => panic_with_error!(&env, PassportError::NotRegistered),
        }
    }

    pub fn get_verifications(env: Env, wallet: Address) -> Vec<Verification> {
        if read_user(&env, &wallet).is_none() {
            panic_with_error!(&env, PassportError::NotRegistered);
        }
        read_verifs(&env, &wallet)
    }

    pub fn upsert_verification(
        env: Env,
        wallet: Address,
        vtype: VerificationType,
        points: i32,
    ) -> i32 {
        wallet.require_auth();
        if points <= 0 {
            panic_with_error!(&env, PassportError::InvalidPoints);
        }

        let mut user = match read_user(&env, &wallet) {
            Some(u) => u,
            None => panic_with_error!(&env, PassportError::NotRegistered),
        };

        let mut verifs = read_verifs(&env, &wallet);
        let now = env.ledger().timestamp();

        let mut old_points = 0i32;
        let mut found_idx: Option<usize> = None;

        for (idx, v) in verifs.iter().enumerate() {
            if v.vtype == vtype {
                old_points = v.points;
                found_idx = Some(idx);
                break;
            }
        }

        let score_minus_old = safe_sub_i32(user.score, old_points).unwrap_or_else(|e| {
            panic_with_error!(&env, e);
        });
        let new_score = safe_add_i32(score_minus_old, points).unwrap_or_else(|e| {
            panic_with_error!(&env, e);
        });

        let issuer = wallet.clone();
        let new_verif = Verification {
            vtype: vtype.clone(),
            points,
            timestamp: now,
            issuer,
        };

        match found_idx {
            Some(i) => verifs.set(i as u32, new_verif),
            None => {
                if user.ver_count >= MAX_VERIFICATIONS_PER_USER {
                    panic_with_error!(&env, PassportError::TooManyVerifications);
                }
                verifs.push_back(new_verif);
                user.ver_count += 1;
            }
        }

        user.score = new_score;
        write_user(&env, &user);
        write_verifs(&env, &wallet, &verifs);

          env.events().publish(
              (Symbol::new(&env, "passport"), Symbol::new(&env, "VerificationUpserted")),
              Event::VerificationUpserted(wallet, vtype, old_points, points, new_score),
          );

        new_score
    }

    pub fn update_profile(env: Env, wallet: Address, name: String, surnames: String) {
        wallet.require_auth();

        let mut user = match read_user(&env, &wallet) {
            Some(u) => u,
            None => panic_with_error!(&env, PassportError::NotRegistered),
        };

        user.name = name;
        user.surnames = surnames;
        write_user(&env, &user);
    }
}

// Include unit tests from `src/test.rs`
#[cfg(test)]
mod test;
