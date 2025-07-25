// backend/src/lib.rs

mod marketplace;
mod marketplace_storage;
mod marketplace_types;
mod nft;
mod storage;
mod types;
mod upload;
mod validation;
// wallet 모듈들 주석 처리 - 호환성 문제 방지
// mod wallet_storage;
// mod wallet_types;

use crate::marketplace::*;
use crate::marketplace_types::*;
use crate::storage::*;
use crate::types::*;
use crate::upload::*;
use crate::validation::*;
use candid::{candid_method, Principal};
use ic_cdk::caller;
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};

// =====================
// 초기화 및 업그레이드 관리
// =====================

/// 캐니스터 초기화
#[init]
fn init() {
    ic_cdk::println!("Initializing backend canister...");

    // 백엔드 저장소 초기화
    storage::init_storage();

    // 마켓플레이스 저장소 초기화
    marketplace_storage::init_marketplace_storage();

    // 지갑 저장소는 현재 비활성화
    // wallet_storage::init_wallet_storage();

    ic_cdk::println!("Backend canister initialization completed");
}

/// 업그레이드 전 처리
#[pre_upgrade]
fn pre_upgrade() {
    ic_cdk::println!("Preparing for upgrade...");
    // 필요한 경우 업그레이드 전 데이터 백업 로직 추가
}

/// 업그레이드 후 처리
#[post_upgrade]
fn post_upgrade() {
    ic_cdk::println!("Post-upgrade initialization...");

    // 저장소 재초기화 (업그레이드 후에도 안전하게)
    storage::init_storage();
    marketplace_storage::init_marketplace_storage();
    // wallet_storage::init_wallet_storage();

    ic_cdk::println!("Post-upgrade initialization completed");
}

/// 수동 저장소 초기화 (테스트/디버깅용)
#[update]
#[candid_method(update)]
fn init_storage_manual() -> String {
    ic_cdk::println!("Manual storage initialization requested...");

    storage::init_storage();
    marketplace_storage::init_marketplace_storage();
    // wallet_storage::init_wallet_storage();

    "Storage manually initialized successfully".to_string()
}

// =====================
// 데이터 업로드 및 관리
// =====================

/// 데이터 업로드
#[update]
#[candid_method(update)]
fn upload(request: UploadRequest) -> Result<UploadResponse, String> {
    // 파일 크기 검증
    validate_data_size(&request.content, 10 * 1024 * 1024)?; // 10MB 제한

    // MIME 타입 검증
    validate_mime_type(&request.mime_type)?;

    // 데이터 파싱
    let parsed_data = upload_data(request.content, &request.mime_type)?;

    // 데이터 검증
    validate_data(&parsed_data)?;

    // 저장소에 저장
    let data_ids = store_upload_data(parsed_data, &request.mime_type)?;

    // 바이트 배열로 변환
    let data_bytes: Vec<Vec<u8>> = data_ids
        .into_iter()
        .map(|id| id.to_le_bytes().to_vec())
        .collect();

    Ok(UploadResponse { data: data_bytes })
}

/// 업로드된 데이터 조회
#[query]
#[candid_method(query)]
fn get_uploaded_data(data_id: u64) -> Option<Vec<u8>> {
    storage::get_uploaded_data(data_id)
}

/// 업로드된 데이터 목록 조회
#[query]
#[candid_method(query)]
fn list_uploaded_data() -> Vec<DataInfo> {
    storage::list_uploaded_data()
}

/// 업로드된 데이터 삭제
#[update]
#[candid_method(update)]
fn delete_uploaded_data(data_id: u64) -> Result<String, String> {
    storage::delete_uploaded_data(data_id)
}

/// 저장소 통계 조회
#[query]
#[candid_method(query)]
fn get_storage_stats() -> StorageStats {
    storage::get_storage_stats()
}

// =====================
// 데이터 존재 및 상태 확인
// =====================

/// 데이터 존재 확인
#[query]
#[candid_method(query)]
fn check_data_exists(data: Vec<u8>) -> Option<u64> {
    storage::check_data_exists(&data)
}

/// 데이터 민팅 여부 확인
#[query]
#[candid_method(query)]
fn check_data_minted(data: Vec<u8>) -> bool {
    storage::check_data_minted(&data)
}

/// 여러 데이터의 상태 확인
#[query]
#[candid_method(query)]
fn check_multiple_data_status(data_list: Vec<Vec<u8>>) -> Vec<(Option<u64>, bool)> {
    storage::check_multiple_data_status(&data_list)
}

// =====================
// 민팅 관련 함수
// =====================

/// 민팅 요청
#[update]
#[candid_method(update)]
fn request_mint(request: MintRequest) -> RequestResponse {
    // 검증 - 실패하면 바로 에러 반환하도록 수정
    if let Err(e) = validate_mint_request(&request.cid, &request.metadata) {
        ic_cdk::println!("Mint request validation failed: {}", e);
        // 검증 실패 시 즉시 에러 상태로 요청 저장
        let request_id = storage::store_mint_request(request);
        let _ = storage::update_mint_status(request_id, MintStatus::Failed(e));
        return RequestResponse { request_id };
    }

    // 사용자 권한 검증
    if let Err(e) = validate_user_permission(request.owner) {
        ic_cdk::println!("User permission validation failed: {}", e);
        let request_id = storage::store_mint_request(request);
        let _ = storage::update_mint_status(request_id, MintStatus::Failed(e));
        return RequestResponse { request_id };
    }

    // 검증 통과 시에만 민팅 요청 저장
    let request_id = storage::store_mint_request(request);

    // 비동기 민팅 처리 시작
    ic_cdk::spawn(async move {
        nft::process_next_mint();
    });

    RequestResponse { request_id }
}

/// 민팅 상태 조회
#[query]
#[candid_method(query)]
fn get_mint_status(request_id: u64) -> Option<MintStatus> {
    storage::get_mint_status(request_id)
}

/// 민팅 요청 목록 조회
#[query]
#[candid_method(query)]
fn list_mint_requests() -> Vec<MintRequestInfo> {
    storage::list_mint_requests()
}

// =====================
// Worker Canister 연동
// =====================

/// Worker에서 토큰 정보 조회
#[update]
#[candid_method(update)]
async fn get_token_info_from_worker(token_id: u64) -> Option<TokenInfo> {
    let worker_canister =
        Principal::from_text("be2us-64aaa-aaaaa-qaabq-cai").expect("Invalid worker canister ID");

    match ic_cdk::call::<(u64,), (Option<TokenInfo>,)>(
        worker_canister,
        "get_token_info",
        (token_id,),
    )
    .await
    {
        Ok((token_info,)) => token_info,
        Err((code, msg)) => {
            ic_cdk::println!("Failed to get token info from worker: {:?} - {}", code, msg);
            None
        }
    }
}

/// Worker에서 토큰 목록 조회
#[update]
#[candid_method(update)]
async fn list_tokens_from_worker() -> Vec<u64> {
    let worker_canister =
        Principal::from_text("be2us-64aaa-aaaaa-qaabq-cai").expect("Invalid worker canister ID");

    match ic_cdk::call::<(), (Vec<u64>,)>(worker_canister, "list_tokens", ()).await {
        Ok((tokens,)) => tokens,
        Err((code, msg)) => {
            ic_cdk::println!("Failed to list tokens from worker: {:?} - {}", code, msg);
            Vec::new()
        }
    }
}

/// 백엔드에서 토큰 정보 조회 (로컬 캐시용)
#[query]
#[candid_method(query)]
fn get_token_info(token_id: u64) -> Option<TokenInfo> {
    // 현재는 worker에서만 관리하므로 None 반환
    // 향후 로컬 캐싱 구현 시 사용
    None
}

/// 백엔드에서 토큰 목록 조회 (로컬 캐시용)
#[query]
#[candid_method(query)]
fn list_tokens() -> Vec<u64> {
    // 현재는 worker에서만 관리하므로 빈 벡터 반환
    // 향후 로컬 캐싱 구현 시 사용
    Vec::new()
}

// =====================
// 마켓플레이스 기능
// =====================

/// 판매글 생성
#[update]
#[candid_method(update)]
fn create_listing(request: CreateListingRequest) -> Result<FavoriteRequest, String> {
    let response = create_listing_service(request)?;
    Ok(FavoriteRequest {
        listing_id: response.listing_id,
    })
}

/// 판매글 업데이트
#[update]
#[candid_method(update)]
fn update_listing(request: UpdateListingRequest) -> Result<SuccessResponse, String> {
    update_listing_service(request)
}

/// 판매글 삭제
#[update]
#[candid_method(update)]
fn delete_listing(listing_id: u64) -> Result<SuccessResponse, String> {
    delete_listing_service(listing_id)
}

/// 판매글 상세 조회
#[query]
#[candid_method(query)]
fn get_listing_detail(listing_id: u64) -> Option<ListingDetail> {
    get_listing_detail_service(listing_id)
}

/// 판매글 목록 조회
#[query]
#[candid_method(query)]
fn list_listings(status: Option<ListingStatus>, limit: Option<u64>) -> Vec<ListingSummary> {
    list_listings_service(status, limit)
}

/// 내 판매글 조회
#[query]
#[candid_method(query)]
fn get_my_listings() -> Vec<ListingSummary> {
    get_my_listings_service()
}

/// 특정 사용자의 판매글 조회
#[query]
#[candid_method(query)]
fn get_user_listings(user: Principal) -> Vec<ListingSummary> {
    get_user_listings_service(user)
}

// =====================
// 검색 기능
// =====================

/// 판매글 검색
#[query]
#[candid_method(query)]
fn search_listings(request: SearchListingsRequest) -> Result<SearchResult, String> {
    search_listings_service(request)
}

/// 고급 검색
#[query]
#[candid_method(query)]
fn advanced_search(
    keywords: Option<String>,
    category: Option<String>,
    min_price: Option<u64>,
    max_price: Option<u64>,
    tags: Option<Vec<String>>,
    seller: Option<Principal>,
    sort_by: Option<SortBy>,
    page: Option<u64>,
) -> Result<SearchResult, String> {
    let price_range = match (min_price, max_price) {
        (Some(min), Some(max)) => Some((min, max)),
        _ => None,
    };

    advanced_search_service(keywords, category, price_range, tags, seller, sort_by, page)
}

/// 검색어 자동완성
#[query]
#[candid_method(query)]
fn get_search_suggestions(partial_query: String, limit: Option<u64>) -> Vec<String> {
    get_search_suggestions_service(partial_query, limit.map(|l| l as usize))
}

/// 인기 검색어 조회
#[query]
#[candid_method(query)]
fn get_trending_keywords(limit: Option<u64>) -> Vec<(String, u32)> {
    get_trending_keywords_service(limit.map(|l| l as usize))
}

/// 검색 결과 통계
#[query]
#[candid_method(query)]
fn get_search_stats(request: SearchListingsRequest) -> Result<SearchStats, String> {
    get_search_stats_service(request)
}

/// 카테고리별 판매글 수 조회
#[query]
#[candid_method(query)]
fn get_categories() -> Vec<(String, u64)> {
    get_categories_service()
}

/// 인기 태그 조회
#[query]
#[candid_method(query)]
fn get_popular_tags(limit: Option<u64>) -> Vec<(String, u64)> {
    get_popular_tags_service(limit)
}

/// 연관 검색어 제안
#[query]
#[candid_method(query)]
fn get_related_keywords(query: String) -> Vec<String> {
    get_related_keywords_service(query)
}

/// 검색어 정규화
#[query]
#[candid_method(query)]
fn normalize_search_query(query: String) -> String {
    normalize_search_query_service(query)
}

// =====================
// 즐겨찾기 기능
// =====================

/// 즐겨찾기 추가
#[update]
#[candid_method(update)]
fn add_favorite(request: FavoriteRequest) -> Result<SuccessResponse, String> {
    add_favorite_service(request)
}

/// 즐겨찾기 제거
#[update]
#[candid_method(update)]
fn remove_favorite(request: FavoriteRequest) -> Result<SuccessResponse, String> {
    remove_favorite_service(request)
}

/// 내 즐겨찾기 목록 조회
#[query]
#[candid_method(query)]
fn get_my_favorites() -> Vec<ListingSummary> {
    get_my_favorites_service()
}

/// 즐겨찾기 여부 확인
#[query]
#[candid_method(query)]
fn is_favorited(listing_id: u64) -> bool {
    is_favorited_service(listing_id)
}

// =====================
// 추천 시스템
// =====================

/// 추천 판매글 조회
#[query]
#[candid_method(query)]
fn get_recommended_listings(limit: u64) -> Vec<ListingSummary> {
    let user = caller();
    get_recommended_listings_service(user, limit)
}

/// 유사한 판매글 찾기
#[query]
#[candid_method(query)]
fn get_similar_listings(listing_id: u64, limit: u64) -> Vec<ListingSummary> {
    get_similar_listings_service(listing_id, limit)
}

/// 트렌딩 판매글 조회
#[query]
#[candid_method(query)]
fn get_trending_listings() -> Vec<ListingSummary> {
    get_trending_listings_service()
}

/// 검색 기반 트렌딩 판매글
#[query]
#[candid_method(query)]
fn get_trending_by_search() -> Vec<ListingSummary> {
    get_trending_by_search_service()
}

// =====================
// 통계 및 관리
// =====================

/// 마켓플레이스 통계 조회
#[query]
#[candid_method(query)]
fn get_marketplace_stats() -> MarketplaceStats {
    get_marketplace_stats_service()
}

/// 최근 활동 조회
#[query]
#[candid_method(query)]
fn get_recent_activities(limit: Option<u64>) -> Vec<ActivityLog> {
    get_recent_activities_service(limit)
}

/// 비활성 판매글 정리
#[update]
#[candid_method(update)]
fn cleanup_inactive_listings() -> u64 {
    cleanup_inactive_listings_service()
}

/// 관리자 판매글 삭제
#[update]
#[candid_method(update)]
fn admin_delete_listing(listing_id: u64) -> Result<SuccessResponse, String> {
    admin_delete_listing_service(listing_id)
}

// =====================
// Candid 인터페이스 생성
// =====================

// Candid 인터페이스 자동 생성
ic_cdk::export_candid!();
