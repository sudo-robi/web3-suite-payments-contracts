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

#[contract]
pub struct SubscriptionContract;

#[contractimpl]
impl SubscriptionContract {
    /// Creates a new subscription plan.
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

        let plan_id = env.storage().instance().get::<_, u64>(&symbol_short!("PLAN_ID")).unwrap_or(0) + 1;

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

        env.storage().instance().set(&symbol_short!("PLAN_ID"), &plan_id);
        env.storage().persistent().set(&plan_key(plan_id), &plan);

        env.events()
            .publish((PLAN_CREATED, creator), plan_id);

        plan_id
    }

    /// Subscribes a user to a plan.
    pub fn subscribe(env: Env, plan_id: u64, subscriber: Address) -> u64 {
        subscriber.require_auth();

        let mut plan = get_plan(&env, plan_id);
        assert!(plan.is_active, "plan is not active");

        if let Some(max) = plan.max_subscribers {
            assert!(plan.current_subscribers < max, "plan has reached max subscribers");
        }

        let now = env.ledger().timestamp();
        let next_billing = Self::next_billing_date(now, &plan.billing_interval, plan.interval_count);

        let sub_id = env.storage().instance().get::<_, u64>(&symbol_short!("SUB_ID")).unwrap_or(0) + 1;

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

        env.storage().instance().set(&symbol_short!("SUB_ID"), &sub_id);
        env.storage().persistent().set(&sub_key(sub_id), &subscription);
        env.storage().persistent().set(&plan_key(plan_id), &plan);

        env.events()
            .publish((SUBSCRIPTION_ACTIVATED, subscriber), (sub_id, plan_id));

        sub_id
    }

    /// Processes billing for a subscription.
    pub fn process_billing(env: Env, sub_id: u64) -> u128 {
        let mut subscription = get_subscription(&env, sub_id);
        assert!(subscription.is_active, "subscription is not active");

        let now = env.ledger().timestamp();
        assert!(now >= subscription.next_billing_date, "not yet billing date");

        let plan = get_plan(&env, subscription.plan_id);

        // Transfer payment from subscriber to merchant
        env.transfer旅途(&subscription.subscriber, &subscription.merchant, plan.amount);

        subscription.total_paid += plan.amount;
        subscription.billing_count += 1;
        subscription.next_billing_date = Self::next_billing_date(
            subscription.next_billing_date,
            &plan.billing_interval,
            plan.interval_count,
        );

        env.storage().persistent().set(&sub_key(sub_id), &subscription);

        env.events()
            .publish((BILLING_PROCESSED, subscription.subscriber.clone()), (sub_id, plan.amount));

        plan.amount
    }

    /// Cancels a subscription.
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

        env.storage().persistent().set(&sub_key(sub_id), &subscription);
        env.storage().persistent().set(&plan_key(subscription.plan_id), &plan);

        env.events()
            .publish((SUBSCRIPTION_CANCELLED, subscription.subscriber.clone()), sub_id);
    }

    /// Returns subscription details.
    pub fn get_subscription(env: Env, sub_id: u64) -> Subscription {
        get_subscription(&env, sub_id)
    }

    /// Returns plan details.
    pub fn get_plan(env: Env, plan_id: u64) -> SubscriptionPlan {
        get_plan(&env, plan_id)
    }

    /// Returns the next billing date based on interval.
    pub fn next_billing_date(current: u64, interval: &BillingInterval, count: u32) -> u64 {
        let seconds = match interval {
            BillingInterval::Daily => 86400 * count as u64,
            BillingInterval::Weekly => 604800 * count as u64,
            BillingInterval::Monthly => 2592000 * count as u64, // 30 days
            BillingInterval::Quarterly => 7776000 * count as u64, // 90 days
            BillingInterval::Yearly => 31536000 * count as u64, // 365 days
        };
        current + seconds
    }

    /// Lists all subscriptions for an address.
    pub fn list_subscriptions(env: Env, address: Address) -> soroban_sdk::Vec<u64> {
        let mut subs = soroban_sdk::Vec::new(&env);
        let all_ids = get_all_sub_ids(&env);

        for i in 0..all_ids.len() {
            let id = all_ids.get_unchecked(i);
            if let Ok(sub) = env.storage().persistent().get::<_, Subscription>(&sub_key(id)) {
                if sub.subscriber == address || sub.merchant == address {
                    subs.push_back(id);
                }
            }
        }

        subs
    }
}

fn plan_key(plan_id: u64) -> Symbol {
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&plan_id.to_be_bytes());
    symbol_short!("PLAN").into_val(&Env::default())
}

fn sub_key(sub_id: u64) -> Symbol {
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&sub_id.to_be_bytes());
    symbol_short!("SUB").into_val(&Env::default())
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

fn get_all_sub_ids(env: &Env) -> soroban_sdk::Vec<u64> {
    env.storage()
        .instance()
        .get(&symbol_short!("ALL_SUB_IDS"))
        .unwrap_or(soroban_sdk::Vec::new(env))
}


