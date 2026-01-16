use spacetimedb::{Identity, ReducerContext, Table, Timestamp, ViewContext};

/// Your SpacetimeAuth OIDC client ID - set this to match your project
const OIDC_CLIENT_ID: &str = "client_XXXXXXXXXXXXXXXXXXXXXX";

#[spacetimedb::table(name = user, public)]
pub struct User {
    #[primary_key]
    identity: Identity,
    name: Option<String>,
    email: Option<String>,
    online: bool,
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

fn validate_message(text: String) -> Result<String, String> {
    if text.is_empty() {
        Err("Messages must not be empty".to_string())
    } else {
        Ok(text)
    }
}

#[spacetimedb::reducer]
pub fn send_message(ctx: &ReducerContext, text: String) -> Result<(), String> {
    // Things to consider:
    // - Rate-limit messages per-user.
    // - Reject messages from unnamed user.
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

#[spacetimedb::reducer(client_connected)]
pub fn identity_connected(ctx: &ReducerContext) {
    // Extract auth info from JWT if present
    let auth_ctx = ctx.sender_auth();
    let (name, email) = if let Some(jwt) = auth_ctx.jwt() {
        // Use subject as fallback name
        let name = jwt.subject().to_string();

        log::info!(
            "User connected with JWT - sub: {}, iss: {}",
            jwt.subject(),
            jwt.issuer()
        );

        (Some(name), None::<String>)
    } else {
        log::info!("User connected anonymously: {}", ctx.sender);
        (None, None)
    };

    if let Some(user) = ctx.db.user().identity().find(ctx.sender) {
        // Returning user - update online status, preserve existing name if set
        ctx.db.user().identity().update(User {
            online: true,
            // Keep existing name if already set, otherwise use JWT name
            name: user.name.or(name),
            email: user.email.or(email),
            ..user
        });
    } else {
        // New user
        ctx.db.user().insert(User {
            identity: ctx.sender,
            name,
            email,
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
