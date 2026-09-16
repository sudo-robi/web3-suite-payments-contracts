# web3-suite-payments-contracts

> Soroban smart contracts for payment streams, invoices, and subscriptions on Stellar

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Issues](https://img.shields.io/github/issues/sudo-robi/web3-suite-payments-contracts)](https://github.com/sudo-robi/web3-suite-payments-contracts/issues)
[![Stars](https://img.shields.io/github/stars/sudo-robi/web3-suite-payments-contracts)](https://github.com/sudo-robi/web3-suite-payments-contracts/stargazers)

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Features](#features)
- [Tech Stack](#tech-stack)
- [Project Structure](#project-structure)
- [Smart Contracts](#smart-contracts)
  - [Payment Stream Contract](#payment-stream-contract)
  - [Invoice Contract](#invoice-contract)
  - [Subscription Contract](#subscription-contract)
- [Getting Started](#getting-started)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Building](#building)
- [Testing](#testing)
- [Deployment](#deployment)
- [Contributing](#contributing)
- [License](#license)

## Overview

Traditional payment systems are limited to discrete, one-time transactions. In the modern creator economy and SaaS landscape, businesses need continuous, programmatic payment primitives that mirror the real-time nature of value exchange. Existing solutions either require custodial intermediaries or lack the composability that on-chain contracts provide.

**web3-suite-payments-contracts** solves this by delivering three production-grade Soroban smart contracts on the Stellar network:

- **Payment Streams** — Continuous real-time payments (salary, subscriptions, escrow-by-the-second)
- **Invoices** — On-chain invoice lifecycle management with line items and status tracking
- **Subscriptions** — Recurring billing with configurable intervals, capacity limits, and plan management

### Target Audience

- **Freelancers & Creators** who need continuous payment streams instead of monthly invoices
- **SaaS Businesses** looking for transparent, on-chain recurring billing
- **DAOs & Web3 Teams** that want programmatic treasury disbursements
- **Developers** building payment-enabled dApps on Stellar/Soroban

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Stellar Network                       │
│  ┌───────────────┐ ┌───────────────┐ ┌───────────────┐  │
│  │    Payment     │ │    Invoice     │ │  Subscription │  │
│  │    Stream      │ │    Contract    │ │   Contract    │  │
│  │   Contract     │ │                │ │               │  │
│  └───────┬───────┘ └───────┬───────┘ └───────┬───────┘  │
│          │                 │                 │           │
│          └────────┬────────┘────────┬────────┘           │
│                   │                 │                    │
│          ┌────────▼─────────────────▼────────┐          │
│          │     Soroban Persistent Storage     │          │
│          │   (Contract State + Ledger Keys)   │          │
│          └───────────────────────────────────┘          │
│                                                         │
│          ┌───────────────────────────────────┐          │
│          │       Event Emissions              │          │
│          │  (STR_CREATED, INV_PAID, etc.)     │          │
│          └───────────────────────────────────┘          │
└─────────────────────────────────────────────────────────┘
                          ▲
                          │  Contract Calls
                          │
┌─────────────────────────┼───────────────────────────────┐
│                    Backend API                           │
│              (Stellar SDK + Soroban RPC)                 │
└─────────────────────────────────────────────────────────┘
```

### Data Flow

1. **Client** calls the backend API with payment intent
2. **Backend** constructs a Soroban transaction using `stellar-sdk`
3. **Client** signs the transaction via Freighter wallet
4. **Backend** submits the signed transaction to Stellar
5. **Contract** validates, updates persistent storage, emits events
6. **Backend** confirms transaction and returns the hash

## Features

### Payment Stream Contract

1. **Create Stream** — Initialize a continuous payment between sender and receiver with per-second rate
2. **Withdraw Funds** — Receiver can claim accrued funds at any time (partial or full)
3. **Pause Stream** — Sender can temporarily halt the stream without ending it
4. **Resume Stream** — Sender can resume a paused stream; pause duration is excluded from accrual
5. **Stop Stream** — Sender can permanently end the stream
6. **Get Withdrawdable** — Query available balance at any moment (accounts for pauses)
7. **Stream Status** — Check if a stream is Active, Paused, or Cancelled
8. **Cumulative Pause Tracking** — Accurate accounting even after multiple pause/resume cycles

### Invoice Contract

9. **Create Invoice** — Issue an invoice with line items, quantity, and due date
10. **Line Items** — Support for up to 50 items with description, amount, and quantity
11. **Automatic Total Calculation** — Contract computes total from items (amount * quantity)
12. **Send Invoice** — Transition from Draft to Sent status
13. **Pay Invoice** — Recipient marks invoice as paid (with timestamp)
14. **Cancel Invoice** — Issuer can cancel unpaid invoices
15. **Overdue Detection** — Check and mark invoices past due date as Overdue
16. **Invoice Status Lifecycle** — Draft → Sent → Paid/Overdue/Cancelled

### Subscription Contract

17. **Create Plan** — Define recurring billing plans with name, amount, interval, and capacity
18. **Subscribe** — Users can subscribe to plans with automatic billing date calculation
19. **Process Billing** — Trigger billing for a subscription; advances next billing date
20. **Cancel Subscription** — Subscriber can cancel; decrements plan subscriber count
21. **Multiple Billing Intervals** — Daily, Weekly, Monthly, Quarterly, Yearly
22. **Max Subscribers Limit** — Optional capacity limits per plan
23. **Subscriber Tracking** — Plan maintains count of active subscribers
24. **Billing History** — Subscription tracks total_paid and billing_count

## Tech Stack

| Component | Technology | Version |
|-----------|-----------|---------|
| Language | Rust | 2021 Edition |
| Smart Contract SDK | Soroban SDK | 21.0.0 |
| Stellar SDK | stellar-sdk | 0.24.0 |
| Target | Stellar / Soroban | — |
| Build Profile | Release (opt-level=z, LTO, strip) | — |
| Test Framework | Soroban testutils | — |
| Package Manager | Cargo Workspace | — |
| License | MIT | — |

## Project Structure

```
contracts/
├── Cargo.toml                          # Workspace root — defines members and shared deps
├── LICENSE                             # MIT License
├── README.md                           # This file
├── contracts/
│   ├── stream/                         # Payment Stream contract
│   │   ├── Cargo.toml                  # Crate config (cdylib target)
│   │   └── src/
│   │       └── lib.rs                  # Contract logic + tests (421 lines)
│   ├── invoice/                        # Invoice contract
│   │   ├── Cargo.toml                  # Crate config (cdylib target)
│   │   └── src/
│   │       └── lib.rs                  # Contract logic + tests (383 lines)
│   └── subscription/                   # Subscription contract
│       ├── Cargo.toml                  # Crate config (cdylib target)
│       └── src/
│           └── lib.rs                  # Contract logic + tests (517 lines)
```

## Smart Contracts

### Payment Stream Contract

**Crate:** `web3-suite-payment-stream`

The Payment Stream contract enables continuous, real-time payments on Stellar. Funds accrue per-second between a sender and receiver, and the receiver can withdraw at any point.

#### Data Structures

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

pub enum StreamStatus {
    Active,
    Paused,
    Completed,
    Cancelled,
}
```

#### Function Signatures

```rust
/// Create a new payment stream.
///
/// # Arguments
/// * `sender` — Address initiating the payment (must authorize)
/// * `receiver` — Address receiving the payment
/// * `amount_per_second` — Payment rate in stroops per second
/// * `start_time` — Unix timestamp when streaming begins
/// * `end_time` — Unix timestamp when streaming ends
///
/// # Returns
/// * `u64` — The assigned stream ID
///
/// # Errors
/// - "end_time must be after start_time"
/// - "amount_per_second must be positive"
/// - "sender and receiver cannot be the same"
pub fn create_stream(
    env: Env,
    sender: Address,
    receiver: Address,
    amount_per_second: u128,
    start_time: u64,
    end_time: u64,
) -> u64;

/// Withdraw accrued funds from a stream.
///
/// # Arguments
/// * `stream_id` — ID of the stream to withdraw from
/// * `amount` — Optional specific amount; None withdraws all available
///
/// # Returns
/// * `u128` — Amount withdrawn
///
/// # Errors
/// - "stream is not active"
/// - "amount exceeds available"
/// - "no funds available to withdraw"
pub fn withdraw(env: Env, stream_id: u64, amount: Option<u128>) -> u128;

/// Pause a stream (sender only). Pause duration excludes from accrual.
///
/// # Errors
/// - "stream is not active"
/// - "stream is already paused"
pub fn pause_stream(env: Env, stream_id: u64);

/// Resume a paused stream (sender only). Accumulates pause duration.
///
/// # Errors
/// - "stream is not active"
/// - "stream is not paused"
pub fn resume_stream(env: Env, stream_id: u64);

/// Permanently stop a stream (sender only).
///
/// # Errors
/// - "stream is not active"
pub fn stop_stream(env: Env, stream_id: u64);

/// Query the withdrawable amount for a stream.
///
/// # Returns
/// * `u128` — Available balance (accounts for pauses)
pub fn get_withdrawable(env: Env, stream_id: u64) -> u128;

/// Get full stream data.
pub fn get_stream(env: Env, stream_id: u64) -> PaymentStream;

/// Get stream status enum.
pub fn get_status(env: Env, stream_id: u64) -> StreamStatus;
```

#### Events

| Event | Symbol | Payload |
|-------|--------|---------|
| Stream Created | `STR_CREATED` | `(sender, stream_id)` |
| Stream Paused | `STR_PAUSED` | `(sender, stream_id)` |
| Stream Resumed | `STR_RESUMED` | `(sender, stream_id)` |
| Stream Stopped | `STR_STOPPED` | `(sender, stream_id)` |
| Withdrawal | `WITHDRAWAL` | `(receiver, (stream_id, amount))` |

#### Example Usage

```rust
// Create a stream: 100 stroops/sec from t=1000 to t=2000
let stream_id = client.create_stream(&sender, &receiver, &100, &1000, &2000);

// Pause at t=1500
client.pause_stream(&stream_id);

// Resume at t=1600 (100 seconds of pause excluded)
client.resume_stream(&stream_id);

// Withdraw at t=1800: (1800-1000-100) * 100 = 70,000 stroops
let amount = client.withdraw(&stream_id, &None);
```

---

### Invoice Contract

**Crate:** `web3-suite-payment-invoice`

The Invoice contract manages the lifecycle of on-chain invoices with line items, status tracking, and payment settlement.

#### Data Structures

```rust
pub struct InvoiceItem {
    pub description: Bytes,
    pub amount: u128,
    pub quantity: u32,
}

pub struct Invoice {
    pub id: u64,
    pub issuer: Address,
    pub recipient: Address,
    pub items: soroban_sdk::Vec<InvoiceItem>,
    pub total_amount: u128,
    pub due_date: u64,
    pub issued_date: u64,
    pub status: InvoiceStatus,
    pub notes: Option<Bytes>,
    pub paid_at: Option<u64>,
    pub payment_tx: Option<Bytes>,
}

pub enum InvoiceStatus {
    Draft,
    Sent,
    Paid,
    Overdue,
    Cancelled,
}
```

#### Function Signatures

```rust
/// Create a new invoice with line items.
///
/// # Arguments
/// * `issuer` — Address of the invoice creator (must authorize)
/// * `recipient` — Address of the payer
/// * `items` — Vector of InvoiceItem (description, amount, quantity)
/// * `due_date` — Unix timestamp for payment deadline
/// * `notes` — Optional notes/memo
///
/// # Returns
/// * `u64` — The assigned invoice ID
///
/// # Errors
/// - "invoice must have at least one item"
/// - "due_date must be in the future"
/// - "issuer and recipient cannot be the same"
/// - "total amount must be positive"
pub fn create_invoice(
    env: Env,
    issuer: Address,
    recipient: Address,
    items: soroban_sdk::Vec<InvoiceItem>,
    due_date: u64,
    notes: Option<Bytes>,
) -> u64;

/// Send invoice — transitions Draft to Sent.
///
/// # Errors
/// - "can only send draft invoices"
pub fn send_invoice(env: Env, invoice_id: u64);

/// Pay invoice — transitions Sent/Overdue to Paid.
///
/// # Errors
/// - "invoice must be sent or overdue to pay"
pub fn pay_invoice(env: Env, invoice_id: u64);

/// Cancel invoice — transitions to Cancelled (not if already Paid).
///
/// # Errors
/// - "cannot cancel paid or already cancelled invoice"
pub fn cancel_invoice(env: Env, invoice_id: u64);

/// Check and mark as overdue if past due date.
pub fn check_overdue(env: Env, invoice_id: u64);

/// Get full invoice data.
pub fn get_invoice(env: Env, invoice_id: u64) -> Invoice;

/// Calculate total from a vector of items (utility function).
pub fn calculate_total(env: Env, items: soroban_sdk::Vec<InvoiceItem>) -> u128;
```

#### Events

| Event | Symbol | Payload |
|-------|--------|---------|
| Invoice Created | `INV_CREATED` | `(issuer, invoice_id)` |
| Invoice Sent | `INV_SENT` | `(issuer, invoice_id)` |
| Invoice Paid | `INV_PAID` | `(recipient, (invoice_id, total_amount))` |
| Invoice Cancelled | `INV_CANCEL` | `(issuer, invoice_id)` |

#### Example Usage

```rust
// Create invoice with two line items
let mut items = soroban_sdk::Vec::new(&env);
items.push_back(InvoiceItem { description: "Web Dev".into(), amount: 5000, quantity: 1 });
items.push_back(InvoiceItem { description: "Design".into(), amount: 2000, quantity: 2 });

let invoice_id = client.create_invoice(&issuer, &recipient, &items, &100000, &None);
// Total: 5000*1 + 2000*2 = 9000 stroops

client.send_invoice(&invoice_id);    // Draft → Sent
client.pay_invoice(&invoice_id);     // Sent → Paid
```

---

### Subscription Contract

**Crate:** `web3-suite-payment-subscription`

The Subscription contract handles recurring billing with plan creation, subscriber management, and configurable billing intervals.

#### Data Structures

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

pub struct Subscription {
    pub id: u64,
    pub plan_id: u64,
    pub subscriber: Address,
    pub merchant: Address,
    pub start_date: u64,
    pub next_billing_date: u64,
    pub total_paid: u128,
    pub billing_count: u32,
    pub is_active: bool,
    pub cancelled_at: Option<u64>,
    pub created_at: u64,
}

pub enum BillingInterval {
    Daily,      // 86,400 seconds
    Weekly,     // 604,800 seconds
    Monthly,    // 2,592,000 seconds (30 days)
    Quarterly,  // 7,776,000 seconds (90 days)
    Yearly,     // 31,536,000 seconds (365 days)
}

pub enum SubscriptionStatus {
    Active,
    PastDue,
    Cancelled,
    Paused,
}
```

#### Function Signatures

```rust
/// Create a new subscription plan.
///
/// # Arguments
/// * `creator` — Address of the plan creator (must authorize)
/// * `name` — Plan name
/// * `description` — Plan description
/// * `amount` — Price per billing cycle in stroops
/// * `billing_interval` — Billing frequency (Daily/Weekly/Monthly/Quarterly/Yearly)
/// * `interval_count` — Multiplier for the interval (e.g., 2 = biweekly)
/// * `max_subscribers` — Optional subscriber cap
///
/// # Returns
/// * `u64` — The assigned plan ID
///
/// # Errors
/// - "amount must be positive"
/// - "interval_count must be positive"
/// - "name cannot be empty"
pub fn create_plan(
    env: Env,
    creator: Address,
    name: Bytes,
    description: Bytes,
    amount: u128,
    billing_interval: BillingInterval,
    interval_count: u32,
    max_subscribers: Option<u32>,
) -> u64;

/// Subscribe to a plan.
///
/// # Arguments
/// * `plan_id` — ID of the plan to subscribe to
/// * `subscriber` — Address subscribing (must authorize)
///
/// # Returns
/// * `u64` — The assigned subscription ID
///
/// # Errors
/// - "plan is not active"
/// - "plan has reached max subscribers"
pub fn subscribe(env: Env, plan_id: u64, subscriber: Address) -> u64;

/// Process billing for a subscription. Advances next_billing_date.
///
/// # Returns
/// * `u128` — Amount billed
///
/// # Errors
/// - "subscription is not active"
/// - "not yet billing date"
pub fn process_billing(env: Env, sub_id: u64) -> u128;

/// Cancel a subscription (subscriber only). Decrements plan subscriber count.
///
/// # Errors
/// - "subscription is not active"
pub fn cancel_subscription(env: Env, sub_id: u64);

/// Get full subscription data.
pub fn get_subscription(env: Env, sub_id: u64) -> Subscription;

/// Get full plan data.
pub fn get_plan(env: Env, plan_id: u64) -> SubscriptionPlan;

/// Calculate the next billing date given a current timestamp and interval.
pub fn next_billing_date(current: u64, interval: &BillingInterval, count: u32) -> u64;
```

#### Events

| Event | Symbol | Payload |
|-------|--------|---------|
| Plan Created | `PLAN_CREATED` | `(creator, plan_id)` |
| Subscription Activated | `SUB_ACT` | `(subscriber, (sub_id, plan_id))` |
| Subscription Cancelled | `SUB_CANCEL` | `(subscriber, sub_id)` |
| Billing Processed | `BILL_PROC` | `(subscriber, (sub_id, amount))` |

#### Example Usage

```rust
// Create a monthly plan at 2500 stroops/month with 100 subscriber cap
let plan_id = client.create_plan(
    &creator,
    &Bytes::from_array(&env, b"Pro Plan"),
    &Bytes::from_array(&env, b"Monthly Pro"),
    &2500,
    &BillingInterval::Monthly,
    &1,
    &Some(100),
);

// Subscribe
let sub_id = client.subscribe(&plan_id, &subscriber);

// Process billing (must be on or after next_billing_date)
let amount = client.process_billing(&sub_id); // Returns 2500

// Cancel
client.cancel_subscription(&sub_id);
```

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain, 2021 edition)
- [Soroban CLI](https://soroban.stellar.org/docs/getting-started/setup) (`soroban`)
- [Stellar CLI](https://soroban.stellar.org/docs/getting-started/setup) (`stellar`)
- [Freighter](https://freighter.app/) browser extension (for deployment)

### Installation

```bash
# Clone the repository
git clone https://github.com/sudo-robi/web3-suite-payments-contracts.git
cd web3-suite-payments-contracts
```

### Building

```bash
# Build all contracts in the workspace
cargo build --target wasm32-unknown-unknown --release

# Build individual contracts
cargo build --target wasm32-unknown-unknown --release -p web3-suite-payment-stream
cargo build --target wasm32-unknown-unknown --release -p web3-suite-payment-invoice
cargo build --target wasm32-unknown-unknown --release -p web3-suite-payment-subscription
```

### Testing

```bash
# Run all tests
cargo test

# Run tests for a specific contract
cargo test -p web3-suite-payment-stream
cargo test -p web3-suite-payment-invoice
cargo test -p web3-suite-payment-subscription

# Run with verbose output
cargo test -- --nocapture
```

### Deployment

```bash
# Install contracts to Stellar testnet
stellar contract install --wasm target/wasm32-unknown-unknown/release/web3_suite_payment_stream.wasm --network testnet
stellar contract install --wasm target/wasm32-unknown-unknown/release/web3_suite_payment_invoice.wasm --network testnet
stellar contract install --wasm target/wasm32-unknown-unknown/release/web3_suite_payment_subscription.wasm --network testnet

# Deploy each contract
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/web3_suite_payment_stream.wasm \
  --network testnet \
  --source <YOUR_KEYPAIR_NAME>

stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/web3_suite_payment_invoice.wasm \
  --network testnet \
  --source <YOUR_KEYPAIR_NAME>

stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/web3_suite_payment_subscription.wasm \
  --network testnet \
  --source <YOUR_KEYPAIR_NAME>
```

### Network Configuration

| Network | Passphrase | RPC URL |
|---------|-----------|---------|
| Testnet | `Test SDF Future Network ; October 2022` | `https://soroban-testnet.stellar.org` |
| Futurenet | `Test SDF Future Network ; October 2022` | `https://soroban-futurenet.stellar.org` |
| Mainnet | `Public Global Stellar Network ; September 2015` | `https://soroban-mainnet.stellar.org` |

## Contributing

### Branch Naming

```
feat/payment-stream-optimization
fix/invoice-status-bug
docs/update-readme
refactor/subscription-storage
test/add-edge-case-tests
```

### Commit Conventions

```
feat: add batch withdraw for multiple streams
fix: correct pause duration calculation
docs: update function signatures
test: add edge case tests for zero-amount streams
refactor: optimize storage key mapping
```

### Pull Request Process

1. Create a feature branch from `main`
2. Write tests for new functionality
3. Ensure all tests pass: `cargo test`
4. Build for WASM target: `cargo build --target wasm32-unknown-unknown --release`
5. Submit PR with clear description of changes
6. Address review feedback
7. Merge after approval

### Code Standards

- All functions must have doc comments with `# Arguments`, `# Returns`, `# Errors`
- All error paths must use descriptive assertion messages
- Tests must cover happy path and error paths
- Use `#[should_panic(expected = "...")]` for error tests
- Follow existing storage key patterns (match on IDs 1-8, fallback for overflow)

## License

MIT License — see [LICENSE](LICENSE) for details.

## Related Repositories

- [web3-suite-payments-backend](https://github.com/sudo-robi/web3-suite-payments-backend) — Express.js API layer
- [web3-suite-payments-frontend](https://github.com/sudo-robi/web3-suite-payments-frontend) — React + Tailwind UI
