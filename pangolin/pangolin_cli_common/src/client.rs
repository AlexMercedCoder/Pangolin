use reqwest::{Client, Response};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use crate::config::CliConfig;
use crate::error::CliError;
use serde_json::json;

pub struct PangolinClient {
    client: Client,
    pub config: CliConfig,
}

impl PangolinClient {
    pub fn new(config: CliConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub fn update_config(&mut self, config: CliConfig) {
        self.config = config;
    }

    fn build_headers(&self) -> Result<HeaderMap, CliError> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if let Some(token) = &self.config.auth_token {
            let auth_value = format!("Bearer {}", token);
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value)
                .map_err(|e| CliError::Internal(format!("Invalid auth header: {}", e)))?);
        } else {
             // eprintln!("DEBUG: No auth token found in config during build_headers.");
        }
        
        if let Some(tenant) = &self.config.tenant_id {
             if let Ok(val) = HeaderValue::from_str(tenant) {
                 headers.insert("x-pangolin-tenant", val);
             }
        }

        Ok(headers)
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.config.base_url.trim_end_matches('/'), path)
    }

    pub async fn get(&self, path: &str) -> Result<Response, CliError> {
        let headers = self.build_headers()?;
        let res = self.client.get(&self.url(path))
            .headers(headers)
            .send()
            .await
            .map_err(|e| CliError::ApiError(e.to_string()))?;
            
        Ok(res)
    }

    pub async fn post<T: serde::Serialize>(&self, path: &str, body: &T) -> Result<Response, CliError> {
        let headers = self.build_headers()?;
        let res = self.client.post(&self.url(path))
            .headers(headers)
            .json(body)
            .send()
            .await
            .map_err(|e| CliError::ApiError(e.to_string()))?;

        Ok(res)
    }

    pub async fn delete(&self, path: &str) -> Result<Response, CliError> {
        let headers = self.build_headers()?;
        let res = self.client.delete(&self.url(path))
            .headers(headers)
            .send()
            .await
            .map_err(|e| CliError::ApiError(e.to_string()))?;
            
        Ok(res)
    }
    
    pub async fn login(&mut self, username: &str, password: &str) -> Result<(), CliError> {
        let body = json!({
            "username": username,
            "password": password
        });

        let res = self.client.post(&self.url("/api/v1/users/login"))
            .json(&body)
            .send()
            .await
            .map_err(|e| CliError::ApiError(e.to_string()))?;

        if !res.status().is_success() {
            return Err(CliError::AuthError("Login failed. Check credentials.".to_string()));
        }

        #[derive(serde::Deserialize)]
        struct LoginResponse {
            token: String,
            user: serde_json::Value,
        }

        let data: LoginResponse = res.json()
            .await
            .map_err(|e| CliError::ApiError(format!("Failed to parse login response: {}", e)))?;

        self.config.auth_token = Some(data.token);
        self.config.username = Some(username.to_string());
        
        // Try to extract tenant from user object if available
        if let Some(tenant_id) = data.user.get("tenant-id").and_then(|t| t.as_str()) {
             self.config.tenant_id = Some(tenant_id.to_string());
        }

        Ok(())
    }
}
