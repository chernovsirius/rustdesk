//! SiriusDesk: server settings baked into the binary at build time.
//!
//! The values come from environment variables at compile time (in CI they are
//! GitHub Actions secrets, see `.github/workflows/flutter-build.yml`):
//!
//! - `SIRIUS_ID_SERVER`    - ID/rendezvous server, e.g. `desk.it-sirius.ru`
//! - `SIRIUS_RELAY_SERVER` - relay server (optional, derived from the ID server if empty)
//! - `SIRIUS_API_SERVER`   - API server, e.g. `https://desk.it-sirius.ru`
//! - `SIRIUS_KEY`          - public key of the server (`id_ed25519.pub`)
//!
//! If `SIRIUS_ID_SERVER` is not set, the build behaves like stock RustDesk.
//!
//! The settings dialog offers "Sirius server" / "Custom settings". The first one
//! writes the baked values into the regular options, the second one lets the user
//! enter a corporate server or leave everything empty to use the public servers.
//! On the very first start of a fresh install (no server configured yet) the
//! Sirius server is applied automatically; an already configured server
//! (e.g. a corporate one) is never touched.

use hbb_common::config::{self, keys, Config};

const OPTION_CUSTOM_RENDEZVOUS_SERVER: &str = "custom-rendezvous-server";
const OPTION_API_SERVER: &str = "api-server";
const OPTION_KEY: &str = "key";

const fn or_empty(v: Option<&'static str>) -> &'static str {
    match v {
        Some(v) => v,
        None => "",
    }
}

pub const ID_SERVER: &str = or_empty(option_env!("SIRIUS_ID_SERVER"));
pub const RELAY_SERVER: &str = or_empty(option_env!("SIRIUS_RELAY_SERVER"));
pub const API_SERVER: &str = or_empty(option_env!("SIRIUS_API_SERVER"));
pub const KEY: &str = or_empty(option_env!("SIRIUS_KEY"));

/// Builtin options read by the Flutter UI via `bind.mainGetBuildinOption`.
pub const BUILTIN_ID_SERVER: &str = "sirius-id-server";
pub const BUILTIN_RELAY_SERVER: &str = "sirius-relay-server";
pub const BUILTIN_API_SERVER: &str = "sirius-api-server";
pub const BUILTIN_KEY: &str = "sirius-key";

/// Set once the first-run defaults have been applied, so that a user who
/// switched to public servers (all fields empty) is not switched back.
const OPTION_INITIALIZED: &str = "sirius-server-initialized";

#[inline]
pub fn is_builtin() -> bool {
    !ID_SERVER.trim().is_empty()
}

/// Expose the baked values to the UI. Called from `load_custom_client`,
/// i.e. in every process.
pub fn load_builtin_settings() {
    if !is_builtin() {
        return;
    }
    let mut builtin = config::BUILTIN_SETTINGS.write().unwrap();
    builtin.insert(BUILTIN_ID_SERVER.to_owned(), ID_SERVER.trim().to_owned());
    builtin.insert(
        BUILTIN_RELAY_SERVER.to_owned(),
        RELAY_SERVER.trim().to_owned(),
    );
    builtin.insert(BUILTIN_API_SERVER.to_owned(), API_SERVER.trim().to_owned());
    builtin.insert(BUILTIN_KEY.to_owned(), KEY.trim().to_owned());
}

/// Apply the Sirius server on the first start of a fresh install.
/// Must run in the process that owns the config (the server/service process),
/// the UI processes receive the options via IPC sync.
pub fn apply_first_run_defaults() {
    if !is_builtin() || !Config::get_option(OPTION_INITIALIZED).is_empty() {
        return;
    }
    if Config::get_option(OPTION_CUSTOM_RENDEZVOUS_SERVER).is_empty() {
        log::info!("Applying built-in Sirius server settings");
        Config::set_option(
            OPTION_CUSTOM_RENDEZVOUS_SERVER.to_owned(),
            ID_SERVER.trim().to_owned(),
        );
        Config::set_option(
            keys::OPTION_RELAY_SERVER.to_owned(),
            RELAY_SERVER.trim().to_owned(),
        );
        Config::set_option(OPTION_API_SERVER.to_owned(), API_SERVER.trim().to_owned());
        Config::set_option(OPTION_KEY.to_owned(), KEY.trim().to_owned());
    } else {
        log::info!("Custom server already configured, keeping it");
    }
    Config::set_option(OPTION_INITIALIZED.to_owned(), "Y".to_owned());
}
