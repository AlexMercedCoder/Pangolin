use anyhow::Result;
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use bytes::Bytes;
use pangolin_core::model::{FederatedAuthType, FederatedCatalogConfig};
use reqwest::RequestBuilder;
use std::time::Duration;

/// Federated Catalog Proxy - forwards Iceberg REST requests to external catalogs
pub struct FederatedCatalogProxy {
    client: reqwest::Client,
}

impl FederatedCatalogProxy {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Forward an Iceberg REST request to a federated catalog
    pub async fn forward_request(
        &self,
        config: &FederatedCatalogConfig,
        method: Method,
        path: &str,
        body: Option<Bytes>,
        headers: HeaderMap,
    ) -> Result<Response> {
        let url = format!("{}{}", config.base_url, path);
        
        // Create request
        let mut request = self.client.request(method.clone(), &url);
        
        // Add authentication
        request = self.add_auth(request, config);
        
        // Forward relevant headers (skip auth headers from client)
        for (key, value) in headers.iter() {
            if !Self::is_auth_header(key.as_str()) {
                request = request.header(key, value);
            }
        }
        
        // Add body if present
        if let Some(body) = body {
            request = request.body(body);
        }
        
        // Send request
        let response = request.send().await.map_err(|e| {
            anyhow::anyhow!("Failed to forward request to federated catalog: {}", e)
        })?;
        
        // Convert reqwest::Response to axum::Response
        let status = StatusCode::from_u16(response.status().as_u16())
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        
        let mut response_headers = HeaderMap::new();
        for (key, value) in response.headers().iter() {
            if let Ok(header_name) = axum::http::HeaderName::from_bytes(key.as_str().as_bytes()) {
                if let Ok(header_value) = axum::http::HeaderValue::from_bytes(value.as_bytes()) {
                    response_headers.insert(header_name, header_value);
                }
            }
        }
        
        let body_bytes = response.bytes().await.map_err(|e| {
            anyhow::anyhow!("Failed to read response body: {}", e)
        })?;
        
        Ok((status, response_headers, body_bytes).into_response())
    }
    
    /// Add authentication to the request based on config
    fn add_auth(&self, mut request: RequestBuilder, config: &FederatedCatalogConfig) -> RequestBuilder {
        match &config.auth_type {
            FederatedAuthType::None => request,
            FederatedAuthType::BasicAuth => {
                if let Some(creds) = &config.credentials {
                    if let (Some(username), Some(password)) = (&creds.username, &creds.password) {
                        request = request.basic_auth(username, Some(password));
                    }
                }
                request
            }
            FederatedAuthType::BearerToken => {
                if let Some(creds) = &config.credentials {
                    if let Some(token) = &creds.token {
                        request = request.bearer_auth(token);
                    }
                }
                request
            }
            FederatedAuthType::ApiKey => {
                if let Some(creds) = &config.credentials {
                    if let Some(api_key) = &creds.api_key {
                        request = request.header("X-API-Key", api_key);
                    }
                }
                request
            }
        }
    }
    
    /// Check if a header is an authentication header that should not be forwarded
    fn is_auth_header(header_name: &str) -> bool {
        matches!(
            header_name.to_lowercase().as_str(),
            "authorization" | "x-api-key" | "cookie"
        )
    }
}

impl Default for FederatedCatalogProxy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pangolin_core::model::FederatedCredentials;

    #[test]
    fn test_is_auth_header() {
        assert!(FederatedCatalogProxy::is_auth_header("Authorization"));
        assert!(FederatedCatalogProxy::is_auth_header("authorization"));
        assert!(FederatedCatalogProxy::is_auth_header("X-API-Key"));
        assert!(FederatedCatalogProxy::is_auth_header("x-api-key"));
        assert!(FederatedCatalogProxy::is_auth_header("Cookie"));
        assert!(!FederatedCatalogProxy::is_auth_header("Content-Type"));
        assert!(!FederatedCatalogProxy::is_auth_header("Accept"));
    }

    #[test]
    fn test_proxy_creation() {
        let proxy = FederatedCatalogProxy::new();
        // Just verify it can be created
        assert!(std::mem::size_of_val(&proxy) > 0);
    }
}
