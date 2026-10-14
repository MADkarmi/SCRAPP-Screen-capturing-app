use crate::oauth::token::TokenResponse;
use url::Url;
use std::env;


pub mod client;
pub mod token;
pub mod session;
pub mod user;
use client::Pkce;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    authorization_endpoint: String,
    token_endpoint: String,
    client_id: String,
    redirect_uri: String,
    logout_endpoint: String,
    post_logout_redirect_uri: String,
    revocation_endpoint: String,
}

impl OAuthConfig {
    /// Creates a new instance of OAuthConfig with the provided parameters.
    /// Arguments
    /// * `authorization_endpoint` - The URL of the OAuth authorization endpoint.
    /// * `token_endpoint` - The URL of the OAuth token endpoint.
    /// * `client_id` - The client ID for the OAuth application.
    /// * `redirect_uri` - The redirect URI for the OAuth application.
    /// * `logout_endpoint` - The URL of the OAuth logout endpoint.
    /// * `post_logout_redirect_uri` - The post-logout redirect URI for the OAuth application.
    /// * `revocation_endpoint` - The URL of the OAuth token revocation endpoint.
    /// Returns
    /// OAuthConfig - A new instance of OAuthConfig with the provided parameters.
    pub fn from_env() -> Result<Self, env::VarError> {
        let authorization_endpoint = env::var("OAUTH_AUTHORIZATION_ENDPOINT")?;
        let token_endpoint = env::var("OAUTH_TOKEN_ENDPOINT")?;
        let client_id = env::var("OAUTH_CLIENT_ID")?;
        let redirect_uri = env::var("OAUTH_REDIRECT_URI")?;
        let logout_endpoint = env::var("OAUTH_LOGOUT_ENDPOINT")?;
        let post_logout_redirect_uri = env::var("OAUTH_POST_LOGOUT_REDIRECT_URI")?;
        let revocation_endpoint = env::var("OAUTH_REVOCATION_ENDPOINT")?;

        Ok(OAuthConfig {
            authorization_endpoint,
            token_endpoint,
            client_id,
            redirect_uri,
            logout_endpoint,
            post_logout_redirect_uri,
            revocation_endpoint,
        })
    }

    /// Builds the authorization URL for the OAuth flow with the provided PKCE and state.
    /// Arguments
    /// * `pkce` - A reference to the Pkce struct containing the code challenge and verifier.
    /// * `state` - A string representing the state parameter for the OAuth flow.
    /// Returns
    /// Result<Url, url::ParseError> - A Result containing the constructed authorization URL or a url::ParseError if the URL could not be constructed.
    pub fn build_authorization_url(
        &self,
        pkce: &Pkce,
        state: &str
        ) -> Result<Url, url::ParseError> {
            let mut url = Url::parse(&self.authorization_endpoint)?;
            url.query_pairs_mut()
                .append_pair("response_type", "code")
                .append_pair("client_id", &self.client_id)
                .append_pair("redirect_uri", &self.redirect_uri)
                .append_pair("scope", "openid profile email offline_access")
                .append_pair("state", state)
                .append_pair("code_challenge", &pkce.challenge)
                .append_pair("code_challenge_method", "S256");
            Ok(url)
    }
}

/// Logs in the user by initiating the OAuth flow, opening the authorization URL in the browser, and exchanging the authorization code for an access token.
/// Arguments
/// * `oauth_config` - A reference to the OAuthConfig struct containing the OAuth configuration.
/// Returns
/// Result<TokenResponse, String> - A Result containing the TokenResponse with the access token or an error message if the login process fails.
/// This function performs the following steps:
/// 1. Generates a PKCE challenge and state.
/// 2. Builds the authorization URL using the provided OAuth configuration.
/// 3. Opens the authorization URL in the user's default web browser.
/// 4. Waits for the OAuth callback to receive the authorization code.
/// 5. Exchanges the authorization code for an access token using the token endpoint.
///
pub fn login_user(oauth_config: &OAuthConfig) -> Result<TokenResponse, String> {

    let pkce = client::Pkce::new();

    let state = client::generate_state();

    let authorization_url = oauth_config
        .build_authorization_url(&pkce, &state)
        .map_err(|e| format!("Failed to build authorization URL: {e}"))?;

    let state_for_callback = state.clone();
    let callback_thread = std::thread::spawn(move||{
        client::wait_for_callback(&state_for_callback)
    });

    client::open_authorization_url(&authorization_url)
        .map_err(|e| format!("Failed to open authorization URL in browser: {e}"))?;

    let code = callback_thread
        .join()
        .map_err(|_| "OAuth callback thread panicked".to_string())?
        .map_err(|e| format!("OAuth callback failed: {e}"))?;

    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create Tokio runtime: {e}"))?;

    runtime.block_on(token::exchange_code_for_token(
        &oauth_config.token_endpoint,
        &oauth_config.client_id,
        &code,
        &oauth_config.redirect_uri,
        &pkce.verifier,
    )).map_err(|e| format!("Failed to exchange code for token: {e}"))

}

/// Logs out the user by revoking the refresh token and clearing the OAuth session.
/// Arguments
/// * `session` - A reference to the OAuthSession struct containing the current OAuth session.
/// * `oauth_config` - A reference to the OAuthConfig struct containing the OAuth configuration.
/// Returns
/// Result<(), String> - A Result indicating success or failure of the logout process.
/// This function performs the following steps:
/// 1. Checks if the user is authenticated.
/// 2. Retrieves the refresh token from the current session.
/// 3. Revokes the refresh token using the revocation endpoint.
/// 4. Clears the OAuth session.
/// 
pub fn logout_user(
    session: &session::OAuthSession,
    oauth_config: &OAuthConfig,
    ) -> Result<(), String> {
        if !session.is_authenticated() {
            return Ok(());
        }

        let refresh_token = session.get_refresh_token()?;

        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create Tokio runtime: {e}"))?;

        runtime.block_on(
        token::revoke_token(
                &oauth_config.revocation_endpoint,
                &oauth_config.client_id,
                &refresh_token,
            )
        )?;

        session.clear();
        Ok(())
    }