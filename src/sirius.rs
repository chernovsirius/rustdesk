//! SiriusDesk: server settings and app name baked into the binary.
//!
//! Build-time environment variables (GitHub Actions secrets/variables in
//! `.github/workflows/flutter-build.yml`):
//!
//! - `SIRIUS_ID_SERVER`    - ID/rendezvous server, e.g. `desk.it-sirius.ru`
//! - `SIRIUS_RELAY_SERVER` - relay server (optional, derived from the ID server if empty)
//! - `SIRIUS_API_SERVER`   - API server, e.g. `https://desk.it-sirius.ru`
//! - `SIRIUS_KEY`          - public key of the server (`id_ed25519.pub`)
//! - `SIRIUS_APP_NAME`     - app name on Windows (service, install dir, config dir),
//!   so the build does not clash with an installed RustDesk. `[a-zA-Z0-9-]+`.
//!
//! A `sirius.ini` next to the executable (`../Resources` on macOS) overrides
//! the baked server settings at run time:
//!
//! ```ini
//! [server]
//! id_server = desk.example.com
//! relay_server =
//! api_server = https://desk.example.com
//! key = ...
//! ```
//!
//! Without `SIRIUS_ID_SERVER` and without `sirius.ini`, the server part is off.
//!
//! `sirius-server-mode` holds the user's choice: `Y` - Sirius server, `N` - custom
//! settings (corporate server, or empty for the public servers), empty - not
//! decided yet. On the first start the Sirius server is chosen unless a server is
//! already configured. While `Y`, the server process keeps the options in sync
//! with the current Sirius values, so a new build or ini moves clients over.

use base::config::keys;
use hbb_common::{
    config::{self, Config},
    log,
};
use std::collections::HashMap;

const fn or_empty(v: Option<&'static str>) -> &'static str {
    match v {
        Some(v) => v,
        None => "",
    }
}

const APP_NAME: &str = or_empty(option_env!("SIRIUS_APP_NAME"));
const ID_SERVER: &str = or_empty(option_env!("SIRIUS_ID_SERVER"));
const RELAY_SERVER: &str = or_empty(option_env!("SIRIUS_RELAY_SERVER"));
const API_SERVER: &str = or_empty(option_env!("SIRIUS_API_SERVER"));
const KEY: &str = or_empty(option_env!("SIRIUS_KEY"));

const INI_FILE: &str = "sirius.ini";

// Builtin options read by the Flutter UI via `bind.mainGetBuildinOption`.
const BUILTIN_ID_SERVER: &str = "sirius-id-server";
const BUILTIN_RELAY_SERVER: &str = "sirius-relay-server";
const BUILTIN_API_SERVER: &str = "sirius-api-server";
const BUILTIN_KEY: &str = "sirius-key";

const OPTION_MODE: &str = "sirius-server-mode";

struct ServerSettings {
    id_server: String,
    relay_server: String,
    api_server: String,
    key: String,
}

impl ServerSettings {
    fn options(&self) -> [(&'static str, &str); 4] {
        [
            (keys::OPTION_CUSTOM_RENDEZVOUS_SERVER, &self.id_server),
            (keys::OPTION_RELAY_SERVER, &self.relay_server),
            (keys::OPTION_API_SERVER, &self.api_server),
            (keys::OPTION_KEY, &self.key),
        ]
    }
}

/// Called from `load_custom_client`, i.e. early in every process.
pub fn load() {
    set_app_name();
    let Some(s) = read_ini().or_else(baked_settings) else {
        return;
    };
    let mut builtin = config::BUILTIN_SETTINGS.write().unwrap();
    builtin.insert(BUILTIN_ID_SERVER.to_owned(), s.id_server);
    builtin.insert(BUILTIN_RELAY_SERVER.to_owned(), s.relay_server);
    builtin.insert(BUILTIN_API_SERVER.to_owned(), s.api_server);
    builtin.insert(BUILTIN_KEY.to_owned(), s.key);
}

// Only on Windows: on Linux and macOS the packages (systemd unit, .desktop,
// .app bundle, launchd plists) are still named after RustDesk.
#[cfg(windows)]
fn set_app_name() {
    let name = APP_NAME.trim();
    if name.is_empty() {
        return;
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        log::error!("SIRIUS_APP_NAME must match [a-zA-Z0-9-]+, ignored: {name}");
        return;
    }
    *config::APP_NAME.write().unwrap() = name.to_owned();
}

#[cfg(not(windows))]
fn set_app_name() {
    let _ = APP_NAME;
}

fn baked_settings() -> Option<ServerSettings> {
    if ID_SERVER.trim().is_empty() {
        return None;
    }
    Some(ServerSettings {
        id_server: ID_SERVER.trim().to_owned(),
        relay_server: RELAY_SERVER.trim().to_owned(),
        api_server: API_SERVER.trim().to_owned(),
        key: KEY.trim().to_owned(),
    })
}

fn read_ini() -> Option<ServerSettings> {
    let dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    #[cfg(target_os = "macos")]
    let dir = dir.join("../Resources");
    let path = dir.join(INI_FILE);
    if !path.is_file() {
        return None;
    }
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) => {
            log::error!("Failed to read {}: {e}", path.display());
            return None;
        }
    };
    let s = parse_ini(&text);
    if s.id_server.is_empty() {
        log::error!("{} has no id_server, ignored", path.display());
        return None;
    }
    log::info!("Server settings loaded from {}", path.display());
    Some(s)
}

fn parse_ini(text: &str) -> ServerSettings {
    let mut map = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(['#', ';', '[']) {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let v = v.trim().trim_matches('"');
            map.insert(k.trim().to_lowercase().replace('-', "_"), v.to_owned());
        }
    }
    let mut get = |names: &[&str]| {
        names
            .iter()
            .find_map(|n| map.remove(*n))
            .unwrap_or_default()
    };
    ServerSettings {
        id_server: get(&["id_server", "host"]),
        relay_server: get(&["relay_server", "relay"]),
        api_server: get(&["api_server", "api"]),
        key: get(&["key"]),
    }
}

fn current_settings() -> Option<ServerSettings> {
    let builtin = config::BUILTIN_SETTINGS.read().unwrap();
    let get = |k: &str| builtin.get(k).cloned().unwrap_or_default();
    let id_server = get(BUILTIN_ID_SERVER);
    if id_server.is_empty() {
        return None;
    }
    Some(ServerSettings {
        id_server,
        relay_server: get(BUILTIN_RELAY_SERVER),
        api_server: get(BUILTIN_API_SERVER),
        key: get(BUILTIN_KEY),
    })
}

/// Must run in the process that owns the config (the server/service process);
/// the UI processes receive the options via IPC sync.
pub fn apply_server_settings() {
    let Some(s) = current_settings() else {
        return;
    };
    let mut mode = Config::get_option(OPTION_MODE);
    if mode.is_empty() {
        let configured = !Config::get_option(keys::OPTION_CUSTOM_RENDEZVOUS_SERVER).is_empty();
        mode = if configured { "N" } else { "Y" }.to_owned();
        log::info!("Sirius server mode on first start: {mode}");
        Config::set_option(OPTION_MODE.to_owned(), mode.clone());
    }
    if mode != "Y" {
        return;
    }
    for (k, v) in s.options() {
        if Config::get_option(k) != v {
            log::info!("Updating {k} to the Sirius server settings");
            Config::set_option(k.to_owned(), v.to_owned());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ini() {
        let s = parse_ini(
            "; comment\n[server]\nID-Server = desk.example.com\napi = \"https://desk.example.com\"\nkey=abc=\n",
        );
        assert_eq!(s.id_server, "desk.example.com");
        assert_eq!(s.relay_server, "");
        assert_eq!(s.api_server, "https://desk.example.com");
        assert_eq!(s.key, "abc=");
    }
}
