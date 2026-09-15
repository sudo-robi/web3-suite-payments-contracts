#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Bytes, Env};

#[test]
fn test_create_plan() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SubscriptionContract);
    let client = SubscriptionContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);

    env.mock_auths(&[MockAuth {
        address: &creator,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "create_plan",
            args: (
                &creator,
                Bytes::from_array(&env, b"Pro Plan"),
                Bytes::from_array(&env, b"Monthly Pro subscription"),
                2500u128,
                BillingInterval::Monthly,
                1u32,
                Some(100u32),
            )
                .into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let plan_id = client.create_plan(
        &creator,
        &Bytes::from_array(&env, b"Pro Plan"),
        &Bytes::from_array(&env, b"Monthly Pro subscription"),
        &2500,
        &BillingInterval::Monthly,
        &1,
        &Some(100),
    );

    assert_eq!(plan_id, 1);

    let plan = client.get_plan(&plan_id);
    assert_eq!(plan.amount, 2500);
    assert!(plan.is_active);
    assert_eq!(plan.current_subscribers, 0);
}

#[test]
fn test_subscribe_and_process_billing() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SubscriptionContract);
    let client = SubscriptionContractClient::new(&env, &contract_id);

    let merchant = Address::generate(&env);
    let subscriber = Address::generate(&env);

    env.mock_auths(&[MockAuth {
        address: &merchant,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "create_plan",
            args: (
                &merchant,
                Bytes::from_array(&env, b"Basic Plan"),
                Bytes::from_array(&env, b"Basic"),
                1000u128,
                BillingInterval::Monthly,
                1u32,
                None::<u32>,
            )
                .into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let plan_id = client.create_plan(
        &merchant,
        &Bytes::from_array(&env, b"Basic Plan"),
        &Bytes::from_array(&env, b"Basic"),
        &1000,
        &BillingInterval::Monthly,
        &1,
        &None,
    );

    env.mock_auths(&[MockAuth {
        address: &subscriber,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "subscribe",
            args: (&plan_id, &subscriber).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let sub_id = client.subscribe(&plan_id, &subscriber);
    assert_eq!(sub_id, 1);

    let subscription = client.get_subscription(&sub_id);
    assert!(subscription.is_active);
    assert_eq!(subscription.total_paid, 0);

    let plan = client.get_plan(&plan_id);
    assert_eq!(plan.current_subscribers, 1);
}

#[test]
fn test_cancel_subscription() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SubscriptionContract);
    let client = SubscriptionContractClient::new(&env, &contract_id);

    let merchant = Address::generate(&env);
    let subscriber = Address::generate(&env);

    env.mock_auths(&[MockAuth {
        address: &merchant,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "create_plan",
            args: (
                &merchant,
                Bytes::from_array(&env, b"Plan"),
                Bytes::from_array(&env, b"Desc"),
                500u128,
                BillingInterval::Weekly,
                1u32,
                None::<u32>,
            )
                .into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let plan_id = client.create_plan(
        &merchant,
        &Bytes::from_array(&env, b"Plan"),
        &Bytes::from_array(&env, b"Desc"),
        &500,
        &BillingInterval::Weekly,
        &1,
        &None,
    );

    env.mock_auths(&[MockAuth {
        address: &subscriber,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "subscribe",
            args: (&plan_id, &subscriber).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let sub_id = client.subscribe(&plan_id, &subscriber);

    env.mock_auths(&[MockAuth {
        address: &subscriber,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "cancel_subscription",
            args: (&sub_id,).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    client.cancel_subscription(&sub_id);

    let subscription = client.get_subscription(&sub_id);
    assert!(!subscription.is_active);
    assert!(subscription.cancelled_at.is_some());

    let plan = client.get_plan(&plan_id);
    assert_eq!(plan.current_subscribers, 0);
}

#[test]
fn test_next_billing_date() {
    let base = 1000000u64;

    assert_eq!(
        SubscriptionContract::next_billing_date(base, &BillingInterval::Daily, 1),
        base + 86400
    );
    assert_eq!(
        SubscriptionContract::next_billing_date(base, &BillingInterval::Weekly, 1),
        base + 604800
    );
    assert_eq!(
        SubscriptionContract::next_billing_date(base, &BillingInterval::Monthly, 1),
        base + 2592000
    );
    assert_eq!(
        SubscriptionContract::next_billing_date(base, &BillingInterval::Quarterly, 1),
        base + 7776000
    );
    assert_eq!(
        SubscriptionContract::next_billing_date(base, &BillingInterval::Yearly, 1),
        base + 31536000
    );
}
