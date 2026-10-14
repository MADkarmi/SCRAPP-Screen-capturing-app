use std::{io::{Read, Write}, net::TcpListener};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::{ fill};
use sha2::{Digest, Sha256};
use url::Url;

/// Represents the Proof Key for Code Exchange (PKCE) used in OAuth 2.0 authorization.
pub struct Pkce {
    pub verifier: String,
    pub challenge: String,
}

impl Pkce{
    /// Creates a new instance of Pkce with a generated verifier and challenge.
    pub fn new() -> Self {
        let verifier = Pkce::generate_verifier();
        let challenge = Pkce::generate_challenge(&verifier);
        Self { verifier, challenge }
    }

    /// Generates a random code verifier for OAuth 2.0 PKCE.
    /// Returns a URL-safe base64-encoded string.
    fn generate_verifier() -> String {
        let mut random_bytes = [0u8; 32];
        fill(&mut random_bytes);
        URL_SAFE_NO_PAD.encode(random_bytes)
    }

    /// Generates a challenge for OAuth 2.0 PKCE based on the code verifier.
    /// Returns a URL-safe base64-encoded string.
    fn generate_challenge(verifier: &str) -> String {
        let hash = Sha256::digest(verifier.as_bytes());
        URL_SAFE_NO_PAD.encode(hash)
    }
}

/// Generates a random state string for OAuth 2.0 authorization requests.
pub fn generate_state() -> String {
        let mut state_bytes = [0u8; 16];
        fill(&mut state_bytes);
        URL_SAFE_NO_PAD.encode(state_bytes)
    }

/// Opens the authorization URL in the default web browser.
///
/// Arguments
/// * `url` - The authorization URL to open in the browser.
pub fn open_authorization_url(url: &Url)
    -> Result<(), Box<dyn std::error::Error>> {
        if webbrowser::open(url.as_str()).is_ok() {
            Ok(())
        } else {
            Err("Failed to open the authorization URL in the browser.".into())
        }
}

/// Waits for the OAuth callback on localhost:4200 and extracts the authorization code from the request.
pub fn wait_for_callback(expected_state: &str) -> Result<String, String>{
    let listener = TcpListener::bind("127.0.0.1:4200").map_err(|e| format!("Failed to bind to port 4200: {}", e))?;

    let (mut stream, _ ) = listener.accept().map_err(|e| format!("Failed to accept connection: {}", e))?;

    let mut buffer: [u8; 8192] = [0; 8192];

    let bytes_read = stream.read(&mut buffer).map_err(|e| format!("Failed to read from stream: {}", e))?;

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);

    let first_line = request.lines().next().ok_or_else(|| "Invalid HTTP request".to_string())?;
    let path = first_line.split_whitespace().nth(1).ok_or_else(|| "Invalid HTTP request".to_string())?;
    let callback_url = Url::parse(&format!("http://localhost{}", path)).map_err(|e| format!("Failed to parse callback URL: {}", e))?;

    let mut code = None;
    let mut state = None;
    let mut error = None;

    for(key, value) in callback_url.query_pairs() {
        match key.as_ref() {
            "code" => code = Some(value.into_owned()),
            "state" => state = Some(value.into_owned()),
            "error" => error = Some(value.into_owned()),
            _ => {}
        }
    }

    if let Some(error) = error{
        send_callback_response(
            &mut stream,
            &format!("OAuth login failed. You can close this browser window. Error: {}", error))?;
        return Err(format!("OAuth authorization failed with error: {}", error));
    }

    let returned_state = state.ok_or_else(|| "Missing state parameter in callback".to_string())?;

    if returned_state != expected_state {
        send_callback_response(
            &mut stream,
            "OAuth login failed. You can close this browser window.")?;
        return Err("State mismatch in OAuth callback".to_string());
    }

    let code = code.ok_or_else(|| "Oauth callback did not contain authorization code".to_string())?;

    send_callback_response(&mut stream, "Login successful. You can close this browser window.")?;

    Ok(code)
}

/// Sends an HTTP response to the browser after receiving the OAuth callback.
///
/// Arguments
/// * `stream` - The TCP stream to write the response to.
/// * `message` - The message to display in the browser.
fn send_callback_response(
    stream: &mut std::net::TcpStream,
    message: &str)
    -> Result<(), String> {

        let body = format!(
            "{}",
            message);

        let response = format!(
            "HTTP/1.1 200 OK\r\n\r\n
            <!DOCTYPE html>\
            <html style=\"margin: 0; padding: 0; background-color: #121212;\">\
            <body>
            <div style=\"font-family: Arial, sans-serif; text-align: center; margin-top: 50px; color: #ffffff; min-height: 100vh; margin: 0;\">
                <h1>OAuth Login</h1>
                <p style=\"color: #e0e0e0;\">{}</p>
            </div>
            </body>
            ",
            body
        );

        stream.write_all(response.as_bytes()).map_err(|e| format!("Failed to write response: {}", e))?;
        stream.flush().map_err(|e| format!("Failed to flush stream: {}", e))?;

        Ok(())
}