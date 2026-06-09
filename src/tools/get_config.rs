use crate::config::runtime_config::RuntimeConfigStore;
use crate::shared::types_v5::{ConfigSources, GetConfigInput, GetConfigResult};
use serde_json::Value;

pub fn handle(store: &RuntimeConfigStore, arguments: Value) -> Value {
    let input: GetConfigInput =
        serde_json::from_value(arguments).unwrap_or(GetConfigInput { include_defaults: None });

    let config = store.current();

    let sources = if input.include_defaults.unwrap_or(false) {
        let defaults = serde_json::to_value(store.defaults()).unwrap_or_default();
        let environment = serde_json::to_value(store.env_overrides()).unwrap_or_default();
        let runtime_overrides =
            serde_json::to_value(RuntimeConfigStore::runtime_overrides()).unwrap_or_default();
        Some(ConfigSources { defaults, environment, runtime_overrides })
    } else {
        None
    };

    serde_json::to_value(GetConfigResult { config, sources }).unwrap_or_else(
        |e| serde_json::json!({ "error": { "code": "internal_error", "message": e.to_string() } }),
    )
}
