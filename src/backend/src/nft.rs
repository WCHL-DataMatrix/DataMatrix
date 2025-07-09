// backend/src/nft.rs

use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::call::call_with_payment;
use ic_cdk::api::call::RejectionCode;
use ic_cdk::api::id;
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

static WORKER_CANISTER_TEXT: &str = "uqqxf-5h777-77774-qaaaa-cai";
static WORKER_CANISTER: Lazy<Principal> =
    Lazy::new(|| Principal::from_text(WORKER_CANISTER_TEXT).expect("잘못된 워커 canister ID"));

thread_local! {
    /// 요청 카운터
    static REQUEST_COUNT: RefCell<u64> = const { RefCell::new(0) };
    /// 민팅 요청 큐: (request_id, MintRequest)
    static MINT_QUEUE: RefCell<VecDeque<(u64, MintRequest)>> = const { RefCell::new(VecDeque::new()) };
    /// request_id → MintStatus 매핑
    static MINT_STATUS: RefCell<HashMap<u64, MintStatus>> = RefCell::new(HashMap::new());
}

/// 민팅 요청 구조체
#[derive(CandidType, Deserialize, Clone)]
pub struct MintRequest {
    pub owner: Option<Principal>,
    pub cid: String,
    pub metadata: Vec<Vec<u8>>,
}

/// 민팅 응답
#[derive(CandidType, Deserialize)]
pub struct MintResponse {
    pub token_id: u64,
}

/// 민팅 요청 상태
#[derive(CandidType, Clone, PartialEq, Eq, Debug)]
pub enum MintStatus {
    Pending,
    InProgress,
    Completed(u64), // token_id
    Failed(String),
}

/// 큐에 요청을 넣었을 때 반환되는 구조
#[derive(CandidType)]
pub struct RequestResponse {
    pub request_id: u64,
}

/// 내부에 저장될 토큰 정보
#[derive(CandidType, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct TokenInfo {
    pub owner: Principal,
    pub cid: String,
    pub metadata: Vec<Vec<u8>>,
}

/// 민팅 요청을 큐에 추가
pub fn request_mint_internal(req: MintRequest) -> RequestResponse {
    // 새로운 request_id 생성
    let request_id = REQUEST_COUNT.with(|c| {
        let mut id = c.borrow_mut();
        *id += 1;
        *id
    });
    // 상태 = Pending
    MINT_STATUS.with(|m| {
        m.borrow_mut().insert(request_id, MintStatus::Pending);
    });
    // 큐에 삽입
    MINT_QUEUE.with(|q| {
        q.borrow_mut().push_back((request_id, req.clone()));
    });
    RequestResponse { request_id }
}

pub fn spawn_next_mint() {
    // 1) 큐에서 하나 꺼내기
    if let Some((request_id, req)) = MINT_QUEUE.with(|q| q.borrow_mut().pop_front()) {
        // 2) 상태 → InProgress
        MINT_STATUS.with(|m| m.borrow_mut().insert(request_id, MintStatus::InProgress));

        // 3) 자기 자신에게 “mint_nft” update call (사이클 0) 비동기 요청
        let me = id();
        ic_cdk::spawn(async move {
            let result: Result<(MintResponse,), (RejectionCode, String)> =
                call_with_payment(*WORKER_CANISTER, "mint_nft", (req.clone(),), 0).await;

            // 4) 결과에 따라 상태 업데이트
            match result {
                Ok((resp,)) => {
                    MINT_STATUS.with(|m| {
                        m.borrow_mut()
                            .insert(request_id, MintStatus::Completed(resp.token_id))
                    });
                }
                Err((code, msg)) => {
                    MINT_STATUS.with(|m| {
                        m.borrow_mut().insert(
                            request_id,
                            MintStatus::Failed(format!("code={:?}, msg={}", code, msg)),
                        )
                    });
                }
            }
        });
    }
}

/// 민팅 상태 조회
pub fn get_mint_status_internal(request_id: u64) -> Option<MintStatus> {
    MINT_STATUS.with(|m| m.borrow().get(&request_id).cloned())
}
