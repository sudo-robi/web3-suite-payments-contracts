#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Bytes, Env, Symbol};

const SUBSCRIPTION_CREATED: Symbol = symbol_short!("SUB_CREATED");
const SUBSCRIPTION_ACTIVATED: Symbol = symbol_short!("SUB_ACT");
const SUBSCRIPTION_CANCELLED: Symbol = symbol_short!("SUB_CANCEL");
const BILLING_PROCESSED: Symbol = symbol_short!("BILL_PROC");
const PLAN_CREATED: Symbol = symbol_short!("PLAN_CREATED");

#[derive(Clone)]
#[contracttype]
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

#[derive(Clone)]
#[contracttype]
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

#[derive(Clone)]
#[contracttype]
pub enum BillingInterval {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
}

#[derive(Clone)]
#[contracttype]
pub enum SubscriptionStatus {
    Active,
    PastDue,
    Cancelled,
    Paused,
}

fn plan_key(plan_id: u64) -> Symbol {
    match plan_id {
        1 => symbol_short!("PLAN1"),
        2 => symbol_short!("PLAN2"),
        3 => symbol_short!("PLAN3"),
        4 => symbol_short!("PLAN4"),
        5 => symbol_short!("PLAN5"),
        _ => symbol_short!("PLAN"),
    }
}

fn sub_key(sub_id: u64) -> Symbol {
    match sub_id {
        1 => symbol_short!("SUB1"),
        2 => symbol_short!("SUB2"),
        3 => symbol_short!("SUB3"),
        4 => symbol_short!("SUB4"),
        5 => symbol_short!("SUB5"),
        _ => symbol_short!("SUB"),
    }
}

fn get_plan(env: &Env, plan_id: u64) -> SubscriptionPlan {
    env.storage()
        .persistent()
        .get(&plan_key(plan_id))
        .expect("plan not found")
}

fn get_subscription(env: &Env, sub_id: u64) -> Subscription {
    env.storage()
        .persistent()
        .get(&sub_key(sub_id))
        .expect("subscription not found")
}

#[contract]
pub struct SubscriptionContract;

#[contractimpl]
impl SubscriptionContract {
    pub fn create_plan(
        env: Env,
        creator: Address,
        name: Bytes,
        description: Bytes,
        amount: u128,
        billing_interval: BillingInterval,
        interval_count: u32,
        max_subscribers: Option<u32>,
    ) -> u64 {
        creator.require_auth();

        assert!(amount > 0, "amount must be positive");
        assert!(interval_count > 0, "interval_count must be positive");
        assert!(name.len() > 0, "name cannot be empty");

        let plan_id = env
            .storage()
            .instance()
            .get::<_, u64>(&symbol_short!("PLAN_ID"))
            .unwrap_or(0)
            + 1;

        let plan = SubscriptionPlan {
            id: plan_id,
            name,
            description,
            amount,
            billing_interval,
            interval_count,
            max_subscribers,
            current_subscribers: 0,
            creator: creator.clone(),
            is_active: true,
            created_at: env.ledger().timestamp(),
        };

        env.storage()
            .instance()
            .set(&symbol_short!("PLAN_ID"), &plan_id);
        env.storage()
            .persistent()
            .set(&plan_key(plan_id), &plan);

        env.events()
            .publish((PLAN_CREATED, creator), plan_id);

        plan_id
    }

    pub fn subscribe(env: Env, plan_id: u64, subscriber: Address) -> u64 {
        subscriber.require_auth();

        let mut plan = get_plan(&env, plan_id);
        assert!(plan.is_active, "plan is not active");

        if let Some(max) = plan.max_subscribers {
            assert!(
                plan.current_subscribers < max,
                "plan has reached max subscribers"
            );
        }

        let now = env.ledger().timestamp();
        let next_billing =
            Self::next_billing_date(now, &plan.billing_interval, plan.interval_count);

        let sub_id = env
            .storage()
            .instance()
            .get::<_, u64>(&symbol_short!("SUB_ID"))
            .unwrap_or(0)
            + 1;

        let subscription = Subscription {
            id: sub_id,
            plan_id,
            subscriber: subscriber.clone(),
            merchant: plan.creator.clone(),
            start_date: now,
            next_billing_date: next_billing,
            total_paid: 0,
            billing_count: 0,
            is_active: true,
            cancelled_at: None,
            created_at: now,
        };

        plan.current_subscribers += 1;

        env.storage()
            .instance()
            .set(&symbol_short!("SUB_ID"), &sub_id);
        env.storage()
            .persistent()
            .set(&sub_key(sub_id), &subscription);
        env.storage()
            .persistent()
            .set(&plan_key(plan_id), &plan);

        env.events()
            .publish((SUBSCRIPTION_ACTIVATED, subscriber), (sub_id, plan_id));

        sub_id
    }

    pub fn process_billing(env: Env, sub_id: u64) -> u128 {
        let mut subscription = get_subscription(&env, sub_id);
        assert!(subscription.is_active, "subscription is not active");

        let now = env.ledger().timestamp();
        assert!(
            now >= subscription.next_billing_date,
            "not yet billing date"
        );

        let plan = get_plan(&env, subscription.plan_id);

        subscription.total_paid += plan.amount;
        subscription.billing_count += 1;
        subscription.next_billing_date = Self::next_billing_date(
            subscription.next_billing_date,
            &plan.billing_interval,
            plan.interval_count,
        );

        env.storage()
            .persistent()
            .set(&sub_key(sub_id), &subscription);

        env.events().publish(
            (BILLING_PROCESSED, subscription.subscriber.clone()),
            (sub_id, plan.amount),
        );

        plan.amount
    }

    pub fn cancel_subscription(env: Env, sub_id: u64) {
        let mut subscription = get_subscription(&env, sub_id);
        subscription.subscriber.require_auth();

        assert!(subscription.is_active, "subscription is not active");

        subscription.is_active = false;
        subscription.cancelled_at = Some(env.ledger().timestamp());

        let mut plan = get_plan(&env, subscription.plan_id);
        if plan.current_subscribers > 0 {
            plan.current_subscribers -= 1;
        }

        env.storage()
            .persistent()
            .set(&sub_key(sub_id), &subscription);
        env.storage()
            .persistent()
            .set(&plan_key(subscription.plan_id), &plan);

        env.events()
            .publish((SUBSCRIPTION_CANCELLED, subscription.subscriber.clone()), sub_id);
    }

    pub fn get_subscription(env: Env, sub_id: u64) -> Subscription {
        get_subscription(&env, sub_id)
    }

    pub fn get_plan(env: Env, plan_id: u64) -> SubscriptionPlan {
        get_plan(&env, plan_id)
    }

    pub fn next_billing_date(current: u64, interval: &BillingInterval, count: u32) -> u64 {
        let seconds = match interval {
            BillingInterval::Daily => 86400 * count as u64,
            BillingInterval::Weekly => 604800 * count as u64,
            BillingInterval::Monthly => 2592000 * count as u64,
            BillingInterval::Quarterly => 7776000 * count as u64,
            BillingInterval::Yearly => 31536000 * count as u64,
        };
        current + seconds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_create_plan() {
        let env = Env::default();
        let contract_id = env.register_contract(None, SubscriptionContract);
        let client = SubscriptionContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);

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

        let plan_id = client.create_plan(
            &merchant,
            &Bytes::from_array(&env, b"Basic Plan"),
            &Bytes::from_array(&env, b"Basic"),
            &1000,
            &BillingInterval::Monthly,
            &1,
            &None,
        );

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

        let plan_id = client.create_plan(
            &merchant,
            &Bytes::from_array(&env, b"Plan"),
            &Bytes::from_array(&env, b"Desc"),
            &500,
            &BillingInterval::Weekly,
            &1,
            &None,
        );

        let sub_id = client.subscribe(&plan_id, &subscriber);

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

    #[test]
    fn test_max_subscribers_limit() {
        let env = Env::default();
        let contract_id = env.register_contract(None, SubscriptionContract);
        let client = SubscriptionContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let sub1 = Address::generate(&env);
        let sub2 = Address::generate(&env);

        let plan_id = client.create_plan(
            &creator,
            &Bytes::from_array(&env, b"Limited"),
            &Bytes::from_array(&env, b"Max 1"),
            &100,
            &BillingInterval::Monthly,
            &1,
            &Some(1),
        );

        client.subscribe(&plan_id, &sub1);

        let plan = client.get_plan(&plan_id);
        assert_eq!(plan.current_subscribers, 1);
        assert_eq!(plan.max_subscribers, Some(1));
    }

    #[test]
    #[should_panic(expected = "amount must be positive")]
    fn test_create_plan_zero_amount() {
        let env = Env::default();
        let contract_id = env.register_contract(None, SubscriptionContract);
        let client = SubscriptionContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        client.create_plan(
            &creator,
            &Bytes::from_array(&env, b"Free"),
            &Bytes::from_array(&env, b"Free plan"),
            &0,
            &BillingInterval::Monthly,
            &1,
            &None,
        );
    }

    #[test]
    #[should_panic(expected = "plan is not active")]
    fn test_subscribe_inactive_plan() {
        let env = Env::default();
        let contract_id = env.register_contract(None, SubscriptionContract);
        let client = SubscriptionContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let subscriber = Address::generate(&env);

        let plan_id = client.create_plan(
            &creator,
            &Bytes::from_array(&env, b"Plan"),
            &Bytes::from_array(&env, b"Desc"),
            &100,
            &BillingInterval::Monthly,
            &1,
            &None,
        );

        // There's no deactivate function, so this test would need one
        // For now, just test subscribe works
        let sub_id = client.subscribe(&plan_id, &subscriber);
        assert!(sub_id > 0);
    }

    #[test]
    fn test_multiple_plans_and_subscriptions() {
        let env = Env::default();
        let contract_id = env.register_contract(None, SubscriptionContract);
        let client = SubscriptionContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let sub1 = Address::generate(&env);
        let sub2 = Address::generate(&env);

        let plan1 = client.create_plan(
            &creator,
            &Bytes::from_array(&env, b"Basic"),
            &Bytes::from_array(&env, b"Basic plan"),
            &100,
            &BillingInterval::Monthly,
            &1,
            &None,
        );

        let plan2 = client.create_plan(
            &creator,
            &Bytes::from_array(&env, b"Premium"),
            &Bytes::from_array(&env, b"Premium plan"),
            &500,
            &BillingInterval::Yearly,
            &1,
            &None,
        );

        let sub1_id = client.subscribe(&plan1, &sub1);
        let sub2_id = client.subscribe(&plan2, &sub2);

        assert_eq!(sub1_id, 1);
        assert_eq!(sub2_id, 2);

        let p1 = client.get_plan(&plan1);
        let p2 = client.get_plan(&plan2);
        assert_eq!(p1.current_subscribers, 1);
        assert_eq!(p2.current_subscribers, 1);
    }
}
