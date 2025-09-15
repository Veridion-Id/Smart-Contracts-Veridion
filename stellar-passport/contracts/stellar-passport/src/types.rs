use soroban_sdk::{contracttype, Address, String, Symbol};

/// Tipos de verificación soportados.
/// `Custom(Symbol)` permite extensiones (p.ej. "over18_cr", "kyc_sumsub").
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum VerificationType {
    Over18,
    Twitter,
    GitHub,
    BrightID,
    WorldID,
    Custom(Symbol),
}

/// Una verificación concreta aplicada a un usuario.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Verification {
    pub vtype: VerificationType,
    pub points: i32,
    pub timestamp: u64,     // epoch seconds (host now)
    pub issuer: Address,    // quién la emite (puede ser el propio usuario o un verificador)
}

/// Datos agregados del usuario.
/// `name` / `surnames` son opcionales a nivel de producto (pueden quedar vacíos para privacidad).
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct User {
    pub wallet: Address,
    pub name: String,
    pub surnames: String,
    pub score: i32,
    pub ver_count: u32,
}

/// Claves de almacenamiento del contrato.
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    User(Address),
    Verifications(Address), // Vec<Verification>
}

/// Eventos de negocio (útiles para indexadores y backends).
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum Event {
    UserRegistered(Address),
    VerificationUpserted(Address, VerificationType, i32, i32, i32),
}
