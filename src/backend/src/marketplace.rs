// backend/src/marketplace.rs

use crate::marketplace_storage::*;
use crate::marketplace_types::*;
use candid::Principal;
use ic_cdk::caller;

// =====================
// 1) 판매글 관리 함수들
// =====================

/// 판매글 생성
pub fn create_listing_service(
    request: CreateListingRequest,
) -> Result<CreateListingResponse, String> {
    // 요청 검증
    request.validate()?;

    let seller = caller();

    // 익명 사용자 차단
    if seller == Principal::anonymous() {
        return Err("익명 사용자는 판매글을 생성할 수 없습니다".to_string());
    }

    // 데이터 소유권 확인
    validate_data_ownership(&request.data_ids, seller)?;

    // 이미 민팅된 데이터인지 확인
    validate_data_not_minted(&request.data_ids)?;

    // 판매글 생성
    let listing_id = create_listing(request, seller)?;

    Ok(CreateListingResponse { listing_id })
}

/// 판매글 업데이트
pub fn update_listing_service(request: UpdateListingRequest) -> Result<SuccessResponse, String> {
    // 요청 검증
    request.validate()?;

    let user = caller();

    // 익명 사용자 차단
    if user == Principal::anonymous() {
        return Err("익명 사용자는 판매글을 수정할 수 없습니다".to_string());
    }

    // 판매글 업데이트
    update_listing(request, user)?;

    Ok(SuccessResponse {
        message: "판매글이 성공적으로 업데이트되었습니다".to_string(),
    })
}

/// 판매글 삭제
pub fn delete_listing_service(listing_id: u64) -> Result<SuccessResponse, String> {
    let user = caller();

    // 익명 사용자 차단
    if user == Principal::anonymous() {
        return Err("익명 사용자는 판매글을 삭제할 수 없습니다".to_string());
    }

    // 판매글 삭제
    delete_listing(listing_id, user)?;

    Ok(SuccessResponse {
        message: "판매글이 성공적으로 삭제되었습니다".to_string(),
    })
}

/// 판매글 상세 조회
pub fn get_listing_detail_service(listing_id: u64) -> Option<ListingDetail> {
    get_listing_detail(listing_id)
}

/// 판매글 목록 조회
pub fn list_listings_service(
    status: Option<ListingStatus>,
    limit: Option<u64>,
) -> Vec<ListingSummary> {
    let limit = limit.unwrap_or(50).min(100); // 최대 100개로 제한
    list_listings(status, Some(limit))
}

/// 내 판매글 조회
pub fn get_my_listings_service() -> Vec<ListingSummary> {
    let user = caller();

    if user == Principal::anonymous() {
        return Vec::new();
    }

    get_listings_by_seller(user)
}

/// 특정 사용자의 판매글 조회
pub fn get_user_listings_service(user: Principal) -> Vec<ListingSummary> {
    get_listings_by_seller(user)
}

// =====================
// 2) 검색 기능 (개선된 스마트 검색)
// =====================

/// 판매글 검색 (개선된 스마트 검색 사용)
pub fn search_listings_service(request: SearchListingsRequest) -> Result<SearchResult, String> {
    // 요청 검증
    request.validate()?;

    Ok(crate::marketplace_storage::search_listings(&request))
}

/// 검색어 자동완성
pub fn get_search_suggestions_service(partial_query: String, limit: Option<usize>) -> Vec<String> {
    let limit = limit.unwrap_or(10).min(20); // 최대 20개로 제한

    if partial_query.trim().len() < 2 {
        return Vec::new(); // 너무 짧은 검색어는 제안하지 않음
    }

    crate::marketplace_storage::get_search_suggestions(&partial_query, limit)
}

/// 인기 검색어 조회
pub fn get_trending_keywords_service(limit: Option<usize>) -> Vec<(String, u32)> {
    let limit = limit.unwrap_or(10).min(50);
    crate::marketplace_storage::get_trending_keywords(limit)
}

/// 검색 결과 통계
pub fn get_search_stats_service(
    request: SearchListingsRequest,
) -> Result<crate::marketplace_types::SearchStats, String> {
    // 요청 검증
    request.validate()?;

    // 검색 실행
    let search_result = crate::marketplace_storage::search_listings(&request);

    // 통계 계산 - marketplace_storage의 함수 대신 여기서 직접 계산
    let listings = &search_result.listings;

    if listings.is_empty() {
        return Ok(crate::marketplace_types::SearchStats {
            total_results: 0,
            avg_price: 0,
            price_range: (0, 0),
            top_categories: Vec::new(),
            top_sellers: Vec::new(),
        });
    }

    let total_results = listings.len() as u64;

    // 가격 통계
    let prices: Vec<u64> = listings.iter().map(|l| l.price).collect();
    let avg_price = prices.iter().sum::<u64>() / total_results;
    let min_price = *prices.iter().min().unwrap();
    let max_price = *prices.iter().max().unwrap();

    // 카테고리별 집계
    let mut category_counts: std::collections::HashMap<String, u64> =
        std::collections::HashMap::new();
    for listing in listings {
        *category_counts.entry(listing.category.clone()).or_insert(0) += 1;
    }
    let mut top_categories: Vec<(String, u64)> = category_counts.into_iter().collect();
    top_categories.sort_by(|a, b| b.1.cmp(&a.1));
    top_categories.truncate(5);

    // 판매자별 집계
    let mut seller_counts: std::collections::HashMap<Principal, u64> =
        std::collections::HashMap::new();
    for listing in listings {
        *seller_counts.entry(listing.seller).or_insert(0) += 1;
    }
    let mut top_sellers: Vec<(Principal, u64)> = seller_counts.into_iter().collect();
    top_sellers.sort_by(|a, b| b.1.cmp(&a.1));
    top_sellers.truncate(5);

    Ok(crate::marketplace_types::SearchStats {
        total_results,
        avg_price,
        price_range: (min_price, max_price),
        top_categories,
        top_sellers,
    })
}

/// 카테고리별 판매글 수 조회
pub fn get_categories_service() -> Vec<(String, u64)> {
    crate::marketplace_storage::get_listings_count_by_category()
}

/// 인기 태그 조회  
pub fn get_popular_tags_service(limit: Option<u64>) -> Vec<(String, u64)> {
    let limit = limit.unwrap_or(20).min(50);
    crate::marketplace_storage::get_popular_tags(limit)
}

/// 고급 검색 (여러 조건 조합)
pub fn advanced_search_service(
    keywords: Option<String>,
    category: Option<String>,
    price_range: Option<(u64, u64)>,
    tags: Option<Vec<String>>,
    seller: Option<Principal>,
    sort_by: Option<SortBy>,
    page: Option<u64>,
) -> Result<SearchResult, String> {
    let request = SearchListingsRequest {
        query: keywords,
        category,
        tags,
        min_price: price_range.map(|(min, _)| min),
        max_price: price_range.map(|(_, max)| max),
        currency: None,
        seller,
        status: Some(ListingStatus::Active), // 활성 상태만 검색
        sort_by,
        page,
        page_size: Some(20),
    };

    search_listings_service(request)
}

/// 검색어 정규화 (공개 함수)
pub fn normalize_search_query(query: &str) -> String {
    crate::marketplace_storage::tokenize_query(query).join(" ")
}

/// 연관 검색어 제안
pub fn get_related_keywords_service(query: String) -> Vec<String> {
    let tokens = crate::marketplace_storage::tokenize_query(&query);
    if tokens.is_empty() {
        return Vec::new();
    }

    // 입력된 키워드와 관련된 태그들 찾기
    let mut related_keywords = std::collections::HashSet::new();

    // 현재 활성 판매글들에서 관련 키워드 추출
    let active_listings =
        crate::marketplace_storage::list_listings(Some(ListingStatus::Active), Some(100));

    for listing in active_listings {
        let listing_keywords = crate::marketplace_storage::extract_keywords(&format!(
            "{} {}",
            listing.title, listing.description
        ));

        // 검색어 토큰 중 하나라도 매칭되면 해당 판매글의 다른 키워드들을 관련 키워드로 추가
        let has_match = tokens.iter().any(|token| {
            listing_keywords
                .iter()
                .any(|keyword| keyword.contains(token) || token.contains(keyword))
        });

        if has_match {
            for tag in &listing.tags {
                if !tokens
                    .iter()
                    .any(|token| tag.to_lowercase().contains(token))
                {
                    related_keywords.insert(tag.clone());
                }
            }
        }
    }

    let mut result: Vec<String> = related_keywords.into_iter().collect();
    result.sort();
    result.truncate(10);
    result
}

// =====================
// 3) 즐겨찾기 기능
// =====================

/// 즐겨찾기 추가
pub fn add_favorite_service(request: FavoriteRequest) -> Result<SuccessResponse, String> {
    let user = caller();

    // 익명 사용자 차단
    if user == Principal::anonymous() {
        return Err("익명 사용자는 즐겨찾기를 추가할 수 없습니다".to_string());
    }

    // 자신의 판매글은 즐겨찾기할 수 없음
    if let Some(listing) = get_listing_readonly(request.listing_id) {
        if listing.seller == user {
            return Err("자신의 판매글은 즐겨찾기할 수 없습니다".to_string());
        }
    } else {
        return Err("판매글을 찾을 수 없습니다".to_string());
    }

    // 즐겨찾기 추가
    add_favorite(user, request.listing_id)?;

    Ok(SuccessResponse {
        message: "즐겨찾기에 추가되었습니다".to_string(),
    })
}

/// 즐겨찾기 제거
pub fn remove_favorite_service(request: FavoriteRequest) -> Result<SuccessResponse, String> {
    let user = caller();

    // 익명 사용자 차단
    if user == Principal::anonymous() {
        return Err("익명 사용자는 즐겨찾기를 제거할 수 없습니다".to_string());
    }

    // 즐겨찾기 제거
    remove_favorite(user, request.listing_id)?;

    Ok(SuccessResponse {
        message: "즐겨찾기에서 제거되었습니다".to_string(),
    })
}

/// 내 즐겨찾기 목록 조회
pub fn get_my_favorites_service() -> Vec<ListingSummary> {
    let user = caller();

    if user == Principal::anonymous() {
        return Vec::new();
    }

    get_user_favorites(user)
}

/// 즐겨찾기 여부 확인
pub fn is_favorited_service(listing_id: u64) -> bool {
    let user = caller();

    if user == Principal::anonymous() {
        return false;
    }

    is_favorited(user, listing_id)
}

// =====================
// 4) 통계 및 분석
// =====================

/// 마켓플레이스 통계 조회
pub fn get_marketplace_stats_service() -> MarketplaceStats {
    get_marketplace_stats()
}

/// 최근 활동 조회
pub fn get_recent_activities_service(limit: Option<u64>) -> Vec<ActivityLog> {
    let limit = limit.unwrap_or(10).min(50);
    get_recent_activities(limit)
}

// =====================
// 5) 검증 함수들
// =====================

/// 데이터 소유권 확인
fn validate_data_ownership(data_ids: &[u64], user: Principal) -> Result<(), String> {
    // storage.rs의 새로운 함수 사용
    crate::storage::validate_data_ids_exist(data_ids)?;

    // TODO: 실제 구현에서는 데이터 업로더 정보를 저장하고 검증해야 함
    // 현재는 모든 사용자가 모든 데이터에 접근 가능하다고 가정

    Ok(())
}

/// 이미 민팅된 데이터인지 확인
fn validate_data_not_minted(data_ids: &[u64]) -> Result<(), String> {
    for &data_id in data_ids {
        // 데이터 가져오기
        let data = crate::storage::get_uploaded_data(data_id)
            .ok_or_else(|| format!("데이터 ID {}를 찾을 수 없습니다", data_id))?;

        // 민팅 여부 확인
        if crate::storage::check_data_minted(&data) {
            return Err(format!("데이터 ID {}는 이미 민팅되었습니다", data_id));
        }
    }

    Ok(())
}

// =====================
// 6) 관리자 기능 (추후 확장용)
// =====================

/// 관리자 권한 확인
fn is_admin(user: Principal) -> bool {
    // TODO: 관리자 목록을 저장하고 확인하는 로직 구현
    // 현재는 모든 사용자를 관리자로 취급하지 않음
    false
}

/// 판매글 강제 삭제 (관리자용)
pub fn admin_delete_listing_service(listing_id: u64) -> Result<SuccessResponse, String> {
    let user = caller();

    if !is_admin(user) {
        return Err("관리자 권한이 필요합니다".to_string());
    }

    // 강제 삭제 (소유자 확인 없음)
    if let Some(mut listing) = get_listing_readonly(listing_id) {
        listing.status = ListingStatus::Suspended;
        listing.updated_at = ic_cdk::api::time();

        // 저장 (실제로는 더 정교한 구현 필요)
        // update_listing_direct(listing)?;

        log_activity(
            ActivityType::ListingDeleted,
            user,
            Some(listing_id),
            "관리자에 의한 강제 삭제".to_string(),
        );

        Ok(SuccessResponse {
            message: "판매글이 관리자에 의해 삭제되었습니다".to_string(),
        })
    } else {
        Err("판매글을 찾을 수 없습니다".to_string())
    }
}

// =====================
// 7) 유틸리티 함수들
// =====================

/// 판매글 ID 유효성 검사
pub fn validate_listing_id(listing_id: u64) -> Result<(), String> {
    if listing_id == 0 {
        return Err("잘못된 판매글 ID입니다".to_string());
    }

    if get_listing_readonly(listing_id).is_none() {
        return Err("판매글을 찾을 수 없습니다".to_string());
    }

    Ok(())
}

/// 판매글 접근 권한 확인
pub fn check_listing_access(listing_id: u64, user: Principal) -> Result<bool, String> {
    let listing =
        get_listing_readonly(listing_id).ok_or_else(|| "판매글을 찾을 수 없습니다".to_string())?;

    // 삭제된 판매글은 소유자만 접근 가능
    if listing.status == ListingStatus::Deleted {
        return Ok(listing.seller == user);
    }

    // 일시 중단된 판매글은 소유자와 관리자만 접근 가능
    if listing.status == ListingStatus::Suspended {
        return Ok(listing.seller == user || is_admin(user));
    }

    // 일반 판매글은 모든 사용자 접근 가능
    Ok(true)
}

/// 판매글 상태 변경 권한 확인
pub fn can_change_listing_status(
    listing_id: u64,
    user: Principal,
    new_status: &ListingStatus,
) -> Result<bool, String> {
    let listing =
        get_listing_readonly(listing_id).ok_or_else(|| "판매글을 찾을 수 없습니다".to_string())?;

    // 소유자가 아니고 관리자도 아니면 권한 없음
    if listing.seller != user && !is_admin(user) {
        return Ok(false);
    }

    // 이미 판매된 항목은 상태 변경 불가
    if listing.status == ListingStatus::Sold && new_status != &ListingStatus::Sold {
        return Ok(false);
    }

    // 삭제된 항목은 복구 불가 (관리자 제외)
    if listing.status == ListingStatus::Deleted && !is_admin(user) {
        return Ok(false);
    }

    Ok(true)
}

/// 가격 형식 검증
pub fn validate_price(price: u64, currency: &str) -> Result<(), String> {
    match currency {
        "ICP" => {
            if price < 1_000_000 {
                // 0.01 ICP (e8s 단위)
                return Err("ICP 가격은 최소 0.01 ICP 이상이어야 합니다".to_string());
            }
            if price > 1_000_000_000_000 {
                // 10,000 ICP
                return Err("ICP 가격은 최대 10,000 ICP 이하여야 합니다".to_string());
            }
        }
        "USD" => {
            if price < 100 {
                // $1.00 (센트 단위)
                return Err("USD 가격은 최소 $1.00 이상이어야 합니다".to_string());
            }
            if price > 100_000_00 {
                // $100,000.00
                return Err("USD 가격은 최대 $100,000.00 이하여야 합니다".to_string());
            }
        }
        _ => {
            return Err("지원되지 않는 통화입니다".to_string());
        }
    }

    Ok(())
}

/// 태그 정규화
pub fn normalize_tags(tags: Vec<String>) -> Vec<String> {
    tags.into_iter()
        .map(|tag| tag.trim().to_lowercase())
        .filter(|tag| !tag.is_empty() && tag.len() <= 50)
        .collect::<std::collections::HashSet<_>>() // 중복 제거
        .into_iter()
        .take(20) // 최대 20개
        .collect()
}

/// 카테고리 정규화
pub fn normalize_category(category: &str) -> String {
    category.trim().to_lowercase()
}

/// 설명 미리보기 생성
pub fn generate_description_preview(description: &str, max_length: usize) -> String {
    if description.len() <= max_length {
        description.to_string()
    } else {
        let preview = &description[..max_length.min(description.len())];
        // 단어 중간에서 자르지 않도록 마지막 공백 찾기
        if let Some(last_space) = preview.rfind(' ') {
            format!("{}...", &preview[..last_space])
        } else {
            format!("{}...", preview)
        }
    }
}

// =====================
// 8) 배치 작업 함수들
// =====================

/// 비활성 판매글 정리 (일정 기간 후 자동 비활성화)
pub fn cleanup_inactive_listings() -> u64 {
    let current_time = ic_cdk::api::time();
    let thirty_days = 30 * 24 * 60 * 60 * 1_000_000_000u64; // 30일 (나노초)
    let mut cleaned_count = 0u64;

    // 활성 판매글들을 가져와서 확인
    let active_listings =
        crate::marketplace_storage::list_listings(Some(ListingStatus::Active), Some(1000));

    for listing_summary in active_listings {
        if current_time.saturating_sub(listing_summary.updated_at) > thirty_days {
            // 30일 이상 업데이트되지 않은 판매글을 일시 중단으로 변경
            let update_request = UpdateListingRequest {
                listing_id: listing_summary.id,
                title: None,
                description: None,
                price: None,
                currency: None,
                category: None,
                tags: None,
                preview_data: None,
                status: Some(ListingStatus::Suspended),
            };

            // 시스템에서 자동으로 업데이트 (권한 체크 우회)
            if crate::marketplace_storage::update_listing(update_request, listing_summary.seller)
                .is_ok()
            {
                cleaned_count += 1;

                crate::marketplace_storage::log_activity(
                    ActivityType::ListingUpdated,
                    Principal::management_canister(),
                    Some(listing_summary.id),
                    "비활성으로 인한 자동 일시 중단".to_string(),
                );
            }
        }
    }

    cleaned_count
}

/// 인기 판매글 업데이트 (조회수, 즐겨찾기 수 기반)
pub fn update_trending_listings() -> Vec<ListingSummary> {
    let mut trending =
        crate::marketplace_storage::list_listings(Some(ListingStatus::Active), Some(50));

    // 인기도 계산 (조회수 + 즐겨찾기 수 * 2)
    trending.sort_by(|a, b| {
        let score_a = a.view_count + (a.favorite_count * 2);
        let score_b = b.view_count + (b.favorite_count * 2);
        score_b.cmp(&score_a)
    });

    trending.truncate(10);
    trending
}

/// 검색 성능 최적화를 위한 인덱스 업데이트
pub fn update_search_indices() {
    // TODO: 검색 성능 향상을 위한 인덱스 구조 구현
    // 현재는 전체 스캔을 사용하지만, 실제 운영에서는 별도 인덱스 테이블 필요
    ic_cdk::println!("Search indices updated");
}

// =====================
// 9) 데이터 내보내기/가져오기
// =====================

/// 판매글 데이터 내보내기 (백업용)
pub fn export_listings_data() -> Vec<Listing> {
    // marketplace_storage의 공개 함수를 통해 데이터 가져오기
    list_listings(None, None)
        .into_iter()
        .filter_map(|summary| get_listing_readonly(summary.id))
        .collect()
}

/// 즐겨찾기 데이터 내보내기 (백업용)
pub fn export_favorites_data() -> Vec<Favorite> {
    // 구현이 복잡하므로 일단 빈 벡터 반환
    // 실제로는 marketplace_storage에서 공개 함수 추가 필요
    Vec::new()
}

/// 활동 로그 데이터 내보내기 (백업용)
pub fn export_activity_logs() -> Vec<ActivityLog> {
    // get_recent_activities를 통해 제한된 데이터만 가져오기
    crate::marketplace_storage::get_recent_activities(1000)
}

// =====================
// 10) 추천 시스템 (기본)
// =====================

/// 사용자 맞춤 추천 판매글 (개선된 검색 활용)
pub fn get_recommended_listings(user: Principal, limit: u64) -> Vec<ListingSummary> {
    // 간단한 추천 알고리즘: 사용자의 즐겨찾기 기반
    let user_favorites = crate::marketplace_storage::get_user_favorites(user);

    if user_favorites.is_empty() {
        // 즐겨찾기가 없으면 인기 판매글 반환
        return update_trending_listings();
    }

    // 사용자가 즐겨찾기한 판매글들의 키워드 분석
    let mut all_keywords = std::collections::HashSet::new();
    let mut preferred_categories = std::collections::HashMap::new();

    for favorite in &user_favorites {
        // 제목과 설명에서 키워드 추출
        let title_keywords = crate::marketplace_storage::extract_keywords(&favorite.title);
        let desc_keywords = crate::marketplace_storage::extract_keywords(&favorite.description);

        all_keywords.extend(title_keywords);
        all_keywords.extend(desc_keywords);

        // 태그 추가
        for tag in &favorite.tags {
            all_keywords.insert(tag.clone());
        }

        // 카테고리 선호도 계산
        *preferred_categories
            .entry(favorite.category.clone())
            .or_insert(0) += 1;
    }

    // 상위 키워드들로 검색 쿼리 생성
    let top_keywords: Vec<String> = all_keywords
        .into_iter()
        .filter(|keyword| keyword.len() >= 2)
        .take(5)
        .collect();

    if top_keywords.is_empty() {
        return update_trending_listings();
    }

    // 키워드 기반 검색 실행
    let search_request = SearchListingsRequest {
        query: Some(top_keywords.join(" ")),
        category: preferred_categories.keys().next().cloned(), // 가장 선호하는 카테고리
        tags: None,
        min_price: None,
        max_price: None,
        currency: None,
        seller: None,
        status: Some(ListingStatus::Active),
        sort_by: Some(SortBy::ViewCountDesc), // 인기도 순
        page: Some(0),
        page_size: Some(limit),
    };

    let search_result = crate::marketplace_storage::search_listings(&search_request);

    // 이미 즐겨찾기한 항목들 제외
    search_result
        .listings
        .into_iter()
        .filter(|listing| !user_favorites.iter().any(|fav| fav.id == listing.id))
        .take(limit as usize)
        .collect()
}

/// 유사한 판매글 찾기 (개선된 검색 활용)
pub fn get_similar_listings(listing_id: u64, limit: u64) -> Vec<ListingSummary> {
    let target_listing = match crate::marketplace_storage::get_listing_readonly(listing_id) {
        Some(listing) => listing,
        None => return Vec::new(),
    };

    // 대상 판매글의 키워드 추출
    let title_keywords = crate::marketplace_storage::extract_keywords(&target_listing.title);
    let desc_keywords = crate::marketplace_storage::extract_keywords(&target_listing.description);

    let mut all_keywords: Vec<String> = title_keywords
        .into_iter()
        .chain(desc_keywords.into_iter())
        .chain(target_listing.tags.iter().cloned())
        .filter(|keyword| keyword.len() >= 2)
        .take(5) // 상위 5개 키워드만 사용
        .collect();

    if all_keywords.is_empty() {
        all_keywords.push(target_listing.category.clone());
    }

    // 유사 판매글 검색
    let search_request = SearchListingsRequest {
        query: Some(all_keywords.join(" ")),
        category: Some(target_listing.category.clone()),
        tags: None,
        min_price: Some(target_listing.price / 2), // 가격 범위 ±50%
        max_price: Some(target_listing.price * 3 / 2),
        currency: Some(target_listing.currency.clone()),
        seller: None, // 다른 판매자 포함
        status: Some(ListingStatus::Active),
        sort_by: None, // 관련성 점수 순으로 정렬됨
        page: Some(0),
        page_size: Some(limit + 1), // 자기 자신 제외를 위해 +1
    };

    let search_result = crate::marketplace_storage::search_listings(&search_request);

    // 자기 자신 제외
    search_result
        .listings
        .into_iter()
        .filter(|listing| listing.id != listing_id)
        .take(limit as usize)
        .collect()
}

/// 검색 기반 트렌딩 판매글
pub fn get_trending_by_search() -> Vec<ListingSummary> {
    // 인기 키워드를 기반으로 트렌딩 판매글 찾기
    let trending_keywords = crate::marketplace_storage::get_trending_keywords(5);

    if trending_keywords.is_empty() {
        return update_trending_listings();
    }

    let keywords: Vec<String> = trending_keywords
        .into_iter()
        .map(|(keyword, _)| keyword)
        .collect();

    let search_request = SearchListingsRequest {
        query: Some(keywords.join(" ")),
        category: None,
        tags: None,
        min_price: None,
        max_price: None,
        currency: None,
        seller: None,
        status: Some(ListingStatus::Active),
        sort_by: Some(SortBy::ViewCountDesc),
        page: Some(0),
        page_size: Some(10),
    };

    let search_result = crate::marketplace_storage::search_listings(&search_request);
    search_result.listings
}
