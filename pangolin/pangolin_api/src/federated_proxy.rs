use anyhow::Result;
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use bytes::Bytes;
use pangolin_core::model::FederatedCatalogConfig;
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
        let base_url = config.properties.get("uri")
            .ok_or_else(|| anyhow::anyhow!("Federated catalog missing 'uri' property"))?;
            
        let url = format!("{}{}", base_url, path);
        
        // Convert axum::http::Method to reqwest::Method
        let reqwest_method = match method {
            Method::GET => reqwest::Method::GET,
            Method::POST => reqwest::Method::POST,
            Method::PUT => reqwest::Method::PUT,
            Method::DELETE => reqwest::Method::DELETE,
            Method::HEAD => reqwest::Method::HEAD,
            Method::OPTIONS => reqwest::Method::OPTIONS,
            Method::PATCH => reqwest::Method::PATCH,
            _ => reqwest::Method::GET, // Default fallback
        };
        
        let mut request = self.client.request(reqwest_method, &url);
        
        // Add authentication
        request = self.add_auth(request, config);
        
        // Forward relevant headers (skip auth headers from client)
        for (key, value) in headers.iter() {
            if !Self::is_auth_header(key.as_str()) {
                // Convert axum headers to reqwest headers via strings
                if let Ok(value_str) = value.to_str() {
                    request = request.header(key.as_str(), value_str);
                }
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
    
    /// Add authentication to the request based on config properties
    fn add_auth(&self, mut request: RequestBuilder, config: &FederatedCatalogConfig) -> RequestBuilder {
        // Bearer Token
        if let Some(token) = config.properties.get("token") {
             request = request.bearer_auth(token);
        } 
        // Basic Auth (username/password)
        else if let (Some(username), Some(password)) = (config.properties.get("username"), config.properties.get("password")) {
            request = request.basic_auth(username, Some(password));
        }
        // X-API-Key
        else if let Some(api_key) = config.properties.get("api_key").or_else(|| config.properties.get("x-api-key")) {
             request = request.header("X-API-Key", api_key);
        }
        
        request
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
