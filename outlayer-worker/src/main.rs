use serde::Serialize;
use std::{env, io::Read};

#[derive(Serialize)]
struct Output {
    ok: bool,
    api_key_present: bool,
    api_key_len: usize,
    input: String,
    note: String,
}

fn main() {
    let mut input = String::new();
    let _ = std::io::stdin().read_to_string(&mut input);

    let api_key = env::var("API_KEY").unwrap_or_default();

    let output = Output {
        ok: true,
        api_key_present: !api_key.is_empty(),
        api_key_len: api_key.len(),
        input,
        note: "Minimal wasm32-wasip2 worker for OutLayer evaluation (secret injection check)."
            .to_string(),
    };

    println!("{}", serde_json::to_string(&output).unwrap());
}
