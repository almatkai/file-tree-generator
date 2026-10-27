//! Compiled-in production adapter. Modified port of V1; see NOTICE.md.
//! Vendor the library at src-tauri/native-extensions/file-tree-generator.
#[path = "../../../native-extensions/file-tree-generator/src/lib.rs"]
mod engine;
use super::v2_sdk as sdk;
use crate::native_core::common::{CoreContext, CoreResult};
use serde_json::{json, Value};
const SLUG: &str = "file-tree-generator";
const FIELDS: [&str; 6] = [
    "input",
    "charset",
    "trailingDirSlash",
    "fullPath",
    "rootDot",
    "output",
];
pub fn commands() -> &'static [&'static str] {
    &["index"]
}
fn token() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn defaults() -> Value {
    let mut v =
        serde_json::to_value(engine::Options::form_defaults()).expect("serializable options");
    v["input"] = json!("");
    v
}
fn options(values: &Value) -> Result<engine::Options, String> {
    serde_json::from_value(json!({"charset":values["charset"],"trailingDirSlash":values["trailingDirSlash"],"fullPath":values["fullPath"],"rootDot":values["rootDot"]})).map_err(|_| "Invalid form options".into())
}
fn compute(values: &Value) -> Result<String, String> {
    engine::generate(
        values["input"].as_str().ok_or("Invalid input")?,
        &options(values)?,
    )
}
fn validate(key: &str, value: &Value) -> Result<(), String> {
    let valid = match key {
        "input" | "output" => value.as_str().is_some_and(|s| {
            s.len() <= engine::MAX_INPUT || key == "output" && s.len() <= engine::MAX_OUTPUT
        }),
        "charset" => value == "ascii" || value == "utf-8",
        "trailingDirSlash" | "fullPath" | "rootDot" => value.is_boolean(),
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err("Invalid form field value or resource limit".into())
    }
}
fn ack(value: CoreResult) -> Result<(), String> {
    let value = value.map_err(|_| "Clipboard copy failed; retry deliberately".to_string())?;
    if value.is_object()
        && value["ok"] == true
        && (value.get("success").is_none() || value["success"] == true)
        && value.get("error").is_none_or(Value::is_null)
    {
        Ok(())
    } else {
        Err("Clipboard did not acknowledge copy; retry deliberately".into())
    }
}
fn field_id(state: &Value, field: &str) -> String {
    format!("{}:{field}", state["nonce"].as_str().unwrap_or("invalid"))
}
fn render(state: &Value, changed: Option<&str>) -> Value {
    let values = &state["values"];
    let mut fields = Vec::new();
    for (id, kind, title) in [
        ("input", "TextArea", "Input"),
        ("charset", "Dropdown", "Charset"),
        ("trailingDirSlash", "Checkbox", "Show Trailing /"),
        ("fullPath", "Checkbox", "Show Full Path"),
        ("rootDot", "Checkbox", "Show Root ."),
        ("output", "TextArea", "Output"),
    ] {
        let mut props = json!({"id":id,"title":title,"actionId":field_id(state,id)});
        if changed.is_none() || changed == Some(id) || id == "output" {
            props["value"] = if id == "output" {
                state["output"].clone()
            } else {
                values[id].clone()
            };
        }
        if kind == "Checkbox" {
            props["label"] = json!(title);
        }
        if id == "charset" {
            props["data"] =
                json!([{"title":"ASCII","value":"ascii"},{"title":"UTF-8","value":"utf-8"}]);
        }
        fields.push(json!({"type":format!("Form.{kind}"),"props":props,"children":[]}));
    }
    if let Some(message) = state["notice"].as_str() {
        fields.push(json!({"type":"Form.Description","props":{"title":"Status","text":message},"children":[]}));
    }
    // Generic action avoids the renderer executing a forged CopyToClipboard.content.
    sdk::view(
        "Create File Tree",
        json!({"type":"Form","props":{},"children":fields}),
        vec![json!({"id":state["copy"],"title":"Copy Output","kind":"submit-form"})],
    )
}
async fn persist(ctx: &CoreContext, state: &mut Value) -> Result<(), String> {
    if state["dirty"] != true {
        return Ok(());
    }
    let values = state["values"].clone();
    sdk::update_store(ctx, SLUG, move |store| {
        for key in ["charset", "fullPath", "rootDot"] {
            if let Some(value) = store.get(key) {
                validate(key, value)?;
            }
        }
        for key in ["charset", "fullPath", "rootDot"] {
            store[key] = values[key].clone();
        }
        Ok(())
    })
    .await
    .map_err(|_| "Settings could not be saved; draft and output retained".to_string())?;
    state["dirty"] = json!(false);
    Ok(())
}
pub async fn handle(
    ctx: &CoreContext,
    command: &str,
    operation: &str,
    payload: &Value,
    state: &mut Value,
) -> CoreResult {
    if !commands().contains(&command) {
        return Err("Unknown File Tree Generator command".into());
    }
    match operation {
        "extension:run-command" => {
            let mut values = defaults();
            // V1 has no arguments or preferences: argumentValues/preferences are intentionally unused.
            let store = sdk::read_store(ctx, SLUG)
                .await
                .map_err(|_| "Saved settings are unreadable; storage was preserved".to_string())?;
            for key in ["charset", "fullPath", "rootDot"] {
                if let Some(value) = store.get(key) {
                    validate(key, value)?;
                    values[key] = value.clone();
                }
            }
            *state =
                json!({"nonce":token(),"copy":token(),"values":values,"output":"","dirty":false});
            Ok(render(state, None))
        }
        "extension:dispose-session" => {
            *state = Value::Null;
            Ok(sdk::unchanged())
        }
        "extension:refresh-session" | "extension:search-text-changed" | "extension:load-more" => {
            if !state["nonce"].is_string() {
                return Err("Session is not active".into());
            }
            Ok(sdk::unchanged())
        }
        "extension:invoke-action" => {
            if !state["nonce"].is_string() {
                return Err("Session is not active".into());
            }
            let id = payload["actionId"]
                .as_str()
                .ok_or("Missing action capability")?;
            let copying = state["copy"].as_str() == Some(id);
            let field = FIELDS
                .iter()
                .find(|field| field_id(state, field) == id)
                .copied();
            if !copying && field.is_none() {
                return Err("Unknown or stale action capability".into());
            }
            let form = payload["formValues"]
                .as_object()
                .ok_or("Missing form values")?;
            let mut values = state["values"].clone();
            if let Some(field) = field {
                if form.len() != 1 {
                    return Err("Field callback requires only value".into());
                }
                let value = form.get("value").ok_or("Missing field value")?;
                validate(field, value)?;
                if field != "output" {
                    values[field] = value.clone();
                }
            } else {
                if form.keys().any(|k| !FIELDS.contains(&k.as_str())) {
                    return Err("Unknown submitted field".into());
                }
                for (key, value) in form {
                    validate(key, value)?;
                    if key != "output" {
                        values[key] = value.clone();
                    }
                }
            }
            let output = compute(&values)?; // Validate the entire candidate before consuming any capability.
            let dirty = state["dirty"] == true
                || ["charset", "fullPath", "rootDot"]
                    .iter()
                    .any(|k| state["values"][k] != values[k]);
            state["values"] = values;
            state["output"] = json!(output);
            state["dirty"] = json!(dirty);
            state["notice"] = Value::Null;
            state["copy"] = json!(token()); // Committed actions only: refresh/search never allocate IDs.
            if let Err(error) = persist(ctx, state).await {
                state["notice"] = json!(error);
                return Ok(render(state, field));
            }
            if copying {
                // Checkpoint before await: uncertain copies are never replayed automatically.
                state["notice"] = json!("Copy pending or uncertain; retry deliberately if needed");
                let result = ack(sdk::copy(ctx, &output).await);
                state["notice"] = json!(match result {
                    Ok(()) => "Copied output".into(),
                    Err(error) => error,
                });
            }
            Ok(render(state, field))
        }
        _ => Err("Unsupported File Tree Generator operation".into()),
    }
}
