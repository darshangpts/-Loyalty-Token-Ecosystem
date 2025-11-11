#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Address, Symbol, symbol_short};

// Store merchant rewards data
#[contracttype]
#[derive(Clone)]
pub struct MerchantData {
    pub merchant_addr: Address,
    pub total_points_issued: u64,
    pub is_active: bool,
}

// Store user loyalty points
#[contracttype]
#[derive(Clone)]
pub struct UserBalance {
    pub user_addr: Address,
    pub points: u64,
}

// Mapping for merchants
#[contracttype]
pub enum MerchantBook {
    Merchant(Address)
}

// Mapping for user balances
#[contracttype]
pub enum UserBook {
    User(Address)
}

// Symbol for total points in circulation
const TOTAL_POINTS: Symbol = symbol_short!("T_POINTS");

#[contract]
pub struct LoyaltyTokenContract;

#[contractimpl]
impl LoyaltyTokenContract {
    
    // Function 1: Register a new merchant to the loyalty ecosystem
    pub fn register_merchant(env: Env, merchant: Address) {
        merchant.require_auth();
        
        let merchant_key = MerchantBook::Merchant(merchant.clone());
        
        // Check if merchant already exists
        let existing: Option<MerchantData> = env.storage().instance().get(&merchant_key);
        
        if existing.is_some() {
            log!(&env, "Merchant already registered");
            panic!("Merchant already registered");
        }
        
        // Create new merchant data
        let merchant_data = MerchantData {
            merchant_addr: merchant.clone(),
            total_points_issued: 0,
            is_active: true,
        };
        
        // Store merchant data
        env.storage().instance().set(&merchant_key, &merchant_data);
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Merchant registered successfully: {:?}", merchant);
    }
    
    // Function 2: Issue loyalty points to a user (only by registered merchants)
    pub fn issue_points(env: Env, merchant: Address, user: Address, points: u64) {
        merchant.require_auth();
        
        // Verify merchant is registered and active
        let merchant_key = MerchantBook::Merchant(merchant.clone());
        let mut merchant_data: MerchantData = env.storage().instance()
            .get(&merchant_key)
            .unwrap_or_else(|| panic!("Merchant not registered"));
        
        if !merchant_data.is_active {
            panic!("Merchant is not active");
        }
        
        // Get or create user balance
        let user_key = UserBook::User(user.clone());
        let mut user_balance: UserBalance = env.storage().instance()
            .get(&user_key)
            .unwrap_or(UserBalance {
                user_addr: user.clone(),
                points: 0,
            });
        
        // Update balances
        user_balance.points += points;
        merchant_data.total_points_issued += points;
        
        // Update total points in circulation
        let mut total_points: u64 = env.storage().instance()
            .get(&TOTAL_POINTS)
            .unwrap_or(0);
        total_points += points;
        
        // Save updated data
        env.storage().instance().set(&user_key, &user_balance);
        env.storage().instance().set(&merchant_key, &merchant_data);
        env.storage().instance().set(&TOTAL_POINTS, &total_points);
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Issued {} points to user from merchant", points);
    }
    
    // Function 3: Redeem loyalty points at any registered merchant
    pub fn redeem_points(env: Env, user: Address, merchant: Address, points: u64) {
        user.require_auth();
        
        // Verify merchant is registered and active
        let merchant_key = MerchantBook::Merchant(merchant.clone());
        let merchant_data: MerchantData = env.storage().instance()
            .get(&merchant_key)
            .unwrap_or_else(|| panic!("Merchant not registered"));
        
        if !merchant_data.is_active {
            panic!("Merchant is not active");
        }
        
        // Get user balance
        let user_key = UserBook::User(user.clone());
        let mut user_balance: UserBalance = env.storage().instance()
            .get(&user_key)
            .unwrap_or_else(|| panic!("User has no loyalty points"));
        
        // Check if user has enough points
        if user_balance.points < points {
            panic!("Insufficient loyalty points");
        }
        
        // Deduct points from user
        user_balance.points -= points;
        
        // Update total points in circulation
        let mut total_points: u64 = env.storage().instance()
            .get(&TOTAL_POINTS)
            .unwrap_or(0);
        total_points -= points;
        
        // Save updated data
        env.storage().instance().set(&user_key, &user_balance);
        env.storage().instance().set(&TOTAL_POINTS, &total_points);
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "User redeemed {} points at merchant", points);
    }
    
    // Function 4: View user's loyalty point balance
    pub fn view_user_balance(env: Env, user: Address) -> u64 {
        let user_key = UserBook::User(user.clone());
        let user_balance: UserBalance = env.storage().instance()
            .get(&user_key)
            .unwrap_or(UserBalance {
                user_addr: user.clone(),
                points: 0,
            });
        
        user_balance.points
    }
}
