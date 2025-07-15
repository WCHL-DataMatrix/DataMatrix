// backend/src/lib.rs

// 매크로 및 타입 임포트
use candid::Principal;
use candid::{CandidType, Deserialize};
use ic_cdk::api::call::call;
use ic_cdk_macros::{init, query, update};
use std::time::Duration;

// 모듈 선언
mod nft;
mod upload;
mod validation;

// #derive: 해당 구조체는 RUST <-> Candid 간 형식 변환을 지원한다
// #update: 수정을 하겠다
// #query: 조회만 하겠다

use once_cell::sync::Lazy;
static WORKER_CANISTER_TEXT: &str = "bw4dl-smaaa-aaaaa-qaacq-cai";
static WORKER_CANISTER: Lazy<Principal> =
    Lazy::new(|| Principal::from_text(WORKER_CANISTER_TEXT).expect("잘못된 워커 canister ID"));

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
    // 3) 각 Value를 CBOR -> 바이트로 재직렬화
    let bytes = parsed
        .into_iter()
        .map(|v| serde_cbor::to_vec(&v).map_err(|e| format!("CBOR 직렬화 실패: {}", e)))
        // .map 자체가 Resut<Vec<u8>, _>의 return type, error의 경우 .map_err로 return을 string으로 변환
        // 따라서 여기까지의 반환 값은 Resut<Vec<u8>, String>
        .collect::<Result<Vec<_>, _>>()?;
    // _는 generic, 따라서 위 반환 값과 type이 동일
    // collect를 이용해서 모든 아이템이 Ok일 때, Vec<Vec<u8>>, 하나라도 Err일 때, Err(String type)를 즉시 반환
    Ok(UploadResponse { data: bytes })
}

// =====================
// 2) 비동기 민팅 인터페이스
// =====================

use nft::{MintRequest, MintStatus, RequestResponse};

/// 민팅 요청을 큐에 추가
#[update]
pub fn request_mint(req: MintRequest) -> RequestResponse {
    nft::request_mint_internal(req)
}

/// 민팅 요청 상태 조회
#[query]
pub fn get_mint_status(request_id: u64) -> Option<MintStatus> {
    nft::get_mint_status_internal(request_id)
}

/// 특정 토큰 정보 조회
#[query]
pub async fn get_token_info(token_id: u64) -> Option<nft::TokenInfo> {
    // worker.get_token_info(query) 호출
    let (info,): (Option<nft::TokenInfo>,) = call(*WORKER_CANISTER, "get_token_info", (token_id,))
        .await
        .unwrap_or_else(|(c, m)| panic!("worker query failed: {:?} {}", c, m));
    info
}

/// 전체 민팅된 토큰 ID 리스트 조회
#[query]
pub async fn list_tokens() -> Vec<u64> {
    // worker.list_tokens(query) 호출
    let (ids,): (Vec<u64>,) = call(*WORKER_CANISTER, "list_tokens", ())
        .await
        .unwrap_or_else(|(c, m)| panic!("worker query failed: {:?} {}", c, m));
    ids
}

// =====================
// 3) 초기화: 백그라운드 작업 예약
// =====================
use ic_cdk_timers::set_timer_interval;

#[init]
fn init() {
    // 10초마다 process_next_mint를 호출
    set_timer_interval(Duration::from_secs(10), || {
        nft::spawn_next_mint();
    });
}

// Candid 연동
use ic_cdk::export_candid;
export_candid!();
