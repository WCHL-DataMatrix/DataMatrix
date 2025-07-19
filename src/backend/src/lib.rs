// backend/src/lib.rs - 마켓플레이스 기능이 추가된 버전

// 매크로 및 타입 임포트
use candid::Principal;
use ic_cdk::api::call::call;
use ic_cdk_macros::{init, query, update};
use std::time::Duration;

// 모듈 선언
mod nft;
mod storage;
mod types;
mod upload;
mod validation;
// 새로 추가된 마켓플레이스 모듈들
mod marketplace;
mod marketplace_storage;
mod marketplace_types;

// Re-export types (기존 타입들)
pub use types::{
    DataInfo, MintRequest, MintRequestInfo, MintStatus, RequestResponse, StorageStats, TokenInfo,
    UploadRequest, UploadResponse,
};

// Re-export 마켓플레이스 타입들
pub use marketplace_types::{
    ActivityLog, ActivityType, CategoryStats, CreateListingRequest, CreateListingResponse,
    FavoriteRequest, Listing, ListingDetail, ListingStatus, ListingSummary, MarketplaceStats,
    SearchListingsRequest, SearchResult, SearchStats, SortBy, SuccessResponse,
    UpdateListingRequest,
};

use once_cell::sync::Lazy;
static WORKER_CANISTER_TEXT: &str = "bw4dl-smaaa-aaaaa-qaacq-cai";
static WORKER_CANISTER: Lazy<Principal> =
    Lazy::new(|| Principal::from_text(WORKER_CANISTER_TEXT).expect("잘못된 워커 canister ID"));

// =====================
// 1) Upload 인터페이스 (기존)
// =====================

/// Upload 엔드포인트
#[update]
pub fn upload(req: UploadRequest) -> Result<UploadResponse, String> {
    // 1) MIME 타입 검증
    upload::validate_mime_type(&req.mime_type)?;

    // 2) 데이터 크기 검증 (10MB 제한)
    upload::validate_data_size(&req.content, 10 * 1024 * 1024)?;

    // 3) 바이너리 → 파싱된 CBOR Value 벡터
    let parsed = upload::upload_data(req.content, &req.mime_type)?;

    // 4) 검증
    validation::validate_data(&parsed)?;

    // 5) 저장소에 저장
    let data_ids = storage::store_upload_data(parsed, &req.mime_type)?;

    // 6) 데이터 ID를 바이트로 변환하여 반환
    let bytes = data_ids
        .into_iter()
        .map(|id| id.to_be_bytes().to_vec())
        .collect();

    Ok(UploadResponse { data: bytes })
}

// =====================
// 2) 비동기 민팅 인터페이스 (기존)
// =====================

/// 민팅 요청을 큐에 추가
#[update]
pub fn request_mint(req: MintRequest) -> RequestResponse {
    // validation 추가
    if let Err(e) = validation::validate_mint_request(&req.cid, &req.metadata) {
        ic_cdk::trap(&format!("민팅 요청 검증 실패: {}", e));
    }

    let request_id = storage::store_mint_request(req);
    RequestResponse { request_id }
}

/// 민팅 요청 상태 조회
#[query]
pub fn get_mint_status(request_id: u64) -> Option<MintStatus> {
    storage::get_mint_status(request_id)
}

/// 업로드된 데이터 조회
#[query]
pub fn get_uploaded_data(data_id: u64) -> Option<Vec<u8>> {
    storage::get_uploaded_data(data_id)
}

/// 업로드된 데이터 목록 조회
#[query]
pub fn list_uploaded_data() -> Vec<DataInfo> {
    storage::list_uploaded_data()
}

/// 민팅 요청 목록 조회
#[query]
pub fn list_mint_requests() -> Vec<MintRequestInfo> {
    storage::list_mint_requests()
}

/// 저장소 통계 조회
#[query]
pub fn get_storage_stats() -> StorageStats {
    storage::get_storage_stats()
}

/// 업로드된 데이터 삭제
#[update]
pub fn delete_uploaded_data(data_id: u64) -> Result<String, String> {
    storage::delete_uploaded_data(data_id)
}

/// 특정 토큰 정보 조회 (기존)
#[query]
pub fn get_token_info(token_id: u64) -> Option<TokenInfo> {
    None // 임시로 None 반환
}

/// 전체 민팅된 토큰 ID 리스트 조회 (기존)
#[query]
pub fn list_tokens() -> Vec<u64> {
    vec![] // 임시로 빈 벡터 반환
}

// Worker canister와 통신하는 update 함수들 (기존)
#[update]
pub async fn get_token_info_from_worker(token_id: u64) -> Option<TokenInfo> {
    let result: Result<(Option<TokenInfo>,), _> =
        call(*WORKER_CANISTER, "get_token_info", (token_id,)).await;

    match result {
        Ok((info,)) => info,
        Err((code, msg)) => {
            ic_cdk::println!("worker query failed: {:?} {}", code, msg);
            None
        }
    }
}

#[update]
pub async fn list_tokens_from_worker() -> Vec<u64> {
    let result: Result<(Vec<u64>,), _> = call(*WORKER_CANISTER, "list_tokens", ()).await;

    match result {
        Ok((ids,)) => ids,
        Err((code, msg)) => {
            ic_cdk::println!("worker query failed: {:?} {}", code, msg);
            vec![]
        }
    }
}

// 기존 저장소 관련 query 함수들 추가
#[query]
pub fn check_data_exists(data: Vec<u8>) -> Option<u64> {
    storage::check_data_exists(&data)
}

#[query]
pub fn check_data_minted(data: Vec<u8>) -> bool {
    storage::check_data_minted(&data)
}

#[query]
pub fn check_multiple_data_status(data_list: Vec<Vec<u8>>) -> Vec<(Option<u64>, bool)> {
    storage::check_multiple_data_status(&data_list)
}

// =====================
// 3) 새로운 마켓플레이스 인터페이스
// =====================

/// 판매글 생성
#[update]
pub fn create_listing(req: CreateListingRequest) -> Result<CreateListingResponse, String> {
    marketplace::create_listing_service(req)
}

/// 판매글 업데이트
#[update]
pub fn update_listing(req: UpdateListingRequest) -> Result<SuccessResponse, String> {
    marketplace::update_listing_service(req)
}

/// 판매글 삭제
#[update]
pub fn delete_listing(listing_id: u64) -> Result<SuccessResponse, String> {
    marketplace::delete_listing_service(listing_id)
}

/// 판매글 상세 조회
#[query]
pub fn get_listing_detail(listing_id: u64) -> Option<ListingDetail> {
    marketplace::get_listing_detail_service(listing_id)
}

/// 판매글 목록 조회
#[query]
pub fn list_listings(status: Option<ListingStatus>, limit: Option<u64>) -> Vec<ListingSummary> {
    marketplace::list_listings_service(status, limit)
}

/// 내 판매글 조회
#[query]
pub fn get_my_listings() -> Vec<ListingSummary> {
    marketplace::get_my_listings_service()
}

/// 특정 사용자의 판매글 조회
#[query]
pub fn get_user_listings(user: Principal) -> Vec<ListingSummary> {
    marketplace::get_user_listings_service(user)
}

/// 판매글 검색
#[query]
pub fn search_listings(req: SearchListingsRequest) -> Result<SearchResult, String> {
    marketplace::search_listings_service(req)
}

/// 카테고리별 판매글 수 조회
#[query]
pub fn get_categories() -> Vec<(String, u64)> {
    marketplace::get_categories_service()
}

/// 인기 태그 조회
#[query]
pub fn get_popular_tags(limit: Option<u64>) -> Vec<(String, u64)> {
    marketplace::get_popular_tags_service(limit)
}

/// 즐겨찾기 추가
#[update]
pub fn add_favorite(req: FavoriteRequest) -> Result<SuccessResponse, String> {
    marketplace::add_favorite_service(req)
}

/// 즐겨찾기 제거
#[update]
pub fn remove_favorite(req: FavoriteRequest) -> Result<SuccessResponse, String> {
    marketplace::remove_favorite_service(req)
}

/// 내 즐겨찾기 목록 조회
#[query]
pub fn get_my_favorites() -> Vec<ListingSummary> {
    marketplace::get_my_favorites_service()
}

/// 즐겨찾기 여부 확인
#[query]
pub fn is_favorited(listing_id: u64) -> bool {
    marketplace::is_favorited_service(listing_id)
}

/// 마켓플레이스 통계 조회
#[query]
pub fn get_marketplace_stats() -> MarketplaceStats {
    marketplace::get_marketplace_stats_service()
}

/// 최근 활동 조회
#[query]
pub fn get_recent_activities(limit: Option<u64>) -> Vec<ActivityLog> {
    marketplace::get_recent_activities_service(limit)
}

/// 추천 판매글 조회 (개선된 버전)
#[query]
pub fn get_recommended_listings(limit: u64) -> Vec<ListingSummary> {
    let user = ic_cdk::caller();
    marketplace::get_recommended_listings(user, limit)
}

/// 유사한 판매글 조회 (개선된 버전)
#[query]
pub fn get_similar_listings(listing_id: u64, limit: u64) -> Vec<ListingSummary> {
    marketplace::get_similar_listings(listing_id, limit)
}

/// 검색어 자동완성
#[query]
pub fn get_search_suggestions(partial_query: String, limit: Option<usize>) -> Vec<String> {
    marketplace::get_search_suggestions_service(partial_query, limit)
}

/// 인기 검색어 조회
#[query]
pub fn get_trending_keywords(limit: Option<usize>) -> Vec<(String, u32)> {
    marketplace::get_trending_keywords_service(limit)
}

/// 검색 결과 통계
#[query]
pub fn get_search_stats(req: SearchListingsRequest) -> Result<SearchStats, String> {
    marketplace::get_search_stats_service(req)
}

/// 고급 검색
#[query]
pub fn advanced_search(
    keywords: Option<String>,
    category: Option<String>,
    price_min: Option<u64>,
    price_max: Option<u64>,
    tags: Option<Vec<String>>,
    seller: Option<Principal>,
    sort_by: Option<SortBy>,
    page: Option<u64>,
) -> Result<SearchResult, String> {
    let price_range = match (price_min, price_max) {
        (Some(min), Some(max)) => Some((min, max)),
        _ => None,
    };

    marketplace::advanced_search_service(
        keywords,
        category,
        price_range,
        tags,
        seller,
        sort_by,
        page,
    )
}

/// 연관 검색어 제안
#[query]
pub fn get_related_keywords(query: String) -> Vec<String> {
    marketplace::get_related_keywords_service(query)
}

/// 검색어 정규화
#[query]
pub fn normalize_search_query(query: String) -> String {
    marketplace::normalize_search_query(&query)
}

/// 검색 기반 트렌딩 판매글
#[query]
pub fn get_trending_by_search() -> Vec<ListingSummary> {
    marketplace::get_trending_by_search()
}

// =====================
// 4) 관리자 기능
// =====================

/// 관리자용 판매글 강제 삭제
#[update]
pub fn admin_delete_listing(listing_id: u64) -> Result<SuccessResponse, String> {
    marketplace::admin_delete_listing_service(listing_id)
}

/// 비활성 판매글 정리
#[update]
pub fn cleanup_inactive_listings() -> u64 {
    marketplace::cleanup_inactive_listings()
}

/// 인기 판매글 업데이트
#[query]
pub fn get_trending_listings() -> Vec<ListingSummary> {
    marketplace::update_trending_listings()
}

// =====================
// 5) 초기화: 백그라운드 작업 예약
// =====================
use ic_cdk_timers::set_timer_interval;

#[init]
fn init() {
    // 기존 저장소 초기화
    storage::init_storage();

    // 마켓플레이스 저장소 초기화
    marketplace_storage::init_marketplace_storage();

    // 10초마다 process_next_mint를 호출
    set_timer_interval(Duration::from_secs(10), || {
        nft::process_next_mint();
    });

    // 1시간마다 비활성 판매글 정리
    set_timer_interval(Duration::from_secs(3600), || {
        let cleaned = marketplace::cleanup_inactive_listings();
        if cleaned > 0 {
            ic_cdk::println!("Cleaned up {} inactive listings", cleaned);
        }
    });

    // 30분마다 검색 인덱스 업데이트
    set_timer_interval(Duration::from_secs(1800), || {
        marketplace::update_search_indices();
    });
}

// Candid 연동
ic_cdk::export_candid!();

// =====================
// 6) 테스트 (기존 + 새로운 마켓플레이스 테스트)
// =====================
#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;

    // 기존 테스트들...

    // 테스트용 더미 데이터
    fn create_test_json_data() -> Vec<u8> {
        br#"[
            {"name": "Alice", "age": 30, "city": "Seoul"},
            {"name": "Bob", "age": 25, "city": "Busan"},
            {"name": "Charlie", "age": 35, "city": "Incheon"}
        ]"#
        .to_vec()
    }

    fn create_test_csv_data() -> Vec<u8> {
        b"name,age,city\nAlice,30,Seoul\nBob,25,Busan\nCharlie,35,Incheon\n".to_vec()
    }

    #[test]
    fn test_marketplace_workflow() {
        println!("=== 마켓플레이스 워크플로우 테스트 ===");

        // 1. 데이터 업로드
        println!("1. 데이터 업로드 중...");
        let upload_req = UploadRequest {
            content: create_test_json_data(),
            mime_type: "application/json".to_string(),
        };
        let upload_result = upload(upload_req).unwrap();
        println!("✓ 데이터 업로드 완료: {} 레코드", upload_result.data.len());

        // 2. 판매글 생성
        println!("2. 판매글 생성 중...");
        let data_ids: Vec<u64> = upload_result
            .data
            .iter()
            .map(|bytes| u64::from_be_bytes(bytes.as_slice().try_into().unwrap()))
            .collect();

        let listing_req = CreateListingRequest {
            title: "고품질 사용자 데이터".to_string(),
            description: "서울, 부산, 인천 지역 사용자들의 프로필 데이터입니다. 마케팅 및 분석용으로 활용하기 좋습니다.".to_string(),
            price: 50_000_000, // 0.5 ICP
            currency: "ICP".to_string(),
            data_ids,
            category: "user_data".to_string(),
            tags: vec!["users".to_string(), "korea".to_string(), "profile".to_string()],
            preview_data: Some(r#"{"sample": "Alice, 30, Seoul"}"#.to_string()),
        };

        // Note: 실제 테스트에서는 caller()가 제대로 설정되지 않으므로 mock 필요
        // let listing_result = create_listing(listing_req);
        println!("✓ 판매글 생성 요청 준비 완료");

        // 3. 검색 테스트
        println!("3. 검색 기능 테스트 중...");
        let search_req = SearchListingsRequest {
            query: Some("사용자".to_string()),
            category: Some("user_data".to_string()),
            tags: Some(vec!["korea".to_string()]),
            min_price: Some(10_000_000),
            max_price: Some(100_000_000),
            currency: Some("ICP".to_string()),
            seller: None,
            status: Some(ListingStatus::Active),
            sort_by: Some(SortBy::CreatedAtDesc),
            page: Some(0),
            page_size: Some(10),
        };

        // 검증만 수행
        assert!(search_req.validate().is_ok());
        println!("✓ 검색 요청 검증 완료");

        println!("=== 마켓플레이스 테스트 완료 ===");
    }

    #[test]
    fn test_listing_validation() {
        // 유효한 판매글 요청
        let valid_req = CreateListingRequest {
            title: "테스트 데이터".to_string(),
            description: "테스트용 데이터입니다.".to_string(),
            price: 1_000_000, // 0.01 ICP
            currency: "ICP".to_string(),
            data_ids: vec![1, 2, 3],
            category: "test".to_string(),
            tags: vec!["test".to_string(), "data".to_string()],
            preview_data: None,
        };
        assert!(valid_req.validate().is_ok());

        // 빈 제목
        let empty_title = CreateListingRequest {
            title: "".to_string(),
            ..valid_req.clone()
        };
        assert!(empty_title.validate().is_err());

        // 너무 긴 제목
        let long_title = CreateListingRequest {
            title: "a".repeat(201),
            ..valid_req.clone()
        };
        assert!(long_title.validate().is_err());

        // 가격이 0
        let zero_price = CreateListingRequest {
            price: 0,
            ..valid_req.clone()
        };
        assert!(zero_price.validate().is_err());

        // 데이터가 없음
        let no_data = CreateListingRequest {
            data_ids: vec![],
            ..valid_req.clone()
        };
        assert!(no_data.validate().is_err());

        // 너무 많은 태그
        let too_many_tags = CreateListingRequest {
            tags: (0..21).map(|i| format!("tag{}", i)).collect(),
            ..valid_req.clone()
        };
        assert!(too_many_tags.validate().is_err());

        println!("✓ 판매글 검증 테스트 완료");
    }

    #[test]
    fn test_search_validation() {
        // 유효한 검색 요청
        let valid_search = SearchListingsRequest {
            query: Some("test".to_string()),
            category: None,
            tags: None,
            min_price: Some(1_000_000),
            max_price: Some(10_000_000),
            currency: Some("ICP".to_string()),
            seller: None,
            status: Some(ListingStatus::Active),
            sort_by: Some(SortBy::CreatedAtDesc),
            page: Some(0),
            page_size: Some(20),
        };
        assert!(valid_search.validate().is_ok());

        // 잘못된 페이지 크기
        let invalid_page_size = SearchListingsRequest {
            page_size: Some(101), // 최대 100
            ..valid_search.clone()
        };
        assert!(invalid_page_size.validate().is_err());

        // 잘못된 가격 범위
        let invalid_price_range = SearchListingsRequest {
            min_price: Some(10_000_000),
            max_price: Some(1_000_000), // min > max
            ..valid_search.clone()
        };
        assert!(invalid_price_range.validate().is_err());

        println!("✓ 검색 검증 테스트 완료");
    }

    #[test]
    fn test_price_validation() {
        // 유효한 ICP 가격
        assert!(marketplace::validate_price(1_000_000, "ICP").is_ok()); // 0.01 ICP
        assert!(marketplace::validate_price(100_000_000, "ICP").is_ok()); // 1 ICP

        // 너무 낮은 ICP 가격
        assert!(marketplace::validate_price(500_000, "ICP").is_err()); // < 0.01 ICP

        // 너무 높은 ICP 가격
        assert!(marketplace::validate_price(2_000_000_000_000, "ICP").is_err()); // > 10,000 ICP

        // 유효한 USD 가격
        assert!(marketplace::validate_price(100, "USD").is_ok()); // $1.00
        assert!(marketplace::validate_price(10_000, "USD").is_ok()); // $100.00

        // 너무 낮은 USD 가격
        assert!(marketplace::validate_price(50, "USD").is_err()); // < $1.00

        // 지원되지 않는 통화
        assert!(marketplace::validate_price(100, "EUR").is_err());

        println!("✓ 가격 검증 테스트 완료");
    }

    #[test]
    fn test_tag_normalization() {
        let input_tags = vec![
            "  Tag1  ".to_string(),
            "TAG2".to_string(),
            "tag1".to_string(), // 중복
            "".to_string(),     // 빈 태그
            "a".repeat(51),     // 너무 긴 태그
            "valid_tag".to_string(),
        ];

        let normalized = marketplace::normalize_tags(input_tags);

        // 중복 제거, 빈 태그 제거, 너무 긴 태그 제거, 소문자 변환
        assert!(normalized.contains(&"tag1".to_string()));
        assert!(normalized.contains(&"tag2".to_string()));
        assert!(normalized.contains(&"valid_tag".to_string()));
        assert!(!normalized.contains(&"".to_string()));
        assert!(!normalized.iter().any(|tag| tag.len() > 50));

        // tag1은 중복이므로 한 번만 있어야 함
        assert_eq!(normalized.iter().filter(|&tag| tag == "tag1").count(), 1);

        println!("✓ 태그 정규화 테스트 완료");
    }

    #[test]
    fn test_description_preview() {
        let short_desc = "짧은 설명";
        let preview = marketplace::generate_description_preview(short_desc, 100);
        assert_eq!(preview, short_desc);

        let long_desc = "이것은 매우 긴 설명입니다. ".repeat(10);
        let preview = marketplace::generate_description_preview(&long_desc, 50);
        assert!(preview.len() <= 53); // 50 + "..."
        assert!(preview.ends_with("..."));

        println!("✓ 설명 미리보기 테스트 완료");
    }

    // 기존 테스트들 유지
    #[test]
    fn test_upload_json_data() {
        let req = UploadRequest {
            content: create_test_json_data(),
            mime_type: "application/json".to_string(),
        };

        let result = upload(req);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.data.len(), 3); // 3개의 레코드
        println!("JSON 업로드 성공: {} 레코드", response.data.len());
    }

    #[test]
    fn test_upload_csv_data() {
        let req = UploadRequest {
            content: create_test_csv_data(),
            mime_type: "text/csv".to_string(),
        };

        let result = upload(req);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.data.len(), 4); // 헤더 + 3개의 데이터 레코드
        println!("CSV 업로드 성공: {} 레코드", response.data.len());
    }

    #[test]
    fn test_mint_request_flow() {
        // 1. 데이터 업로드 시뮬레이션
        let upload_req = UploadRequest {
            content: create_test_json_data(),
            mime_type: "application/json".to_string(),
        };

        let upload_result = upload(upload_req).unwrap();
        assert_eq!(upload_result.data.len(), 3);

        // 2. 각 업로드된 데이터에 대해 민팅 요청
        let test_owner = Principal::anonymous();

        for (i, data_blob) in upload_result.data.iter().enumerate() {
            let mint_req = MintRequest {
                owner: Some(test_owner),
                cid: format!("QmTest{}", i + 1), // IPFS CID 시뮬레이션
                metadata: vec![data_blob.clone()],
            };

            // 민팅 요청
            let request_response = request_mint(mint_req);
            println!("민팅 요청 ID: {}", request_response.request_id);

            // 요청 상태 확인
            let status = get_mint_status(request_response.request_id);
            assert!(status.is_some());
            assert_eq!(status.unwrap(), MintStatus::Pending);
        }
    }

    #[test]
    fn test_full_integration_workflow() {
        println!("=== 전체 통합 워크플로우 테스트 ===");

        // 1. 데이터 업로드
        println!("1. 데이터 업로드 중...");
        let upload_req = UploadRequest {
            content: create_test_json_data(),
            mime_type: "application/json".to_string(),
        };
        let upload_result = upload(upload_req).unwrap();
        println!("✓ 데이터 업로드 완료: {} 레코드", upload_result.data.len());

        // 2. 데이터 ID 추출
        let data_ids: Vec<u64> = upload_result
            .data
            .iter()
            .map(|bytes| u64::from_be_bytes(bytes.as_slice().try_into().unwrap()))
            .collect();
        println!("✓ 데이터 ID 추출 완료: {:?}", data_ids);

        // 3. 판매글 생성 요청 검증
        println!("3. 판매글 생성 요청 검증 중...");
        let listing_req = CreateListingRequest {
            title: "테스트 사용자 데이터".to_string(),
            description: "테스트용 사용자 프로필 데이터입니다.".to_string(),
            price: 50_000_000, // 0.5 ICP
            currency: "ICP".to_string(),
            data_ids: data_ids.clone(),
            category: "user_data".to_string(),
            tags: vec!["test".to_string(), "users".to_string(), "korea".to_string()],
            preview_data: Some(r#"{"sample": "Alice, 30, Seoul"}"#.to_string()),
        };

        // 요청 검증
        assert!(listing_req.validate().is_ok());
        println!("✓ 판매글 생성 요청 검증 완료");

        // 4. 검색 요청 검증
        println!("4. 검색 기능 검증 중...");
        let search_req = SearchListingsRequest {
            query: Some("사용자".to_string()),
            category: Some("user_data".to_string()),
            tags: Some(vec!["korea".to_string()]),
            min_price: Some(10_000_000),
            max_price: Some(100_000_000),
            currency: Some("ICP".to_string()),
            seller: None,
            status: Some(ListingStatus::Active),
            sort_by: Some(SortBy::CreatedAtDesc),
            page: Some(0),
            page_size: Some(10),
        };

        assert!(search_req.validate().is_ok());
        println!("✓ 검색 요청 검증 완료");

        // 5. 즐겨찾기 요청 검증
        println!("5. 즐겨찾기 요청 검증 중...");
        let favorite_req = FavoriteRequest { listing_id: 1 };
        println!("✓ 즐겨찾기 요청 생성 완료");

        println!("=== 전체 통합 워크플로우 테스트 완료 ===");
    }

    #[test]
    fn test_error_handling() {
        // 잘못된 JSON 데이터 테스트
        let invalid_json = UploadRequest {
            content: b"invalid json data".to_vec(),
            mime_type: "application/json".to_string(),
        };
        let result = upload(invalid_json);
        assert!(result.is_err());
        println!("✓ 잘못된 JSON 데이터 에러 처리 확인");

        // 지원하지 않는 MIME 타입 테스트
        let unsupported_mime = UploadRequest {
            content: b"some data".to_vec(),
            mime_type: "application/pdf".to_string(),
        };
        let result = upload(unsupported_mime);
        assert!(result.is_err());
        println!("✓ 지원하지 않는 MIME 타입 에러 처리 확인");

        // 빈 데이터 테스트
        let empty_data = UploadRequest {
            content: Vec::new(),
            mime_type: "application/json".to_string(),
        };
        let result = upload(empty_data);
        assert!(result.is_err());
        println!("✓ 빈 데이터 에러 처리 확인");
    }
}
