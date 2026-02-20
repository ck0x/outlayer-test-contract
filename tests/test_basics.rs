use near_api::{AccountId, NearGas, NearToken};
use near_sdk::serde_json::json;

#[derive(near_sdk::serde::Deserialize)]
#[serde(crate = "near_sdk::serde")]
struct ExecutionRequest {
    pub status: String,
    pub result: Option<String>,
}

async fn test_basics_on(contract_wasm: Vec<u8>) -> testresult::TestResult<()> {
    let sandbox = near_sandbox::Sandbox::start_sandbox().await?;
    let sandbox_network =
        near_api::NetworkConfig::from_rpc_url("sandbox", sandbox.rpc_addr.parse()?);

    let owner = create_subaccount(&sandbox, "owner.sandbox").await?;
    let operator = create_subaccount(&sandbox, "operator.sandbox").await?;
    let alice = create_subaccount(&sandbox, "alice.sandbox").await?;
    let contract = create_subaccount(&sandbox, "contract.sandbox")
        .await?
        .as_contract();

    let signer = near_api::Signer::from_secret_key(
        near_sandbox::config::DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY
            .parse()
            .unwrap(),
    )?;

    near_api::Contract::deploy(contract.account_id().clone())
        .use_code(contract_wasm)
        .with_init_call(
            "init",
            json!({
                "owner_id": owner.account_id(),
                "outlayer_operator_id": operator.account_id()
            }),
        )?
        .with_signer(signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    contract
        .call_function(
            "register_wasi_binary",
            json!({
                "name": "fetcher-v1",
                "wasm_sha256": "sha256:abc123",
                "dashboard_upload_ref": "outlayer://artifact/fetcher-v1"
            }),
        )
        .transaction()
        .gas(NearGas::from_tgas(30))
        .with_signer(owner.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    contract
        .call_function(
            "upsert_encrypted_env",
            json!({
                "env_key": "API_KEY",
                "ciphertext_b64": "BASE64_ENCRYPTED_VALUE"
            }),
        )
        .transaction()
        .gas(NearGas::from_tgas(30))
        .with_signer(owner.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    let request_execution_id: String = contract
        .call_function(
            "request_execution",
            json!({
                "program_id": "hello-program",
                "input_payload": "{\"hello\":\"world\"}"
            }),
        )
        .transaction()
        .gas(NearGas::from_tgas(30))
        .with_signer(alice.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .json()?;

    contract
        .call_function(
            "submit_result",
            json!({
                "request_id": request_execution_id.clone(),
                "result": "{\"ok\":true}"
            }),
        )
        .transaction()
        .gas(NearGas::from_tgas(30))
        .with_signer(operator.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    let execution_request: Option<ExecutionRequest> = contract
        .call_function("get_request", json!({"request_id": request_execution_id}))
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    let execution_request = execution_request.expect("request should exist");

    assert_eq!(execution_request.status, "Completed");
    assert_eq!(execution_request.result.as_deref(), Some("{\"ok\":true}"));

    let request_secret_fetch_id: String = contract
        .call_function(
            "request_secret_fetch",
            json!({
                "binary_id": "0",
                "env_key": "API_KEY",
                "url": "https://httpbin.org/get"
            }),
        )
        .transaction()
        .gas(NearGas::from_tgas(30))
        .with_signer(alice.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .json()?;

    contract
        .call_function(
            "submit_result",
            json!({
                "request_id": request_secret_fetch_id.clone(),
                "result": "{\"status\":200}"
            }),
        )
        .transaction()
        .gas(NearGas::from_tgas(30))
        .with_signer(operator.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    let secret_fetch_request: Option<ExecutionRequest> = contract
        .call_function(
            "get_request",
            json!({"request_id": request_secret_fetch_id}),
        )
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    let secret_fetch_request = secret_fetch_request.expect("request should exist");

    assert_eq!(secret_fetch_request.status, "Completed");
    assert_eq!(
        secret_fetch_request.result.as_deref(),
        Some("{\"status\":200}")
    );

    Ok(())
}

async fn create_subaccount(
    sandbox: &near_sandbox::Sandbox,
    name: &str,
) -> testresult::TestResult<near_api::Account> {
    let account_id: AccountId = name.parse().unwrap();
    sandbox
        .create_account(account_id.clone())
        .initial_balance(NearToken::from_near(10))
        .send()
        .await?;
    Ok(near_api::Account(account_id))
}

#[tokio::test]
async fn test_contract_is_operational() -> testresult::TestResult<()> {
    let contract_wasm_path = cargo_near_build::build_with_cli(Default::default())?;
    let contract_wasm = std::fs::read(contract_wasm_path)?;

    test_basics_on(contract_wasm).await
}
