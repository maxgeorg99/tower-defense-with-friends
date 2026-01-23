use log::info;
use spacetimedb::{Identity, JwtClaims, ReducerContext, SpacetimeType, Table, Timestamp, ViewContext};
use serde::{Deserialize, Serialize};

#[spacetimedb::table(name = user, public)]
pub struct User {
    #[primary_key]
    identity: Identity,
    name: Option<String>,
    color: Color,
    online: bool,
}

#[derive(SpacetimeType, Debug, Clone)]
pub enum Color {
    Blue,
    Yellow,
    Purple,
    Black
}
#[spacetimedb::table(name = message, public)]
pub struct Message {
    sender: Identity,
    sent: Timestamp,
    text: String,
}

#[spacetimedb::view(name = my_user, public)]
fn my_user(ctx: &ViewContext) -> Option<User> {
    ctx.db.user().identity().find(ctx.sender)
}

fn validate_name(name: String) -> Result<String, String> {
    if name.is_empty() {
        Err("Names must not be empty".to_string())
    } else {
        Ok(name)
    }
}

#[spacetimedb::reducer]
pub fn set_name(ctx: &ReducerContext, name: String) -> Result<(), String> {
    let name = validate_name(name)?;
    if let Some(user) = ctx.db.user().identity().find(ctx.sender) {
        log::info!("User {} sets name to {name}", ctx.sender);
        ctx.db.user().identity().update(User {
            name: Some(name),
            ..user
        });
        Ok(())
    } else {
        Err("Cannot set name for unknown user".to_string())
    }
}


#[spacetimedb::reducer]
pub fn set_color(ctx: &ReducerContext, color: Color) -> Result<(), String> {
    if let Some(user) = ctx.db.user().identity().find(ctx.sender) {
        log::info!("User {} sets color to {:?}", ctx.sender, color);
        ctx.db.user().identity().update(User {
            color,
            ..user
        });
        Ok(())
    } else {
        Err("Cannot set color for unknown user".to_string())
    }
}

fn validate_message(text: String) -> Result<String, String> {
    if text.is_empty() {
        Err("Messages must not be empty".to_string())
    } else {
        Ok(text)
    }
}

#[spacetimedb::reducer]
pub fn send_message(ctx: &ReducerContext, text: String) -> Result<(), String> {
    let text = validate_message(text)?;
    log::info!("User {}: {text}", ctx.sender);
    ctx.db.message().insert(Message {
        sender: ctx.sender,
        text,
        sent: ctx.timestamp,
    });
    Ok(())
}

#[spacetimedb::reducer(init)]
// Called when the module is initially published
pub fn init(_ctx: &ReducerContext) {}

#[derive(Debug, Serialize, Deserialize)]
struct UserProfile {
    name: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
    preferred_username: Option<String>,
    email: Option<String>,
    email_verified: Option<bool>,
    picture: Option<String>,
}

fn extract_user_profile(jwt: &JwtClaims) -> Option<UserProfile> {
    let payload = jwt.raw_payload();
    let json: serde_json::Value = serde_json::from_slice(payload.as_ref()).ok()?;

    info!("{}", json.to_string().as_str());
    Some(UserProfile {
        name: json["name"].as_str().map(|s| s.to_string()),
        given_name: json["given_name"].as_str().map(|s| s.to_string()),
        family_name: json["family_name"].as_str().map(|s| s.to_string()),
        preferred_username: json["preferred_username"].as_str().map(|s| s.to_string()),
        email: json["email"].as_str().map(|s| s.to_string()),
        email_verified: json["email_verified"].as_bool(),
        picture: json["picture"].as_str().map(|s| s.to_string()),
    })
}

#[spacetimedb::reducer(client_connected)]
pub fn identity_connected(ctx: &ReducerContext) {
    let auth_ctx = ctx.sender_auth();

    let name = if let Some(jwt) = auth_ctx.jwt() {
        if let Some(profile) = extract_user_profile(jwt) {
            let display_name = profile.name
                .or_else(|| profile.given_name.clone());

            display_name
        } else {
            log::warn!("Failed to parse JWT payload for user: {}", ctx.sender);
            None
        }
    } else {
        log::info!("User connected anonymously: {}", ctx.sender);
        None
    };

    if let Some(user) = ctx.db.user().identity().find(ctx.sender) {
        // Returning user - update online status
        ctx.db.user().identity().update(User {
            online: true,
            // Keep existing data if already set, otherwise use JWT data
            name: user.name.or(name),
            color: user.color,
            ..user
        });
    } else {
        // New user
        ctx.db.user().insert(User {
            identity: ctx.sender,
            name,
            color: Color::Purple,
            online: true,
        });
    }
}

#[spacetimedb::reducer(client_disconnected)]
pub fn identity_disconnected(ctx: &ReducerContext) {
    if let Some(user) = ctx.db.user().identity().find(ctx.sender) {
        ctx.db.user().identity().update(User {
            online: false,
            ..user
        });
    } else {
        // This branch should be unreachable,
        // as it doesn't make sense for a client to disconnect without connecting first.
        log::warn!(
            "Disconnect event for unknown user with identity {:?}",
            ctx.sender
        );
    }
}
