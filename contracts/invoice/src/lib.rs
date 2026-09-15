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

fn invoice_key(invoice_id: u64) -> Symbol {
    match invoice_id {
        1 => symbol_short!("INV1"),
        2 => symbol_short!("INV2"),
        3 => symbol_short!("INV3"),
        4 => symbol_short!("INV4"),
        5 => symbol_short!("INV5"),
        6 => symbol_short!("INV6"),
        7 => symbol_short!("INV7"),
        8 => symbol_short!("INV8"),
        _ => symbol_short!("INV"),
    }
}

fn get_invoice(env: &Env, invoice_id: u64) -> Invoice {
    env.storage()
        .persistent()
        .get(&invoice_key(invoice_id))
        .expect("invoice not found")
}

#[contract]
pub struct InvoiceContract;

#[contractimpl]
impl InvoiceContract {
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
        assert!(
            due_date > env.ledger().timestamp(),
            "due_date must be in the future"
        );
        assert!(issuer != recipient, "issuer and recipient cannot be the same");

        let mut total: u128 = 0;
        for item in items.iter() {
            total += item.amount * item.quantity as u128;
        }
        assert!(total > 0, "total amount must be positive");

        let invoice_id = env
            .storage()
            .instance()
            .get::<_, u64>(&symbol_short!("INV_ID"))
            .unwrap_or(0)
            + 1;

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

        env.storage()
            .instance()
            .set(&symbol_short!("INV_ID"), &invoice_id);
        env.storage()
            .persistent()
            .set(&invoice_key(invoice_id), &invoice);

        env.events()
            .publish((INVOICE_CREATED, issuer), invoice_id);

        invoice_id
    }

    pub fn send_invoice(env: Env, invoice_id: u64) {
        let mut invoice = get_invoice(&env, invoice_id);
        invoice.issuer.require_auth();

        assert!(
            matches!(invoice.status, InvoiceStatus::Draft),
            "can only send draft invoices"
        );

        invoice.status = InvoiceStatus::Sent;
        env.storage()
            .persistent()
            .set(&invoice_key(invoice_id), &invoice);

        env.events()
            .publish((INVOICE_SENT, invoice.issuer.clone()), invoice_id);
    }

    pub fn pay_invoice(env: Env, invoice_id: u64) {
        let mut invoice = get_invoice(&env, invoice_id);
        invoice.recipient.require_auth();

        assert!(
            matches!(invoice.status, InvoiceStatus::Sent | InvoiceStatus::Overdue),
            "invoice must be sent or overdue to pay"
        );

        invoice.status = InvoiceStatus::Paid;
        invoice.paid_at = Some(env.ledger().timestamp());

        env.storage()
            .persistent()
            .set(&invoice_key(invoice_id), &invoice);

        env.events().publish(
            (INVOICE_PAID, invoice.recipient.clone()),
            (invoice_id, invoice.total_amount),
        );
    }

    pub fn cancel_invoice(env: Env, invoice_id: u64) {
        let mut invoice = get_invoice(&env, invoice_id);
        invoice.issuer.require_auth();

        assert!(
            !matches!(invoice.status, InvoiceStatus::Paid | InvoiceStatus::Cancelled),
            "cannot cancel paid or already cancelled invoice"
        );

        invoice.status = InvoiceStatus::Cancelled;
        env.storage()
            .persistent()
            .set(&invoice_key(invoice_id), &invoice);

        env.events()
            .publish((INVOICE_CANCELLED, invoice.issuer.clone()), invoice_id);
    }

    pub fn check_overdue(env: Env, invoice_id: u64) {
        let mut invoice = get_invoice(&env, invoice_id);
        let now = env.ledger().timestamp();

        if matches!(invoice.status, InvoiceStatus::Sent) && now > invoice.due_date {
            invoice.status = InvoiceStatus::Overdue;
            env.storage()
                .persistent()
                .set(&invoice_key(invoice_id), &invoice);
        }
    }

    pub fn get_invoice(env: Env, invoice_id: u64) -> Invoice {
        get_invoice(&env, invoice_id)
    }

    pub fn calculate_total(env: Env, items: soroban_sdk::Vec<InvoiceItem>) -> u128 {
        let mut total: u128 = 0;
        for item in items.iter() {
            total += item.amount * item.quantity as u128;
        }
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    fn create_test_item(env: &Env, desc: &str, amount: u128, quantity: u32) -> InvoiceItem {
        InvoiceItem {
            description: Bytes::from_array(env, desc.as_bytes()),
            amount,
            quantity,
        }
    }

    #[test]
    fn test_create_invoice() {
        let env = Env::default();
        let contract_id = env.register_contract(None, InvoiceContract);
        let client = InvoiceContractClient::new(&env, &contract_id);

        let issuer = Address::generate(&env);
        let recipient = Address::generate(&env);

        let mut items = soroban_sdk::Vec::new(&env);
        items.push_back(create_test_item(&env, "Web Development", 5000, 1));
        items.push_back(create_test_item(&env, "Design Review", 2000, 2));

        let invoice_id =
            client.create_invoice(&issuer, &recipient, &items, &100000, &None::<Bytes>);
        assert_eq!(invoice_id, 1);

        let invoice = client.get_invoice(&invoice_id);
        assert_eq!(invoice.total_amount, 9000); // 5000*1 + 2000*2
        assert!(matches!(invoice.status, InvoiceStatus::Draft));
    }

    #[test]
    fn test_send_and_pay_invoice() {
        let env = Env::default();
        let contract_id = env.register_contract(None, InvoiceContract);
        let client = InvoiceContractClient::new(&env, &contract_id);

        let issuer = Address::generate(&env);
        let recipient = Address::generate(&env);

        let mut items = soroban_sdk::Vec::new(&env);
        items.push_back(create_test_item(&env, "Service", 1000, 1));

        let invoice_id =
            client.create_invoice(&issuer, &recipient, &items, &100000, &None::<Bytes>);

        client.send_invoice(&invoice_id);
        let invoice = client.get_invoice(&invoice_id);
        assert!(matches!(invoice.status, InvoiceStatus::Sent));

        client.pay_invoice(&invoice_id);
        let invoice = client.get_invoice(&invoice_id);
        assert!(matches!(invoice.status, InvoiceStatus::Paid));
        assert!(invoice.paid_at.is_some());
    }

    #[test]
    fn test_cancel_invoice() {
        let env = Env::default();
        let contract_id = env.register_contract(None, InvoiceContract);
        let client = InvoiceContractClient::new(&env, &contract_id);

        let issuer = Address::generate(&env);
        let recipient = Address::generate(&env);

        let mut items = soroban_sdk::Vec::new(&env);
        items.push_back(create_test_item(&env, "Service", 1000, 1));

        let invoice_id =
            client.create_invoice(&issuer, &recipient, &items, &100000, &None::<Bytes>);

        client.cancel_invoice(&invoice_id);
        let invoice = client.get_invoice(&invoice_id);
        assert!(matches!(invoice.status, InvoiceStatus::Cancelled));
    }

    #[test]
    fn test_calculate_total() {
        let env = Env::default();
        let contract_id = env.register_contract(None, InvoiceContract);
        let client = InvoiceContractClient::new(&env, &contract_id);

        let mut items = soroban_sdk::Vec::new(&env);
        items.push_back(create_test_item(&env, "Item A", 100, 3));
        items.push_back(create_test_item(&env, "Item B", 250, 2));

        let total = client.calculate_total(&items);
        assert_eq!(total, 800); // 100*3 + 250*2
    }

    #[test]
    #[should_panic(expected = "invoice must have at least one item")]
    fn test_create_empty_invoice() {
        let env = Env::default();
        let contract_id = env.register_contract(None, InvoiceContract);
        let client = InvoiceContractClient::new(&env, &contract_id);

        let issuer = Address::generate(&env);
        let recipient = Address::generate(&env);

        let items = soroban_sdk::Vec::new(&env);
        client.create_invoice(&issuer, &recipient, &items, &100000, &None::<Bytes>);
    }

    #[test]
    #[should_panic(expected = "issuer and recipient cannot be the same")]
    fn test_create_invoice_same_parties() {
        let env = Env::default();
        let contract_id = env.register_contract(None, InvoiceContract);
        let client = InvoiceContractClient::new(&env, &contract_id);

        let addr = Address::generate(&env);
        let mut items = soroban_sdk::Vec::new(&env);
        items.push_back(create_test_item(&env, "Service", 1000, 1));

        client.create_invoice(&addr, &addr, &items, &100000, &None::<Bytes>);
    }

    #[test]
    #[should_panic(expected = "cannot cancel paid or already cancelled invoice")]
    fn test_cancel_paid_invoice() {
        let env = Env::default();
        let contract_id = env.register_contract(None, InvoiceContract);
        let client = InvoiceContractClient::new(&env, &contract_id);

        let issuer = Address::generate(&env);
        let recipient = Address::generate(&env);

        let mut items = soroban_sdk::Vec::new(&env);
        items.push_back(create_test_item(&env, "Service", 1000, 1));

        let invoice_id =
            client.create_invoice(&issuer, &recipient, &items, &100000, &None::<Bytes>);

        client.send_invoice(&invoice_id);
        client.pay_invoice(&invoice_id);
        client.cancel_invoice(&invoice_id);
    }

    #[test]
    fn test_invoice_workflow_full() {
        let env = Env::default();
        let contract_id = env.register_contract(None, InvoiceContract);
        let client = InvoiceContractClient::new(&env, &contract_id);

        let issuer = Address::generate(&env);
        let recipient = Address::generate(&env);

        let mut items = soroban_sdk::Vec::new(&env);
        items.push_back(create_test_item(&env, "Consulting", 5000, 2));

        // Create
        let invoice_id =
            client.create_invoice(&issuer, &recipient, &items, &200000, &None::<Bytes>);
        let invoice = client.get_invoice(&invoice_id);
        assert_eq!(invoice.total_amount, 10000);
        assert!(matches!(invoice.status, InvoiceStatus::Draft));

        // Send
        client.send_invoice(&invoice_id);
        let invoice = client.get_invoice(&invoice_id);
        assert!(matches!(invoice.status, InvoiceStatus::Sent));

        // Pay
        client.pay_invoice(&invoice_id);
        let invoice = client.get_invoice(&invoice_id);
        assert!(matches!(invoice.status, InvoiceStatus::Paid));
        assert!(invoice.paid_at.is_some());
    }
}
