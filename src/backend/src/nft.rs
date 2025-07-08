// backend/src/nft.rs

use candid::{CandidType, Deserialize, Principal};
use ic_cdk::caller;
use ic_cdk_macros::{query, update};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

thread_local! {
    /// 전역 토큰 카운터
    static TOKEN_COUNT: RefCell<u64> = RefCell::new(0);
    /// token_id → TokenInfo 매핑
    static TOKENS: RefCell<HashMap<u64, TokenInfo>> = RefCell::new(HashMap::new());
    /// 민팅 요청 큐: (request_id, MintRequest)
    static MINT_QUEUE: RefCell<VecDeque<(u64, MintRequest)>> = RefCell::new(VecDeque::new());
    /// request_id → MintStatus 매핑
    static MINT_STATUS: RefCell<HashMap<u64, MintStatus>> = RefCell::new(HashMap::new());
}

/// 민팅 요청 상태
#[derive(CandidType, Clone)]
pub enum MintStatus {
    Pending,
    InProgress,
    Completed(u64), // token_id
    Failed(String),
}

/// 민팅 요청 구조체
#[derive(CandidType, Deserialize, Clone)]
pub struct MintRequest {
    pub owner: Option<Principal>,
    pub cid: String,
    pub metadata: Vec<Vec<u8>>,
}

/// 민팅 응답
#[derive(CandidType)]
pub struct MintResponse {
    pub token_id: u64,
}

/// 큐에 요청을 넣었을 때 반환되는 구조
#[derive(CandidType)]
pub struct RequestResponse {
    pub request_id: u64,
}

/// 내부에 저장될 토큰 정보
#[derive(CandidType, Clone)]
pub struct TokenInfo {
    pub owner: Principal,
    pub cid: String,
    pub metadata: Vec<Vec<u8>>,
}

/// 민팅 요청을 큐에 추가
pub fn request_mint_internal(req: MintRequest) -> RequestResponse {
    // 새로운 request_id 생성
    let request_id = TOKEN_COUNT.with(|c| {
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

/// 큐에서 다음 요청을 하나 꺼내 처리
pub fn process_next_mint() {
    if let Some((request_id, req)) = MINT_QUEUE.with(|q| q.borrow_mut().pop_front()) {
        // 상태 업데이트
        MINT_STATUS.with(|m| {
            m.borrow_mut().insert(request_id, MintStatus::InProgress);
        });
        // 실제 민팅 로직
        match mint_nft_internal(req.clone()) {
            Ok(resp) => {
                MINT_STATUS.with(|m| {
                    m.borrow_mut()
                        .insert(request_id, MintStatus::Completed(resp.token_id));
                });
            }
            Err(err) => {
                MINT_STATUS.with(|m| {
                    m.borrow_mut().insert(request_id, MintStatus::Failed(err));
                });
            }
        }
    }
}

/// 민팅 바로 실행 (로컬 테스트용)
#[update]
pub fn mint_nft_internal(req: MintRequest) -> Result<MintResponse, String> {
    let caller = caller();
    let owner = req.owner.unwrap_or(caller);

    // 토큰 ID 증가
    let token_id = TOKEN_COUNT.with(|c| {
        let mut count = c.borrow_mut();
        *count += 1;
        *count
    });
    let info = TokenInfo {
        owner,
        cid: req.cid,
        metadata: req.metadata,
    };

    // 저장
    TOKENS.with(|t| t.borrow_mut().insert(token_id, info));

    Ok(MintResponse { token_id })
}

/// 민팅 상태 조회
pub fn get_mint_status_internal(request_id: u64) -> Option<MintStatus> {
    MINT_STATUS.with(|m| m.borrow().get(&request_id).cloned())
}

/// 특정 토큰 정보 조회
#[query]
pub fn get_token_info_internal(token_id: u64) -> Option<TokenInfo> {
    TOKENS.with(|t| t.borrow().get(&token_id).cloned())
}

/// 전체 토큰 ID 리스트 조회
#[query]
pub fn list_tokens_internal() -> Vec<u64> {
    TOKENS.with(|t| t.borrow().keys().copied().collect())
}
