#![allow(clippy::disallowed_macros)]

mod module_bindings;
mod tui;

use module_bindings::*;
use std::env;
use std::sync::{Arc, Mutex};

use spacetimedb_sdk::{credentials, DbContext, Error, Event, Identity, Status, Table, TableWithPrimaryKey};
use tui::AppState;

// ## Define the main function

fn main() {
    // Connect to devtools for hot-reloading if feature is enabled
    #[cfg(feature = "devtools")]
    {
        std::thread::spawn(|| {
            dioxus_devtools::connect_subsecond();
        });
    }

    // Create shared app state
    let app_state = Arc::new(Mutex::new(AppState::new()));
    let my_identity = Arc::new(Mutex::new(None));

    // Connect to the database
    let ctx = connect_to_db(my_identity.clone());

    // Register callbacks to run in response to database events.
    register_callbacks(&ctx, app_state.clone());

    // Subscribe to SQL queries in order to construct a local partial replica of the database.
    subscribe_to_tables(&ctx, app_state.clone());

    // Spawn a thread, where the connection will process messages and invoke callbacks.
    ctx.run_threaded();

    // Run the TUI
    if let Err(e) = tui::run_tui(ctx, app_state, my_identity) {
        eprintln!("TUI error: {e}");
        std::process::exit(1);
    }
}

// ## Connect to the database

/// Load credentials from a file and connect to the database.
fn connect_to_db(my_identity: Arc<Mutex<Option<Identity>>>) -> DbConnection {
    // The URI of the SpacetimeDB instance hosting our chat module.
    let host: String = env::var("SPACETIMEDB_HOST").unwrap_or("http://localhost:3000".to_string());

    // The module name we chose when we published our module.
    let db_name: String = env::var("SPACETIMEDB_DB_NAME").unwrap_or("quickstart-chat".to_string());

    // Check if we should use a fresh identity (for testing multiple users)
    let use_fresh_identity = env::var("FRESH_IDENTITY").is_ok();

    let mut builder = DbConnection::builder()
        // Register our `on_connect` callback, which will save our auth token.
        .on_connect(move |ctx, identity, token| on_connected(ctx, identity, token, my_identity.clone()))
        // Register our `on_connect_error` callback, which will print a message, then exit the process.
        .on_connect_error(on_connect_error)
        // Our `on_disconnect` callback, which will print a message, then exit the process.
        .on_disconnect(on_disconnected);

    // Only load saved credentials if we're not using a fresh identity
    if !use_fresh_identity {
        if let Ok(token) = creds_store().load() {
            builder = builder.with_token(token);
        }
    }

    builder
        // Set the database name we chose when we called `spacetime publish`.
        .with_module_name(db_name)
        // Set the URI of the SpacetimeDB host that's running our database.
        .with_uri(host)
        // Finalize configuration and connect!
        .build()
        .expect("Failed to connect")
}

// ### Save credentials

fn creds_store() -> credentials::File {
    credentials::File::new("quickstart-chat")
}

/// Our `on_connect` callback: save our credentials to a file (unless using fresh identity).
fn on_connected(_ctx: &DbConnection, identity: Identity, token: &str, my_identity: Arc<Mutex<Option<Identity>>>) {
    // Only save credentials if not using a fresh identity
    if env::var("FRESH_IDENTITY").is_err() {
        if let Err(e) = creds_store().save(token) {
            eprintln!("Failed to save credentials: {e:?}");
        }
    }
    *my_identity.lock().unwrap() = Some(identity);
}

// ### Handle errors and disconnections

/// Our `on_connect_error` callback: print the error, then exit the process.
fn on_connect_error(_ctx: &ErrorContext, err: Error) {
    eprintln!("Connection error: {err}");
    std::process::exit(1);
}

/// Our `on_disconnect` callback: print a note, then exit the process.
fn on_disconnected(_ctx: &ErrorContext, err: Option<Error>) {
    if let Some(err) = err {
        eprintln!("Disconnected: {err}");
        std::process::exit(1);
    } else {
        println!("Disconnected.");
        std::process::exit(0);
    }
}

// ## Register callbacks

/// Register all the callbacks our app will use to respond to database events.
fn register_callbacks(ctx: &DbConnection, state: Arc<Mutex<AppState>>) {
    let state_clone = state.clone();
    ctx.db.user().on_insert(move |ctx, user| {
        on_user_inserted(ctx, user, state_clone.clone());
    });

    let state_clone = state.clone();
    ctx.db.user().on_update(move |ctx, old, new| {
        on_user_updated(ctx, old, new, state_clone.clone());
    });

    let state_clone = state.clone();
    ctx.db.message().on_insert(move |ctx, message| {
        on_message_inserted(ctx, message, state_clone.clone());
    });

    ctx.reducers.on_set_name(on_name_set);
    ctx.reducers.on_send_message(on_message_sent);
}

// ### Notify about new users

/// Our `User::on_insert` callback: trigger redraw.
fn on_user_inserted(_ctx: &EventContext, _user: &User, state: Arc<Mutex<AppState>>) {
    let mut app_state = state.lock().unwrap();
    app_state.trigger_redraw();
}

// ### Notify about updated users

/// Our `User::on_update` callback: trigger redraw.
fn on_user_updated(_ctx: &EventContext, _old: &User, _new: &User, state: Arc<Mutex<AppState>>) {
    let mut app_state = state.lock().unwrap();
    app_state.trigger_redraw();
}

// ### Handle messages

/// Our `Message::on_insert` callback: trigger redraw.
fn on_message_inserted(ctx: &EventContext, _message: &Message, state: Arc<Mutex<AppState>>) {
    if !matches!(ctx.event, Event::SubscribeApplied) {
        let mut app_state = state.lock().unwrap();
        app_state.trigger_redraw();
    }
}

// ### Handle reducer failures

/// Our `on_set_name` callback: print a warning if the reducer failed.
fn on_name_set(ctx: &ReducerEventContext, name: &String) {
    if let Status::Failed(err) = &ctx.event.status {
        eprintln!("Failed to change name to {name:?}: {err}");
    }
}

/// Our `on_send_message` callback: print a warning if the reducer failed.
fn on_message_sent(ctx: &ReducerEventContext, text: &String) {
    if let Status::Failed(err) = &ctx.event.status {
        eprintln!("Failed to send message {text:?}: {err}");
    }
}

// ## Subscribe to tables

/// Register subscriptions for all rows of both tables.
fn subscribe_to_tables(ctx: &DbConnection, state: Arc<Mutex<AppState>>) {
    ctx.subscription_builder()
        .on_applied(move |ctx| on_sub_applied(ctx, state.clone()))
        .on_error(on_sub_error)
        .subscribe(["SELECT * FROM user", "SELECT * FROM message"]);
}

// ### Load past messages in order

/// Our `on_applied` callback: trigger initial redraw.
fn on_sub_applied(_ctx: &SubscriptionEventContext, state: Arc<Mutex<AppState>>) {
    // All data is now loaded in the database, trigger a redraw
    let mut app_state = state.lock().unwrap();
    app_state.trigger_redraw();
}

// ### Notify about failed subscriptions

/// Or `on_error` callback:
/// print the error, then exit the process.
fn on_sub_error(_ctx: &ErrorContext, err: Error) {
    eprintln!("Subscription failed: {err}");
    std::process::exit(1);
}
