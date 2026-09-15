# web3-suite-payments-contracts

[![Rust](https://img.shields.io/badge/Rust-2021-blue?logo=rust)](https://www.rust-lang.org/)
[![Soroban](https://img.shields.io/badge/Soroban-21.0-7B61FF?logo=stellar)](https://soroban.stellar.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](./LICENSE)
[![Stellar](https://img.shields.io/badge/Network-Stellar-08B5E5?logo=stellar)](https://stellar.org)

Soroban smart contracts powering the payment primitives for the **web3-suite** ecosystem on Stellar. Provides continuous payment streams, on-chain invoicing, and recurring subscription billing — all natively on-chain with minimal trust assumptions.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    web3-suite-payments-contracts                │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌───────────────────┐  ┌───────────────────┐  ┌─────────────┐ │
│  │  Payment Stream   │  │     Invoice       │  │ Subscription│ │
│  │    Contract       │  │    Contract       │  │  Contract   │ │
│  ├───────────────────┤  ├───────────────────┤  ├─────────────┤ │
│  │ • create_stream   │  │ • create_invoice  │  │ • create_pl │ │
│  │ • withdraw        │  │ • send_invoice    │  │ • subscribe │ │
│  │ • pause_stream    │  │ • pay_invoice     │  │ • process_bi│ │
│  │ • resume_stream   │  │ • cancel_invoice  │  │ • cancel_sub│ │
│  │ • stop_stream     │  │ • check_overdue   │  │ • get_plan  │ │
│  │ • get_stream      │  │ • get_invoice     │  │ • list_sub  │ │
│  │ • list_streams    │  │ • list_invoices   │  │             │ │
│  └─────────┬─────────┘  └─────────┬─────────┘  └──────┬──────┘ │
│            │                      │                    │        │
│            └──────────────────────┼────────────────────┘        │
│                                   │                             │
│                          ┌────────▼────────┐                    │
│                          │   Soroban SDK   │                    │
│                          │  (Stellar VM)   │                    │
│                          └─────────────────┘                    │
└─────────────────────────────────────────────────────────────────┘
```

### Data Flow

```
User (Sender)                 Smart Contract              User (Receiver)
     │                              │                           │
     │──── create_stream() ────────►│                           │
     │                              │─── STREAM_CREATED ──────►│
     │                              │                           │
     │                              │◄──── withdraw() ─────────│
     │                              │───── transfer() ────────►│
     │                              │                           │
     │◄─── pause_stream() ─────────│                           │
     │                              │                           │
     │──── resume_stream() ────────►│                           │
     │                              │                           │
     │──── stop_stream() ──────────►│                           │
     │                              │─── remaining funds ─────►│
```

---

## Contracts

### 1. Payment Stream Contract

**Purpose:** Enables continuous, real-time payment streams between two parties. Funds flow at a configurable rate per second, with support for pausing, resuming, and early termination.

**Key Features:**
- Configurable payment rate (tokens per second)
- Pause/resume without losing accumulated state
- Pro-rata withdrawal at any time
- Automatic completion at end time
- Event emission for all state changes

**Storage Model:**
| Field | Type | Description |
|-------|------|-------------|
| `id` | `u64` | Unique stream identifier |
| `sender` | `Address` | Account funding the stream |
| `receiver` | `Address` | Account receiving payments |
| `amount_per_second` | `u128` | Payment rate |
| `start_time` | `u64` | Stream start (unix timestamp) |
| `end_time` | `u64` | Stream end (unix timestamp) |
| `total_streamed` | `u128` | Cumulative amount streamed |
| `withdrawn` | `u128` | Amount already withdrawn |
| `is_active` | `bool` | Whether stream is live |
| `is_paused` | `bool` | Whether stream is paused |
| `pause_time` | `Option<u64>` | When pause started |
| `cumulative_pause_duration` | `u64` | Total time paused |

### 2. Invoice Contract

**Purpose:** On-chain invoice creation, sending, payment, and lifecycle management. Supports line-item invoices with automatic total calculation.

**Key Features:**
- Line-item invoices with descriptions, amounts, and quantities
- Automatic total calculation
- Draft → Sent → Paid/Overdue/Cancelled lifecycle
- Due date enforcement
- Event emission for audit trail

**Storage Model:**
| Field | Type | Description |
|-------|------|-------------|
| `id` | `u64` | Unique invoice identifier |
| `issuer` | `Address` | Account that created the invoice |
| `recipient` | `Address` | Account that should pay |
| `items` | `Vec<InvoiceItem>` | Line items |
| `total_amount` | `u128` | Calculated total |
| `due_date` | `u64` | Payment deadline |
| `status` | `InvoiceStatus` | Current lifecycle state |
| `paid_at` | `Option<u64>` | When payment was made |
| `payment_tx` | `Option<Bytes>` | Reference to payment transaction |

### 3. Subscription Contract

**Purpose:** Recurring subscription billing with configurable plans and billing intervals. Supports merchant-created plans and subscriber management.

**Key Features:**
- Multiple billing intervals (daily, weekly, monthly, quarterly, yearly)
- Plan capacity limits
- Automatic next-billing-date calculation
- Subscriber count tracking
- Pause and cancellation support

**Storage Model:**

*SubscriptionPlan:*
| Field | Type | Description |
|-------|------|-------------|
| `id` | `u64` | Unique plan identifier |
| `name` | `Bytes` | Plan display name |
| `amount` | `u128` | Price per billing cycle |
| `billing_interval` | `BillingInterval` | How often to bill |
| `interval_count` | `u32` | Multiplier for interval |
| `max_subscribers` | `Option<u32>` | Capacity limit |
| `current_subscribers` | `u32` | Active subscriber count |

*Subscription:*
| Field | Type | Description |
|-------|------|-------------|
| `id` | `u64` | Unique subscription identifier |
| `plan_id` | `u64` | Associated plan |
| `subscriber` | `Address` | Paying user |
| `merchant` | `Address` | Plan creator |
| `next_billing_date` | `u64` | Next charge date |
| `total_paid` | `u128` | Cumulative payments |
| `is_active` | `bool` | Subscription status |

---

## API Reference

### Payment Stream Contract

```rust
// Create a new payment stream
fn create_stream(
    env: Env,
    sender: Address,
    receiver: Address,
    amount_per_second: u128,
    start_time: u64,
    end_time: u64,
) -> u64;

// Withdraw available funds
fn withdraw(env: Env, stream_id: u64, amount: Option<u128>) -> u128;

// Pause stream (sender only)
fn pause_stream(env: Env, stream_id: u64);

// Resume paused stream (sender only)
fn resume_stream(env: Env, stream_id: u64);

// Stop stream and return remaining funds
fn stop_stream(env: Env, stream_id: u64);

// Get withdrawable amount
fn get_withdrawable(env: Env, stream_id: u64) -> u128;

// Get stream details
fn get_stream(env: Env, stream_id: u64) -> PaymentStream;

// List streams for an address
fn list_streams(env: Env, address: Address) -> Vec<u64>;
```

### Invoice Contract

```rust
// Create a new invoice
fn create_invoice(
    env: Env,
    issuer: Address,
    recipient: Address,
    items: Vec<InvoiceItem>,
    due_date: u64,
    notes: Option<Bytes>,
) -> u64;

// Send invoice to recipient
fn send_invoice(env: Env, invoice_id: u64);

// Pay an invoice
fn pay_invoice(env: Env, invoice_id: u64);

// Cancel an invoice
fn cancel_invoice(env: Env, invoice_id: u64);

// Check and mark overdue
fn check_overdue(env: Env, invoice_id: u64);

// Calculate total for items
fn calculate_total(env: Env, items: Vec<InvoiceItem>) -> u128;

// Get invoice details
fn get_invoice(env: Env, invoice_id: u64) -> Invoice;

// List invoices for an address
fn list_invoices(env: Env, address: Address) -> Vec<u64>;
```

### Subscription Contract

```rust
// Create a subscription plan
fn create_plan(
    env: Env,
    creator: Address,
    name: Bytes,
    description: Bytes,
    amount: u128,
    billing_interval: BillingInterval,
    interval_count: u32,
    max_subscribers: Option<u32>,
) -> u64;

// Subscribe to a plan
fn subscribe(env: Env, plan_id: u64, subscriber: Address) -> u64;

// Process billing cycle
fn process_billing(env: Env, sub_id: u64) -> u128;

// Cancel subscription
fn cancel_subscription(env: Env, sub_id: u64);

// Get subscription details
fn get_subscription(env: Env, sub_id: u64) -> Subscription;

// Get plan details
fn get_plan(env: Env, plan_id: u64) -> SubscriptionPlan;

// List subscriptions for an address
fn list_subscriptions(env: Env, address: Address) -> Vec<u64>;
```

---

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Soroban CLI](https://soroban.stellar.org/docs/getting-started/installation)
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools)

## Building

```bash
# Clone the repository
git clone https://github.com/sudo-robi/web3-suite-payments-contracts.git
cd web3-suite-payments-contracts

# Build all contracts
cargo build --target wasm32-unknown-unknown --release

# Build individual contracts
cargo build --target wasm32-unknown-unknown --release -p web3-suite-payment-stream
cargo build --target wasm32-unknown-unknown --release -p web3-suite-payment-invoice
cargo build --target wasm32-unknown-unknown --release -p web3-suite-payment-subscription
```

## Testing

```bash
# Run all tests
cargo test

# Run tests for a specific contract
cargo test -p web3-suite-payment-stream
cargo test -p web3-suite-payment-invoice
cargo test -p web3-suite-payment-subscription
```

## Deploying

### Deploy to Testnet

```bash
# Install Soroban CLI
cargo install --locked soroban-cli

# Generate a keypair (if you don't have one)
soroban keys generate my-account

# Fund the account on testnet
soroban keys fund my-account --network testnet

# Deploy the stream contract
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/web3_suite_payment_stream.wasm \
  --source my-account \
  --network testnet

# Deploy the invoice contract
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/web3_suite_payment_invoice.wasm \
  --source my-account \
  --network testnet

# Deploy the subscription contract
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/web3_suite_payment_subscription.wasm \
  --source my-account \
  --network testnet
```

### Initialize Contracts

```bash
# Initialize a stream (replace CONTRACT_ID with deployed address)
soroban contract invoke \
  --id CONTRACT_ID \
  --source my-account \
  --network testnet \
  -- create_stream \
  --sender sender_address \
  --receiver receiver_address \
  --amount-per-second 100 \
  --start-time $(date +%s) \
  --end-time $(($(date +%s) + 86400))
```

---

## Project Structure

```
web3-suite-payments-contracts/
├── Cargo.toml                    # Workspace configuration
├── LICENSE                       # MIT License
├── README.md                     # This file
├── .gitignore
└── contracts/
    ├── stream/
    │   ├── Cargo.toml
    │   ├── src/
    │   │   └── lib.rs           # Payment stream contract
    │   └── tests/
    │       └── test_stream.rs   # Stream contract tests
    ├── invoice/
    │   ├── Cargo.toml
    │   ├── src/
    │   │   └── lib.rs           # Invoice contract
    │   └── tests/
    │       └── test_invoice.rs  # Invoice contract tests
    └── subscription/
        ├── Cargo.toml
        ├── src/
        │   └── lib.rs           # Subscription contract
        └── tests/
            └── test_subscription.rs  # Subscription tests
```

---

## Security Considerations

- All state-changing functions require authorization (`require_auth()`)
- Input validation on all public functions
- Overflow checks enabled in release builds
- Stream math uses `u128` to prevent overflow on large amounts
- Paused streams don't accrue time, preventing double-spend
- Subscription billing is pull-based (merchant calls `process_billing`)

## Contributing

Contributions are welcome! Please follow these steps:

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Commit** your changes (`git commit -m 'Add amazing feature'`)
4. **Push** to the branch (`git push origin feature/amazing-feature`)
5. **Open** a Pull Request

### Development Guidelines

- Write tests for all new functionality
- Follow Rust idioms and `clippy` lints
- Keep functions focused and under 50 lines
- Document public API with doc comments
- Run `cargo fmt` and `cargo clippy` before committing

### Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help newcomers learn
- Credit contributors appropriately

---

## License

This project is licensed under the MIT License — see the [LICENSE](./LICENSE) file for details.

## Acknowledgments

- [Stellar Development Foundation](https://stellar.org/) for the Stellar network
- [Soroban](https://soroban.stellar.org/) for the smart contract platform
- The open-source Rust community
