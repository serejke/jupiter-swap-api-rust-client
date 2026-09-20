use serde::{Deserialize, Serialize};
use solana_pubkey::Pubkey;

use crate::serde_helpers::{field_as_string, option_field_as_string};

/// Topologically sorted DAG with additional metadata for rendering
pub type RoutePlanWithMetadata = Vec<RoutePlanStep>;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RoutePlanStep {
    pub swap_info: SwapInfo,
    /// Share of the input routed through this step, as a whole percent.
    ///
    /// Optional because the routing algorithm introduced with `instructionVersion=V2`
    /// reports `null` here and states the share in `bps` instead. A step that carries
    /// neither is a routing response this client does not understand.
    #[serde(default)]
    pub percent: Option<u8>,
    /// Share of the input routed through this step, in basis points (`10000` = 100%).
    /// Canonical since `instructionVersion=V2`; absent on older responses.
    #[serde(default)]
    pub bps: Option<u16>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SwapInfo {
    #[serde(with = "field_as_string")]
    pub amm_key: Pubkey,
    pub label: String,
    #[serde(with = "field_as_string")]
    pub input_mint: Pubkey,
    #[serde(with = "field_as_string")]
    pub output_mint: Pubkey,
    /// An estimation of the input amount into the AMM
    #[serde(with = "field_as_string")]
    pub in_amount: u64,
    /// An estimation of the output amount into the AMM
    #[serde(with = "field_as_string")]
    pub out_amount: u64,
    /// Deprecated by Jupiter and absent from live `/quote` responses since
    /// November 2025: first reported as `0` and the input mint, then dropped
    /// from the payload entirely. The fee is already accounted for in
    /// `out_amount`. Optional so a response without the field still decodes.
    #[serde(default, with = "option_field_as_string")]
    pub fee_amount: Option<u64>,
    /// Deprecated by Jupiter, as `fee_amount`.
    #[serde(default, with = "option_field_as_string")]
    pub fee_mint: Option<Pubkey>,
}
