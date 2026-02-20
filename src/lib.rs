use near_sdk::json_types::U64;
use near_sdk::{
    AccountId, PanicOnDefault, env, near, require,
};

#[near(serializers = [json, borsh])]
#[derive(Clone, PartialEq, Eq)]
pub enum RequestStatus {
    Pending,
    Completed,
}

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub enum RequestKind {
    Execution {
        program_id: String,
        input_payload: String,
    },
    SecretFetch {
        binary_id: U64,
        env_key: String,
        url: String,
    },
}

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct ExecutionRequest {
    pub id: U64,
    pub requester: AccountId,
    pub kind: RequestKind,
    pub status: RequestStatus,
    pub result: Option<String>,
}

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct WasiBinary {
    pub id: U64,
    pub name: String,
    pub wasm_sha256: String,
    pub dashboard_upload_ref: String,
}

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct EncryptedEnvVar {
    pub env_key: String,
    pub ciphertext_b64: String,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    owner_id: AccountId,
    outlayer_operator_id: AccountId,
    next_request_id: u64,
    next_binary_id: u64,
    requests: Vec<ExecutionRequest>,
    binaries: Vec<WasiBinary>,
    encrypted_envs: Vec<EncryptedEnvVar>,
}

#[near]
impl Contract {
    #[init]
    pub fn init(owner_id: AccountId, outlayer_operator_id: AccountId) -> Self {
        Self {
            owner_id,
            outlayer_operator_id,
            next_request_id: 0,
            next_binary_id: 0,
            requests: vec![],
            binaries: vec![],
            encrypted_envs: vec![],
        }
    }

    pub fn get_owner_id(&self) -> AccountId {
        self.owner_id.clone()
    }

    pub fn get_outlayer_operator_id(&self) -> AccountId {
        self.outlayer_operator_id.clone()
    }

    pub fn set_outlayer_operator(&mut self, outlayer_operator_id: AccountId) {
        self.assert_owner();
        self.outlayer_operator_id = outlayer_operator_id;
    }

    pub fn register_wasi_binary(
        &mut self,
        name: String,
        wasm_sha256: String,
        dashboard_upload_ref: String,
    ) -> U64 {
        self.assert_owner();
        let binary_id = self.next_binary_id;
        self.next_binary_id += 1;

        let binary = WasiBinary {
            id: binary_id.into(),
            name,
            wasm_sha256,
            dashboard_upload_ref,
        };
        self.binaries.push(binary);
        binary_id.into()
    }

    pub fn get_binary(&self, binary_id: U64) -> Option<WasiBinary> {
        self.binaries
            .iter()
            .find(|binary| binary.id == binary_id)
            .cloned()
    }

    pub fn list_binaries(&self) -> Vec<WasiBinary> {
        self.binaries.clone()
    }

    pub fn upsert_encrypted_env(&mut self, env_key: String, ciphertext_b64: String) {
        self.assert_owner();
        if let Some(existing) = self
            .encrypted_envs
            .iter_mut()
            .find(|env_var| env_var.env_key == env_key)
        {
            existing.ciphertext_b64 = ciphertext_b64;
            return;
        }

        self.encrypted_envs.push(EncryptedEnvVar {
            env_key,
            ciphertext_b64,
        });
    }

    pub fn get_encrypted_env_keys(&self) -> Vec<String> {
        self.encrypted_envs
            .iter()
            .map(|env_var| env_var.env_key.clone())
            .collect()
    }

    pub fn request_execution(&mut self, program_id: String, input_payload: String) -> U64 {
        self.push_request(RequestKind::Execution {
            program_id,
            input_payload,
        })
    }

    pub fn request_secret_fetch(&mut self, binary_id: U64, env_key: String, url: String) -> U64 {
        require!(
            self.binaries.iter().any(|binary| binary.id == binary_id),
            "Unknown binary_id"
        );
        require!(
            self.encrypted_envs
                .iter()
                .any(|env_var| env_var.env_key == env_key),
            "Unknown encrypted env key"
        );

        self.push_request(RequestKind::SecretFetch {
            binary_id,
            env_key,
            url,
        })
    }

    pub fn submit_result(&mut self, request_id: U64, result: String) {
        self.assert_outlayer_operator();

        let Some(index) = self
            .requests
            .iter()
            .position(|request| request.id == request_id)
        else {
            env::panic_str("Unknown request_id");
        };

        require!(
            self.requests[index].status == RequestStatus::Pending,
            "Request already completed"
        );

        self.requests[index].status = RequestStatus::Completed;
        self.requests[index].result = Some(result);
    }

    pub fn get_request(&self, request_id: U64) -> Option<ExecutionRequest> {
        self.requests
            .iter()
            .find(|request| request.id == request_id)
            .cloned()
    }

    pub fn list_requests(&self) -> Vec<ExecutionRequest> {
        self.requests.clone()
    }

    fn push_request(&mut self, kind: RequestKind) -> U64 {
        let request_id = self.next_request_id;
        self.next_request_id += 1;

        let request = ExecutionRequest {
            id: request_id.into(),
            requester: env::predecessor_account_id(),
            kind,
            status: RequestStatus::Pending,
            result: None,
        };

        self.requests.push(request);
        env::log_str(&format!("request_created:{}", request_id));
        request_id.into()
    }

    fn assert_owner(&self) {
        require!(
            env::predecessor_account_id() == self.owner_id,
            "Only owner can call this method"
        );
    }

    fn assert_outlayer_operator(&self) {
        require!(
            env::predecessor_account_id() == self.outlayer_operator_id,
            "Only OutLayer operator can submit results"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{VMContextBuilder, accounts};
    use near_sdk::testing_env;

    fn get_context(predecessor: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor.clone())
            .predecessor_account_id(predecessor);
        builder
    }

    #[test]
    fn init_contract() {
        let owner = accounts(1);
        let operator = accounts(2);
        let contract = Contract::init(owner.clone(), operator.clone());

        assert_eq!(contract.get_owner_id(), owner);
        assert_eq!(contract.get_outlayer_operator_id(), operator);
        assert!(contract.list_requests().is_empty());
    }

    #[test]
    fn callback_loop_works() {
        let owner = accounts(1);
        let operator = accounts(2);
        let alice = accounts(3);

        let mut contract = Contract::init(owner.clone(), operator.clone());

        let alice_context = get_context(alice.clone());
        testing_env!(alice_context.build());
        let request_id = contract.request_execution("hello-program".to_string(), "{\"x\":1}".to_string());

        let request = contract.get_request(request_id).expect("request should exist");
        assert_eq!(request.requester, alice);
        assert!(matches!(request.status, RequestStatus::Pending));

        let operator_context = get_context(operator);
        testing_env!(operator_context.build());
        contract.submit_result(request_id, "{\"ok\":true}".to_string());

        let completed_request = contract.get_request(request_id).expect("request should still exist");
        assert!(matches!(completed_request.status, RequestStatus::Completed));
        assert_eq!(completed_request.result.as_deref(), Some("{\"ok\":true}"));
    }

    #[test]
    fn owner_registers_binary() {
        let owner = accounts(1);
        let operator = accounts(2);
        let mut contract = Contract::init(owner.clone(), operator);

        let owner_context = get_context(owner.clone());
        testing_env!(owner_context.build());

        let binary_id = contract.register_wasi_binary(
            "fetcher-v1".to_string(),
            "abc123".to_string(),
            "outlayer://artifact/fetcher-v1".to_string(),
        );

        let binary = contract.get_binary(binary_id).expect("binary should exist");
        assert_eq!(binary.name, "fetcher-v1");
        assert_eq!(binary.wasm_sha256, "abc123");
    }

    #[test]
    #[should_panic(expected = "Unknown encrypted env key")]
    fn secret_fetch_requires_secret_registration() {
        let owner = accounts(1);
        let operator = accounts(2);
        let alice = accounts(3);
        let mut contract = Contract::init(owner.clone(), operator);

        let owner_context = get_context(owner);
        testing_env!(owner_context.build());
        let binary_id = contract.register_wasi_binary(
            "fetcher-v1".to_string(),
            "abc123".to_string(),
            "outlayer://artifact/fetcher-v1".to_string(),
        );

        let alice_context = get_context(alice);
        testing_env!(alice_context.build());
        let _ = contract.request_secret_fetch(
            binary_id,
            "API_KEY".to_string(),
            "https://httpbin.org/get".to_string(),
        );
    }

    #[test]
    fn secret_fetch_flow_works() {
        let owner = accounts(1);
        let operator = accounts(2);
        let alice = accounts(3);
        let mut contract = Contract::init(owner.clone(), operator.clone());

        let owner_context = get_context(owner);
        testing_env!(owner_context.build());
        let binary_id = contract.register_wasi_binary(
            "fetcher-v1".to_string(),
            "abc123".to_string(),
            "outlayer://artifact/fetcher-v1".to_string(),
        );
        contract.upsert_encrypted_env("API_KEY".to_string(), "BASE64_CIPHERTEXT".to_string());

        let alice_context = get_context(alice.clone());
        testing_env!(alice_context.build());
        let request_id = contract.request_secret_fetch(
            binary_id,
            "API_KEY".to_string(),
            "https://httpbin.org/get".to_string(),
        );

        let request = contract.get_request(request_id).expect("request should exist");
        assert_eq!(request.requester, alice);
        assert!(matches!(request.kind, RequestKind::SecretFetch { .. }));

        let operator_context = get_context(operator);
        testing_env!(operator_context.build());
        contract.submit_result(request_id, "{\"status\":200}".to_string());

        let completed_request = contract.get_request(request_id).expect("request should exist");
        assert!(matches!(completed_request.status, RequestStatus::Completed));
    }
}
