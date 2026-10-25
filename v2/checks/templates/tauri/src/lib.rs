//! HEADLESS STUB: no native Tauri window or OS effects are available.
#[derive(Clone)]
pub struct AppHandle;
pub trait Emitter {
    fn emit(&self, channel: &str, payload: serde_json::Value) -> Result<(), String>;
}
impl Emitter for AppHandle {
    fn emit(&self, _: &str, _: serde_json::Value) -> Result<(), String> {
        Err("Headless Tauri emitter is disabled".into())
    }
}
