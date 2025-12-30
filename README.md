# Untrace Protocol - Rust/Solana Implementation

A comprehensive privacy-focused blockchain protocol built on Solana with modular architecture for enterprise and developer adoption.

## Overview

This codebase implements the core Untrace privacy protocol with the following features:

- **On-chain and off-chain privacy protection** for transactions and communications
- **Modular architecture** allowing adoption of individual components or the full stack
- **UntraceOS wallet** integration with existing Web3 infrastructures
- **Anti-MEV protection** with time-locks, batching, and private order flow
- **Decentralized governance** with token-based voting and treasury management

## Architecture

The project is organized as a Cargo workspace with the following modules:

```
CODE/
├── common/              # Shared types, crypto utilities, and error handling
├── privacy-program/     # On-chain Solana program (smart contract)
├── privacy-client/      # Off-chain client library for interacting with the protocol
├── wallet-sdk/          # UntraceOS wallet with Web3 adapter support
├── anti-mev/           # MEV protection mechanisms
└── governance/         # Decentralized governance system
```

## Modules

### 1. Common (`untrace-common`)

Shared types and cryptographic utilities used across all modules.

**Features:**
- Privacy levels (Basic, Enhanced, Maximum)
- Encrypted transaction structures
- Pedersen commitments for privacy pools
- Zero-knowledge proof utilities
- Merkle tree verification
- Encryption/decryption helpers

**Key Types:**
- `PrivacyLevel` - Transaction privacy configuration
- `EncryptedTransaction` - Encrypted transaction data
- `PrivateTransfer` - Private transfer instruction
- `CrossChainTransfer` - Cross-chain bridge data
- `PrivacyPool` - Privacy pool state
- `Commitment` - Privacy pool commitment

### 2. Privacy Program (`untrace-privacy-program`)

On-chain Solana program implementing privacy features using Anchor framework.

**Instructions:**
- `initialize_pool` - Create a new privacy pool
- `deposit` - Deposit funds into privacy pool with commitment
- `withdraw` - Withdraw from privacy pool with ZK proof
- `private_transfer` - Execute private transfer with encryption
- `cross_chain_transfer` - Bridge assets to other chains

**Accounts:**
- `PrivacyPoolAccount` - Pool state and merkle root
- `CommitmentAccount` - Stored commitments
- `NullifierAccount` - Spent commitment tracking
- `PrivateTransferAccount` - Encrypted transfer data
- `CrossChainBridgeAccount` - Bridge transfer state

### 3. Privacy Client (`untrace-privacy-client`)

Off-chain Rust client for interacting with the privacy protocol.

**Components:**
- `UntraceClient` - Main client for protocol interaction
- `PrivacyPoolClient` - Privacy pool operations
- `PrivateTransferClient` - Private transfer execution
- `CrossChainClient` - Cross-chain bridge operations

**Example Usage:**
```rust
use untrace_privacy_client::{UntraceClient, PrivacyLevel};

let client = UntraceClient::new(
    "https://api.mainnet-beta.solana.com",
    program_id,
    payer_keypair,
);

// Execute private transfer
let signature = client
    .private_transfer()
    .transfer(&recipient, 1_000_000, PrivacyLevel::Maximum)
    .await?;

// Deposit to privacy pool
let (sig, commitment, randomness) = client
    .privacy_pool()
    .deposit(pool_id, &recipient, 1_000_000)
    .await?;
```

### 4. Wallet SDK (`untrace-wallet-sdk`)

UntraceOS wallet with Web3 integration capabilities.

**Features:**
- Keypair management with secure storage
- Web3 wallet adapter support (Phantom, Solflare, etc.)
- Private transaction execution
- Cross-chain transfers
- Privacy pool interaction
- Encrypted wallet export/import

**Supported Adapters:**
- Phantom
- Solflare
- Generic Web3 wallets

**Example Usage:**
```rust
use untrace_wallet_sdk::{UntraceWallet, WalletConfig};

let config = WalletConfig::default();
let mut wallet = UntraceWallet::new(config)?;

wallet.init_privacy_client()?;

// Send private transaction
let signature = wallet
    .send_private_transaction(&recipient, 1_000_000, None)
    .await?;

// Connect external wallet
wallet.connect_adapter("phantom", Box::new(PhantomAdapter::new()))?;
```

### 5. Anti-MEV (`untrace-anti-mev`)

MEV (Maximal Extractable Value) protection mechanisms.

**Components:**
- `AntiMevService` - Main MEV protection service
- `TimeLockManager` - Transaction time-locking
- `BatchProcessor` - Transaction batching for anonymity
- `PrivateOrderFlow` - Encrypted order submission
- `MevDetector` - Sandwich attack and frontrunning detection

**Protection Levels:**
- **Basic** - Time-lock delays
- **Enhanced** - Time-lock + batching
- **Maximum** - Time-lock + batching + encrypted orders

**Example Usage:**
```rust
use untrace_anti_mev::{AntiMevService, MevProtectionLevel};

let mut service = AntiMevService::new(config);

let protected = service.protect_transaction(
    instruction,
    MevProtectionLevel::Maximum,
)?;
```

### 6. Governance (`untrace-governance`)

Decentralized governance system with token-based voting.

**Components:**
- `GovernanceSystem` - Main governance controller
- `GovernanceToken` (UNT) - Voting token
- `VotingSystem` - Proposal voting mechanism
- `Treasury` - Protocol treasury and fee management

**Features:**
- Proposal creation and voting
- Vote delegation
- Quorum requirements
- Treasury allocations
- Fee configuration
- Revenue tracking

**Example Usage:**
```rust
use untrace_governance::GovernanceSystem;

let mut gov = GovernanceSystem::new(
    1_000_000_000, // Token supply
    86400,         // Voting period
    100_000_000,   // Quorum
);

// Create proposal
let proposal_id = gov.create_proposal(
    proposer,
    "Increase bridge fee to 0.5%",
    start_time,
    end_time,
)?;

// Vote
gov.vote(proposal_id, voter, true)?;

// Execute if passed
gov.execute_proposal(proposal_id)?;
```

## Building

Build all modules:
```bash
cd CODE
cargo build --release
```

Build specific module:
```bash
cargo build -p untrace-privacy-program --release
cargo build -p untrace-wallet-sdk --release
```

## Testing

Run all tests:
```bash
cargo test
```

Run tests for specific module:
```bash
cargo test -p untrace-privacy-client
cargo test -p untrace-anti-mev
```

## Security Features

### Privacy Protection
- Pedersen commitments for amount hiding
- Zero-knowledge proofs for transaction validity
- Encrypted transaction data
- Privacy pools for mixing
- Cross-chain privacy bridges

### MEV Protection
- Time-locked transactions prevent frontrunning
- Transaction batching for anonymity sets
- Private order flow encryption
- Sandwich attack detection
- Risk scoring system

### Cryptography
- Curve25519 for key exchange
- SHA3-256 for hashing
- Blake3 for key derivation
- Ed25519 for signatures
- AES-GCM for encryption (production use)

### Fees
- Transaction Fee: 0.3%
- Bridge Fee: 0.5%
- Privacy Pool Fee: 0.2%

### Voting
- Minimum tokens to propose: 1,000,000 UNT
- Quorum: 100,000,000 UNT
- Voting period: 24 hours (configurable)
- Vote delegation supported

## Deployment

### On-chain Program
```bash
# Build program
anchor build

# Deploy to devnet
anchor deploy --provider.cluster devnet

# Deploy to mainnet
anchor deploy --provider.cluster mainnet
```

### Client Integration
```toml
[dependencies]
untrace-privacy-client = { path = "../CODE/privacy-client" }
untrace-wallet-sdk = { path = "../CODE/wallet-sdk" }
```

## Supported Chains

- Solana (native)
- Ethereum
- BinanceSmartChain
- Polygon
- Avalanche
- Arbitrum
- Optimism

## Contributing

This is a demonstration codebase. For production use:

1. Replace simplified ZK proofs with proper ZK-SNARKs (e.g., Groth16, Plonk)
2. Implement proper AEAD encryption (ChaCha20-Poly1305)
3. Add comprehensive integration tests
4. Conduct security audits
5. Implement proper key management
6. Add monitoring and observability

## License

MIT License

## Resources

- [Solana Documentation](https://docs.solana.com)
- [Anchor Framework](https://www.anchor-lang.com)
- [Zero-Knowledge Proofs](https://z.cash/technology/zksnarks/)
- [MEV Protection](https://docs.flashbots.net)

## Contact

For questions and support, join our community channels.
