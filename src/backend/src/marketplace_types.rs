// backend/src/marketplace_types.rs

use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::Storable;
use serde::Serialize;
use std::borrow::Cow;

// =====================
// 1) 판매글 관련 타입
// =====================

/// 판매글 생성 요청
#[derive(CandidType, Deserialize, Clone)]
pub struct CreateListingRequest {
    pub title: String,
    pub description: String,
    pub price: u64,                   // ICP 단위 (e8s)
    pub currency: String,             // "ICP", "USD" 등
    pub data_ids: Vec<u64>,           // 판매할 데이터 ID들
    pub category: String,             // 카테고리
    pub tags: Vec<String>,            // 태그들
    pub preview_data: Option<String>, // 미리보기 데이터 (JSON 문자열)
}

/// 판매글 업데이트 요청
#[derive(CandidType, Deserialize, Clone)]
pub struct UpdateListingRequest {
    pub listing_id: u64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub price: Option<u64>,
    pub currency: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub preview_data: Option<String>,
    pub status: Option<ListingStatus>,
}

/// 판매글 상태
#[derive(CandidType, Deserialize, Serialize, Clone, PartialEq, Debug)]
pub enum ListingStatus {
    Active,    // 판매 중
    Sold,      // 판매 완료
    Suspended, // 일시 중단
    Deleted,   // 삭제됨
}

/// 판매글 상세 정보 (내부 저장용)
#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct Listing {
    pub id: u64,
    pub seller: Principal,
    pub title: String,
    pub description: String,
    pub price: u64,
    pub currency: String,
    pub data_ids: Vec<u64>,
    pub category: String,
    pub tags: Vec<String>,
    pub preview_data: Option<String>,
    pub status: ListingStatus,
    pub created_at: u64,
    pub updated_at: u64,
    pub view_count: u64,
    pub favorite_count: u64,
}

/// 판매글 요약 정보 (목록 조회용)
#[derive(CandidType, Deserialize)]
pub struct ListingSummary {
    pub id: u64,
    pub seller: Principal,
    pub title: String,
    pub description: String, // 축약된 설명
    pub price: u64,
    pub currency: String,
    pub category: String,
    pub tags: Vec<String>,
    pub status: ListingStatus,
    pub created_at: u64,
    pub updated_at: u64,
    pub view_count: u64,
    pub favorite_count: u64,
    pub data_count: u64, // 포함된 데이터 개수
}

/// 판매글 상세 정보 (조회용)
#[derive(CandidType, Deserialize)]
pub struct ListingDetail {
    pub listing: Listing,
    pub data_info: Vec<crate::types::DataInfo>, // 기존 types.rs의 DataInfo 사용
}

// =====================
// 2) 검색 및 필터링 관련 타입
// =====================

/// 판매글 검색 요청
#[derive(CandidType, Deserialize, Clone)]
pub struct SearchListingsRequest {
    pub query: Option<String>,         // 검색어 (제목, 설명에서 검색)
    pub category: Option<String>,      // 카테고리 필터
    pub tags: Option<Vec<String>>,     // 태그 필터
    pub min_price: Option<u64>,        // 최소 가격
    pub max_price: Option<u64>,        // 최대 가격
    pub currency: Option<String>,      // 통화 필터
    pub seller: Option<Principal>,     // 판매자 필터
    pub status: Option<ListingStatus>, // 상태 필터
    pub sort_by: Option<SortBy>,       // 정렬 기준
    pub page: Option<u64>,             // 페이지 번호 (0부터 시작)
    pub page_size: Option<u64>,        // 페이지 크기 (기본 20)
}

/// 정렬 기준
#[derive(CandidType, Deserialize, Clone)]
pub enum SortBy {
    CreatedAtDesc,     // 생성일 내림차순 (최신순)
    CreatedAtAsc,      // 생성일 오름차순
    PriceDesc,         // 가격 내림차순
    PriceAsc,          // 가격 오름차순
    ViewCountDesc,     // 조회수 내림차순
    FavoriteCountDesc, // 즐겨찾기 수 내림차순
    UpdatedAtDesc,     // 수정일 내림차순
}

/// 검색 결과
#[derive(CandidType, Deserialize)]
pub struct SearchResult {
    pub listings: Vec<ListingSummary>,
    pub total_count: u64,
    pub page: u64,
    pub page_size: u64,
    pub total_pages: u64,
}

// =====================
// 3) 즐겨찾기 관련 타입
// =====================

/// 즐겨찾기 정보
#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct Favorite {
    pub user: Principal,
    pub listing_id: u64,
    pub created_at: u64,
}

/// 즐겨찾기 요청
#[derive(CandidType, Deserialize)]
pub struct FavoriteRequest {
    pub listing_id: u64,
}

// =====================
// 4) 통계 관련 타입
// =====================

/// 마켓플레이스 통계
#[derive(CandidType, Deserialize)]
pub struct MarketplaceStats {
    pub total_listings: u64,
    pub active_listings: u64,
    pub sold_listings: u64,
    pub total_sellers: u64,
    pub total_views: u64,
    pub total_favorites: u64,
    pub categories: Vec<CategoryStats>,
    pub recent_activity: Vec<ActivityLog>,
}

/// 검색 결과 통계
#[derive(CandidType, Deserialize)]
pub struct SearchStats {
    pub total_results: u64,
    pub avg_price: u64,
    pub price_range: (u64, u64), // (min, max)
    pub top_categories: Vec<(String, u64)>,
    pub top_sellers: Vec<(Principal, u64)>,
}

/// 카테고리별 통계
#[derive(CandidType, Deserialize)]
pub struct CategoryStats {
    pub category: String,
    pub count: u64,
    pub avg_price: u64,
}

/// 활동 로그
#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct ActivityLog {
    pub timestamp: u64,
    pub activity_type: ActivityType,
    pub user: Principal,
    pub listing_id: Option<u64>,
    pub details: String,
}

/// 활동 타입
#[derive(CandidType, Deserialize, Serialize, Clone)]
pub enum ActivityType {
    ListingCreated,
    ListingUpdated,
    ListingViewed,
    ListingFavorited,
    ListingSold,
    ListingDeleted,
}

// =====================
// 5) 응답 타입
// =====================

/// 판매글 생성 응답
#[derive(CandidType, Deserialize)]
pub struct CreateListingResponse {
    pub listing_id: u64,
}

/// 일반적인 성공 응답
#[derive(CandidType, Deserialize)]
pub struct SuccessResponse {
    pub message: String,
}

// =====================
// 6) Storable 구현
// =====================

impl Storable for Listing {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(serde_cbor::to_vec(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        serde_cbor::from_slice(&bytes).unwrap()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

impl Storable for Favorite {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(serde_cbor::to_vec(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        serde_cbor::from_slice(&bytes).unwrap()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

impl Storable for ActivityLog {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(serde_cbor::to_vec(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        serde_cbor::from_slice(&bytes).unwrap()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

// =====================
// 7) 검증 함수들
// =====================

impl CreateListingRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty() {
            return Err("제목은 필수입니다".to_string());
        }

        if self.title.len() > 200 {
            return Err("제목은 200자를 초과할 수 없습니다".to_string());
        }

        if self.description.trim().is_empty() {
            return Err("설명은 필수입니다".to_string());
        }

        if self.description.len() > 10000 {
            return Err("설명은 10,000자를 초과할 수 없습니다".to_string());
        }

        if self.price == 0 {
            return Err("가격은 0보다 커야 합니다".to_string());
        }

        if self.data_ids.is_empty() {
            return Err("판매할 데이터가 최소 하나는 있어야 합니다".to_string());
        }

        if self.data_ids.len() > 100 {
            return Err("한 번에 판매할 수 있는 데이터는 최대 100개입니다".to_string());
        }

        if self.category.trim().is_empty() {
            return Err("카테고리는 필수입니다".to_string());
        }

        if self.tags.len() > 20 {
            return Err("태그는 최대 20개까지 가능합니다".to_string());
        }

        for tag in &self.tags {
            if tag.len() > 50 {
                return Err("각 태그는 50자를 초과할 수 없습니다".to_string());
            }
        }

        Ok(())
    }
}

impl UpdateListingRequest {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(ref title) = self.title {
            if title.trim().is_empty() {
                return Err("제목은 비어있을 수 없습니다".to_string());
            }
            if title.len() > 200 {
                return Err("제목은 200자를 초과할 수 없습니다".to_string());
            }
        }

        if let Some(ref description) = self.description {
            if description.trim().is_empty() {
                return Err("설명은 비어있을 수 없습니다".to_string());
            }
            if description.len() > 10000 {
                return Err("설명은 10,000자를 초과할 수 없습니다".to_string());
            }
        }

        if let Some(price) = self.price {
            if price == 0 {
                return Err("가격은 0보다 커야 합니다".to_string());
            }
        }

        if let Some(ref tags) = self.tags {
            if tags.len() > 20 {
                return Err("태그는 최대 20개까지 가능합니다".to_string());
            }
            for tag in tags {
                if tag.len() > 50 {
                    return Err("각 태그는 50자를 초과할 수 없습니다".to_string());
                }
            }
        }

        Ok(())
    }
}

impl SearchListingsRequest {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(page_size) = self.page_size {
            if page_size == 0 || page_size > 100 {
                return Err("페이지 크기는 1-100 사이여야 합니다".to_string());
            }
        }

        if let Some(min_price) = self.min_price {
            if let Some(max_price) = self.max_price {
                if min_price > max_price {
                    return Err("최소 가격은 최대 가격보다 클 수 없습니다".to_string());
                }
            }
        }

        Ok(())
    }

    pub fn get_page(&self) -> u64 {
        self.page.unwrap_or(0)
    }

    pub fn get_page_size(&self) -> u64 {
        self.page_size.unwrap_or(20).min(100)
    }

    pub fn get_sort_by(&self) -> SortBy {
        self.sort_by.clone().unwrap_or(SortBy::CreatedAtDesc)
    }
}
