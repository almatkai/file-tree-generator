use file_tree_beta_contract::native_core::{
    common::{CoreContext, CorePaths},
    extensions::{v2_file_tree_generator as adapter, v2_sdk as sdk, Sessions},
};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
type RecordedCalls = Arc<Mutex<Vec<(String, Vec<Value>)>>>;
struct Fixture {
    dir: tempfile::TempDir,
    ctx: CoreContext,
    calls: RecordedCalls,
    reply: Arc<Mutex<Result<Value, String>>>,
}
impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let ctx = CoreContext::new(
            CorePaths {
                data_dir: root.join("data"),
                config_dir: root.join("config"),
                resource_dir: root.join("resources"),
                temp_dir: root.join("tmp"),
                home_dir: root.into(),
            },
            None,
        );
        let calls = Arc::new(Mutex::new(Vec::new()));
        let reply = Arc::new(Mutex::new(Ok(json!({"ok":true}))));
        let c = calls.clone();
        let r = reply.clone();
        ctx.set_router(Arc::new(move |channel, args| {
            let c = c.clone();
            let r = r.clone();
            Box::pin(async move {
                assert_eq!(channel, "clipboard:write");
                c.lock().unwrap().push((channel, args));
                r.lock().unwrap().clone()
            })
        }))
        .unwrap();
        Self {
            dir,
            ctx,
            calls,
            reply,
        }
    }
    async fn run(&self, state: &mut Value) -> Value {
        adapter::handle(
            &self.ctx,
            "index",
            "extension:run-command",
            &json!({"argumentValues":{"input":"ignored"},"preferences":{"rootDot":true}}),
            state,
        )
        .await
        .unwrap()
    }
    async fn action(&self, state: &mut Value, id: Value, values: Value) -> Result<Value, String> {
        adapter::handle(
            &self.ctx,
            "index",
            "extension:invoke-action",
            &json!({"actionId":id,"formValues":values}),
            state,
        )
        .await
    }
    async fn edit(&self, state: &mut Value, key: &str, value: Value) -> Value {
        let view = field_id(state, key);
        self.action(state, json!(view), json!({"value":value}))
            .await
            .unwrap()
    }
}
fn field_id(state: &Value, key: &str) -> String {
    format!("{}:{key}", state["nonce"].as_str().unwrap())
}
fn child(view: &Value, key: &str) -> Value {
    view["root"]["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["props"]["id"] == key)
        .unwrap()["props"]
        .clone()
}
#[tokio::test]
async fn form_defaults_wire_and_computed_copy_ignore_forged_output() {
    let manifest: Value = serde_json::from_str(include_str!("../manifest.json")).unwrap();
    assert_eq!(manifest["id"], "com.tezbar.native.file-tree-generator");
    assert_eq!(manifest["replaces"], json!(["raycast.file-tree-generator"]));
    let declared: Vec<_> = manifest["commands"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    assert_eq!(declared, adapter::commands());
    assert_eq!(manifest["commands"][0]["runtime"], "rust");
    assert_eq!(
        manifest["commands"][0]["platforms"],
        json!(["macOS", "Windows"])
    );
    assert_eq!(manifest["verification"].as_array().unwrap().len(), 2);
    assert!(manifest["verification"]
        .as_array()
        .unwrap()
        .iter()
        .all(|v| v["state"] == "not-run"));

    let f = Fixture::new();
    let mut s = json!({});
    let v = f.run(&mut s).await;
    assert_eq!(v["actions"][0]["kind"], "submit-form");
    assert_eq!(child(&v, "charset")["value"], "utf-8");
    assert_eq!(child(&v, "trailingDirSlash")["value"], true);
    assert_eq!(child(&v, "rootDot")["value"], false);
    let id = s["copy"].clone();
    let v = f
        .action(&mut s, id, json!({"input":"a\n b","output":"FORGED"}))
        .await
        .unwrap();
    assert_eq!(child(&v, "output")["value"], "a/\n└── b");
    assert_eq!(f.calls.lock().unwrap()[0].1, vec![json!("a/\n└── b")]);
    assert_eq!(s["notice"], "Copied output");
}
#[tokio::test]
async fn draft_ack_omits_unrelated_controlled_values() {
    let f = Fixture::new();
    let mut s = json!({});
    f.run(&mut s).await;
    let v = f.edit(&mut s, "input", json!("a\n b")).await;
    assert!(child(&v, "charset").get("value").is_none());
    assert!(child(&v, "fullPath").get("value").is_none());
    assert_eq!(child(&v, "input")["value"], "a\n b");
    let id = field_id(&s, "charset");
    f.edit(&mut s, "rootDot", json!(true)).await;
    f.action(&mut s, json!(id), json!({"value":"ascii"}))
        .await
        .unwrap();
    assert_eq!(s["values"]["input"], "a\n b");
}
#[tokio::test]
async fn invalid_calls_do_not_consume_capabilities() {
    let f = Fixture::new();
    let mut s = json!({});
    f.run(&mut s).await;
    let before = s.clone();
    let copy = s["copy"].clone();
    for form in [
        Value::Null,
        json!({"charset":"bad"}),
        json!({"input":true}),
        json!({"evil":"x"}),
        json!({"fullPath":null}),
    ] {
        assert!(f.action(&mut s, copy.clone(), form).await.is_err());
        assert_eq!(s, before);
    }
    f.action(&mut s, copy.clone(), json!({"input":"yes"}))
        .await
        .unwrap();
    assert!(f.action(&mut s, copy, json!({})).await.is_err());
}
#[tokio::test]
async fn cross_session_and_disposed_actions_rejected() {
    let f = Fixture::new();
    let mut a = json!({});
    let mut b = json!({});
    f.run(&mut a).await;
    f.run(&mut b).await;
    assert!(f
        .action(&mut b, a["copy"].clone(), json!({}))
        .await
        .is_err());
    let id = a["copy"].clone();
    adapter::handle(
        &f.ctx,
        "index",
        "extension:dispose-session",
        &json!({}),
        &mut a,
    )
    .await
    .unwrap();
    assert!(f.action(&mut a, id, json!({})).await.is_err());
}
#[tokio::test]
async fn refresh_search_and_load_more_are_mutation_free() {
    let f = Fixture::new();
    let mut s = json!({});
    f.run(&mut s).await;
    f.edit(&mut s, "input", json!("draft")).await;
    let original = s.clone();
    for op in [
        "extension:refresh-session",
        "extension:search-text-changed",
        "extension:load-more",
    ] {
        let mut staged = s.clone();
        assert_eq!(
            adapter::handle(
                &f.ctx,
                "index",
                op,
                &json!({"searchText":"discarded"}),
                &mut staged
            )
            .await
            .unwrap()["mode"],
            "unchanged"
        );
        assert_eq!(staged, original);
    }
    assert_eq!(s, original);
}
#[tokio::test]
async fn strict_effect_acknowledgements_and_deliberate_retries() {
    let f = Fixture::new();
    let mut s = json!({});
    f.run(&mut s).await;
    for reply in [
        Ok(Value::Null),
        Ok(json!(true)),
        Ok(json!(false)),
        Ok(json!({})),
        Ok(json!({"ok":null})),
        Ok(json!({"ok":true,"success":false})),
        Ok(json!({"ok":true,"success":null})),
        Ok(json!({"ok":true,"error":"private"})),
        Ok(json!({"ok":false})),
        Err("SECRET/USER/PATH".into()),
    ] {
        *f.reply.lock().unwrap() = reply;
        let id = s["copy"].clone();
        f.action(&mut s, id, json!({"input":"retained"}))
            .await
            .unwrap();
        assert_ne!(s["notice"], "Copied output");
        assert!(!s["notice"].to_string().contains("SECRET"));
        assert_eq!(s["output"], "retained");
        let calls = f.calls.lock().unwrap().len();
        adapter::handle(
            &f.ctx,
            "index",
            "extension:refresh-session",
            &json!({}),
            &mut s,
        )
        .await
        .unwrap();
        assert_eq!(calls, f.calls.lock().unwrap().len());
    }
    for reply in [
        json!({"ok":true}),
        json!({"ok":true,"success":true}),
        json!({"ok":true,"success":true,"error":null}),
    ] {
        *f.reply.lock().unwrap() = Ok(reply);
        let id = s["copy"].clone();
        f.action(&mut s, id, json!({})).await.unwrap();
        assert_eq!(s["notice"], "Copied output");
    }
}
#[tokio::test]
async fn source_store_value_fields_only_persist_unknown_keys_survive() {
    let f = Fixture::new();
    sdk::update_store(&f.ctx, "file-tree-generator", |v| {
        v["future"] = json!({"keep":1});
        Ok(())
    })
    .await
    .unwrap();
    let mut s = json!({});
    f.run(&mut s).await;
    f.edit(&mut s, "input", json!("draft")).await;
    f.edit(&mut s, "charset", json!("ascii")).await;
    f.edit(&mut s, "fullPath", json!(true)).await;
    f.edit(&mut s, "rootDot", json!(true)).await;
    f.edit(&mut s, "trailingDirSlash", json!(false)).await;
    let store = sdk::read_store(&f.ctx, "file-tree-generator")
        .await
        .unwrap();
    assert_eq!(
        store,
        json!({"future":{"keep":1},"charset":"ascii","fullPath":true,"rootDot":true})
    );
    let mut next = json!({});
    f.run(&mut next).await;
    assert_eq!(next["values"]["input"], "");
    assert_eq!(next["values"]["trailingDirSlash"], true);
    assert_eq!(next["values"]["charset"], "ascii");
}
#[tokio::test]
async fn corrupt_store_preserved_and_draft_checkpoint_survives_failed_save() {
    let f = Fixture::new();
    let mut s = json!({});
    f.run(&mut s).await;
    f.edit(&mut s, "input", json!("a\n b")).await;
    let path = f
        .ctx
        .paths
        .data_dir
        .join("native-extensions-v2/file-tree-generator/storage.json");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, b"{broken").unwrap();
    f.edit(&mut s, "rootDot", json!(true)).await;
    assert_eq!(s["output"], ".\n└── a/\n    └── b");
    assert_eq!(s["dirty"], true);
    assert_eq!(std::fs::read(&path).unwrap(), b"{broken");
    assert!(f.calls.lock().unwrap().is_empty());
    let mut next = json!({});
    assert!(adapter::handle(
        &f.ctx,
        "index",
        "extension:run-command",
        &json!({}),
        &mut next
    )
    .await
    .is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"{broken");
    // Valid JSON with corrupt known settings must also survive a concurrent update.
    let malformed = br#"{"charset":42,"unknown":"preserve"}"#;
    std::fs::write(&path, malformed).unwrap();
    f.edit(&mut s, "fullPath", json!(true)).await;
    assert_eq!(s["dirty"], true);
    assert_eq!(std::fs::read(&path).unwrap(), malformed);
    assert!(f.calls.lock().unwrap().is_empty());
}
#[tokio::test]
async fn sdk_atomic_transactions_and_rejected_update() {
    let f = Fixture::new();
    let mut tasks = vec![];
    for _ in 0..20 {
        let ctx = f.ctx.clone();
        tasks.push(tokio::spawn(async move {
            sdk::update_store(&ctx, "file-tree-generator", |v| {
                v["count"] = json!(v["count"].as_u64().unwrap_or(0) + 1);
                Ok(())
            })
            .await
            .unwrap()
        }));
    }
    for task in tasks {
        task.await.unwrap();
    }
    assert_eq!(
        sdk::read_store(&f.ctx, "file-tree-generator")
            .await
            .unwrap()["count"],
        20
    );
    assert!(sdk::update_store(&f.ctx, "file-tree-generator", |v| {
        v["count"] = json!(99);
        Err("rollback".into())
    })
    .await
    .is_err());
    assert_eq!(
        sdk::read_store(&f.ctx, "file-tree-generator")
            .await
            .unwrap()["count"],
        20
    );
}
#[tokio::test]
async fn real_native_sessions_launch_dispatch_refresh_dispose_and_clear() {
    let f = Fixture::new();
    let sessions = Sessions::default();
    let v = sessions
        .call(
            &f.ctx,
            "extension:run-command",
            json!({"extensionId":"com.tezbar.native.file-tree-generator","commandName":"index"}),
        )
        .await;
    let id = v["sessionId"].clone();
    assert!(id.as_str().unwrap().starts_with("native-extension:"));
    let action = child(&v, "input")["actionId"].clone();
    let result = sessions
        .call(
            &f.ctx,
            "extension:invoke-action",
            json!({"sessionId":id,"actionId":action,"formValues":{"value":"x\n y"}}),
        )
        .await;
    assert_eq!(child(&result, "output")["value"], "x/\n└── y");
    for op in [
        "extension:refresh-session",
        "extension:search-text-changed",
        "extension:load-more",
    ] {
        assert_eq!(
            sessions.call(&f.ctx, op, json!({"sessionId":id})).await["mode"],
            "unchanged"
        );
    }
    assert_eq!(
        sessions
            .call(&f.ctx, "extension:dispose-session", json!({"sessionId":id}))
            .await,
        true
    );
    assert_eq!(
        sessions
            .call(&f.ctx, "extension:refresh-session", json!({"sessionId":id}))
            .await["ok"],
        false
    );
    sessions.clear_slug().await;
    sessions.clear().await;
}
#[tokio::test]
async fn native_sessions_cross_session_stale_and_uninstall() {
    let f = Fixture::new();
    let sessions = Sessions::default();
    let launch =
        json!({"extensionId":"com.tezbar.native.file-tree-generator","commandName":"index"});
    let a = sessions
        .call(&f.ctx, "extension:run-command", launch.clone())
        .await;
    let b = sessions.call(&f.ctx, "extension:run-command", launch).await;
    assert_eq!(
        sessions
            .call(
                &f.ctx,
                "extension:invoke-action",
                json!({"sessionId":b["sessionId"],"actionId":a["actions"][0]["id"],"formValues":{}})
            )
            .await["ok"],
        false
    );
    std::fs::create_dir_all(&f.ctx.paths.data_dir).unwrap();
    std::fs::write(f.ctx.paths.data_dir.join("uninstalled"), b"stub").unwrap();
    assert_eq!(
        sessions
            .call(
                &f.ctx,
                "extension:invoke-action",
                json!({"sessionId":b["sessionId"],"actionId":b["actions"][0]["id"],"formValues":{}})
            )
            .await["ok"],
        false
    );
    assert_eq!(
        sessions
            .call(
                &f.ctx,
                "extension:refresh-session",
                json!({"sessionId":b["sessionId"]})
            )
            .await["ok"],
        false
    );
}
#[tokio::test]
async fn invalid_large_input_retains_previous_output_and_capability() {
    let f = Fixture::new();
    let mut s = json!({});
    f.run(&mut s).await;
    f.edit(&mut s, "input", json!("safe")).await;
    let before = s.clone();
    let id = json!(field_id(&s, "input"));
    assert!(f
        .action(&mut s, id, json!({"value":"x".repeat(1_048_577)}))
        .await
        .is_err());
    assert_eq!(s, before);
}
#[tokio::test]
async fn export_actual_wire_for_renderer() {
    let f = Fixture::new();
    let mut s = json!({});
    let before = f.run(&mut s).await;
    let after = f.edit(&mut s, "input", json!("one\n two")).await;
    if let Ok(path) = std::env::var("FILE_TREE_WIRE_OUT") {
        std::fs::write(
            path,
            serde_json::to_vec(&json!({"before":before,"after":after})).unwrap(),
        )
        .unwrap();
    }
    assert!(f.dir.path().exists());
}

#[tokio::test]
async fn explicit_retry_after_save_failure_keeps_output_and_preserves_unknown_storage() {
    let f = Fixture::new();
    let mut s = json!({});
    f.run(&mut s).await;
    f.edit(&mut s, "input", json!("root\n child")).await;
    let path = f
        .ctx
        .paths
        .data_dir
        .join("native-extensions-v2/file-tree-generator/storage.json");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, b"not-json").unwrap();
    let id = s["copy"].clone();
    f.action(&mut s, id, json!({"charset":"ascii"}))
        .await
        .unwrap();
    assert_eq!(s["output"], "root/\n`-- child");
    assert!(f.calls.lock().unwrap().is_empty());
    assert_eq!(s["dirty"], true);
    // Simulate an explicit external repair, never a silent storage reset by the adapter.
    std::fs::write(&path, br#"{"unknown":{"keep":true}}"#).unwrap();
    let id = s["copy"].clone();
    f.action(&mut s, id, json!({})).await.unwrap();
    assert_eq!(s["notice"], "Copied output");
    assert_eq!(f.calls.lock().unwrap().len(), 1);
    assert_eq!(
        sdk::read_store(&f.ctx, "file-tree-generator")
            .await
            .unwrap()["unknown"],
        json!({"keep":true})
    );
}

#[tokio::test]
async fn actual_native_clear_invalidates_live_ids_and_relaunch_does_not_revive_them() {
    let f = Fixture::new();
    let sessions = Sessions::default();
    let request =
        json!({"extensionId":"com.tezbar.native.file-tree-generator","commandName":"index"});
    let before = sessions
        .call(&f.ctx, "extension:run-command", request.clone())
        .await;
    sessions.clear().await;
    assert_eq!(
        sessions
            .call(
                &f.ctx,
                "extension:refresh-session",
                json!({"sessionId":before["sessionId"]})
            )
            .await["ok"],
        false
    );
    let after = sessions
        .call(&f.ctx, "extension:run-command", request)
        .await;
    assert_ne!(before["sessionId"], after["sessionId"]);
    assert_eq!(sessions.call(&f.ctx,"extension:invoke-action",json!({"sessionId":after["sessionId"],"actionId":before["actions"][0]["id"],"formValues":{}})).await["ok"],false);
    assert_eq!(sessions.call(&f.ctx,"extension:invoke-action",json!({"sessionId":after["sessionId"],"actionId":after["actions"][0]["id"],"formValues":{"input":"new"}})).await["ok"],true);
}
