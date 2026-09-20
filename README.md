# jupiter-swap-api-client

A Rust client for the [Jupiter](https://jup.ag/) quote and swap APIs: quote a swap,
get a built transaction back, and send it yourself.

This is a fork of [`jup-ag/jupiter-swap-api-client`](https://github.com/jup-ag/jupiter-swap-api-client),
which is **no longer maintained** — its `main` branch has not moved since December 2025,
its open pull requests go unreviewed, and the published crate still pins Solana 2.x.
Jupiter itself no longer documents a Rust path at all; the API is REST/JSON and the
official examples are JavaScript, Python and curl. So this fork exists to keep a working
Rust client alive, not to stage changes for upstream.

## What this fork changes

Request and response coverage upstream lacks:

- `prioritizationFeeLamports` and `dynamicComputeUnitLimit` on `/swap`
- `blockhashSlotsToExpiry` on `/swap`
- `excludeDexes` under its correct name, with `dexes`/`excludeDexes` serialized as
  comma-separated strings rather than repeated query parameters

Client behavior:

- an optional Jupiter API key, sent as the `x-api-key` header
- one `reqwest::Client` built once and reused, so connections are pooled
- request and idle-connection timeouts, configurable by environment variable

Kept current against the live API:

- `feeAmount` and `feeMint` in `routePlan[].swapInfo` are `Option`. Jupiter deprecated
  both in November 2025 — first reporting `0` and the input mint, then omitting them
  entirely — and the fee is already accounted for in `outAmount`. Without this, a quote
  from the live API does not deserialize.
- `routePlan[].percent` is `Option`, and `bps` sits alongside it. The routing algorithm
  behind `instructionVersion=V2` reports `percent: null` and states the share in basis
  points instead.
- The Solana dependencies are the granular crates matched to the 4.3 client train, not
  the `solana-sdk` umbrella. The library needs only `solana-pubkey` and
  `solana-instruction`.

## Transaction versions

The library is transaction-version agnostic: `SwapResponse::swap_transaction` is raw
bytes, and nothing here decodes them. That matters for **transaction v1** (SIMD-0385),
which writes its signatures last, as a fixed-length array sized by the message header.
Serde's derived layout — signatures first, behind a `short_vec` length prefix — cannot
express that, so `bincode` cannot read a v1 transaction even though it still compiles
against `VersionedTransaction`.

Decode with [`wincode`](https://crates.io/crates/wincode), as the example does:

```rust
let transaction: VersionedTransaction =
    wincode::deserialize(&swap_response.swap_transaction)?;
```

## Usage

```rust
use jupiter_swap_api_client::{
    quote::QuoteRequest, swap::SwapRequest, transaction_config::TransactionConfig,
    JupiterSwapApiClient,
};
use solana_pubkey::{pubkey, Pubkey};

const USDC_MINT: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const NATIVE_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
const TEST_WALLET: Pubkey = pubkey!("2AQdpHJ2JpcEgPiATUXjQxA8QmafFegfQwSLWSprPicm");

#[tokio::main]
async fn main() {
    let client = JupiterSwapApiClient::new("https://api.jup.ag/swap/v1".to_string());
    // or, with a key:
    // let client = JupiterSwapApiClient::new_with_api_key(base_url, api_key);

    let quote_response = client
        .quote(&QuoteRequest {
            amount: 1_000_000,
            input_mint: USDC_MINT,
            output_mint: NATIVE_MINT,
            slippage_bps: 50,
            ..QuoteRequest::default()
        })
        .await
        .unwrap();

    let swap_response = client
        .swap(&SwapRequest {
            user_public_key: TEST_WALLET,
            quote_response: quote_response.clone(),
            config: TransactionConfig::default(),
        })
        .await
        .unwrap();

    println!("raw tx len: {}", swap_response.swap_transaction.len());

    // POST /swap-instructions, when you would rather assemble the transaction yourself
    let swap_instructions = client
        .swap_instructions(&SwapRequest {
            user_public_key: TEST_WALLET,
            quote_response,
            config: TransactionConfig::default(),
        })
        .await
        .unwrap();
    println!("{swap_instructions:#?}");
}
```

The full program, including signing and sending, is in [`example/`](./example/).

## Endpoints

| Base URL | Notes |
| --- | --- |
| `https://api.jup.ag/swap/v1` | Current. Keyless requests are rate limited; an API key raises the limit. |
| `https://lite-api.jup.ag/swap/v1` | Keyless tier. A deprecation was announced for 31 January 2026 and then postponed with no new date. |
| `https://quote-api.jup.ag/v6` | **Dead.** The host no longer resolves in DNS. |

Self-hosted and paid-hosted deployments work the same way — point the base URL at them.

## Environment variables

The example reads these; the library takes the base URL and key as constructor arguments.

| Variable | Default | Meaning |
| --- | --- | --- |
| `API_BASE_URL` | `https://api.jup.ag/swap/v1` | API base URL |
| `JUPITER_API_KEY` | unset | Sent as `x-api-key` when present |
| `JUPITER_REQUEST_TIMEOUT_SECS` | `3` | Per-request timeout |
| `JUPITER_CONNECTION_TIMEOUT_SECS` | `1800` | Idle-connection pool timeout |

## Keeping up with Jupiter

Because upstream will not do it, these are what to watch:

- the OpenAPI specs, which are the earliest machine-readable signal of a field being
  added, removed, or dropped from `required` — this is where `feeAmount`'s demotion
  showed up first
- Jupiter's changelog, posted monthly, where breaking changes and their deadlines land
- `lite-api.jup.ag` liveness, since its deprecation is postponed rather than cancelled
- Swap API v2 (`/order`, `/build`), which Jupiter now recommends over v1 and which this
  client does not implement. Note that v2 instructions do not emit fee events, so any
  downstream parser reading fee transfers has to switch to token balance deltas.

## Additional resources

- [Jupiter developer docs](https://developers.jup.ag/)
- [jup.ag](https://jup.ag/)
