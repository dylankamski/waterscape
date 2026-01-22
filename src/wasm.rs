//! WebAssembly bindings for the Waterscape protocol
//!
//! This module provides JavaScript-friendly APIs for use in browsers and Node.js.

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
use crate::agent::Agent;
#[cfg(feature = "wasm")]
use crate::protocol::{Waterscape, WaterscapeGroup};

/// JavaScript-friendly Agent wrapper
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmAgent {
    inner: Agent,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmAgent {
    /// Create a new agent with the given name
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str) -> Self {
        Self {
            inner: Agent::new(name),
        }
    }

    /// Get the agent's name
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.inner.name().to_string()
    }

    /// Get the agent's fingerprint (first 8 bytes of signing key as hex)
    #[wasm_bindgen(getter)]
    pub fn fingerprint(&self) -> String {
        self.inner.public_identity().fingerprint()
    }

    /// Get the agent's public identity as JSON
    #[wasm_bindgen(js_name = publicIdentityJson)]
    pub fn public_identity_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.inner.public_identity())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Export the signing key (for backup)
    #[wasm_bindgen(js_name = exportSigningKey)]
    pub fn export_signing_key(&self) -> Vec<u8> {
        self.inner.export_signing_key().to_vec()
    }
}

/// JavaScript-friendly Waterscape API
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmWaterscape;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmWaterscape {
    /// Encode a secret message for a recipient
    /// 
    /// # Arguments
    /// * `sender` - The sending agent
    /// * `recipient_json` - The recipient's public identity as JSON
    /// * `cover_text` - The visible cover text
    /// * `secret` - The secret message to hide
    /// 
    /// # Returns
    /// The cover text with hidden encrypted message
    #[wasm_bindgen]
    pub fn encode(
        sender: &WasmAgent,
        recipient_json: &str,
        cover_text: &str,
        secret: &str,
    ) -> Result<String, JsValue> {
        let recipient: crate::agent::PublicIdentity = serde_json::from_str(recipient_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid recipient JSON: {}", e)))?;

        Waterscape::encode(&sender.inner, &recipient, cover_text, secret)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Decode a hidden message
    /// 
    /// # Arguments
    /// * `receiver` - The receiving agent
    /// * `sender_json` - The sender's public identity as JSON
    /// * `text` - The text containing the hidden message
    /// 
    /// # Returns
    /// The decoded secret message
    #[wasm_bindgen]
    pub fn decode(
        receiver: &WasmAgent,
        sender_json: &str,
        text: &str,
    ) -> Result<String, JsValue> {
        let sender: crate::agent::PublicIdentity = serde_json::from_str(sender_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid sender JSON: {}", e)))?;

        Waterscape::decode(&receiver.inner, &sender, text)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Check if text contains a hidden message
    #[wasm_bindgen(js_name = hasHiddenMessage)]
    pub fn has_hidden_message(text: &str) -> bool {
        Waterscape::has_hidden_message(text)
    }

    /// Extract only the visible text (remove hidden data)
    #[wasm_bindgen(js_name = visibleText)]
    pub fn visible_text(text: &str) -> String {
        Waterscape::visible_text(text)
    }
}

/// JavaScript-friendly Group wrapper
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmWaterscapeGroup {
    inner: WaterscapeGroup,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmWaterscapeGroup {
    /// Create a new group
    /// 
    /// # Arguments
    /// * `name` - Group name
    /// * `creator` - The agent creating the group
    /// * `members_json` - JSON array of member public identities
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str, creator: &WasmAgent, members_json: &str) -> Result<WasmWaterscapeGroup, JsValue> {
        let members: Vec<crate::agent::PublicIdentity> = serde_json::from_str(members_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid members JSON: {}", e)))?;

        Ok(Self {
            inner: WaterscapeGroup::new(name, &creator.inner, members),
        })
    }

    /// Get the group name
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.inner.name().to_string()
    }

    /// Encode a message for the group
    #[wasm_bindgen]
    pub fn encode(
        &self,
        sender: &WasmAgent,
        cover_text: &str,
        secret: &str,
    ) -> Result<String, JsValue> {
        self.inner
            .encode(&sender.inner, cover_text, secret)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Decode a group message
    #[wasm_bindgen]
    pub fn decode(&self, text: &str) -> Result<String, JsValue> {
        self.inner
            .decode(text)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

/// Initialize panic hook for better error messages in WASM
#[cfg(feature = "wasm")]
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Log to browser console
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn log(message: &str) {
    web_sys::console::log_1(&JsValue::from_str(message));
}
