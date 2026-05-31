#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env};

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    InvalidAmount = 2,
    OfferNotFound = 3,
    OfferCancelled = 4,
    FillExceedsRemaining = 5,
    Unauthorized = 6,
    FillBelowMinimum = 7,
}

mod storage {
    use soroban_sdk::Env;

    const INSTANCE_BUMP_AMOUNT: u32 = 16777215;
    const INSTANCE_LIFETIME_THRESHOLD: u32 = 10368000;

    const OFFERS_PREFIX: &str = "offers";
    const OFFER_COUNT_KEY: &str = "offer_count";
    const INITIALIZED_KEY: &str = "initialized";

    pub fn extend_ttl(env: &Env) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
    }

    pub fn is_initialized(env: &Env) -> bool {
        env.storage()
            .instance()
            .get::<_, bool>(&INITIALIZED_KEY)
            .unwrap_or(false)
    }

    pub fn set_initialized(env: &Env) {
        env.storage().instance().set(&INITIALIZED_KEY, &true);
    }

    pub fn read_offer_count(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get::<_, u64>(&OFFER_COUNT_KEY)
            .unwrap_or(0)
    }

    pub fn write_offer_count(env: &Env, count: u64) {
        env.storage().instance().set(&OFFER_COUNT_KEY, &count);
    }

    pub fn store_offer(env: &Env, offer_id: u64, offer: &super::Offer) {
        let key = (OFFERS_PREFIX.as_bytes(), offer_id);
        env.storage().instance().set(&key, offer);
    }

    pub fn get_offer(env: &Env, offer_id: u64) -> Option<super::Offer> {
        let key = (OFFERS_PREFIX.as_bytes(), offer_id);
        env.storage().instance().get(&key)
    }

    pub fn remove_offer(env: &Env, offer_id: u64) {
        let key = (OFFERS_PREFIX.as_bytes(), offer_id);
        env.storage().instance().remove(&key);
    }
}

mod events {
    use soroban_sdk::{contracttype, Address};

    #[derive(Clone)]
    #[contracttype]
    pub struct OfferCreatedEvent {
        pub offer_id: u64,
        pub seller: Address,
        pub carbon_amount: i128,
        pub usdc_amount: i128,
    }

    #[derive(Clone)]
    #[contracttype]
    pub struct OfferFilledEvent {
        pub offer_id: u64,
        pub buyer: Address,
        pub filled_carbon: i128,
        pub filled_usdc: i128,
    }

    #[derive(Clone)]
    #[contracttype]
    pub struct OfferCancelledEvent {
        pub offer_id: u64,
        pub seller: Address,
        pub remaining_carbon: i128,
    }
}

#[derive(Clone)]
#[contracttype]
pub struct Offer {
    pub offer_id: u64,
    pub seller: Address,
    pub carbon_amount: i128,
    pub usdc_amount: i128,
    pub filled_carbon: i128,
    pub filled_usdc: i128,
    pub carbon_token: Address,
    pub usdc_token: Address,
    pub is_cancelled: bool,
    pub min_fill_amount: i128,
}

impl Offer {
    pub fn remaining_carbon(&self) -> i128 {
        self.carbon_amount - self.filled_carbon
    }

    pub fn remaining_usdc(&self) -> i128 {
        self.usdc_amount - self.filled_usdc
    }

    pub fn is_fully_filled(&self) -> bool {
        self.filled_carbon >= self.carbon_amount
    }
}

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Initialize the escrow contract
    pub fn initialize(env: Env) -> Result<(), Error> {
        storage::extend_ttl(&env);
        if storage::is_initialized(&env) {
            return Err(Error::AlreadyInitialized);
        }
        storage::set_initialized(&env);
        storage::write_offer_count(&env, 0);
        Ok(())
    }

    /// Create a new offer - seller deposits Carbon tokens into escrow
    /// Returns the offer_id
    pub fn create_offer(
        env: Env,
        seller: Address,
        carbon_amount: i128,
        usdc_amount: i128,
        carbon_token: Address,
        usdc_token: Address,
        min_fill_amount: i128,
    ) -> Result<u64, Error> {
        seller.require_auth();

        if carbon_amount <= 0 || usdc_amount <= 0 || min_fill_amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        if min_fill_amount <= 0 {
            panic!("min_fill_amount must be positive");
        }
        if min_fill_amount > carbon_amount {
            panic!("min_fill_amount cannot exceed carbon_amount");
        }

        storage::extend_ttl(&env);

        let offer_id = storage::read_offer_count(&env) + 1;
        storage::write_offer_count(&env, offer_id);

        let offer = Offer {
            offer_id,
            seller: seller.clone(),
            carbon_amount,
            usdc_amount,
            filled_carbon: 0,
            filled_usdc: 0,
            carbon_token: carbon_token.clone(),
            usdc_token: usdc_token.clone(),
            is_cancelled: false,
            min_fill_amount,
        };

        storage::store_offer(&env, offer_id, &offer);

        let carbon_client = soroban_sdk::token::Client::new(&env, &carbon_token);
        carbon_client.transfer(&seller, &env.current_contract_address(), &carbon_amount);

        env.events().publish(
            ("offer_created",),
            (offer_id, seller.clone(), carbon_amount, usdc_amount),
        );

        Ok(offer_id)
    }

    /// Fill an offer - buyer pays USDC and receives Carbon tokens
    /// Supports partial fills - amount specifies how much carbon to buy
    pub fn fill_offer(
        env: Env,
        offer_id: u64,
        buyer: Address,
        fill_carbon_amount: i128,
    ) -> Result<(), Error> {
        buyer.require_auth();

        if fill_carbon_amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        storage::extend_ttl(&env);

        let mut offer = storage::get_offer(&env, offer_id).ok_or(Error::OfferNotFound)?;

        if offer.is_cancelled {
            return Err(Error::OfferCancelled);
        }

        let remaining_carbon = offer.remaining_carbon();
        if fill_carbon_amount > remaining_carbon {
            return Err(Error::FillExceedsRemaining);
        }

        // Allow fills below minimum only when consuming the entire remaining amount
        if fill_carbon_amount < offer.min_fill_amount && fill_carbon_amount < remaining_carbon {
            return Err(Error::FillBelowMinimum);
        }

        // Calculate proportional USDC amount, rounding up in favor of the seller
        let fill_usdc_amount = (fill_carbon_amount * offer.usdc_amount + offer.carbon_amount - 1)
            / offer.carbon_amount;

        // Transfer USDC from buyer to escrow
        let usdc_client = soroban_sdk::token::Client::new(&env, &offer.usdc_token);
        usdc_client.transfer(&buyer, &env.current_contract_address(), &fill_usdc_amount);

        let carbon_client = soroban_sdk::token::Client::new(&env, &offer.carbon_token);
        carbon_client.transfer(&env.current_contract_address(), &buyer, &fill_carbon_amount);

        // Transfer USDC from escrow to seller
        usdc_client.transfer(
            &env.current_contract_address(),
            &offer.seller,
            &fill_usdc_amount,
        );

        offer.filled_carbon += fill_carbon_amount;
        offer.filled_usdc += fill_usdc_amount;

        if offer.is_fully_filled() {
            storage::remove_offer(&env, offer_id);
        } else {
            storage::store_offer(&env, offer_id, &offer);
        }

        env.events().publish(
            ("offer_filled",),
            (
                offer_id,
                buyer.clone(),
                fill_carbon_amount,
                fill_usdc_amount,
            ),
        );

        Ok(())
    }

    /// Cancel an offer - only the seller can cancel
    /// Returns remaining carbon tokens to seller
    pub fn cancel_offer(env: Env, offer_id: u64, caller: Address) -> Result<(), Error> {
        caller.require_auth();

        storage::extend_ttl(&env);

        let mut offer = storage::get_offer(&env, offer_id).ok_or(Error::OfferNotFound)?;

        if caller != offer.seller {
            return Err(Error::Unauthorized);
        }

        if offer.is_cancelled {
            return Err(Error::OfferCancelled);
        }

        let remaining_carbon = offer.remaining_carbon();
        if remaining_carbon > 0 {
            let carbon_client = soroban_sdk::token::Client::new(&env, &offer.carbon_token);
            carbon_client.transfer(
                &env.current_contract_address(),
                &offer.seller,
                &remaining_carbon,
            );
        }

        offer.is_cancelled = true;
        storage::store_offer(&env, offer_id, &offer);

        env.events().publish(
            ("offer_cancelled",),
            (offer_id, offer.seller.clone(), remaining_carbon),
        );

        Ok(())
    }

    /// Get offer details
    pub fn get_offer(env: Env, offer_id: u64) -> Option<Offer> {
        storage::extend_ttl(&env);
        storage::get_offer(&env, offer_id)
    }

    /// Get remaining amount for an offer
    pub fn get_remaining_amount(env: Env, offer_id: u64) -> (i128, i128) {
        storage::extend_ttl(&env);
        if let Some(offer) = storage::get_offer(&env, offer_id) {
            (offer.remaining_carbon(), offer.remaining_usdc())
        } else {
            (0, 0)
        }
    }
}

mod test;
