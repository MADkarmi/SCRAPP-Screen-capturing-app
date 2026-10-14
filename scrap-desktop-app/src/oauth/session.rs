use std::{
    sync::{Arc, Mutex}, time::{Duration, Instant}
};

use crate::oauth::{token};

use super::{
    OAuthConfig,
    user::OAuthUser,
    token::TokenResponse,
    };

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct SessionData {
    token_response: TokenResponse,
    expires_at: Instant,
    user: Option<OAuthUser>,
}

/// Represents an OAuth session, managing the access token, refresh token, and user information.
#[derive(Debug, Clone)]
pub struct OAuthSession {
    session: Arc<Mutex<Option<SessionData>>>,
}

#[allow(dead_code)]
impl OAuthSession {
    /// Creates a new instance of OAuthSession with no active session.
    pub fn new() -> Self {
       OAuthSession {
           session: Arc::new(Mutex::new(None))
       }
    }

    /// Sets the session data with the provided TokenResponse.
    ///
    /// Arguments
    /// * `token_response` - The TokenResponse containing the access token, refresh token, and expiration information.
    pub fn set(&self, token_response: TokenResponse) {
        let expires_at = Instant::now() + Duration::from_secs(token_response.expires_in.saturating_sub(60)); // Subtract 60 seconds to account for clock skew

        let user = token_response
            .id_token
            .as_ref()
            .and_then(|id_token| {
                match token::parse_id_token(id_token){
                    Ok(claims) =>Some(
                        OAuthUser{
                            name: claims.name,
                            email: claims.email,
                        }
                    ),
                    Err(e) => {
                        eprintln!("Failed to parse ID token: {e}");
                        None
                    }
                }
            });

        if let Ok( mut session) = self.session.lock() {
            *session = Some(SessionData {
                token_response,
                expires_at,
                user,
            });
        }
    }

    /// Clears the current session data, effectively logging out the user from local application.
    pub fn clear(&self) {
        if let Ok( mut session) = self.session.lock() {
            *session = None;
        }
    }

    /// Retrieves the current TokenResponse if a session is active.
    ///
    /// Returns
    /// Some(TokenResponse) if a session is active, or None if no session is active.
    pub fn get(&self) -> Option<TokenResponse> {
        self.session
            .lock()
            .ok()
            .and_then(|session| {
                session.as_ref()
                    .map(|data| data.token_response.clone())
            })
    }

    /// Checks if the user is currently authenticated (i.e., if a session is active).
    ///
    /// Returns
    /// true if a session is active, false otherwise.
    pub fn is_authenticated(&self) -> bool {
        self.session
            .lock()
            .map(|session| session.is_some())
            .unwrap_or(false)
    }

    /// Checks if the access token is expired.
    ///
    /// Returns
    /// true if the access token is expired or if there is no active session, false otherwise
    pub fn is_access_token_expired(&self) -> bool {
        self.session
            .lock()
            .map(|session| {
                match session.as_ref() {
                    Some(data) => Instant::now() >= data.expires_at,
                    None => true,
                }
            })
            .unwrap_or(true)
    }

    /// Retrieves the refresh token from the current session if available.
    ///
    /// Returns
    /// Ok(String) containing the refresh token if available, or Err(String) with an error message if not available.
    pub fn get_refresh_token(&self) -> Result<String, String>{
        let refresh_token = {
            let session = self
                .session
                .lock()
                .map_err(|_| "Failed to acquire lock on session".to_string())?;

            let session = session
                .as_ref()
                .ok_or_else(||  "No active OAuth session".to_string())?;

            session
                .token_response
                .refresh_token
                .clone()
                .ok_or_else(|| "No refresh token available".to_string())?
        };

        Ok(refresh_token)
    }

    /// Refreshes the access token using the refresh token.
    /// Arguments
    /// * `config` - The OAuth configuration containing the token endpoint and client ID.
    /// Returns
    /// Ok(()) if the refresh is successful, or Err(String) with an error message if the refresh fails.
    pub fn refresh(&self, config: &OAuthConfig) -> Result<(), String> {
        let refresh_token = self.get_refresh_token()?;

        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create Tokio runtime: {e}"))?;

        let new_token = runtime.block_on(
            super::token::refresh_access_token(
                &config.token_endpoint,
                &config.client_id,
                &refresh_token,
            )
        )?;
        self.set(new_token);

        Ok(())
    }

    /// Ensures that the access token is valid and refreshes it if necessary.
    /// Arguments
    /// * `config` - The OAuth configuration containing the token endpoint and client ID.
    /// Returns
    /// Ok(()) if the access token is valid or successfully refreshed, or Err(String)
    pub fn ensure_valid(&self, config: &OAuthConfig) -> Result<(), String> {
        if !self.is_authenticated(){
            return Err("User is not authenticated".to_string());
        }

        if !self.is_access_token_expired() {
            return Ok(());
        }

        self.refresh(config)
    }

    /// Retrieves the user information from the current session if available.
    ///
    /// Returns
    /// Some(OAuthUser) if user information is available, or None if no session is active or user information is not available.
    pub fn user(&self) -> Option<OAuthUser> {
        self.session
            .lock()
            .ok()
            .and_then(|session| {
                session.as_ref()
                    .and_then(|data| data.user.clone())
            })
    }

    /// Retrieves the access token from the current session if available.
    ///
    /// Arguments
    /// * `config` - The OAuth configuration containing the token endpoint and client ID.
    /// Returns
    /// Ok(String) containing the access token if available, or Err(String) with an error
    pub fn get_access_token(&self, config: &OAuthConfig) -> Result<String, String> {
        self.ensure_valid(config)?;

        let access_token = self
            .session
            .lock()
            .map_err(|_| "Failed to acquire lock on session".to_string())?
            .as_ref()
            .ok_or_else(|| "No active OAuth session".to_string())?
            .token_response
            .access_token
            .clone();
        Ok(access_token)
    }

    // pub fn logout(&self, config: &OAuthConfig) {
    //     let id_token = {
    //         let session = self
    //             .session
    //             .lock()
    //             .expect("Failed to acquire lock on session");

    //         let session = session
    //             .as_ref().ok_or_else(|| "No active OAuth session".to_string());

    //         session
    //             .token_response
    //             .id_token
    //             .clone()
    //             .ok_or_else(|| "No ID token available".to_string())
    //     };


    //     self.clear();
    // }
}
