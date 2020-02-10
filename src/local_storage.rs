//! Convenient wrappers to acces browser's local storage
//!
//! The idea here is to get rid of the `Result<T, JsValue>` wrapper in
//! all web_sys functions, as we can choose to degrade functionality when
//! local storage is absent.

use wasm_bindgen::JsValue;
use web_sys::Storage;

fn storage() -> Option<Storage> {
    web_sys::window()?.local_storage().ok()?
}

/// Check the availability of local storage
pub fn has_storage() -> bool {
    storage().is_some()
}

/// Check the availability of a key in local storage
pub fn has_key(key: &str) -> bool {
    if let Some(stor) = storage() {
        if let Ok(item) = stor.get_item(key) {
            item.is_some()
        } else {
            false
        }
    } else {
        false
    }
}

/// Retrieve an item in local storage
pub fn get_item(key: &str) -> Option<String> {
    storage().and_then(|s| s.get_item(key).ok())?
}

/// Store an item in local storage
pub fn set_item(key: &str, value: &str) -> Result<(), JsValue> {
    let stor =
        storage().ok_or(JsValue::from_str(&"No local storage available"))?;
    stor.set_item(key, value)
}
