// backend/src/lib.rs

// 매크로 및 타입 임포트
use candid::{CandidType, Deserialize};
use ic_cdk_macros::{query, update};

// 모듈 선언
mod nft;
mod upload;
mod validation;

// =====================
// 1) Upload 인터페이스
// =====================

/// 업로드 요청: 바이너리 컨텐츠와 MIME 타입
#[derive(CandidType, Deserialize)]
pub struct UploadRequest {
    pub content: Vec<u8>,
    pub mime_type: String,
}

/// 업로드 응답: 각 레코드의 CBOR 직렬화 바이트 배열
#[derive(CandidType)]
pub struct UploadResponse {
    pub data: Vec<Vec<u8>>,
}

/// Upload 엔드포인트
#[update]
pub fn upload(req: UploadRequest) -> Result<UploadResponse, String> {
    // 1) 바이너리 → 파싱된 CBOR Value 벡터
    let parsed = upload::upload_data(req.content, &req.mime_type)?;
    // 2) 검증
    validation::validate_data(&parsed)?;
    // 3) 각 Value를 CBOR 바이트로 재직렬화
    let bytes = parsed
        .into_iter()
        .map(|v| serde_cbor::to_vec(&v).map_err(|e| format!("CBOR 직렬화 실패: {}", e)))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(UploadResponse { data: bytes })
}

// =====================
// 2) NFT 민팅/조회 인터페이스
// =====================

// NFT 모듈 internal 함수 및 타입 임포트
use nft::{MintRequest, MintResponse, TokenInfo};

/// NFT 민팅 엔드포인트
#[update]
pub fn mint_nft(req: MintRequest) -> Result<MintResponse, String> {
    nft::mint_nft_internal(req)
}

/// 특정 토큰 정보 조회
#[query]
pub fn get_token_info(token_id: u64) -> Option<TokenInfo> {
    nft::get_token_info_internal(token_id)
}

/// 전체 민팅된 토큰 ID 리스트 조회
#[query]
pub fn list_tokens() -> Vec<u64> {
    nft::list_tokens_internal()
}

use ic_cdk::export_candid;
export_candid!();
