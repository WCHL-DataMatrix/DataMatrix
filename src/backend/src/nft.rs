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
        // 상태를 InProgress로 업데이트
        if let Err(e) = storage::update_mint_status(request_id, MintStatus::InProgress) {
            ic_cdk::println!("Failed to update mint status: {}", e);
            return;
        }

        // 비동기로 worker canister 호출
        ic_cdk::spawn(async move {
            let result: Result<(MintResponse,), _> =
                call_with_payment(*WORKER_CANISTER, "mint_nft", (req,), 0).await;

            // 결과에 따라 상태 업데이트
            match result {
                Ok((resp,)) => {
                    if let Err(e) = storage::update_mint_status(
                        request_id,
                        MintStatus::Completed(resp.token_id),
                    ) {
                        ic_cdk::println!("Failed to update mint status to completed: {}", e);
                    }
                }
                Err((code, msg)) => {
                    if let Err(e) = storage::update_mint_status(
                        request_id,
                        MintStatus::Failed(format!("code={:?}, msg={}", code, msg)),
                    ) {
                        ic_cdk::println!("Failed to update mint status to failed: {}", e);
                    }
                }
            }
        });
    }
}
