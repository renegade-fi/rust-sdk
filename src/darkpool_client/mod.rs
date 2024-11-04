//! A client for interacting with the darkpool

use ethers::signers::LocalWallet;
use eyre::Result;
use num_bigint::BigUint;
use renegade_circuit_types::{fixed_point::FixedPoint, max_price, order::OrderSide, Amount};
use renegade_common::types::wallet::{
    derivation::{
        derive_blinder_seed, derive_share_seed, derive_wallet_id, derive_wallet_keychain,
    },
    keychain::KeyChain,
    Order, WalletIdentifier,
};
use renegade_constants::Scalar;

use crate::{http::RelayerHttpClient, wrap_eyre};

mod auth;
mod wallet_ops;

/// The Arbitrum Sepolia chain ID
const ARBITRUM_SEPOLIA_CHAIN_ID: u64 = 421611;
/// The Arbitrum mainnet chain ID
const ARBITRUM_MAINNET_CHAIN_ID: u64 = 42161;
/// The base URL for the Renegade relayer in testnet
const TESTNET_BASE_URL: &str = "https://testnet.cluster0.renegade.fi:3000";
/// The base URL for the Renegade relayer in mainnet
const MAINNET_BASE_URL: &str = "https://mainnet.cluster0.renegade.fi:3000";

/// The secrets held by a wallet
#[derive(Clone)]
pub struct WalletSecrets {
    /// The wallet's ID
    wallet_id: WalletIdentifier,
    /// The wallet's blinder seed
    blinder_seed: Scalar,
    /// The wallet's secret share seed
    share_seed: Scalar,
    /// The wallet keychain
    keychain: KeyChain,
}

impl WalletSecrets {
    /// Derive a set of wallet secrets from an ethereum private key
    pub fn from_ethereum_pkey(chain_id: u64, pkey: &LocalWallet) -> Result<Self> {
        // Derive the seeds and keychain
        let wallet_id = wrap_eyre!(derive_wallet_id(pkey))?;
        let blinder_seed = wrap_eyre!(derive_blinder_seed(pkey))?;
        let share_seed = wrap_eyre!(derive_share_seed(pkey))?;
        let keychain = wrap_eyre!(derive_wallet_keychain(pkey, chain_id))?;

        Ok(Self { wallet_id, blinder_seed, share_seed, keychain })
    }
}

/// The darkpool client for interacting with a Renegade relayer
#[derive(Clone)]
pub struct DarkpoolClient {
    /// The wallet secrets
    wallet_secrets: WalletSecrets,
    /// The HTTP client
    http_client: RelayerHttpClient,
}

impl DarkpoolClient {
    /// Create a new darkpool client using the given Ethereum private key to
    /// derive the wallet secrets
    pub fn new(chain_id: u64, base_url: &str, pkey: &LocalWallet) -> Result<Self> {
        // Derive the wallet secrets
        let wallet_secrets = wrap_eyre!(WalletSecrets::from_ethereum_pkey(chain_id, pkey))?;

        // Create the client
        let hmac_key = wallet_secrets.keychain.secret_keys.symmetric_key;
        let http_client = RelayerHttpClient::new(base_url.to_string(), hmac_key);
        Ok(Self { wallet_secrets, http_client })
    }

    /// Create a new Arbitrum Sepolia darkpool client
    pub fn new_arbitrum_sepolia(pkey: &LocalWallet) -> Result<Self> {
        Self::new(ARBITRUM_SEPOLIA_CHAIN_ID, TESTNET_BASE_URL, pkey)
    }

    /// Create a new Arbitrum mainnet darkpool client
    pub fn new_arbitrum_mainnet(pkey: &LocalWallet) -> Result<Self> {
        Self::new(ARBITRUM_MAINNET_CHAIN_ID, MAINNET_BASE_URL, pkey)
    }
}

/// An order builder for creating orders in the darkpool
pub struct OrderBuilder {
    /// The quote mint (token address)
    quote_mint: Option<String>,
    /// The base mint (token address)
    base_mint: Option<String>,
    /// The side of the order
    side: Option<OrderSide>,
    /// The amount of the order
    amount: Option<Amount>,
    /// The worst case price for the order
    worst_case_price: Option<FixedPoint>,
    /// The minimum fill size for the order
    min_fill_size: Option<Amount>,
    /// Whether to allow external matches
    allow_external_matches: Option<bool>,
}

impl OrderBuilder {
    /// Create a new order builder
    pub fn new() -> Self {
        Self {
            quote_mint: None,
            base_mint: None,
            side: None,
            amount: None,
            worst_case_price: None,
            min_fill_size: None,
            allow_external_matches: None,
        }
    }

    /// Set the quote mint (token address) as a hex string
    pub fn quote_mint(mut self, quote_mint: impl ToString) -> Self {
        self.quote_mint = Some(quote_mint.to_string());
        self
    }

    /// Set the base mint (token address) as a hex string
    pub fn base_mint(mut self, base_mint: impl ToString) -> Self {
        self.base_mint = Some(base_mint.to_string());
        self
    }

    /// Set whether this is a buy or sell order
    pub fn side(mut self, side: OrderSide) -> Self {
        self.side = Some(side);
        self
    }

    /// Set the order amount
    pub fn amount(mut self, amount: Amount) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Set the worst case price for the order
    pub fn worst_case_price(mut self, price: FixedPoint) -> Self {
        self.worst_case_price = Some(price);
        self
    }

    /// Set the minimum fill size
    pub fn min_fill_size(mut self, size: Amount) -> Self {
        self.min_fill_size = Some(size);
        self
    }

    /// Set whether to allow external matches
    pub fn allow_external_matches(mut self, allow: bool) -> Self {
        self.allow_external_matches = Some(allow);
        self
    }

    /// Build the order
    pub fn build(self) -> Result<Order, String> {
        let quote_mint = self.quote_mint.ok_or("Quote mint is required")?;
        let base_mint = self.base_mint.ok_or("Base mint is required")?;

        // Parse hex strings to BigUint
        let quote_mint = BigUint::parse_bytes(quote_mint.trim_start_matches("0x").as_bytes(), 16)
            .ok_or("Invalid quote mint hex string")?;
        let base_mint = BigUint::parse_bytes(base_mint.trim_start_matches("0x").as_bytes(), 16)
            .ok_or("Invalid base mint hex string")?;

        let side = self.side.ok_or("Side is required")?;
        let amount = self.amount.ok_or("Amount is required")?;
        let worst_case_price = self.worst_case_price.unwrap_or_else(|| match side {
            OrderSide::Buy => max_price(),
            OrderSide::Sell => FixedPoint::from_integer(0),
        });
        let min_fill_size = self.min_fill_size.unwrap_or(0);
        let allow_external_matches = self.allow_external_matches.unwrap_or(false);

        Order::new(
            quote_mint,
            base_mint,
            side,
            amount,
            worst_case_price,
            min_fill_size,
            allow_external_matches,
        )
    }
}

impl Default for OrderBuilder {
    fn default() -> Self {
        Self::new()
    }
}
