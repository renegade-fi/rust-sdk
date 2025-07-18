# Wallet Examples

This directory contains examples demonstrating how to use the Renegade SDK for wallet operations. These examples show the core functionality for managing wallets, funds, and orders in the Renegade darkpool.

## Prerequisites

All examples require setting the `PKEY` environment variable with your Ethereum private key:

```bash
export PKEY="your_private_key_here"
```

The examples are configured to work with Arbitrum Sepolia testnet and use predefined token addresses:
- **WETH**: `0xc3414a7ef14aaaa9c4522dfc00a4e66e74e9c25a`
- **USDC**: `0xdf8d259c04020562717557f2b5a3cf28e92707d1`

## Examples

### 1. `generate_wallet.rs`
**Purpose**: Generate a random Ethereum private key and create a Renegade wallet client.

**What it does**:
- Generates a random Ethereum private key
- Creates a Renegade client for Arbitrum Sepolia
- Displays the wallet ID

```bash
cargo run --features darkpool-client,examples --example generate_wallet
```

### 2. `create_wallet.rs`
**Purpose**: Create a new wallet in the Renegade darkpool.

**What it does**:
- Creates a Renegade client
- Registers the wallet in the darkpool

```bash
cargo run --features darkpool-client,examples --example create_wallet
```

### 3. `get_wallet.rs`
**Purpose**: Retrieve and display the current state of your wallet.

**What it does**:
- Fetches wallet state from the relayer
- Displays active orders with details (ID, side, amount, token)
- Shows current token balances

```bash
cargo run --features darkpool-client,examples --example get_wallet
```

### 4. `place_order.rs`
**Purpose**: Place a new trading order in your wallet.

**What it does**:
- Places a sell order for 1 WETH in exchange for USDC
- Demonstrates the order placement flow using `OrderBuilder`
- Shows how to specify order parameters (base/quote tokens, side, amount)

```bash
cargo run --features darkpool-client,examples --example place_order
```

### 5. `cancel_order.rs`
**Purpose**: Cancel an existing order in your wallet.

**What it does**:
- Retrieves current wallet state
- Cancels the first available order
- Handles the case where no orders exist

```bash
cargo run --features darkpool-client,examples --example cancel_order
```

### 6. `deposit.rs`
**Purpose**: Deposit funds from your Ethereum address into your Renegade wallet.

**What it does**:
- Deposits 1 WETH from your Ethereum address into the wallet
- Demonstrates the deposit flow for bringing external funds into the darkpool

```bash
cargo run --features darkpool-client,examples --example deposit
```

### 7. `withdraw.rs`
**Purpose**: Withdraw funds from your wallet back to your Ethereum address.

**What it does**:
- Withdraws 0.5 WETH from the wallet to your Ethereum address
- Demonstrates the withdrawal flow for moving funds out of the darkpool

```bash
cargo run --features darkpool-client,examples --example withdraw
```

## Typical Workflow

A typical workflow for using these examples might be:

1. **Generate or use existing keys**: Use `generate_wallet.rs` as an example of how a wallet's keychain is generated.
2. **Create wallet**: Run `create_wallet.rs` to register in the darkpool
3. **Fund wallet**: Use `deposit.rs` to add tokens to your wallet
4. **Check status**: Use `get_wallet.rs` to verify your balance
5. **Trade**: Use `place_order.rs` to place orders
6. **Manage orders**: Use `cancel_order.rs` to remove unwanted orders
7. **Withdraw**: Use `withdraw.rs` to move funds back to Ethereum

## Token Addresses

The examples use Arbitrum Sepolia testnet addresses. For mainnet usage, you'll need to update the token addresses in each example file. 