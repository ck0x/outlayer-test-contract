# OutLayer Evaluation Contract (NEAR + Rust)

This repository contains a minimal NEAR smart contract specifically for evaluating OutLayer integration flows.

The contract is intentionally small and focuses on three checks:

1. Callback loop: `request_execution` + `submit_result`
2. Binary deployment metadata: `register_wasi_binary`
3. Secret injection flow shape: `upsert_encrypted_env` + `request_secret_fetch`

---

## Build

Install [`cargo-near`](https://github.com/near/cargo-near) and run:

```bash
cargo near build
```

## Test

```bash
cargo test
```

You can also run only unit or integration tests:

```bash
cargo test --lib
cargo test --test test_basics
```

## Deploy

Deployment is automated with GitHub Actions CI/CD pipeline.
To deploy manually, install [`cargo-near`](https://github.com/near/cargo-near) and run:

If you deploy for debugging purposes:

```bash
cargo near deploy build-non-reproducible-wasm
```

If you deploy production ready smart contract:

```bash
cargo near deploy build-reproducible-wasm
```

## Initialize

```bash
near contract call-function as-transaction '<CONTRACT_ACCOUNT_ID>' init json-args '{"owner_id":"<OWNER_ACCOUNT_ID>","outlayer_operator_id":"<OUTLAYER_OPERATOR_ACCOUNT_ID>"}' prepaid-gas '30.0 Tgas' attached-deposit '0 NEAR'
```

## Minimal evaluation flow

### 1) Register uploaded wasm32-wasi binary metadata

```bash
near contract call-function as-transaction '<CONTRACT_ACCOUNT_ID>' register_wasi_binary json-args '{"name":"fetcher-v1","wasm_sha256":"sha256:abc123","dashboard_upload_ref":"outlayer://artifact/fetcher-v1"}' prepaid-gas '30.0 Tgas' attached-deposit '0 NEAR'
```

### 2) Register encrypted env var reference

```bash
near contract call-function as-transaction '<CONTRACT_ACCOUNT_ID>' upsert_encrypted_env json-args '{"env_key":"API_KEY","ciphertext_b64":"BASE64_ENCRYPTED_VALUE"}' prepaid-gas '30.0 Tgas' attached-deposit '0 NEAR'
```

### 3) Request execution and callback result

```bash
near contract call-function as-transaction '<CONTRACT_ACCOUNT_ID>' request_execution json-args '{"program_id":"hello-program","input_payload":"{\"hello\":\"world\"}"}' prepaid-gas '30.0 Tgas' attached-deposit '0 NEAR'

near contract call-function as-transaction '<CONTRACT_ACCOUNT_ID>' submit_result json-args '{"request_id":"0","result":"{\"ok\":true}"}' prepaid-gas '30.0 Tgas' attached-deposit '0 NEAR'
```

### 4) Request secret-backed external fetch shape

```bash
near contract call-function as-transaction '<CONTRACT_ACCOUNT_ID>' request_secret_fetch json-args '{"binary_id":"0","env_key":"API_KEY","url":"https://httpbin.org/get"}' prepaid-gas '30.0 Tgas' attached-deposit '0 NEAR'
```

The contract does not perform the external fetch itself; it records requests and accepts OutLayer callback results via `submit_result`.

## OutLayer Project Setup

This repo now includes a monorepo worker at `outlayer-worker/` for `wasm32-wasip2`.

### Build the OutLayer worker

```bash
cd outlayer-worker
rustup target add wasm32-wasip2
cargo build --release --target wasm32-wasip2
```

Worker artifact:

- `outlayer-worker/target/wasm32-wasip2/release/outlayer-worker.wasm`

### Stable WASM URL (for OutLayer WASM URL mode)

This repo includes workflow `.github/workflows/publish-outlayer-worker.yml` that builds and publishes:

- `outlayer-worker/latest.wasm`

Expected URL:

- `https://ck0x.github.io/outlayer-test-contract/outlayer-worker/latest.wasm`

One-time setup in GitHub repository settings:

1. Go to **Settings → Pages**
2. Under **Build and deployment**, set **Source** to **GitHub Actions**

Then run the workflow once manually (Actions tab) or push changes under `outlayer-worker/`.

### Dashboard values (from your screenshot)

- **Project Name**: `outlayer-test`
- **Code Source**:
	- Prefer **WASM URL** if dashboard does not support subdirectory builds
	- Use **GitHub Repository** only if OutLayer supports building from `outlayer-worker/` path
- **WASM URL (recommended)**: `https://ck0x.github.io/outlayer-test-contract/outlayer-worker/latest.wasm`
- **Repository**: `https://github.com/ck0x/outlayer-test-contract`
- **Commit/Branch**: `main`
- **Build Target**: `wasm32-wasip2`

If your OutLayer dashboard currently has no repository subdirectory field, the most reliable option is to upload/host the built wasm and choose **WASM URL**.

## Useful Links

- [cargo-near](https://github.com/near/cargo-near) - NEAR smart contract development toolkit for Rust
- [near CLI](https://near.cli.rs) - Interact with NEAR blockchain from command line
- [NEAR Rust SDK Documentation](https://docs.near.org/sdk/rust/introduction)
- [NEAR Documentation](https://docs.near.org)
- [NEAR StackOverflow](https://stackoverflow.com/questions/tagged/nearprotocol)
- [NEAR Discord](https://near.chat)
- [NEAR Telegram Developers Community Group](https://t.me/neardev)
- NEAR DevHub: [Telegram](https://t.me/neardevhub), [Twitter](https://twitter.com/neardevhub)
