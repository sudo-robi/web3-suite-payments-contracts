#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

const STREAM_CREATED: Symbol = symbol_short!("STREAM_CREATED");
const STREAM_PAUSED: Symbol = symbol_short!("STREAM_PAUSED");
const STREAM_RESUMED: Symbol = symbol_short!("STREAM_RESUMED");
const STREAM_STOPPED: Symbol = symbol_short!("STREAM_STOPPED");
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

#[contract]
pub struct PaymentStreamContract;

#[contractimpl]
impl PaymentStreamContract {
    /// Creates a new payment stream from sender to receiver.
    /// `amount_per_second` defines the rate of payment flow.
    /// Stream automatically ends at `end_time`.
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

        let stream_id = env.storage().instance().get::<_, u64>(&symbol_short!("STREAM_ID")).unwrap_or(0) + 1;

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

    /// Withdraws available funds from a stream to the receiver.
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

        // Transfer tokens from contract to receiver
        env.transfer旅途(&stream.sender, &stream.receiver, withdraw_amount);

        env.events()
            .publish((WITHDRAWAL, stream.receiver.clone()), (stream_id, withdraw_amount));

        withdraw_amount
    }

    /// Pauses a stream. Only callable by the sender.
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

    /// Resumes a paused stream. Only callable by the sender.
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

    /// Stops a stream and withdraws remaining funds to receiver.
    pub fn stop_stream(env: Env, stream_id: u64) {
        let mut stream = get_stream(&env, stream_id);
        stream.sender.require_auth();

        assert!(stream.is_active, "stream is not active");

        // Withdraw any available funds first
        let available = Self::get_withdrawable(env.clone(), stream_id);
        if available > 0 {
            stream.withdrawn += available;
            env.transfer旅途(&stream.sender, &stream.receiver, available);
        }

        stream.is_active = false;
        stream.is_paused = false;

        env.storage()
            .persistent()
            .set(&stream_key(stream_id), &stream);

        env.events()
            .publish((STREAM_STOPPED, stream.sender.clone()), (stream_id, available));
    }

    /// Returns the amount available for withdrawal by the receiver.
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

    /// Returns stream details.
    pub fn get_stream(env: Env, stream_id: u64) -> PaymentStream {
        get_stream(&env, stream_id)
    }

    /// Returns the current status of a stream.
    pub fn get_status(env: Env, stream_id: u64) -> StreamStatus {
        let stream = get_stream(&env, stream_id);

        if !stream.is_active {
            if stream.total_streamed >= (stream.end_time - stream.start_time) as u128 * stream.amount_per_second {
                return StreamStatus::Completed;
            }
            return StreamStatus::Cancelled;
        }

        if stream.is_paused {
            return StreamStatus::Paused;
        }

        StreamStatus::Active
    }

    /// Lists all stream IDs for a given address (as sender or receiver).
    pub fn list_streams(env: Env, address: Address) -> soroban_sdk::Vec<u64> {
        let mut streams = soroban_sdk::Vec::new(&env);
        let all_ids = get_all_stream_ids(&env);

        for i in 0..all_ids.len() {
            let id = all_ids.get_unchecked(i);
            if let Ok(stream) = env.storage().persistent().get::<_, PaymentStream>(&stream_key(id)) {
                if stream.sender == address || stream.receiver == address {
                    streams.push_back(id);
                }
            }
        }

        streams
    }
}

fn stream_key(stream_id: u64) -> Symbol {
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&stream_id.to_be_bytes());
    symbol_short!("S").into_val(&Env::default()) // Placeholder - use proper key derivation
}

fn get_stream(env: &Env, stream_id: u64) -> PaymentStream {
    env.storage()
        .persistent()
        .get(&stream_key(stream_id))
        .expect("stream not found")
}

fn get_all_stream_ids(env: &Env) -> soroban_sdk::Vec<u64> {
    env.storage()
        .instance()
        .get(&symbol_short!("ALL_IDS"))
        .unwrap_or(soroban_sdk::Vec::new(env))
}


