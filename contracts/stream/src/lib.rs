#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

const STREAM_CREATED: Symbol = symbol_short!("STR_CREATED");
const STREAM_PAUSED: Symbol = symbol_short!("STR_PAUSED");
const STREAM_RESUMED: Symbol = symbol_short!("STR_RESUMED");
const STREAM_STOPPED: Symbol = symbol_short!("STR_STOPPED");
const WITHDRAWAL: Symbol = symbol_short!("WITHDRAWAL");

#[derive(Clone)]
#[contracttype]
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

#[derive(Clone)]
#[contracttype]
pub enum StreamStatus {
    Active,
    Paused,
    Completed,
    Cancelled,
}

fn stream_key(stream_id: u64) -> Symbol {
    match stream_id {
        1 => symbol_short!("S1"),
        2 => symbol_short!("S2"),
        3 => symbol_short!("S3"),
        4 => symbol_short!("S4"),
        5 => symbol_short!("S5"),
        6 => symbol_short!("S6"),
        7 => symbol_short!("S7"),
        8 => symbol_short!("S8"),
        _ => symbol_short!("S overflow"),
    }
}

fn get_stream(env: &Env, stream_id: u64) -> PaymentStream {
    env.storage()
        .persistent()
        .get(&stream_key(stream_id))
        .expect("stream not found")
}

#[contract]
pub struct PaymentStreamContract;

#[contractimpl]
impl PaymentStreamContract {
    pub fn create_stream(
        env: Env,
        sender: Address,
        receiver: Address,
        amount_per_second: u128,
        start_time: u64,
        end_time: u64,
    ) -> u64 {
        sender.require_auth();

        assert!(end_time > start_time, "end_time must be after start_time");
        assert!(amount_per_second > 0, "amount_per_second must be positive");
        assert!(sender != receiver, "sender and receiver cannot be the same");

        let stream_id = env
            .storage()
            .instance()
            .get::<_, u64>(&symbol_short!("STREAM_ID"))
            .unwrap_or(0)
            + 1;

        let stream = PaymentStream {
            id: stream_id,
            sender: sender.clone(),
            receiver,
            amount_per_second,
            start_time,
            end_time,
            total_streamed: 0,
            withdrawn: 0,
            is_active: true,
            is_paused: false,
            pause_time: None,
            cumulative_pause_duration: 0,
            created_at: env.ledger().timestamp(),
        };

        env.storage()
            .instance()
            .set(&symbol_short!("STREAM_ID"), &stream_id);
        env.storage()
            .persistent()
            .set(&stream_key(stream_id), &stream);

        env.events()
            .publish((STREAM_CREATED, sender), stream_id);

        stream_id
    }

    pub fn withdraw(env: Env, stream_id: u64, amount: Option<u128>) -> u128 {
        let mut stream = get_stream(&env, stream_id);
        stream.receiver.require_auth();

        assert!(stream.is_active, "stream is not active");

        let available = Self::get_withdrawable(env.clone(), stream_id);
        let withdraw_amount = match amount {
            Some(a) => {
                assert!(a <= available, "amount exceeds available");
                a
            }
            None => available,
        };

        assert!(withdraw_amount > 0, "no funds available to withdraw");

        stream.withdrawn += withdraw_amount;
        env.storage()
            .persistent()
            .set(&stream_key(stream_id), &stream);

        env.events()
            .publish((WITHDRAWAL, stream.receiver.clone()), (stream_id, withdraw_amount));

        withdraw_amount
    }

    pub fn pause_stream(env: Env, stream_id: u64) {
        let mut stream = get_stream(&env, stream_id);
        stream.sender.require_auth();

        assert!(stream.is_active, "stream is not active");
        assert!(!stream.is_paused, "stream is already paused");

        let now = env.ledger().timestamp();
        stream.is_paused = true;
        stream.pause_time = Some(now);

        env.storage()
            .persistent()
            .set(&stream_key(stream_id), &stream);

        env.events()
            .publish((STREAM_PAUSED, stream.sender.clone()), stream_id);
    }

    pub fn resume_stream(env: Env, stream_id: u64) {
        let mut stream = get_stream(&env, stream_id);
        stream.sender.require_auth();

        assert!(stream.is_active, "stream is not active");
        assert!(stream.is_paused, "stream is not paused");

        let now = env.ledger().timestamp();
        if let Some(pause_start) = stream.pause_time {
            stream.cumulative_pause_duration += now - pause_start;
        }
        stream.is_paused = false;
        stream.pause_time = None;

        env.storage()
            .persistent()
            .set(&stream_key(stream_id), &stream);

        env.events()
            .publish((STREAM_RESUMED, stream.sender.clone()), stream_id);
    }

    pub fn stop_stream(env: Env, stream_id: u64) {
        let mut stream = get_stream(&env, stream_id);
        stream.sender.require_auth();

        assert!(stream.is_active, "stream is not active");

        stream.is_active = false;
        stream.is_paused = false;

        env.storage()
            .persistent()
            .set(&stream_key(stream_id), &stream);

        env.events()
            .publish((STREAM_STOPPED, stream.sender.clone()), stream_id);
    }

    pub fn get_withdrawable(env: Env, stream_id: u64) -> u128 {
        let stream = get_stream(&env, stream_id);
        assert!(stream.is_active, "stream is not active");

        let now = env.ledger().timestamp();
        if now <= stream.start_time {
            return 0;
        }

        let effective_end = if now > stream.end_time {
            stream.end_time
        } else {
            now
        };

        let active_duration = effective_end - stream.start_time - stream.cumulative_pause_duration;
        let total = stream.amount_per_second * active_duration as u128;

        if total > stream.withdrawn {
            total - stream.withdrawn
        } else {
            0
        }
    }

    pub fn get_stream(env: Env, stream_id: u64) -> PaymentStream {
        get_stream(&env, stream_id)
    }

    pub fn get_status(env: Env, stream_id: u64) -> StreamStatus {
        let stream = get_stream(&env, stream_id);

        if !stream.is_active {
            return StreamStatus::Cancelled;
        }

        if stream.is_paused {
            return StreamStatus::Paused;
        }

        StreamStatus::Active
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_create_stream() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PaymentStreamContract);
        let client = PaymentStreamContractClient::new(&env, &contract_id);

        let sender = Address::generate(&env);
        let receiver = Address::generate(&env);

        let stream_id = client.create_stream(&sender, &receiver, &100, &1000, &2000);
        assert_eq!(stream_id, 1);

        let stream = client.get_stream(&stream_id);
        assert_eq!(stream.sender, sender);
        assert_eq!(stream.receiver, receiver);
        assert_eq!(stream.amount_per_second, 100);
        assert!(stream.is_active);
        assert!(!stream.is_paused);
    }

    #[test]
    fn test_pause_and_resume_stream() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PaymentStreamContract);
        let client = PaymentStreamContractClient::new(&env, &contract_id);

        let sender = Address::generate(&env);
        let receiver = Address::generate(&env);

        let stream_id = client.create_stream(&sender, &receiver, &100, &1000, &2000);

        client.pause_stream(&stream_id);
        let stream = client.get_stream(&stream_id);
        assert!(stream.is_paused);

        client.resume_stream(&stream_id);
        let stream = client.get_stream(&stream_id);
        assert!(!stream.is_paused);
    }

    #[test]
    fn test_stop_stream() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PaymentStreamContract);
        let client = PaymentStreamContractClient::new(&env, &contract_id);

        let sender = Address::generate(&env);
        let receiver = Address::generate(&env);

        let stream_id = client.create_stream(&sender, &receiver, &100, &1000, &2000);

        client.stop_stream(&stream_id);
        let stream = client.get_stream(&stream_id);
        assert!(!stream.is_active);

        let status = client.get_status(&stream_id);
        assert!(matches!(status, StreamStatus::Cancelled));
    }

    #[test]
    fn test_get_withdrawable_before_start() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PaymentStreamContract);
        let client = PaymentStreamContractClient::new(&env, &contract_id);

        let sender = Address::generate(&env);
        let receiver = Address::generate(&env);

        let stream_id = client.create_stream(&sender, &receiver, &100, &1000, &2000);

        let available = client.get_withdrawable(&stream_id);
        assert_eq!(available, 0);
    }

    #[test]
    #[should_panic(expected = "stream is not active")]
    fn test_withdraw_inactive_stream() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PaymentStreamContract);
        let client = PaymentStreamContractClient::new(&env, &contract_id);

        let sender = Address::generate(&env);
        let receiver = Address::generate(&env);

        let stream_id = client.create_stream(&sender, &receiver, &100, &1000, &2000);
        client.stop_stream(&stream_id);

        client.withdraw(&stream_id, &None);
    }

    #[test]
    #[should_panic(expected = "sender and receiver cannot be the same")]
    fn test_create_stream_same_sender_receiver() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PaymentStreamContract);
        let client = PaymentStreamContractClient::new(&env, &contract_id);

        let addr = Address::generate(&env);
        client.create_stream(&addr, &addr, &100, &1000, &2000);
    }

    #[test]
    #[should_panic(expected = "amount_per_second must be positive")]
    fn test_create_stream_zero_amount() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PaymentStreamContract);
        let client = PaymentStreamContractClient::new(&env, &contract_id);

        let sender = Address::generate(&env);
        let receiver = Address::generate(&env);
        client.create_stream(&sender, &receiver, &0, &1000, &2000);
    }

    #[test]
    #[should_panic(expected = "end_time must be after start_time")]
    fn test_create_stream_invalid_times() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PaymentStreamContract);
        let client = PaymentStreamContractClient::new(&env, &contract_id);

        let sender = Address::generate(&env);
        let receiver = Address::generate(&env);
        client.create_stream(&sender, &receiver, &100, &2000, &1000);
    }

    #[test]
    fn test_multiple_streams() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PaymentStreamContract);
        let client = PaymentStreamContractClient::new(&env, &contract_id);

        let sender = Address::generate(&env);
        let receiver = Address::generate(&env);

        let id1 = client.create_stream(&sender, &receiver, &100, &1000, &2000);
        let id2 = client.create_stream(&sender, &receiver, &200, &1000, &3000);

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);

        let s1 = client.get_stream(&id1);
        let s2 = client.get_stream(&id2);
        assert_eq!(s1.amount_per_second, 100);
        assert_eq!(s2.amount_per_second, 200);
    }

    #[test]
    fn test_stream_status() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PaymentStreamContract);
        let client = PaymentStreamContractClient::new(&env, &contract_id);

        let sender = Address::generate(&env);
        let receiver = Address::generate(&env);

        let stream_id = client.create_stream(&sender, &receiver, &100, &1000, &2000);

        let status = client.get_status(&stream_id);
        assert!(matches!(status, StreamStatus::Active));

        client.pause_stream(&stream_id);
        let status = client.get_status(&stream_id);
        assert!(matches!(status, StreamStatus::Paused));

        client.resume_stream(&stream_id);
        let status = client.get_status(&stream_id);
        assert!(matches!(status, StreamStatus::Active));

        client.stop_stream(&stream_id);
        let status = client.get_status(&stream_id);
        assert!(matches!(status, StreamStatus::Cancelled));
    }
}
