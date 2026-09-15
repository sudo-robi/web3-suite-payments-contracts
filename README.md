# web3-suite Payments Contracts

![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)
![Soroban](https://img.shields.io/badge/Soroban-21.0.0-green.svg)
![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)
![Stellar](https://img.shields.io/badge/Stellar-Network-black.svg)

Soroban smart contracts for the web3-suite payment ecosystem on Stellar. Three production-ready contracts enabling continuous payment streams, on-chain invoicing, and recurring subscription billing.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Stellar Network                          │
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │   Payment     │  │   Invoice    │  │   Subscription   │  │
│  │   Stream      │  │   Contract   │  │   Contract       │  │
│  │   Contract    │  │              │  │                  │  │
│  │              │  │  - Create    │  │  - Create Plan   │  │
│  │  - Create    │  │  - Send      │  │  - Subscribe     │  │
│  │  - Withdraw  │  │  - Pay       │  │  - Process Bill  │  │
│  │  - Pause     │  │  - Cancel    │  │  - Cancel        │  │
│  │  - Resume    │  │  - Overdue   │  │  - Next Bill     │  │
│  │  - Stop      │  │              │  │                  │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬─────────┘  │
│         │                 │                    │             │
│         └─────────────────┼────────────────────┘             │
│                           │                                  │
│                    Soroban VM (WASM)                         │
└─────────────────────────────────────────────────────────────┘
                            │
              ┌─────────────┼─────────────┐
              │             │             │
     ┌────────▼──────┐ ┌───▼────┐ ┌──────▼───────┐
     │   Backend     │ │Freighter│ │   Frontend   │
     │   API         │ │ Wallet │ │   React App  │
     │   (Express)   │ │        │ │   (Vite)     │
     └───────────────┘ └────────┘ └──────────────┘
```

## Contracts

### Payment Stream Contract (`contracts/stream`)

Continuous real-time payment streams. Funds flow at a constant rate per second from sender to receiver.

**Key Features:**
- Create streams with configurable rate and duration
- Withdraw available funds at any time
- Pause/resume streams (sender-controlled)
- Stop streams and settle remaining balance
- Accurate withdrawable amount calculation with pause compensation

**API:**

| Function | Auth | Description |
|----------|------|-------------|
| `create_stream(sender, receiver, amount_per_second, start_time, end_time)` | sender | Create a new payment stream |
| `withdraw(stream_id, amount?)` | receiver | Withdraw available funds |
| `pause_stream(stream_id)` | sender | Pause the stream |
| `resume_stream(stream_id)` | sender | Resume a paused stream |
| `stop_stream(stream_id)` | sender | Stop and settle the stream |
| `get_withdrawable(stream_id)` | none | Get withdrawable amount |
| `get_stream(stream_id)` | none | Get stream details |
| `get_status(stream_id)` | none | Get stream status |

### Invoice Contract (`contracts/invoice`)

On-chain invoicing with line items, status tracking, and payment settlement.

**Key Features:**
- Create invoices with multiple line items
- Draft -> Sent -> Paid/Cancelled lifecycle
- Automatic overdue detection
- Total calculation from line items
- Issuer and recipient role enforcement

**API:**

| Function | Auth | Description |
|----------|------|-------------|
| `create_invoice(issuer, recipient, items, due_date, notes?)` | issuer | Create a new invoice |
| `send_invoice(invoice_id)` | issuer | Send invoice to recipient |
| `pay_invoice(invoice_id)` | recipient | Pay the invoice |
| `cancel_invoice(invoice_id)` | issuer | Cancel the invoice |
| `check_overdue(invoice_id)` | none | Mark overdue invoices |
| `get_invoice(invoice_id)` | none | Get invoice details |
| `calculate_total(items)` | none | Calculate total from items |

### Subscription Contract (`contracts/subscription`)

Recurring billing with configurable intervals and subscriber management.

**Key Features:**
- Create plans with daily/weekly/monthly/quarterly/yearly billing
- Optional subscriber caps
- Automated billing cycle management
- Subscription cancellation with plan counter update
- Configurable interval multipliers

**API:**

| Function | Auth | Description |
|----------|------|-------------|
| `create_plan(creator, name, description, amount, interval, count, max?)` | creator | Create a subscription plan |
| `subscribe(plan_id, subscriber)` | subscriber | Subscribe to a plan |
| `process_billing(sub_id)` | none | Process billing cycle |
| `cancel_subscription(sub_id)` | subscriber | Cancel subscription |
| `get_plan(plan_id)` | none | Get plan details |
| `get_subscription(sub_id)` | none | Get subscription details |
| `next_billing_date(current, interval, count)` | none | Calculate next billing date |

## Data Structures

### PaymentStream
```rust
pub struct PaymentStream {
    pub id: u64,
    pub sender: Address,
    pub receiver: Address,
    pub amount_per_second: u128,
    pub start_time: u64,
    pub end_time: u64,
    pub total_streamed: u128,
    pub withdrawn: u128,
    pub is_active: bool,
    pub is_paused: bool,
    pub pause_time: Option<u64>,
    pub cumulative_pause_duration: u64,
    pub created_at: u64,
}
```

### Invoice
```rust
pub struct Invoice {
    pub id: u64,
    pub issuer: Address,
    pub recipient: Address,
    pub items: Vec<InvoiceItem>,
    pub total_amount: u128,
    pub due_date: u64,
    pub issued_date: u64,
    pub status: InvoiceStatus,
    pub notes: Option<Bytes>,
    pub paid_at: Option<u64>,
    pub payment_tx: Option<Bytes>,
}
```

### SubscriptionPlan
```rust
pub struct SubscriptionPlan {
    pub id: u64,
    pub name: Bytes,
    pub description: Bytes,
    pub amount: u128,
    pub billing_interval: BillingInterval,
    pub interval_count: u32,
    pub max_subscribers: Option<u32>,
    pub current_subscribers: u32,
    pub creator: Address,
    pub is_active: bool,
    pub created_at: u64,
}
```

## Build

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Soroban CLI](https://soroban.stellar.org/docs/getting-started/setup)
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools)

### Build All Contracts

```bash
# From workspace root
cargo build --release
```

### Build Individual Contract

```bash
# Stream contract
cargo build --release -p web3-suite-payment-stream

# Invoice contract
cargo build --release -p web3-suite-payment-invoice

# Subscription contract
cargo build --release -p web3-suite-payment-subscription
```

### Run Tests

```bash
# All tests
cargo test

# Specific contract tests
cargo test -p web3-suite-payment-stream
cargo test -p web3-suite-payment-invoice
cargo test -p web3-suite-payment-subscription
```

## Deploy

### 1. Deploy to Testnet

```bash
# Deploy stream contract
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/web3_suite_payment_stream.wasm \
  --source admin \
  --network testnet

# Deploy invoice contract
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/web3_suite_payment_invoice.wasm \
  --source admin \
  --network testnet

# Deploy subscription contract
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/web3_suite_payment_subscription.wasm \
  --source admin \
  --network testnet
```

### 2. Initialize Contracts

```bash
# After deployment, save the contract IDs and configure your backend:
# STREAM_CONTRACT_ID=<contract-address>
# INVOICE_CONTRACT_ID=<contract-address>
# SUBSCRIPTION_CONTRACT_ID=<contract-address>
```

### 3. Fund Admin Account

```bash
# Get testnet tokens
stellar keys fund admin --network testnet
```

## Network Configuration

| Network | Passphrase | RPC URL |
|---------|-----------|---------|
| Testnet | `Test SDF Network ; September 2015` | `https://soroban-testnet.stellar.org` |
| Futurenet | `Test SDF Future Network ; October 2022` | `https://soroban-futurenet.stellar.org` |
| Mainnet | `Public Global Stellar Network ; September 2015` | `https://soroban-mainnet.stellar.org` |

## Security Considerations

- All state-changing functions require authorization from the appropriate party
- Streams enforce `sender != receiver` at creation time
- Invoice payments are only accepted from the designated recipient
- Subscription billing checks `next_billing_date` to prevent premature charges
- Withdrawal amounts are calculated with pause compensation for fair settlement

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Guidelines

- Write tests for all new functionality
- Follow Soroban SDK conventions
- Use `cargo fmt` and `cargo clippy` before committing
- Keep contracts small and focused
- Document public API functions

## License

MIT License - see [LICENSE](../LICENSE) for details.
