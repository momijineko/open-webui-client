//! HTTP Proxy Module
//!
//! Proxies HTTP requests from the frontend to the backend,
//! bypassing browser CORS restrictions using Tauri IPC.
//! Supports both local and remote backend modes.

use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use reqwest::cookie::Jar;

/// HTTP request options from the frontend
#[derive(Debug, Deserialize)]
pub struct ProxyRequest {
    pub method: String,
    pub path: String,
    pub headers: Option<Vec<HeaderTuple>>,
    pub body: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HeaderTuple(pub String, pub String);

/// HTTP response to return to the frontend
#[derive(Debug, Serialize)]
pub struct ProxyResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<HeaderTuple>,
    pub body: String,
}

/// Get the default local backend URL
fn get_local_backend_url() -> String {
    "http://127.0.0.1:8080".to_string()
}

/// Parse HTTP method string
fn parse_method(method: &str) -> Result<Method, String> {
    match method.to_uppercase().as_str() {
        "GET" => Ok(Method::GET),
        "POST" => Ok(Method::POST),
        "PUT" => Ok(Method::PUT),
        "DELETE" => Ok(Method::DELETE),
        "PATCH" => Ok(Method::PATCH),
        "HEAD" => Ok(Method::HEAD),
        "OPTIONS" => Ok(Method::OPTIONS),
        _ => Err(format!("Invalid HTTP method: {}", method)),
    }
}

// Global cookie store for maintaining session across requests
lazy_static::lazy_static! {
    static ref COOKIE_STORE: Arc<Jar> = Arc::new(Jar::default());
}

/// Proxy an HTTP request to the backend (local or remote)
#[tauri::command]
pub async fn proxy_request(request: ProxyRequest) -> Result<ProxyResponse, String> {
    // Determine the backend URL from headers or use default
    let backend_url = if let Some(headers) = &request.headers {
        headers.iter()
            .find(|h| h.0.eq_ignore_ascii_case("X-Remote-Backend-URL"))
            .map(|h| h.1.clone())
            .unwrap_or_else(get_local_backend_url)
    } else {
        get_local_backend_url()
    };

    // Build the full URL
    let full_url = format!("{}{}", backend_url.trim_end_matches('/'), request.path);

    // Parse the HTTP method
    let method = parse_method(&request.method)?;

    // Build HTTP client with cookie support
    let cookie_store = COOKIE_STORE.clone();
    let client = Client::builder()
        .cookie_provider(cookie_store)
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let mut req_builder = client.request(method.clone(), &full_url);

    // Add headers if provided
    if let Some(headers) = &request.headers {
        for header in headers {
            let HeaderTuple(key, value) = header;

            // Skip internal headers
            if key.eq_ignore_ascii_case("X-Remote-Backend-URL") ||
               key.eq_ignore_ascii_case("X-Remote-Auth") {
                continue;
            }

            // Skip certain headers that reqwest handles automatically
            if !key.eq_ignore_ascii_case("content-length")
                && !key.eq_ignore_ascii_case("host")
                && !key.eq_ignore_ascii_case("connection")
            {
                req_builder = req_builder.header(key.as_str(), value.as_str());
            }
        }

        // Add Basic Auth if provided via X-Remote-Auth header
        if let Some(HeaderTuple(_, auth_value)) = headers.iter()
            .find(|h| h.0.eq_ignore_ascii_case("X-Remote-Auth")) {
            req_builder = req_builder.header("Authorization", auth_value.as_str());
        }
    }

    // Add body if provided
    if let Some(body) = &request.body {
        req_builder = req_builder.body(body.clone());
    }

    // Execute the request
    let response: reqwest::Response = req_builder.send().await.map_err(|e| {
        format!("Failed to send request to backend: {}", e)
    })?;

    // Get response status
    let status = response.status().as_u16();
    let status_text = response.status().canonical_reason().unwrap_or("Unknown").to_string();

    // Get response headers
    let headers = response
        .headers()
        .iter()
        .map(|(name, value): (&reqwest::header::HeaderName, &reqwest::header::HeaderValue)| {
            HeaderTuple(
                name.as_str().to_string(),
                value.to_str().unwrap_or("").to_string(),
            )
        })
        .collect();

    // Get response body
    let body = response.text().await.map_err(|e| {
        format!("Failed to read response body: {}", e)
    })?;

    Ok(ProxyResponse {
        status,
        status_text,
        headers,
        body,
    })
}

/// Response for fetching images (returns base64 data)
#[derive(Debug, Serialize)]
pub struct ImageResponse {
    pub mime_type: String,
    pub data: String, // base64 encoded
}

/// Fetch an image and return it as base64
/// This is needed because <img> tags can't use custom headers or go through IPC proxy
#[tauri::command]
pub async fn fetch_image(path: String, remote_url: Option<String>) -> Result<ImageResponse, String> {
    // Determine the backend URL
    let backend_url = remote_url.unwrap_or_else(get_local_backend_url);

    // Build the full URL
    let full_url = format!("{}{}", backend_url.trim_end_matches('/'), path);

    // Build HTTP client with cookie support
    let cookie_store = COOKIE_STORE.clone();
    let client = Client::builder()
        .cookie_provider(cookie_store)
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // Execute the request
    let response = client.get(&full_url).send().await.map_err(|e| {
        format!("Failed to fetch image: {}", e)
    })?;

    if !response.status().is_success() {
        return Err(format!("Image request failed with status: {}", response.status()));
    }

    // Get content type
    let mime_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/png")
        .to_string();

    // Get image bytes
    let bytes = response.bytes().await.map_err(|e| {
        format!("Failed to read image bytes: {}", e)
    })?;

    // Encode to base64 (using the base64 crate)
    let data = base64::encode(&bytes);

    Ok(ImageResponse {
        mime_type,
        data,
    })
}
