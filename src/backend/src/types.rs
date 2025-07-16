// backend/src/types.rs

use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::Storable;
use serde::Serialize;
use std::borrow::Cow;

// =====================
// 1) 업로드 관련 타입
// =====================

/// 업로드 요청
#[derive(CandidType, Deserialize)]
pub struct UploadRequest {
    pub content: Vec<u8>,
    pub mime_type: String,
}

/// 업로드 응답
#[derive(CandidType)]
pub struct UploadResponse {
    pub data: Vec<Vec<u8>>, // 데이터 ID들을 바이트로 변환
}

/// 저장된 데이터 정보
#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct DataBlob {
    pub data: Vec<u8>,
    pub mime_type: String,
    pub timestamp: u64,
}

/// 데이터 정보 (조회용)
#[derive(CandidType, Deserialize)]
pub struct DataInfo {
    pub id: u64,
    pub mime_type: String,
    pub timestamp: u64,
    pub size: u64,
}

// =====================
// 2) 민팅 관련 타입
// =====================

/// 민팅 요청
#[derive(CandidType, Deserialize, Serialize, Clone)]
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

/// 민팅 상태
#[derive(CandidType, Deserialize, Serialize, Clone, PartialEq, Debug)]
pub enum MintStatus {
    Pending,
    InProgress,
    Completed(u64), // token_id
    Failed(String),
}

/// 요청 응답
#[derive(CandidType)]
pub struct RequestResponse {
    pub request_id: u64,
}

/// 토큰 정보
#[derive(CandidType, Deserialize, Clone, PartialEq, Debug)]
pub struct TokenInfo {
    pub owner: Principal,
    pub cid: String,
    pub metadata: Vec<Vec<u8>>,
}

/// 민팅 요청 정보 (조회용)
#[derive(CandidType, Deserialize)]
pub struct MintRequestInfo {
    pub request_id: u64,
    pub owner: Option<Principal>,
    pub cid: String,
    pub status: MintStatus,
    pub timestamp: u64,
}

// =====================
// 3) 통계 관련 타입
// =====================

/// 저장소 통계
#[derive(CandidType, Deserialize)]
pub struct StorageStats {
    pub total_uploads: u64,
    pub total_mint_requests: u64,
    pub pending_mints: u64,
    pub completed_mints: u64,
    pub failed_mints: u64,
    pub storage_size: u64,
}

// =====================
// 4) Storable 구현
// =====================

impl Storable for DataBlob {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(serde_cbor::to_vec(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        serde_cbor::from_slice(&bytes).unwrap()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

impl Storable for MintRequest {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(serde_cbor::to_vec(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        serde_cbor::from_slice(&bytes).unwrap()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

impl Storable for MintStatus {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(serde_cbor::to_vec(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        serde_cbor::from_slice(&bytes).unwrap()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

/// 민팅 요청 상세 정보 (내부 저장용)
#[derive(Serialize, Deserialize, Clone)]
pub struct MintRequestData {
    pub request: MintRequest,
    pub timestamp: u64,
}

impl Storable for MintRequestData {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(serde_cbor::to_vec(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        serde_cbor::from_slice(&bytes).unwrap()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}
