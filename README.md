# Smart Contracts - Veridion Identity Verification System

A comprehensive identity verification and reputation system built on the Stellar blockchain using Soroban smart contracts. This system enables users to build verifiable digital identities through various verification methods and accumulate reputation scores.

## 📋 Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Contracts](#contracts)
- [Installation & Setup](#installation--setup)
- [API Reference](#api-reference)
- [Usage Examples](#usage-examples)
- [Security Considerations](#security-considerations)
- [Testing](#testing)
- [Deployment](#deployment)
- [Contributing](#contributing)

## 🎯 Overview

The Veridion Identity Verification System provides a decentralized platform for users to:

- **Register** their digital identity with basic profile information
- **Accumulate verifications** from various sources (age verification, social media, identity providers)
- **Build reputation scores** based on verified credentials
- **Maintain privacy** while proving specific attributes
- **Enable trust** in decentralized applications through verifiable credentials

### Key Features

- ✅ **Multi-source verification** support (Over18, Twitter, GitHub, BrightID, WorldID, Custom)
- ✅ **Reputation scoring** system with configurable point values
- ✅ **Privacy-preserving** design with optional profile information
- ✅ **Event-driven architecture** for easy integration with frontends
- ✅ **Comprehensive testing** with security-focused test cases
- ✅ **Overflow protection** and safe arithmetic operation

## 🏗️ Architecture

The system consists of a single main contract that handles all identity and verification operations:

```
┌─────────────────────────────────────┐
│           StellarPassport           │
├─────────────────────────────────────┤
│ • User Registration                 │
│ • Verification Management           │
│ • Score Calculation                 │
│ • Profile Updates                   │
│ • Event Publishing                  │
└─────────────────────────────────────┘
```

### Data Structures

- **User**: Core user profile with wallet, name, surnames, score, and verification count
- **Verification**: Individual verification with type, points, timestamp, issuer, and status
- **VerificationType**: Enum supporting predefined and custom verification types
- **Status**: Verification status (Approved, Rejected, Pending)

## 📦 Contracts

### StellarPassport Contract

**Location**: `stellar-passport/contracts/stellar-passport/`

The main contract implementing the identity verification system.

**Key Components**:
- `src/lib.rs` - Main contract implementation
- `src/types.rs` - Data structures and enums
- `src/errors.rs` - Custom error definitions
- `src/test.rs` - Comprehensive test suite

## 🚀 Installation & Setup

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- [Stellar CLI](https://soroban.stellar.org/docs/getting-started/install)
- [Soroban SDK](https://soroban.stellar.org/docs/getting-started/hello-world)

### Installation

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd Smart-Contracts-Veridion
   ```

2. **Navigate to the contract directory**:
   ```bash
   cd stellar-passport/contracts/stellar-passport
   ```

3. **Build the contract**:
   ```bash
   make build
   # or
   stellar contract build
   ```

4. **Run tests**:
   ```bash
   make test
   # or
   cargo test
   ```

## 📚 API Reference

### Core Functions

#### `version(env: Env) -> u32`
Returns the contract version.

**Parameters**: None  
**Returns**: `u32` - Contract version number  
**Access**: Public

---

#### `register(env: Env, wallet: Address, name: String, surnames: String)`
Registers a new user in the system.

**Parameters**:
- `wallet: Address` - User's wallet address (must be authenticated)
- `name: String` - User's first name
- `surnames: String` - User's last name(s)

**Returns**: None  
**Access**: Authenticated (wallet must sign)  
**Events**: `UserRegistered(Address)`

**Errors**:
- `AlreadyRegistered` - User is already registered

---

#### `get_score(env: Env, wallet: Address) -> i32`
Retrieves the reputation score for a user.

**Parameters**:
- `wallet: Address` - User's wallet address

**Returns**: `i32` - User's current reputation score  
**Access**: Public

**Errors**:
- `NotRegistered` - User is not registered

---

#### `get_verifications(env: Env, wallet: Address) -> Vec<Verification>`
Retrieves all verifications for a user.

**Parameters**:
- `wallet: Address` - User's wallet address

**Returns**: `Vec<Verification>` - List of user's verifications  
**Access**: Public

**Errors**:
- `NotRegistered` - User is not registered

---

#### `upsert_verification(env: Env, wallet: Address, vtype: VerificationType, points: i32) -> i32`
Adds or updates a verification for a user.

**Parameters**:
- `wallet: Address` - User's wallet address (must be authenticated)
- `vtype: VerificationType` - Type of verification
- `points: i32` - Points to award (must be > 0)

**Returns**: `i32` - User's new total score  
**Access**: Authenticated (wallet must sign)  
**Events**: `VerificationUpserted(Address, VerificationType, i32, i32, i32)`

**Errors**:
- `NotRegistered` - User is not registered
- `InvalidPoints` - Points must be positive
- `TooManyVerifications` - User has reached verification limit (50)
- `Overflow` - Score calculation would overflow

---

#### `update_profile(env: Env, wallet: Address, name: String, surnames: String)`
Updates user's profile information.

**Parameters**:
- `wallet: Address` - User's wallet address (must be authenticated)
- `name: String` - New first name
- `surnames: String` - New last name(s)

**Returns**: None  
**Access**: Authenticated (wallet must sign)

**Errors**:
- `NotRegistered` - User is not registered

### Verification Types

The system supports the following verification types:

- `Over18` - Age verification (18+)
- `Twitter` - Twitter account verification
- `GitHub` - GitHub account verification
- `BrightID` - BrightID verification
- `WorldID` - World ID verification
- `Custom(Symbol)` - Custom verification types

### Events

The contract emits the following events:

- `UserRegistered(Address)` - Emitted when a user registers
- `VerificationUpserted(Address, VerificationType, i32, i32, i32)` - Emitted when a verification is added/updated
  - Parameters: wallet, verification_type, old_points, new_points, total_score

## 💡 Usage Examples

### Basic User Registration

```rust
use soroban_sdk::{Address, String, Symbol};
use stellar_passport::{StellarPassportClient, VerificationType};

// Initialize client
let client = StellarPassportClient::new(&env, &contract_id);

// Register a new user
client.register(
    &user_wallet,
    &String::from_str(&env, "Alice"),
    &String::from_str(&env, "Smith")
);
```

### Adding Verifications

```rust
// Add age verification
let score = client.upsert_verification(
    &user_wallet,
    &VerificationType::Over18,
    &25
);

// Add custom verification
let custom_type = VerificationType::Custom(Symbol::new(&env, "kyc_sumsub"));
let score = client.upsert_verification(
    &user_wallet,
    &custom_type,
    &50
);
```

### Retrieving User Data

```rust
// Get user's reputation score
let score = client.get_score(&user_wallet);

// Get all verifications
let verifications = client.get_verifications(&user_wallet);

// Iterate through verifications
for i in 0..verifications.len() {
    let verification = verifications.get(i).unwrap();
    println!("Type: {:?}, Points: {}, Status: {:?}", 
             verification.vtype, 
             verification.points, 
             verification.status);
}
```

### Updating Profile

```rust
client.update_profile(
    &user_wallet,
    &String::from_str(&env, "Alice Updated"),
    &String::from_str(&env, "Smith Updated")
);
```

## ⚠️ Security Considerations

### Known Security Issues

The current implementation has several security considerations that should be addressed:

#### 1. Self-Issued Verifications
**Issue**: Users can issue verifications to themselves with arbitrary point values.  
**Impact**: Complete undermining of the trust system.  
**Mitigation**: Implement a trusted issuer system where only authorized addresses can issue verifications.

#### 2. Public Score Access
**Issue**: Anyone can query any user's score and verifications without authorization.  
**Impact**: Privacy concerns as all user data is publicly accessible.  
**Mitigation**: Implement access control or use privacy-preserving techniques.

#### 3. Empty String Validation
**Issue**: Users can register and update profiles with empty strings.  
**Impact**: Data integrity issues and potential UI/UX problems.  
**Mitigation**: Add input validation for required fields.

#### 4. Verification Status Management
**Issue**: No mechanism to change verification status once set.  
**Impact**: Verifications cannot be revoked or updated by authorized parties.  
**Mitigation**: Add admin functions for status management.

### Recommended Security Improvements

1. **Implement Trusted Issuers**: Only allow specific addresses to issue verifications
2. **Add Access Control**: Implement role-based access for sensitive operations
3. **Input Validation**: Validate all inputs for format and content
4. **Status Management**: Add functions to approve/reject verifications
5. **Rate Limiting**: Implement limits on verification frequency
6. **Audit Trail**: Enhanced logging for all operations

## 🧪 Testing

The contract includes comprehensive tests covering:

- **End-to-end flows** - Complete user journeys
- **Error handling** - All error conditions
- **Security tests** - Known vulnerabilities and edge cases
- **Overflow protection** - Arithmetic safety
- **Limit enforcement** - Maximum verification limits

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Test Categories

- **Functional Tests**: Core functionality verification
- **Security Tests**: Vulnerability and edge case testing
- **Integration Tests**: End-to-end workflow testing
- **Performance Tests**: Overflow and limit testing

## 🚀 Deployment

### Build for Production

```bash
# Build optimized contract
stellar contract build --release

# Verify build
ls -la target/wasm32v1-none/release/*.wasm
```

### Deploy to Stellar Network

```bash
# Deploy to testnet
stellar contract deploy \
  --wasm target/wasm32v1-none/release/stellar_passport.wasm \
  --source-account YOUR_ACCOUNT \
  --network testnet

# Deploy to mainnet (production)
stellar contract deploy \
  --wasm target/wasm32v1-none/release/stellar_passport.wasm \
  --source-account YOUR_ACCOUNT \
  --network mainnet
```

### Contract Configuration

After deployment, configure the contract with:

1. **Trusted Issuers**: Set up authorized verification issuers
2. **Point Values**: Configure default point values for verification types
3. **Access Controls**: Set up admin roles and permissions

## 🤝 Contributing

We welcome contributions! Please follow these guidelines:

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Commit** your changes (`git commit -m 'Add amazing feature'`)
4. **Push** to the branch (`git push origin feature/amazing-feature`)
5. **Open** a Pull Request

### Development Guidelines

- Follow Rust best practices and conventions
- Add comprehensive tests for new features
- Update documentation for API changes
- Ensure all tests pass before submitting
- Consider security implications of changes

### Code Style

- Use `cargo fmt` for formatting
- Follow existing code patterns
- Add meaningful comments for complex logic
- Use descriptive variable and function names

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 📞 Support

For support and questions:

- **Issues**: Open an issue on GitHub
- **Discussions**: Use GitHub Discussions for questions
- **Documentation**: Check the [Soroban documentation](https://soroban.stellar.org/docs)

## 🔗 Related Links

- [Soroban Documentation](https://soroban.stellar.org/docs)
- [Stellar Network](https://stellar.org/)
- [Rust Documentation](https://doc.rust-lang.org/)
- [Smart Contract Best Practices](https://soroban.stellar.org/docs/basic-tutorials/hello-world)

---

**⚠️ Disclaimer**: This software is provided as-is for educational and development purposes. Please conduct thorough security audits before using in production environments.
