use candid::{CandidType, Deserialize, Principal};
use ic_cdk::caller;
use ic_cdk_macros::{query, update};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    /// 전역 토큰 카운터
    static TOKEN_COUNT: RefCell<u64> = const { RefCell::new(0) };
    /// token_id → TokenInfo 매핑
    static TOKENS: RefCell<HashMap<u64, TokenInfo>> = RefCell::new(HashMap::new());
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

/// 내부에 저장될 토큰 정보
#[derive(CandidType, Clone, PartialEq, Eq, Debug)]
pub struct TokenInfo {
    pub owner: Principal,
    pub cid: String,
    pub metadata: Vec<Vec<u8>>,
}

/// 민팅
#[update]
pub fn mint_nft(req: MintRequest) -> Result<MintResponse, String> {
    let owner = req.owner.unwrap_or_else(caller);

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

/// 특정 토큰 정보 조회
#[query]
pub fn get_token_info(token_id: u64) -> Option<TokenInfo> {
    TOKENS.with(|t| t.borrow().get(&token_id).cloned())
}

/// 전체 토큰 ID 리스트 조회
#[query]
pub fn list_tokens() -> Vec<u64> {
    // HashMap 키는 순서가 불확실하므로 정렬해서 반환
    let mut ids: Vec<u64> = TOKENS.with(|t| t.borrow().keys().copied().collect());
    ids.sort();
    ids
}

ic_cdk::export_candid!();
