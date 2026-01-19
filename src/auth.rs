//! OAuth2/OIDC Authentication with PKCE for SpacetimeAuth
//!
//! Implements the authorization code flow with PKCE (Proof Key for Code Exchange)
//! for secure authentication in native applications.

use bevy::prelude::*;
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::resources::AppState;

// ============================================================================
// Configuration
// ============================================================================

const AUTH_ENDPOINT: &str = "https://auth.spacetimedb.com/oidc/auth";
const TOKEN_ENDPOINT: &str = "https://auth.spacetimedb.com/oidc/token";
const SCOPES: &str = "openid profile email";
const REDIRECT_PORT: u16 = 31419;

/// SpacetimeAuth configuration
#[derive(Resource, Clone)]
pub struct AuthConfig {
    /// Your SpacetimeAuth client ID
    pub client_id: String,
    /// Local callback port for desktop OAuth
    pub callback_port: u16,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            client_id: std::env::var("SPACETIMEDB_CLIENT_ID")
                .unwrap_or_else(|_| "client_032BJ1Hcqe1lvzV379770F".to_string()),
            callback_port: REDIRECT_PORT,
        }
    }
}

/// Current authentication state
#[derive(Resource, Default)]
pub struct AuthState {
    /// The access token once authenticated
    pub access_token: Option<String>,
    /// The ID token (contains user profile)
    pub id_token: Option<String>,
    /// The refresh token
    pub refresh_token: Option<String>,
    /// Token expiry timestamp
    pub token_expiry: Option<u64>,
    /// User profile from ID token
    pub user_profile: Option<UserProfile>,
    /// Whether we're currently waiting for auth
    pub pending: bool,
    /// Any error message
    pub error: Option<String>,
}

/// User profile from ID token
#[derive(Clone, Debug, Default)]
pub struct UserProfile {
    pub name: String,
    pub preferred_username: Option<String>,
    pub email: Option<String>,
}

// ============================================================================
// PKCE Implementation
// ============================================================================

/// PKCE codes for the authorization flow
struct PkceCodes {
    verifier: String,
    challenge: String,
}

fn generate_pkce_codes() -> PkceCodes {
    use rand::Rng;

    // Generate random 32-byte verifier
    let mut rng = rand::thread_rng();
    let verifier_bytes: Vec<u8> = (0..32).map(|_| rng.r#gen::<u8>()).collect();
    let verifier = base64_url_encode(&verifier_bytes);

    // Generate challenge = base64url(sha256(verifier))
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    let challenge = base64_url_encode(&hash);

    PkceCodes { verifier, challenge }
}

fn base64_url_encode(data: &[u8]) -> String {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    URL_SAFE_NO_PAD.encode(data)
}

// ============================================================================
// OAuth Callback Server State
// ============================================================================

/// Shared state for the auth callback server
#[derive(Clone)]
pub struct CallbackState {
    pub auth_code: Arc<Mutex<Option<String>>>,
    pub error: Arc<Mutex<Option<String>>>,
    pub pkce_verifier: Arc<Mutex<String>>,
}

/// Resource to hold the callback server state
#[derive(Resource)]
pub struct CallbackServerState(pub CallbackState);

// ============================================================================
// OAuth Flow
// ============================================================================

/// Start the OAuth2 PKCE authentication flow
pub fn start_login(config: &AuthConfig) -> CallbackState {
    let pkce = generate_pkce_codes();

    let state = CallbackState {
        auth_code: Arc::new(Mutex::new(None)),
        error: Arc::new(Mutex::new(None)),
        pkce_verifier: Arc::new(Mutex::new(pkce.verifier.clone())),
    };

    let callback_state = state.clone();
    let port = config.callback_port;
    let client_id = config.client_id.clone();
    let code_challenge = pkce.challenge.clone();

    // Start local callback server in background thread
    thread::spawn(move || {
        if let Err(e) = run_callback_server(port, callback_state) {
            log::error!("Auth callback server error: {}", e);
        }
    });

    // Build the authorization URL with PKCE
    let redirect_uri = format!("http://127.0.0.1:{}", port);
    let auth_url = format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&code_challenge={}&code_challenge_method=S256",
        AUTH_ENDPOINT,
        client_id,
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(SCOPES),
        code_challenge
    );

    info!("Opening browser for authentication...");
    if let Err(e) = open::that(&auth_url) {
        error!("Failed to open browser: {}. Please visit: {}", e, auth_url);
    }

    state
}

/// Run the local HTTP server to catch the OAuth callback
fn run_callback_server(port: u16, state: CallbackState) -> Result<(), String> {
    let addr = format!("127.0.0.1:{}", port);
    let server = tiny_http::Server::http(&addr).map_err(|e| e.to_string())?;
    log::info!("Auth callback server listening on {}", addr);

    for request in server.incoming_requests() {
        let url = request.url().to_string();
        log::info!("Received callback: {}", url);

        // Extract authorization code from query params
        if let Some(code) = extract_code_from_url(&url) {
            log::info!("Received authorization code");
            *state.auth_code.lock().unwrap() = Some(code);

            // Send success response
            let html = get_success_html();
            let response = tiny_http::Response::from_string(html)
                .with_header(
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap(),
                );
            let _ = request.respond(response);
            break;
        } else if url.contains("error=") {
            let error = extract_error_from_url(&url).unwrap_or_else(|| "Unknown error".to_string());
            log::error!("OAuth error: {}", error);
            *state.error.lock().unwrap() = Some(error.clone());

            let html = get_error_html(&error);
            let response = tiny_http::Response::from_string(html)
                .with_header(
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap(),
                );
            let _ = request.respond(response);
            break;
        }
    }

    Ok(())
}

fn extract_code_from_url(url: &str) -> Option<String> {
    let query_start = url.find('?')?;
    let query = &url[query_start + 1..];

    for param in query.split('&') {
        if let Some(value) = param.strip_prefix("code=") {
            return Some(value.to_string());
        }
    }
    None
}

fn extract_error_from_url(url: &str) -> Option<String> {
    let query_start = url.find('?')?;
    let query = &url[query_start + 1..];

    for param in query.split('&') {
        if let Some(value) = param.strip_prefix("error=") {
            return Some(urlencoding::decode(value).unwrap_or_default().to_string());
        }
    }
    None
}

// ============================================================================
// Token Exchange
// ============================================================================

/// Exchange authorization code for tokens (blocking HTTP request)
fn exchange_code_for_tokens(
    code: &str,
    verifier: &str,
    client_id: &str,
    redirect_uri: &str,
) -> Result<TokenResponse, String> {
    let body = format!(
        "grant_type=authorization_code&code={}&client_id={}&redirect_uri={}&code_verifier={}",
        urlencoding::encode(code),
        urlencoding::encode(client_id),
        urlencoding::encode(redirect_uri),
        urlencoding::encode(verifier)
    );

    // Use ureq for blocking HTTP request
    let response = ureq::post(TOKEN_ENDPOINT)
        .set("Content-Type", "application/x-www-form-urlencoded")
        .send_string(&body)
        .map_err(|e| format!("Token request failed: {}", e))?;

    let body_str = response
        .into_string()
        .map_err(|e| format!("Failed to read token response: {}", e))?;

    let json: serde_json::Value = serde_json::from_str(&body_str)
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    Ok(TokenResponse {
        access_token: json["access_token"].as_str().map(String::from),
        id_token: json["id_token"].as_str().map(String::from),
        refresh_token: json["refresh_token"].as_str().map(String::from),
        expires_in: json["expires_in"].as_u64(),
    })
}

struct TokenResponse {
    access_token: Option<String>,
    id_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
}

/// Decode JWT payload to extract user profile
fn decode_jwt_payload(jwt: &str) -> Option<UserProfile> {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

    let parts: Vec<&str> = jwt.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    // Add padding if needed for base64 decode
    let mut payload_b64 = parts[1].to_string();
    while payload_b64.len() % 4 != 0 {
        payload_b64.push('=');
    }

    let payload = URL_SAFE_NO_PAD.decode(&payload_b64).ok()?;
    let json: serde_json::Value = serde_json::from_slice(&payload).ok()?;

    Some(UserProfile {
        name: json["name"].as_str().unwrap_or_default().to_string(),
        preferred_username: json["preferred_username"].as_str().map(|s| s.to_string()),
        email: json["email"].as_str().map(|s| s.to_string()),
    })
}

// ============================================================================
// HTML Responses
// ============================================================================

fn get_success_html() -> String {
    r#"<!DOCTYPE html>
<html>
<head>
    <title>Authentication Successful</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
        }
        .container {
            text-align: center;
            padding: 40px;
            background: rgba(255, 255, 255, 0.1);
            border-radius: 20px;
            backdrop-filter: blur(10px);
        }
        h1 { margin: 0 0 20px 0; }
    </style>
</head>
<body>
    <div class="container">
        <h1>Authentication Successful!</h1>
        <p>You can close this window and return to the game.</p>
    </div>
</body>
</html>"#
        .to_string()
}

fn get_error_html(error: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Authentication Error</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #ea5455 0%, #feb692 100%);
            color: white;
        }}
        .container {{
            text-align: center;
            padding: 40px;
            background: rgba(255, 255, 255, 0.1);
            border-radius: 20px;
            backdrop-filter: blur(10px);
        }}
        h1 {{ margin: 0 0 20px 0; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>❌ Authentication Error</h1>
        <p>{}</p>
        <p>Please close this window and try again.</p>
    </div>
</body>
</html>"#,
        error
    )
}

// ============================================================================
// URL Encoding Helper
// ============================================================================

mod urlencoding {
    pub fn encode(s: &str) -> String {
        url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
    }

    pub fn decode(s: &str) -> Result<String, std::string::FromUtf8Error> {
        let decoded: String = url::form_urlencoded::parse(s.as_bytes())
            .map(|(k, v)| if v.is_empty() { k.to_string() } else { format!("{}={}", k, v) })
            .collect();
        Ok(decoded)
    }
}

// ============================================================================
// Login Screen UI
// ============================================================================

/// Marker for the login screen root entity
#[derive(Component)]
pub struct LoginScreen;

/// Marker for the login button
#[derive(Component)]
pub struct LoginButton;

/// Marker for anonymous play button
#[derive(Component)]
pub struct AnonymousPlayButton;

/// Setup the login screen UI
pub fn setup_login_screen(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(30.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
            LoginScreen,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Tower Defense MMO"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Subtitle
            parent.spawn((
                Text::new("Defend together with friends!"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
            ));

            // Login button
            parent
                .spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(40.0), Val::Px(15.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor::all(Color::srgb(0.3, 0.7, 0.3)),
                    BackgroundColor(Color::srgb(0.2, 0.5, 0.2)),
                    LoginButton,
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("Login to Play"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Anonymous play option
            parent
                .spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(30.0), Val::Px(10.0)),
                        margin: UiRect::top(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    AnonymousPlayButton,
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("Play Anonymously"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.5, 0.5, 0.5, 1.0)),
                    ));
                });
        });
}

/// Handle login button clicks
pub fn handle_login_button(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<LoginButton>)>,
    config: Res<AuthConfig>,
    mut auth_state: ResMut<AuthState>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            info!("Login button pressed, starting OAuth PKCE flow...");
            let callback_state = start_login(&config);
            commands.insert_resource(CallbackServerState(callback_state));
            auth_state.pending = true;
        }
    }
}

/// Handle anonymous play button clicks
pub fn handle_anonymous_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<AnonymousPlayButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            info!("Playing anonymously...");
            next_state.set(AppState::InGame);
        }
    }
}

/// Update button colors on hover
pub fn update_login_button_colors(
    mut login_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (With<LoginButton>, Changed<Interaction>),
    >,
    mut anon_query: Query<
        (&Interaction, &mut BackgroundColor),
        (With<AnonymousPlayButton>, Changed<Interaction>, Without<LoginButton>),
    >,
) {
    for (interaction, mut bg_color, mut border_color) in &mut login_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.4, 0.15));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.6, 0.25));
                *border_color = BorderColor::all(Color::srgb(0.4, 0.8, 0.4));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.5, 0.2));
                *border_color = BorderColor::all(Color::srgb(0.3, 0.7, 0.3));
            }
        }
    }

    for (interaction, mut bg_color) in &mut anon_query {
        match *interaction {
            Interaction::Pressed | Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.3));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::NONE);
            }
        }
    }
}

/// Check if auth completed and exchange code for tokens, then connect to SpacetimeDB
pub fn check_auth_and_connect(
    callback_state: Option<Res<CallbackServerState>>,
    mut auth_state: ResMut<AuthState>,
    config: Res<AuthConfig>,
    mut stdb_config: ResMut<crate::resources::StdbConfig>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Some(callback) = callback_state else {
        return;
    };

    if !auth_state.pending {
        return;
    }

    // Check if we got an authorization code
    if let Ok(code_lock) = callback.0.auth_code.lock() {
        if let Some(code) = code_lock.as_ref() {
            info!("Got authorization code, exchanging for tokens...");

            let verifier = callback.0.pkce_verifier.lock().unwrap().clone();
            let redirect_uri = format!("http://127.0.0.1:{}", config.callback_port);

            match exchange_code_for_tokens(code, &verifier, &config.client_id, &redirect_uri) {
                Ok(tokens) => {
                    info!("Token exchange successful!");

                    if let Some(ref access_token) = tokens.access_token {
                        auth_state.access_token = Some(access_token.clone());

                        // Store token in StdbConfig for the delayed connection
                        stdb_config.token = Some(access_token.clone());
                    }

                    auth_state.id_token = tokens.id_token.clone();
                    auth_state.refresh_token = tokens.refresh_token;

                    if let Some(expires_in) = tokens.expires_in {
                        auth_state.token_expiry = Some(
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs()
                                + expires_in,
                        );
                    }

                    // Decode user profile from ID token
                    if let Some(ref id_token) = tokens.id_token {
                        if let Some(profile) = decode_jwt_payload(id_token) {
                            info!(
                                "Logged in as: {}",
                                profile.preferred_username.as_deref().unwrap_or(&profile.name)
                            );
                            auth_state.user_profile = Some(profile);
                        }
                    }

                    auth_state.pending = false;

                    // Transition to InGame state - connection will be established there
                    info!("Authentication successful! Connecting to game...");
                    next_state.set(AppState::InGame);
                }
                Err(e) => {
                    error!("Token exchange failed: {}", e);
                    auth_state.error = Some(e);
                    auth_state.pending = false;
                }
            }
        }
    }

    // Check for errors
    if let Ok(error) = callback.0.error.lock() {
        if let Some(e) = error.as_ref() {
            error!("Authentication failed: {}", e);
            auth_state.error = Some(e.clone());
            auth_state.pending = false;
        }
    }
}

/// Cleanup login screen when leaving
pub fn cleanup_login_screen(mut commands: Commands, query: Query<Entity, With<LoginScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// ============================================================================
// Token Persistence (for loading saved tokens on startup)
// ============================================================================

const TOKEN_FILE: &str = ".spacetimedb_token";

/// Load the access token from file if it exists
pub fn load_token_from_file() -> Option<String> {
    match std::fs::read_to_string(TOKEN_FILE) {
        Ok(token) if !token.is_empty() => {
            info!("Loaded token from {}", TOKEN_FILE);
            Some(token.trim().to_string())
        }
        _ => None,
    }
}
