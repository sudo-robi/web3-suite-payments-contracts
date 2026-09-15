#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Bytes, Env};

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
    items.push_back(&env, create_test_item(&env, "Design Review", 2000, 2));

    env.mock_auths(&[MockAuth {
        address: &issuer,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "create_invoice",
            args: (&issuer, &recipient, &items, &100000u64, &None::<Bytes>).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let invoice_id = client.create_invoice(&issuer, &recipient, &items, &100000, &None);
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

    env.mock_auths(&[MockAuth {
        address: &issuer,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "create_invoice",
            args: (&issuer, &recipient, &items, &100000u64, &None::<Bytes>).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let invoice_id = client.create_invoice(&issuer, &recipient, &items, &100000, &None);

    env.mock_auths(&[MockAuth {
        address: &issuer,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "send_invoice",
            args: (&invoice_id,).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    client.send_invoice(&invoice_id);
    let invoice = client.get_invoice(&invoice_id);
    assert!(matches!(invoice.status, InvoiceStatus::Sent));

    env.mock_auths(&[MockAuth {
        address: &recipient,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "pay_invoice",
            args: (&invoice_id,).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    client.pay_invoice(&invoice_id);
    let invoice = client.get_invoice(&invoice_id);
    assert!(matches!(invoice.status, InvoiceStatus::Paid));
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
