use serde::Deserialize;
use reqwest::Client;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct IdTokenClaims {
    pub sub: String,

    #[serde(default)]
    pub name: Option<String>,

    #[serde(default)]
    pub email: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,

    #[serde(default)]
    pub scope: Option<String>,

    #[serde(default)]
    pub id_token: Option<String>,

    #[serde(default)]
    pub refresh_token: Option<String>,
}

/// Exchanges the authorization code for an access token and refresh token.
///
/// Arguments
/// * `token_endpoint` - The token endpoint URL of the OAuth provider.
/// * `client_id` - The client ID of the OAuth application.
/// * `code` - The authorization code received from the OAuth provider.
/// * `redirect_uri` - The redirect URI used in the authorization request.
/// * `code_verifier` - The code verifier used in the PKCE flow.
/// Returns
/// Ok(TokenResponse) if the exchange is successful, or Err(String) with an error message if the exchange fails.
pub async fn exchange_code_for_token(
    token_endpoint: &str,
    client_id: &str,
    code: &str,
    redirect_uri: &str,
    code_verifier: &str,
) -> Result<TokenResponse, String> {
    let client = Client::new();

    let params = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("client_id", client_id),
        ("code_verifier", code_verifier),
    ];

    let response = client
        .post(token_endpoint)
        .form(&params)
        .send()
        .await
        .map_err(|e|
            format!("Failed to send token request: {}", e)
        )?;

    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<unable to read response>".to_string());


        return Err(format!(
            "Token request failed with status {}: {}",
            status, body
        ));
    }

    response
        .json::<TokenResponse>()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))

}

/// Refreshes the access token using the refresh token.
///
/// Arguments
/// * `token_endpoint` - The token endpoint URL of the OAuth provider.
/// * `client_id` - The client ID of the OAuth application.
/// * `refresh_token` - The refresh token to use for obtaining a new access token.
/// Returns
/// Ok(TokenResponse) if the refresh is successful, or Err(String) with an error message if the refresh fails.
pub async fn refresh_access_token(
    token_endpoint: &str,
    client_id: &str,
    refresh_token: &str,
) -> Result<TokenResponse, String> {
    let client = Client::new();

    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", client_id),
    ];

    let response = client
        .post(token_endpoint)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to send token refresh request: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<unable to read response>".to_string());

        return Err(format!(
            "Token refresh request failed with status {}: {}",
            status, body
        ));
    }

    response
        .json::<TokenResponse>()
        .await
        .map_err(|e| format!("Failed to parse token refresh response: {}", e))
}

/// Revokes the access token or refresh token.
///
/// Arguments
/// * `revocation_endpoint` - The revocation endpoint URL of the OAuth provider.
/// * `client_id` - The client ID of the OAuth application.
/// * `token` - The access token or refresh token to revoke.
/// Returns
/// Ok(()) if the revocation is successful, or Err(String) with an error message if the revocation fails.
pub async fn revoke_token(
    revocation_endpoint: &str,
    client_id: &str,
    token: &str,
) -> Result<(), String> {
    let client = Client::new();

    let params = [
        ("token", token),
        ("client_id", client_id),
    ];

    let response = client
        .post(revocation_endpoint)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to send token revocation request: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<unable to read response>".to_string());

        return Err(format!(
            "Token revocation request failed with status {}: {}",
            status, body
        ));
    }

    Ok(())
}

/// Parses the ID token and extracts the claims.
///
/// Arguments
/// * `id_token` - The ID token to parse.
/// Returns
/// Ok(IdTokenClaims) if the parsing is successful, or Err(String) with an error message if the parsing fails.
pub fn parse_id_token(id_token: &str) -> Result<IdTokenClaims, String> {
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() != 3 {
        return Err("Invalid ID token format".to_string());
    }

    let payload = parts[1];
    let decoded_payload = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|e| format!("Failed to decode ID token payload: {}", e))?;

    serde_json::from_slice::<IdTokenClaims>(&decoded_payload)
        .map_err(|e| format!("Failed to parse ID token claims: {}", e))
}