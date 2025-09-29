use anyhow::{anyhow, Result};
use quote::{QuoteRequest, QuoteResponse};
use reqwest::{Client, Response};
use serde::de::DeserializeOwned;
use swap::{SwapInstructionsResponse, SwapInstructionsResponseInternal, SwapRequest, SwapResponse};

pub mod quote;
mod route_plan_with_metadata;
mod serde_helpers;
pub mod swap;
pub mod transaction_config;

#[derive(Clone)]
pub struct JupiterSwapApiClient {
    pub base_path: String,
    pub api_key: Option<String>,
}

async fn check_is_success(response: Response) -> Result<Response> {
    if !response.status().is_success() {
        return Err(anyhow!(
            "Request status not ok: {}, body: {:?}",
            response.status(),
            response.text().await
        ));
    }
    Ok(response)
}

async fn check_status_code_and_deserialize<T: DeserializeOwned>(response: Response) -> Result<T> {
    check_is_success(response)
        .await?
        .json::<T>()
        .await
        .map_err(Into::into)
}

impl JupiterSwapApiClient {
    pub fn new(base_path: String) -> Self {
        Self { 
            base_path,
            api_key: None,
        }
    }

    pub fn new_with_api_key(base_path: String, api_key: String) -> Self {
        Self { 
            base_path,
            api_key: Some(api_key),
        }
    }

    fn build_client(&self) -> Client {
        let mut client_builder = Client::builder();
        
        if let Some(ref api_key) = self.api_key {
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                "x-api-key", 
                reqwest::header::HeaderValue::from_str(api_key).unwrap()
            );
            client_builder = client_builder.default_headers(headers);
        }
        
        client_builder.build().unwrap()
    }

    pub async fn quote(&self, quote_request: &QuoteRequest) -> Result<QuoteResponse> {
        let query = serde_qs::to_string(&quote_request)?;
        let response = self.build_client()
            .get(format!("{}/quote?{query}", self.base_path))
            .send()
            .await?;
        check_status_code_and_deserialize(response).await
    }

    pub async fn swap(&self, swap_request: &SwapRequest) -> Result<SwapResponse> {
        let response = self.build_client()
            .post(format!("{}/swap", self.base_path))
            .json(swap_request)
            .send()
            .await?;
        check_status_code_and_deserialize(response).await
    }

    pub async fn swap_instructions(
        &self,
        swap_request: &SwapRequest,
    ) -> Result<SwapInstructionsResponse> {
        let response = self.build_client()
            .post(format!("{}/swap-instructions", self.base_path))
            .json(swap_request)
            .send()
            .await?;
        check_status_code_and_deserialize::<SwapInstructionsResponseInternal>(response)
            .await
            .map(Into::into)
    }
}