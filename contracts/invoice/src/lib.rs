#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Bytes, Env, Symbol};

const INVOICE_CREATED: Symbol = symbol_short!("INV_CREATED");
const INVOICE_SENT: Symbol = symbol_short!("INV_SENT");
const INVOICE_PAID: Symbol = symbol_short!("INV_PAID");
const INVOICE_CANCELLED: Symbol = symbol_short!("INV_CANCEL");

#[derive(Clone)]
#[contracttype]
pub struct InvoiceItem {
    pub description: Bytes,
    pub amount: u128,
    pub quantity: u32,
}

#[derive(Clone)]
#[contracttype]
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

#[derive(Clone)]
#[contracttype]
pub enum InvoiceStatus {
    Draft,
    Sent,
    Paid,
    Overdue,
    Cancelled,
}

#[contract]
pub struct InvoiceContract;

#[contractimpl]
impl InvoiceContract {
    /// Creates a new invoice with line items.
    pub fn create_invoice(
        env: Env,
        issuer: Address,
        recipient: Address,
        items: soroban_sdk::Vec<InvoiceItem>,
        due_date: u64,
        notes: Option<Bytes>,
    ) -> u64 {
        issuer.require_auth();

        assert!(items.len() > 0, "invoice must have at least one item");
        assert!(due_date > env.ledger().timestamp(), "due_date must be in the future");
        assert!(issuer != recipient, "issuer and recipient cannot be the same");

        let mut total: u128 = 0;
        for item in items.iter() {
            total += item.amount * item.quantity as u128;
        }
        assert!(total > 0, "total amount must be positive");

        let invoice_id = env.storage().instance().get::<_, u64>(&symbol_short!("INV_ID")).unwrap_or(0) + 1;

        let invoice = Invoice {
            id: invoice_id,
            issuer: issuer.clone(),
            recipient,
            items,
            total_amount: total,
            due_date,
            issued_date: env.ledger().timestamp(),
            status: InvoiceStatus::Draft,
            notes,
            paid_at: None,
            payment_tx: None,
        };

        env.storage().instance().set(&symbol_short!("INV_ID"), &invoice_id);
        env.storage().persistent().set(&invoice_key(invoice_id), &invoice);

        env.events()
            .publish((INVOICE_CREATED, issuer), invoice_id);

        invoice_id
    }

    /// Sends an invoice to the recipient, changing status to Sent.
    pub fn send_invoice(env: Env, invoice_id: u64) {
        let mut invoice = get_invoice(&env, invoice_id);
        invoice.issuer.require_auth();

        assert!(matches!(invoice.status, InvoiceStatus::Draft), "can only send draft invoices");

        invoice.status = InvoiceStatus::Sent;
        env.storage().persistent().set(&invoice_key(invoice_id), &invoice);

        env.events()
            .publish((INVOICE_SENT, invoice.issuer.clone()), invoice_id);
    }

    /// Pays an invoice. Only callable by the recipient.
    pub fn pay_invoice(env: Env, invoice_id: u64) {
        let mut invoice = get_invoice(&env, invoice_id);
        invoice.recipient.require_auth();

        assert!(
            matches!(invoice.status, InvoiceStatus::Sent | InvoiceStatus::Overdue),
            "invoice must be sent or overdue to pay"
        );

        // Transfer payment from recipient to issuer
        env.transfer旅途(&invoice.recipient, &invoice.issuer, invoice.total_amount);

        invoice.status = InvoiceStatus::Paid;
        invoice.paid_at = Some(env.ledger().timestamp());

        env.storage().persistent().set(&invoice_key(invoice_id), &invoice);

        env.events()
            .publish((INVOICE_PAID, invoice.recipient.clone()), (invoice_id, invoice.total_amount));
    }

    /// Cancels an invoice. Only callable by the issuer.
    pub fn cancel_invoice(env: Env, invoice_id: u64) {
        let mut invoice = get_invoice(&env, invoice_id);
        invoice.issuer.require_auth();

        assert!(
            !matches!(invoice.status, InvoiceStatus::Paid | InvoiceStatus::Cancelled),
            "cannot cancel paid or already cancelled invoice"
        );

        invoice.status = InvoiceStatus::Cancelled;
        env.storage().persistent().set(&invoice_key(invoice_id), &invoice);

        env.events()
            .publish((INVOICE_CANCELLED, invoice.issuer.clone()), invoice_id);
    }

    /// Checks and marks overdue invoices.
    pub fn check_overdue(env: Env, invoice_id: u64) {
        let mut invoice = get_invoice(&env, invoice_id);
        let now = env.ledger().timestamp();

        if matches!(invoice.status, InvoiceStatus::Sent) && now > invoice.due_date {
            invoice.status = InvoiceStatus::Overdue;
            env.storage().persistent().set(&invoice_key(invoice_id), &invoice);
        }
    }

    /// Returns invoice details.
    pub fn get_invoice(env: Env, invoice_id: u64) -> Invoice {
        get_invoice(&env, invoice_id)
    }

    /// Calculates the total amount for an invoice.
    pub fn calculate_total(env: Env, items: soroban_sdk::Vec<InvoiceItem>) -> u128 {
        let mut total: u128 = 0;
        for item in items.iter() {
            total += item.amount * item.quantity as u128;
        }
        total
    }

    /// Lists all invoices for an address (as issuer or recipient).
    pub fn list_invoices(env: Env, address: Address) -> soroban_sdk::Vec<u64> {
        let mut invoices = soroban_sdk::Vec::new(&env);
        let all_ids = get_all_invoice_ids(&env);

        for i in 0..all_ids.len() {
            let id = all_ids.get_unchecked(i);
            if let Ok(invoice) = env.storage().persistent().get::<_, Invoice>(&invoice_key(id)) {
                if invoice.issuer == address || invoice.recipient == address {
                    invoices.push_back(id);
                }
            }
        }

        invoices
    }
}

fn invoice_key(invoice_id: u64) -> Symbol {
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&invoice_id.to_be_bytes());
    symbol_short!("INV").into_val(&Env::default())
}

fn get_invoice(env: &Env, invoice_id: u64) -> Invoice {
    env.storage()
        .persistent()
        .get(&invoice_key(invoice_id))
        .expect("invoice not found")
}

fn get_all_invoice_ids(env: &Env) -> soroban_sdk::Vec<u64> {
    env.storage()
        .instance()
        .get(&symbol_short!("ALL_INV_IDS"))
        .unwrap_or(soroban_sdk::Vec::new(env))
}


