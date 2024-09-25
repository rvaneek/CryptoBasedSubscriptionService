#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, symbol_short};

// Structure to store Subscription details
#[contracttype]
#[derive(Clone)]
pub struct Subscription {
    pub sub_id: u64,
    pub user: String,
    pub provider: String,
    pub start_time: u64,
    pub end_time: u64,
    pub is_active: bool,
}

// Structure to store Subscription Status for admin view
#[contracttype]
#[derive(Clone)]
pub struct SubscriptionStatus {
    pub active: u64,
    pub inactive: u64,
    pub total: u64,
}

// Symbol for storage keys
const ALL_SUBSCRIPTION_STATUS: Symbol = symbol_short!("ALSUBSTA");
const SUBSCRIPTION_COUNT: Symbol = symbol_short!("SUB_COUNT");

// Mapping subscription ID to Subscription details
#[contracttype]
pub enum SubscriptionBook {
    Subscription(u64),
}

#[contract]
pub struct CryptoSubscriptionContract;

#[contractimpl]
impl CryptoSubscriptionContract {
    // Creates a new subscription
    pub fn create_subscription(env: Env, user: String, provider: String, duration: u64) -> u64 {
        let mut sub_count: u64 = env.storage().instance().get(&SUBSCRIPTION_COUNT).unwrap_or(0);
        sub_count += 1;

        let time = env.ledger().timestamp();
        let end_time = time + duration;

        let new_sub = Subscription {
            sub_id: sub_count,
            user: user,
            provider: provider,
            start_time: time,
            end_time: end_time,
            is_active: true,
        };

        // Update subscription status
        let mut status = Self::view_subscription_status(env.clone());
        status.active += 1;
        status.total += 1;

        env.storage().instance().set(&SubscriptionBook::Subscription(sub_count), &new_sub);
        env.storage().instance().set(&ALL_SUBSCRIPTION_STATUS, &status);
        env.storage().instance().set(&SUBSCRIPTION_COUNT, &sub_count);

        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "Subscription Created with ID: {}", sub_count);

        sub_count
    }

    // Activate or deactivate a subscription
    pub fn update_subscription(env: Env, sub_id: u64, is_active: bool) {
        let mut subscription = Self::view_subscription(env.clone(), sub_id);

        if subscription.is_active != is_active {
            let mut status = Self::view_subscription_status(env.clone());
            if is_active {
                status.active += 1;
                status.inactive -= 1;
            } else {
                status.active -= 1;
                status.inactive += 1;
            }

            subscription.is_active = is_active;
            env.storage().instance().set(&SubscriptionBook::Subscription(sub_id), &subscription);
            env.storage().instance().set(&ALL_SUBSCRIPTION_STATUS, &status);

            env.storage().instance().extend_ttl(5000, 5000);

            log!(&env, "Subscription ID: {}, status updated to: {}", sub_id, if is_active { "Active" } else { "Inactive" });
        } else {
            log!(&env, "No change in subscription status for ID: {}", sub_id);
        }
    }

    // View subscription details by ID
    pub fn view_subscription(env: Env, sub_id: u64) -> Subscription {
        env.storage().instance().get(&SubscriptionBook::Subscription(sub_id)).unwrap_or(Subscription {
            sub_id: 0,
            user: String::from_str(&env, "Not Found"),
            provider: String::from_str(&env, "Not Found"),
            start_time: 0,
            end_time: 0,
            is_active: false,
        })
    }

    // View subscription status for admin
    pub fn view_subscription_status(env: Env) -> SubscriptionStatus {
        env.storage().instance().get(&ALL_SUBSCRIPTION_STATUS).unwrap_or(SubscriptionStatus {
            active: 0,
            inactive: 0,
            total: 0,
        })
    }
}
