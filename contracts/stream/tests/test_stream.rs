#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_create_stream() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PaymentStreamContract);
    let client = PaymentStreamContractClient::new(&env, &contract_id);

    let sender = Address::generate(&env);
    let receiver = Address::generate(&env);

    env.mock_auths(&[MockAuth {
        address: &sender,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "create_stream",
            args: (&sender, &receiver, 100u128, 1000u64, 2000u64).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let stream_id = client.create_stream(&sender, &receiver, &100, &1000, &2000);
    assert_eq!(stream_id, 1);

    let stream = client.get_stream(&stream_id);
    assert_eq!(stream.sender, sender);
    assert_eq!(stream.receiver, receiver);
    assert_eq!(stream.amount_per_second, 100);
    assert!(stream.is_active);
}

#[test]
fn test_pause_and_resume_stream() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PaymentStreamContract);
    let client = PaymentStreamContractClient::new(&env, &contract_id);

    let sender = Address::generate(&env);
    let receiver = Address::generate(&env);

    env.mock_auths(&[MockAuth {
        address: &sender,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "create_stream",
            args: (&sender, &receiver, 100u128, 1000u64, 2000u64).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let stream_id = client.create_stream(&sender, &receiver, &100, &1000, &2000);

    env.mock_auths(&[MockAuth {
        address: &sender,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "pause_stream",
            args: (&stream_id,).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    client.pause_stream(&stream_id);
    let stream = client.get_stream(&stream_id);
    assert!(stream.is_paused);

    env.mock_auths(&[MockAuth {
        address: &sender,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "resume_stream",
            args: (&stream_id,).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    client.resume_stream(&stream_id);
    let stream = client.get_stream(&stream_id);
    assert!(!stream.is_paused);
}

#[test]
#[should_panic(expected = "stream is not active")]
fn test_withdraw_inactive_stream() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PaymentStreamContract);
    let client = PaymentStreamContractClient::new(&env, &contract_id);

    let sender = Address::generate(&env);
    let receiver = Address::generate(&env);

    env.mock_auths(&[MockAuth {
        address: &sender,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "create_stream",
            args: (&sender, &receiver, 100u128, 1000u64, 2000u64).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let stream_id = client.create_stream(&sender, &receiver, &100, &1000, &2000);

    env.mock_auths(&[MockAuth {
        address: &sender,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "stop_stream",
            args: (&stream_id,).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    client.stop_stream(&stream_id);

    env.mock_auths(&[MockAuth {
        address: &receiver,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "withdraw",
            args: (&stream_id, &None::<u128>).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    client.withdraw(&stream_id, &None);
}
