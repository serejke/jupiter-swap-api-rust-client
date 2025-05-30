use serde::{Deserialize, Deserializer, Serialize};
use solana_sdk::pubkey::Pubkey;

use crate::serde_helpers::option_field_as_string;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum ComputeUnitPriceMicroLamports {
    MicroLamports(u64),
    #[serde(deserialize_with = "auto")]
    Auto,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
// #[serde(untagged)]
pub enum PrioritizationFeeLamports {
    /// Jupiter will automatically set a priority fee,
    /// and it will be capped at 5,000,000 lamports / 0.005 SOL
    #[serde(deserialize_with = "auto")]
    Auto,
    /// The priority fee will be a multiplier on the auto fee.
    AutoMultiplier(u64),
    /// A tip instruction will be included to Jito and no priority fee will be set.
    JitoTipLamports(u64)
}

fn auto<'de, D>(deserializer: D) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    enum Helper {
        #[serde(rename = "auto")]
        Variant,
    }
    Helper::deserialize(deserializer)?;
    Ok(())
}

#[derive(Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct TransactionConfig {
    /// Wrap and unwrap SOL. Will be ignored if `destination_token_account` is set because the `destination_token_account` may belong to a different user that we have no authority to close.
    pub wrap_and_unwrap_sol: bool,
    /// Fee token account for the output token, it is derived using the seeds = ["referral_ata", referral_account, mint] and the `REFER4ZgmyYx9c6He5XfaTMiGfdLwRnkV4RPp9t9iF3` referral contract (only pass in if you set a feeBps and make sure that the feeAccount has been created)
    #[serde(with = "option_field_as_string")]
    pub fee_account: Option<Pubkey>,
    /// Public key of the token account that will be used to receive the token out of the swap. If not provided, the user's ATA will be used. If provided, we assume that the token account is already initialized.
    #[serde(with = "option_field_as_string")]
    pub destination_token_account: Option<Pubkey>,
    /// compute unit price to prioritize the transaction, the additional fee will be compute unit consumed * computeUnitPriceMicroLamports
    pub compute_unit_price_micro_lamports: Option<ComputeUnitPriceMicroLamports>,
    /// Prioritization fee lamports paid for the transaction in addition to the signatures fee.
    /// Mutually exclusive with `compute_unit_price_micro_lamports`.
    pub prioritization_fee_lamports: Option<PrioritizationFeeLamports>,
    /// When enabled, it will do a swap simulation to get the compute unit used and set it in ComputeBudget's compute unit limit.
    /// This will increase latency slightly since there will be one extra RPC call to simulate this. Default is false.
    pub dynamic_compute_unit_limit: bool,
    /// Request a legacy transaction rather than the default versioned transaction, needs to be paired with a quote using asLegacyTransaction otherwise the transaction might be too large
    ///
    /// Default: false
    pub as_legacy_transaction: bool,
    /// This enables the usage of shared program accounts. That means no intermediate token accounts or open orders accounts need to be created.
    /// But it also means that the likelihood of hot accounts is higher.
    ///
    /// Default: true
    pub use_shared_accounts: bool,
    /// This is useful when the instruction before the swap has a transfer that increases the input token amount.
    /// Then, the swap will just use the difference between the token ledger token amount and post token amount.
    ///
    /// Default: false
    pub use_token_ledger: bool,
    /// Number of slots from the current blockhash to replace with a new blockhash.
    /// This can be used to ensure the transaction is confirmed before the blockhash expires.
    /// Default is 150 slots, which is around ~60 seconds.
    pub blockhash_slots_to_expiry: Option<u64>,
}

impl Default for TransactionConfig {
    fn default() -> Self {
        Self {
            wrap_and_unwrap_sol: true,
            fee_account: None,
            destination_token_account: None,
            compute_unit_price_micro_lamports: None,
            prioritization_fee_lamports: None,
            dynamic_compute_unit_limit: false,
            as_legacy_transaction: false,
            use_shared_accounts: true,
            use_token_ledger: false,
            blockhash_slots_to_expiry: None,
        }
    }
}
