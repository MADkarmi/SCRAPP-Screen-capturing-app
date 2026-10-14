# Scrap Desktop App

Desktop client for the Scrap video-sharing platform, built with **Rust** and **Slint**.

> **⚠️ Development status**
>
> This application is still under active development and is **not yet ready for production use**.

## Configuration

Because the Scrap server is currently run **locally** and has not yet been deployed to a public server, the desktop application requires a `.env` file to configure the server API endpoints.

The `.env` file is intentionally exposed during development so that users can adjust the configuration to match the local server instance.

### `.env`

Edit a `.env` file next to the application executable and configure the server endpoints, including the appropriate ports

```env
OAUTH_AUTHORIZATION_ENDPOINT=https://localhost:7104/connect/authorize
OAUTH_TOKEN_ENDPOINT=https://localhost:7104/connect/token
OAUTH_CLIENT_ID=scrap-desktop-app
OAUTH_REDIRECT_URI=http://localhost:4200/callback 
OAUTH_LOGOUT_ENDPOINT=https://localhost:7104/connect/logout
OAUTH_POST_LOGOUT_REDIRECT_URI=http://localhost:4200/logout-callback #to be changed to the actual post-logout redirect URI of application
OAUTH_REVOCATION_ENDPOINT=https://localhost:7104/connect/revoke
UPLOAD_API_ENDPOINT=https://localhost:7104/api/uploads
UPLOAD_RAW_API_ENDPOINT=stream
UPLOAD_MULTIPART_API_ENDPOINT=multipart
UPLOAD_CHUNKED_API_ENDPOINT=chunks
UPLOAD_COMPLETE_API_ENDPOINT=complete
DEFAULT_CHUNK_SIZE=52428800 # 50MB 26214400 # 25MB #104857600 # 10MB
MAX_CONCURRENT_CHUNKS=10 # Limit the number of concurrent chunk uploads
```