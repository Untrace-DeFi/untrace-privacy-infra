# Untrace Protocol - Quick Start Guide

Get started with the Untrace privacy protocol in minutes.

## Prerequisites

- Rust 1.70+ and Cargo
- Solana CLI tools
- Anchor Framework 0.29+
- Node.js 16+ (for testing)

## Installation

### 1. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Install Solana CLI
```bash
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
```

### 3. Install Anchor
```bash
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest
```

## Building the Project

```bash
# Clone and enter the CODE directory
cd CODE

# Build all modules
cargo build --release

# Run tests
cargo test
```

## Quick Examples

### Example 1: Privacy Pool Deposit & Withdraw

```rust
use untrace_privacy_client::{UntraceClient, PrivacyLevel};
use solana_sdk::{signature::Keypair, pubkey::Pubkey};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize client
    let payer = Keypair::new();
    let program_id = "UnTrAcE1111111111111111111111111111111111111"
        .parse::<Pubkey>()?;

    let client = UntraceClient::new(
        "https://api.devnet.solana.com",
        program_id,
        payer,
    );

    // Create privacy pool
    let pool_id = 1u64;
    client.privacy_pool()
        .initialize_pool(pool_id, 10)
        .await?;

    // Deposit to pool
    let recipient = Pubkey::new_unique();
    let amount = 1_000_000; // 0.001 SOL

    let (signature, commitment, randomness) = client
        .privacy_pool()
        .deposit(pool_id, &recipient, amount)
        .await?;

    println!("Deposited! Signature: {}", signature);
    println!("Commitment: {:?}", commitment);

    // Later: Withdraw from pool
    let secret = randomness.to_vec();
    let withdraw_sig = client
        .privacy_pool()
        .withdraw(pool_id, &commitment, &secret, &recipient)
        .await?;

    println!("Withdrawn! Signature: {}", withdraw_sig);

    Ok(())
}
```

### Example 2: Private Transfer

```rust
use untrace_privacy_client::{UntraceClient, PrivacyLevel};
use solana_sdk::{signature::Keypair, pubkey::Pubkey};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let payer = Keypair::new();
    let program_id = "UnTrAcE1111111111111111111111111111111111111"
        .parse::<Pubkey>()?;

    let client = UntraceClient::new(
        "https://api.devnet.solana.com",
        program_id,
        payer,
    );

    // Send private transfer with maximum privacy
    let recipient = Pubkey::new_unique();
    let amount = 5_000_000; // 0.005 SOL

    let signature = client
        .private_transfer()
        .transfer(&recipient, amount, PrivacyLevel::Maximum)
        .await?;

    println!("Private transfer sent! Signature: {}", signature);

    Ok(())
}
```

### Example 3: Cross-Chain Bridge

```rust
use untrace_privacy_client::UntraceClient;
use untrace_privacy_client::cross_chain::SupportedChain;
use solana_sdk::{signature::Keypair, pubkey::Pubkey};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let payer = Keypair::new();
    let program_id = "UnTrAcE1111111111111111111111111111111111111"
        .parse::<Pubkey>()?;

    let client = UntraceClient::new(
        "https://api.devnet.solana.com",
        program_id,
        payer,
    );

    // Bridge SOL to Ethereum
    let eth_recipient = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb";
    let amount = 10_000_000; // 0.01 SOL

    let signature = client
        .cross_chain()
        .bridge_transfer(
            SupportedChain::Solana,
            SupportedChain::Ethereum,
            eth_recipient,
            amount,
            "SOL",
        )
        .await?;

    println!("Bridge transfer initiated! Signature: {}", signature);

    // Check bridge status
    let bridge_account = Pubkey::new_unique(); // Get from signature
    let status = client
        .cross_chain()
        .get_bridge_status(&bridge_account)
        .await?;

    println!("Bridge status: {:?}", status);

    Ok(())
}
```

### Example 4: UntraceOS Wallet

```rust
use untrace_wallet_sdk::{UntraceWallet, WalletConfig};
use solana_sdk::pubkey::Pubkey;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create wallet with default config
    let config = WalletConfig::default();
    let mut wallet = UntraceWallet::new(config)?;

    // Initialize privacy client
    wallet.init_privacy_client()?;

    println!("Wallet address: {}", wallet.public_key());

    // Send private transaction
    let recipient = Pubkey::new_unique();
    let amount = 2_000_000; // 0.002 SOL

    let signature = wallet
        .send_private_transaction(&recipient, amount, None)
        .await?;

    println!("Transaction sent: {}", signature);

    // Deposit to privacy pool
    let pool_id = 1;
    let (sig, commitment, _) = wallet
        .deposit_to_pool(pool_id, &recipient, amount)
        .await?;

    println!("Deposited to pool: {}", sig);

    // Export wallet (encrypted)
    let password = "my_secure_password";
    let encrypted = wallet.export_encrypted(password)?;
    println!("Encrypted wallet: {}", encrypted);

    Ok(())
}
```

### Example 5: Anti-MEV Protection

```rust
use untrace_anti_mev::{
    AntiMevService, MevProtectionLevel, MevDetector, TransactionEvent, TransactionType
};
use untrace_common::AntiMevConfig;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

fn main() -> anyhow::Result<()> {
    // Configure MEV protection
    let config = AntiMevConfig {
        time_lock_enabled: true,
        min_time_lock: 10, // 10 slots
        batching_enabled: true,
        batch_size: 5,
    };

    let mut service = AntiMevService::new(config);

    // Create instruction to protect
    let instruction = Instruction::new_with_bytes(
        Pubkey::new_unique(),
        &[1, 2, 3],
        vec![],
    );

    // Apply maximum MEV protection
    let protected = service.protect_transaction(
        instruction,
        MevProtectionLevel::Maximum,
    )?;

    println!("Transaction protected: {:?}", protected);

    // MEV Detection
    let mut detector = MevDetector::new(1000);

    let tx = TransactionEvent {
        account: Pubkey::new_unique(),
        amount: 10_000_000,
        timestamp: 100,
        tx_type: TransactionType::Swap,
    };

    let risk_score = detector.calculate_risk_score(&tx);
    println!("MEV risk score: {:.2}", risk_score);

    if risk_score > 0.5 {
        println!("⚠️ High MEV risk detected!");
    }

    Ok(())
}
```

### Example 6: Governance

```rust
use untrace_governance::GovernanceSystem;
use solana_sdk::pubkey::Pubkey;

fn main() -> anyhow::Result<()> {
    // Initialize governance
    let mut gov = GovernanceSystem::new(
        1_000_000_000, // 1B total supply
        86400,         // 24 hour voting period
        100_000_000,   // 100M quorum
    );

    // Create proposal
    let proposer = Pubkey::new_unique();

    // Mint tokens to proposer
    gov.token.mint(proposer, 10_000_000)?;

    let proposal_id = gov.create_proposal(
        proposer,
        "Reduce bridge fee to 0.3%".to_string(),
        0,     // Start now
        86400, // End in 24 hours
    )?;

    println!("Proposal {} created", proposal_id);

    // Vote on proposal
    let voter = Pubkey::new_unique();
    gov.token.mint(voter, 150_000_000)?;

    gov.vote(proposal_id, voter, true)?;
    println!("Voted yes!");

    // Check if proposal passed
    let proposal = gov.get_proposal(proposal_id).unwrap();
    println!("Yes votes: {}", proposal.yes_votes);
    println!("No votes: {}", proposal.no_votes);

    // Execute proposal (after voting period)
    // gov.execute_proposal(proposal_id)?;

    Ok(())
}
```

## Running Tests

```bash
# Test all modules
cargo test

# Test specific module
cargo test -p untrace-privacy-client

# Test with output
cargo test -- --nocapture

# Run specific test
cargo test test_privacy_pool
```

## Deployment to Devnet

### 1. Configure Solana CLI
```bash
solana config set --url devnet
solana-keygen new
solana airdrop 2
```

### 2. Build Program
```bash
cd privacy-program
anchor build
```

### 3. Deploy Program
```bash
anchor deploy
```

### 4. Get Program ID
```bash
solana address -k target/deploy/untrace_privacy_program-keypair.json
```

### 5. Update Client Configuration
```rust
let program_id = "YOUR_DEPLOYED_PROGRAM_ID".parse::<Pubkey>()?;
```

## Configuration

### Wallet Config
```rust
use untrace_wallet_sdk::WalletConfig;
use untrace_common::PrivacyLevel;

let config = WalletConfig {
    default_privacy_level: PrivacyLevel::Enhanced,
    anti_mev_enabled: true,
    rpc_url: "https://api.devnet.solana.com".to_string(),
    program_id: "YOUR_PROGRAM_ID".to_string(),
    auto_mix_enabled: true,
    min_pool_size: 10,
};
```

### Anti-MEV Config
```rust
use untrace_common::AntiMevConfig;

let config = AntiMevConfig {
    time_lock_enabled: true,
    min_time_lock: 10,
    batching_enabled: true,
    batch_size: 5,
};
```

## Common Issues

### Issue: "Program not found"
**Solution:** Deploy the program first or use correct program ID

### Issue: "Insufficient funds"
**Solution:** Airdrop SOL on devnet: `solana airdrop 2`

### Issue: "Account not found"
**Solution:** Initialize accounts before using them

### Issue: "Cargo not found"
**Solution:** Install Rust toolchain

## Next Steps

1. Read [ARCHITECTURE.md](ARCHITECTURE.md) for system design
2. Review [README.md](README.md) for module details
3. Check example code in each module's tests
4. Join community Discord for support
5. Review security best practices

## Resources

- [Solana Cookbook](https://solanacookbook.com)
- [Anchor Book](https://book.anchor-lang.com)
- [Rust Documentation](https://doc.rust-lang.org)
- [Privacy Protocol Research](https://eprint.iacr.org)

## Support

For questions and issues:
- GitHub Issues
- Discord Community
- Twitter: @UntraceProtocol
- Documentation: docs.untrace.org

## License

MIT License - See LICENSE file for details
