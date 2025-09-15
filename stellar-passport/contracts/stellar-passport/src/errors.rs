use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PassportError {
    AlreadyRegistered = 1,
    NotRegistered = 2,
    Unauthorized = 3,
    InvalidPoints = 4,
    Overflow = 5,
    TooManyVerifications = 6,
}
