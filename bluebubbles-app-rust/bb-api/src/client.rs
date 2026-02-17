//! HTTP client for the BlueBubbles server REST API.
//!
//! Handles authentication, custom headers, timeout management, SSL
//! certificate handling, exponential backoff retry, and request/response lifecycle.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, Method, RequestBuilder, Response, StatusCode};
use serde::de::DeserializeOwned;
use tokio::sync::RwLock;
use tracing::{debug, warn};

use bb_core::config::ServerConfig;
use bb_core::constants;
use bb_core::error::{BbError, BbResult};

use crate::response::ServerResponse;

/// Retry configuration for HTTP requests.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts.
    pub max_retries: u32,
    /// Base delay between retries (doubles each attempt).
    pub base_delay: Duration,
    /// Maximum delay cap.
    pub max_delay: Duration,
    /// HTTP status codes that trigger a retry.
    pub retryable_statuses: Vec<u16>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(4),
            retryable_statuses: vec![502, 503, 504],
        }
    }
}

/// HTTP client for communicating with the BlueBubbles server.
///
/// Wraps reqwest::Client with BlueBubbles-specific authentication,
/// header injection, retry logic, and error handling.
#[derive(Clone)]
pub struct ApiClient {
    inner: Client,
    /// Base URL for the API (e.g. "https://example.com/api/v1").
    api_root: String,
    /// Server origin (scheme + host, no path).
    origin: String,
    /// GUID authentication key appended to every request.
    guid_auth_key: String,
    /// Default request timeout.
    timeout: Duration,
    /// Extended timeout for large transfers (12x default).
    extended_timeout: Duration,
    /// Optional origin override for localhost connections.
    origin_override: Arc<RwLock<Option<String>>>,
    /// Custom headers from server config.
    custom_headers: Vec<(String, String)>,
    /// Retry configuration.
    retry_config: RetryConfig,
    /// Health check interval for connection monitoring.
    health_check_interval: Duration,
    /// Whether the Cloudflare 502 retry is enabled.
    cloudflare_retry: bool,
}

impl ApiClient {
    /// Create a new ApiClient from server configuration.
    pub fn new(config: &ServerConfig) -> BbResult<Self> {
        let sanitized_address =
            bb_core::config::AppConfig::sanitize_server_address(&config.address);

        let mut builder = Client::builder()
            .timeout(Duration::from_millis(config.api_timeout_ms))
            .connect_timeout(Duration::from_secs(15))
            .pool_max_idle_per_host(5)
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Duration::from_secs(30));

        // Handle self-signed certificates
        if config.accept_self_signed_certs {
            builder = builder.danger_accept_invalid_certs(true);
        }

        let inner = builder
            .build()
            .map_err(|e| BbError::Http(format!("failed to build HTTP client: {e}")))?;

        let origin = derive_origin(&sanitized_address);
        let api_root = format!("{origin}/api/{}", constants::API_VERSION);
        let timeout = Duration::from_millis(config.api_timeout_ms);
        let extended_timeout = timeout * constants::EXTENDED_TIMEOUT_MULTIPLIER as u32;

        // Build custom headers based on server tunnel type
        let mut custom_headers = Vec::new();
        if sanitized_address.contains("ngrok") {
            custom_headers.push(("ngrok-skip-browser-warning".into(), "true".into()));
        }
        if sanitized_address.contains("zrok") {
            custom_headers.push(("skip_zrok_interstitial".into(), "true".into()));
        }
        for (k, v) in config.custom_headers.iter() {
            custom_headers.push((k.clone(), v.clone()));
        }

        let cloudflare_retry = sanitized_address.contains("trycloudflare");

        Ok(Self {
            inner,
            api_root,
            origin,
            guid_auth_key: config.guid_auth_key.clone(),
            timeout,
            extended_timeout,
            origin_override: Arc::new(RwLock::new(None)),
            custom_headers,
            retry_config: RetryConfig::default(),
            health_check_interval: Duration::from_secs(30),
            cloudflare_retry,
        })
    }

    /// Set custom retry configuration.
    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    /// Get the current API root URL.
    pub fn api_root(&self) -> &str {
        &self.api_root
    }

    /// Get the server origin.
    pub fn origin(&self) -> &str {
        &self.origin
    }

    /// Get the health check interval.
    pub fn health_check_interval(&self) -> Duration {
        self.health_check_interval
    }

    /// Set an origin override for localhost connections.
    pub async fn set_origin_override(&self, origin: Option<String>) {
        let mut guard = self.origin_override.write().await;
        *guard = origin;
        if let Some(ref o) = *guard {
            debug!("origin override set to {o}");
        } else {
            debug!("origin override cleared");
        }
    }

    /// Get the current effective origin (with override if set).
    pub async fn effective_origin(&self) -> String {
        let guard = self.origin_override.read().await;
        guard.as_deref().unwrap_or(&self.origin).to_string()
    }

    /// Build the full URL for an API path, including auth parameter.
    async fn url(&self, path: &str) -> String {
        let override_guard = self.origin_override.read().await;
        let base = if let Some(ref override_origin) = *override_guard {
            format!("{override_origin}/api/{}", constants::API_VERSION)
        } else {
            self.api_root.clone()
        };

        let separator = if path.contains('?') { "&" } else { "?" };
        format!("{base}{path}{separator}guid={}", self.guid_auth_key)
    }

    /// Apply custom headers to a request builder.
    fn apply_headers(&self, mut builder: RequestBuilder) -> RequestBuilder {
        for (key, value) in &self.custom_headers {
            builder = builder.header(key.as_str(), value.as_str());
        }
        builder
    }

    /// Internal: build a request for the given method, URL, timeout, and optional JSON body.
    fn build_request(
        &self,
        method: Method,
        url: &str,
        timeout: Duration,
        body: Option<&serde_json::Value>,
    ) -> RequestBuilder {
        let mut builder = self.inner.request(method, url).timeout(timeout);
        if let Some(b) = body {
            builder = builder.json(b);
        }
        self.apply_headers(builder)
    }

    /// Execute a request with exponential backoff retry.
    async fn request_with_retry(
        &self,
        method: Method,
        path: &str,
        timeout: Duration,
        body: Option<&serde_json::Value>,
    ) -> BbResult<Response> {
        let url = self.url(path).await;
        debug!("{} {}", method, path);

        let mut last_error: Option<BbError> = None;

        for attempt in 0..=self.retry_config.max_retries {
            if attempt > 0 {
                let delay = self.calculate_retry_delay(attempt - 1);
                warn!(
                    "retrying {} {} (attempt {}/{}) after {:.1}s",
                    method,
                    path,
                    attempt + 1,
                    self.retry_config.max_retries + 1,
                    delay.as_secs_f64()
                );
                tokio::time::sleep(delay).await;
            }

            let builder = self.build_request(method.clone(), &url, timeout, body);

            match builder.send().await {
                Ok(response) => {
                    let status = response.status();

                    // Cloudflare 502 retry for trycloudflare URLs (single extra retry)
                    if status == StatusCode::BAD_GATEWAY && self.cloudflare_retry && attempt == 0 {
                        warn!("cloudflare 502 detected, retrying");
                        last_error = Some(BbError::ServerError {
                            status: 502,
                            message: "Cloudflare 502".into(),
                        });
                        continue;
                    }

                    // Check if this status code is retryable
                    if self
                        .retry_config
                        .retryable_statuses
                        .contains(&status.as_u16())
                        && attempt < self.retry_config.max_retries
                    {
                        warn!("retryable status {} from {}", status.as_u16(), path);
                        last_error = Some(BbError::ServerError {
                            status: status.as_u16(),
                            message: format!("retryable status {status}"),
                        });
                        continue;
                    }

                    return Self::check_status(response).await;
                }
                Err(e) => {
                    let is_retryable = e.is_timeout() || e.is_connect();
                    let err = Self::classify_error(e);

                    if is_retryable && attempt < self.retry_config.max_retries {
                        warn!("retryable error on {}: {}", path, err);
                        last_error = Some(err);
                        continue;
                    }

                    return Err(err);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| BbError::Http("max retries exceeded".into())))
    }

    /// Calculate retry delay with exponential backoff.
    fn calculate_retry_delay(&self, attempt: u32) -> Duration {
        let base_ms = self.retry_config.base_delay.as_millis() as u64;
        let delay_ms = base_ms.saturating_mul(1u64 << attempt);
        let max_ms = self.retry_config.max_delay.as_millis() as u64;
        Duration::from_millis(delay_ms.min(max_ms))
    }

    // --- Public HTTP methods ---

    /// Execute a GET request with automatic retry.
    pub async fn get(&self, path: &str) -> BbResult<Response> {
        self.request_with_retry(Method::GET, path, self.timeout, None)
            .await
    }

    /// Execute a GET request with extended timeout (for large downloads).
    pub async fn get_extended(&self, path: &str) -> BbResult<Response> {
        self.request_with_retry(Method::GET, path, self.extended_timeout, None)
            .await
    }

    /// Execute a POST request with a JSON body.
    pub async fn post(&self, path: &str, body: &serde_json::Value) -> BbResult<Response> {
        self.request_with_retry(Method::POST, path, self.timeout, Some(body))
            .await
    }

    /// Execute a POST request with extended timeout.
    pub async fn post_extended(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> BbResult<Response> {
        self.request_with_retry(Method::POST, path, self.extended_timeout, Some(body))
            .await
    }

    /// Execute a PUT request with a JSON body.
    pub async fn put(&self, path: &str, body: &serde_json::Value) -> BbResult<Response> {
        self.request_with_retry(Method::PUT, path, self.timeout, Some(body))
            .await
    }

    /// Execute a DELETE request.
    pub async fn delete(&self, path: &str) -> BbResult<Response> {
        self.request_with_retry(Method::DELETE, path, self.timeout, None)
            .await
    }

    /// Execute a DELETE request with a JSON body.
    pub async fn delete_with_body(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> BbResult<Response> {
        self.request_with_retry(Method::DELETE, path, self.timeout, Some(body))
            .await
    }

    /// Execute a POST request with a multipart form (for file uploads).
    /// Multipart forms cannot be cloned, so no automatic retry on this method.
    pub async fn post_multipart(
        &self,
        path: &str,
        form: reqwest::multipart::Form,
    ) -> BbResult<Response> {
        let url = self.url(path).await;
        debug!("POST (multipart) {}", path);

        let builder = self
            .inner
            .post(&url)
            .multipart(form)
            .timeout(self.extended_timeout);
        let builder = self.apply_headers(builder);

        let response = builder
            .send()
            .await
            .map_err(|e| Self::classify_error(e))?;

        Self::check_status(response).await
    }

    // --- Response helpers ---

    /// Ping the server to check health. Returns the round-trip latency.
    pub async fn health_check(&self) -> BbResult<Duration> {
        let start = std::time::Instant::now();
        let resp: ServerResponse = self.get_json("/ping").await?;
        if resp.is_success() {
            Ok(start.elapsed())
        } else {
            Err(BbError::Http("health check failed".into()))
        }
    }

    /// Deserialize a response body into a ServerResponse<T>.
    pub async fn parse_response<T: DeserializeOwned>(
        response: Response,
    ) -> BbResult<ServerResponse<T>> {
        response
            .json::<ServerResponse<T>>()
            .await
            .map_err(|e| BbError::Serialization(format!("failed to parse response: {e}")))
    }

    /// Get raw bytes from a response (for file downloads).
    pub async fn response_bytes(response: Response) -> BbResult<Vec<u8>> {
        response
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| BbError::Http(format!("failed to read response bytes: {e}")))
    }

    /// Download a response body as bytes with progress reporting.
    ///
    /// The progress callback receives (bytes_downloaded, total_bytes).
    /// If the server does not send Content-Length, total_bytes will be 0.
    pub async fn response_bytes_with_progress<F>(
        response: Response,
        progress: F,
    ) -> BbResult<Vec<u8>>
    where
        F: Fn(u64, u64) + Send + 'static,
    {
        let total = response.content_length().unwrap_or(0);
        let mut downloaded: u64 = 0;
        let mut bytes = Vec::with_capacity(if total > 0 { total as usize } else { 8192 });

        let mut stream = response;
        while let Some(chunk) = stream
            .chunk()
            .await
            .map_err(|e| BbError::Http(format!("download stream error: {e}")))?
        {
            downloaded += chunk.len() as u64;
            bytes.extend_from_slice(&chunk);
            progress(downloaded, total);
        }

        Ok(bytes)
    }

    /// Convenience: GET + parse into ServerResponse<T>.
    pub async fn get_json<T: DeserializeOwned>(&self, path: &str) -> BbResult<ServerResponse<T>> {
        let resp = self.get(path).await?;
        Self::parse_response(resp).await
    }

    /// Convenience: GET with extended timeout + parse.
    pub async fn get_extended_json<T: DeserializeOwned>(
        &self,
        path: &str,
    ) -> BbResult<ServerResponse<T>> {
        let resp = self.get_extended(path).await?;
        Self::parse_response(resp).await
    }

    /// Convenience: POST + parse into ServerResponse<T>.
    pub async fn post_json<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> BbResult<ServerResponse<T>> {
        let resp = self.post(path, body).await?;
        Self::parse_response(resp).await
    }

    /// Convenience: POST with extended timeout + parse into ServerResponse<T>.
    pub async fn post_extended_json<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> BbResult<ServerResponse<T>> {
        let resp = self.post_extended(path, body).await?;
        Self::parse_response(resp).await
    }

    /// Convenience: PUT + parse into ServerResponse<T>.
    pub async fn put_json<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> BbResult<ServerResponse<T>> {
        let resp = self.put(path, body).await?;
        Self::parse_response(resp).await
    }

    /// Check the HTTP status code and convert to BbError if needed.
    async fn check_status(response: Response) -> BbResult<Response> {
        let status = response.status();

        if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
            return Err(BbError::AuthFailed(format!("server returned {status}")));
        }

        if status.is_server_error() {
            let body = response.text().await.unwrap_or_default();
            return Err(BbError::ServerError {
                status: status.as_u16(),
                message: body,
            });
        }

        Ok(response)
    }

    /// Classify a reqwest error into a BbError variant.
    fn classify_error(e: reqwest::Error) -> BbError {
        if e.is_timeout() {
            BbError::Timeout(e.to_string())
        } else if e.is_connect() {
            BbError::Http(format!("connection failed: {e}"))
        } else {
            BbError::Http(e.to_string())
        }
    }

    /// Update the server address at runtime (for URL changes without restart).
    pub fn update_server_address(&mut self, new_address: &str) {
        let sanitized = bb_core::config::AppConfig::sanitize_server_address(new_address);
        self.origin = derive_origin(&sanitized);
        self.api_root = format!("{}/api/{}", self.origin, constants::API_VERSION);
        self.cloudflare_retry = sanitized.contains("trycloudflare");

        // Update tunnel headers
        self.custom_headers
            .retain(|(k, _)| k != "ngrok-skip-browser-warning" && k != "skip_zrok_interstitial");
        if sanitized.contains("ngrok") {
            self.custom_headers
                .push(("ngrok-skip-browser-warning".into(), "true".into()));
        }
        if sanitized.contains("zrok") {
            self.custom_headers
                .push(("skip_zrok_interstitial".into(), "true".into()));
        }

        debug!("server address updated to {}", self.origin);
    }
}

/// Derive the origin (scheme + host + optional port) from a server address.
fn derive_origin(address: &str) -> String {
    if let Ok(url) = reqwest::Url::parse(address) {
        let host = url.host_str().unwrap_or("localhost");
        match url.port() {
            Some(port) => format!("{}://{}:{}", url.scheme(), host, port),
            None => format!("{}://{}", url.scheme(), host),
        }
    } else {
        address.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> ServerConfig {
        ServerConfig {
            address: "http://localhost:1234".into(),
            guid_auth_key: "test".into(),
            custom_headers: std::collections::HashMap::new(),
            api_timeout_ms: 30000,
            accept_self_signed_certs: false,
        }
    }

    #[test]
    fn test_derive_origin() {
        assert_eq!(
            derive_origin("https://abc123.trycloudflare.com/api/v1"),
            "https://abc123.trycloudflare.com"
        );
        assert_eq!(
            derive_origin("http://192.168.1.100:1234"),
            "http://192.168.1.100:1234"
        );
    }

    #[test]
    fn test_retry_delay_calculation() {
        let client = ApiClient::new(&test_config()).unwrap();
        assert_eq!(client.calculate_retry_delay(0), Duration::from_secs(1));
        assert_eq!(client.calculate_retry_delay(1), Duration::from_secs(2));
        assert_eq!(client.calculate_retry_delay(2), Duration::from_secs(4));
    }

    #[test]
    fn test_retry_delay_capped() {
        let client = ApiClient::new(&test_config()).unwrap();
        let d10 = client.calculate_retry_delay(10);
        assert!(d10 <= Duration::from_secs(4));
    }

    #[test]
    fn test_cloudflare_detection() {
        let config = ServerConfig {
            address: "https://abc.trycloudflare.com".into(),
            guid_auth_key: "test".into(),
            custom_headers: std::collections::HashMap::new(),
            api_timeout_ms: 30000,
            accept_self_signed_certs: false,
        };
        let client = ApiClient::new(&config).unwrap();
        assert!(client.cloudflare_retry);
    }

    #[test]
    fn test_ngrok_headers() {
        let config = ServerConfig {
            address: "https://abc.ngrok.io".into(),
            guid_auth_key: "test".into(),
            custom_headers: std::collections::HashMap::new(),
            api_timeout_ms: 30000,
            accept_self_signed_certs: false,
        };
        let client = ApiClient::new(&config).unwrap();
        assert!(client
            .custom_headers
            .iter()
            .any(|(k, _)| k == "ngrok-skip-browser-warning"));
    }

    #[test]
    fn test_update_server_address() {
        let mut client = ApiClient::new(&test_config()).unwrap();
        client.update_server_address("https://new.trycloudflare.com");
        assert!(client.origin.contains("trycloudflare"));
        assert!(client.cloudflare_retry);
    }
}
