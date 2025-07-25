// backend/src/nft.rs

use crate::storage;
use crate::types::{MintResponse, MintStatus};
use candid::Principal;
use ic_cdk::api::call::call_with_payment;
use once_cell::sync::Lazy;

// Worker canister ID
static WORKER_CANISTER_TEXT: &str = "be2us-64aaa-aaaaa-qaabq-cai";
static WORKER_CANISTER: Lazy<Principal> =
    Lazy::new(|| Principal::from_text(WORKER_CANISTER_TEXT).expect("잘못된 워커 canister ID"));

/// 다음 민팅 요청 처리
pub fn process_next_mint() {
    // storage에서 다음 대기 중인 민팅 요청 가져오기
    if let Some((request_id, req)) = storage::get_next_pending_mint() {
        ic_cdk::println!("Processing mint request {}", request_id);

        // 상태를 InProgress로 업데이트
        if let Err(e) = storage::update_mint_status(request_id, MintStatus::InProgress) {
            ic_cdk::println!("Failed to update mint status to InProgress: {}", e);
            return;
        }

        // Worker canister 호출
        ic_cdk::spawn(async move {
            let worker_canister = Principal::from_text("be2us-64aaa-aaaaa-qaabq-cai")
                .expect("Invalid worker canister ID");

            // Worker canister의 mint_nft 함수 호출
            let mint_request = worker::MintRequest {
                owner: req.owner,
                cid: req.cid,
                metadata: req.metadata,
            };

            match ic_cdk::call::<(worker::MintRequest,), (Result<worker::MintResponse, String>,)>(
                worker_canister,
                "mint_nft",
                (mint_request,),
            )
            .await
            {
                Ok((Ok(response),)) => {
                    ic_cdk::println!("Mint successful, token_id: {}", response.token_id);
                    if let Err(e) = storage::update_mint_status(
                        request_id,
                        MintStatus::Completed(response.token_id),
                    ) {
                        ic_cdk::println!("Failed to update mint status to completed: {}", e);
                    }
                }
                Ok((Err(error),)) => {
                    ic_cdk::println!("Worker returned error: {}", error);
                    if let Err(e) =
                        storage::update_mint_status(request_id, MintStatus::Failed(error))
                    {
                        ic_cdk::println!("Failed to update mint status to failed: {}", e);
                    }
                }
                Err((code, msg)) => {
                    let error_msg = format!("Worker call failed: code={:?}, msg={}", code, msg);
                    ic_cdk::println!("{}", error_msg);
                    if let Err(e) =
                        storage::update_mint_status(request_id, MintStatus::Failed(error_msg))
                    {
                        ic_cdk::println!("Failed to update mint status to failed: {}", e);
                    }
                }
            }
        });
    } else {
        ic_cdk::println!("No pending mint requests found");
    }
}

mod worker {
    use candid::{CandidType, Deserialize, Principal};

    #[derive(CandidType, Deserialize)]
    pub struct MintRequest {
        pub owner: Option<Principal>,
        pub cid: String,
        pub metadata: Vec<Vec<u8>>,
    }

    #[derive(CandidType, Deserialize)]
    pub struct MintResponse {
        pub token_id: u64,
    }
}
