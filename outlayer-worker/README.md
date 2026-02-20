# OutLayer Worker (wasm32-wasip2)

This is a tiny Rust WASI component for OutLayer project creation tests.

## Build

```bash
cd outlayer-worker
rustup target add wasm32-wasip2
cargo build --release --target wasm32-wasip2
```

Artifact:

- `target/wasm32-wasip2/release/outlayer-worker.wasm`

## Runtime behavior

- Reads stdin as input payload.
- Reads `API_KEY` from environment.
- Prints JSON with:
  - `ok`
  - `api_key_present`
  - `api_key_len`
  - `input`

This is enough to validate secret injection in OutLayer.
