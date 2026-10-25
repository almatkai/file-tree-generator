use std::io::{self, BufRead};
use tezbar_file_tree_generator::{generate, Options};
fn main() {
    for line in io::stdin().lock().lines() {
        let value: serde_json::Value = serde_json::from_str(&line.expect("stdin")).expect("JSON");
        let options: Options = serde_json::from_value(value["options"].clone()).expect("options");
        println!(
            "{}",
            serde_json::to_string(
                &generate(value["input"].as_str().expect("input"), &options)
                    .expect("bounded input")
            )
            .expect("JSON")
        );
    }
}
