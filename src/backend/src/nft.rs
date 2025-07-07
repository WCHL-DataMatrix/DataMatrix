// backend/src/nft.rs

use candid::{CandidType, Deserialize, Principal};
use ic_cdk::caller;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    /// 전역 토큰 카운터
    static TOKEN_COUNT: RefCell<u64> = RefCell::new(0);
    /// token_id → TokenInfo 매핑
    static TOKENS: RefCell<HashMap<u64, TokenInfo>> = RefCell::new(HashMap::new());
}

/// 민팅 요청 구조체
#[derive(CandidType, Deserialize)]
pub struct MintRequest {
    /// 소유자를 수동 지정하고 싶으면(Some), 아니면 None으로 두면 caller가 소유자로 자동 지정됩니다.
    pub owner: Option<Principal>,
    /// IPFS 등에 올린 데이터의 CID
    pub cid: String,
    /// 업로드된 CBOR 직렬화 바이트 배열 리스트
    pub metadata: Vec<Vec<u8>>,
}

/// 민팅 응답
#[derive(CandidType)]
pub struct MintResponse {
    pub token_id: u64,
}

/// 내부에 저장될 토큰 정보
#[derive(CandidType, Clone)]
pub struct TokenInfo {
    pub owner: Principal,
    pub cid: String,
    pub metadata: Vec<Vec<u8>>,
}

/// 내부용 민팅 함수 (엔드포인트에서 호출)
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

/// 내부용 조회 함수
pub fn get_token_info_internal(token_id: u64) -> Option<TokenInfo> {
    TOKENS.with(|t| t.borrow().get(&token_id).cloned())
}

/// 내부용 전체 리스트 조회 함수
pub fn list_tokens_internal() -> Vec<u64> {
    TOKENS.with(|t| t.borrow().keys().copied().collect())
}
