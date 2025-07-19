// backend/src/marketplace_storage.rs

use crate::marketplace_types::*;
use candid::Principal;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl, StableBTreeMap, StableCell,
};
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

// 메모리 관리
type Memory = VirtualMemory<DefaultMemoryImpl>;

// =====================
// 0) 유틸리티 함수들
// =====================

/// 안전한 ID 생성 (타임스탬프 + 해시 기반)
fn generate_unique_id(prefix: &str, additional_data: &str) -> u64 {
    let timestamp = ic_cdk::api::time();
    let mut hasher = Sha256::new();

    hasher.update(prefix.as_bytes());
    hasher.update(additional_data.as_bytes());
    hasher.update(timestamp.to_le_bytes());

    let hash = hasher.finalize();
    let mut id_bytes = [0u8; 8];
    id_bytes.copy_from_slice(&hash[0..8]);

    u64::from_le_bytes(id_bytes)
}

// 마켓플레이스 전용 메모리 관리자
thread_local! {
    static MARKETPLACE_MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

// 메모리 헬퍼 함수들
fn get_listings_memory() -> Memory {
    MARKETPLACE_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
}

fn get_favorites_memory() -> Memory {
    MARKETPLACE_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
}

fn get_activity_logs_memory() -> Memory {
    MARKETPLACE_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
}

fn get_listing_counter_memory() -> Memory {
    MARKETPLACE_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
}

// 저장소들
thread_local! {
    static LISTINGS: RefCell<Option<StableBTreeMap<u64, Listing, Memory>>> = const { RefCell::new(None) };
    static FAVORITES: RefCell<Option<StableBTreeMap<u64, Favorite, Memory>>> = const { RefCell::new(None) };
    static ACTIVITY_LOGS: RefCell<Option<StableBTreeMap<u64, ActivityLog, Memory>>> = const { RefCell::new(None) };
    static LISTING_COUNTER: RefCell<Option<StableCell<u64, Memory>>> = const { RefCell::new(None) };
}

// =====================
// 1) 저장소 초기화
// =====================

pub fn init_marketplace_storage() {
    // 판매글 저장소 초기화
    LISTINGS.with(|storage| {
        let mut storage = storage.borrow_mut();
        if storage.is_none() {
            *storage = Some(StableBTreeMap::init(get_listings_memory()));
        }
    });

    // 즐겨찾기 저장소 초기화
    FAVORITES.with(|storage| {
        let mut storage = storage.borrow_mut();
        if storage.is_none() {
            *storage = Some(StableBTreeMap::init(get_favorites_memory()));
        }
    });

    // 활동 로그 저장소 초기화
    ACTIVITY_LOGS.with(|storage| {
        let mut storage = storage.borrow_mut();
        if storage.is_none() {
            *storage = Some(StableBTreeMap::init(get_activity_logs_memory()));
        }
    });

    // 판매글 카운터 초기화
    LISTING_COUNTER.with(|counter| {
        let mut counter = counter.borrow_mut();
        if counter.is_none() {
            *counter = Some(
                StableCell::init(get_listing_counter_memory(), 0)
                    .expect("Failed to initialize listing counter"),
            );
        }
    });

    ic_cdk::println!("Marketplace storage initialized");
}

// =====================
// 2) 스마트 검색 시스템
// =====================

/// 검색어를 정규화하고 토큰화 (순수 Rust 구현)
pub fn tokenize_query(query: &str) -> Vec<String> {
    query
        .to_lowercase()
        .split_whitespace()
        .filter(|word| word.len() > 1) // 1글자 단어 제외
        .map(|word| {
            // 특수문자 제거, 한글과 영숫자만 허용
            word.chars()
                .filter(|c| {
                    c.is_alphanumeric() ||
                    (*c >= '\u{AC00}' && *c <= '\u{D7AF}') || // 한글 완성형
                    (*c >= '\u{1100}' && *c <= '\u{11FF}') || // 한글 자모
                    (*c >= '\u{3130}' && *c <= '\u{318F}') // 한글 호환 자모
                })
                .collect::<String>()
        })
        .filter(|word| !word.is_empty())
        .collect()
}

/// 텍스트에서 키워드 추출
pub fn extract_keywords(text: &str) -> HashSet<String> {
    let tokens = tokenize_query(text);
    let mut keywords = HashSet::new();

    for token in tokens {
        // 원본 단어 추가
        keywords.insert(token.clone());

        // 3글자 이상인 경우 부분 문자열도 추가 (검색 확장)
        if token.len() >= 3 {
            for i in 0..token.len() - 1 {
                for j in i + 2..=token.len() {
                    if j > i + 1 {
                        keywords.insert(token[i..j].to_string());
                    }
                }
            }
        }
    }

    keywords
}

/// 검색 관련성 점수 계산
pub fn calculate_relevance_score(listing: &Listing, query_tokens: &[String]) -> f64 {
    if query_tokens.is_empty() {
        return 0.0;
    }

    let mut total_score = 0.0;
    let title_keywords = extract_keywords(&listing.title);
    let desc_keywords = extract_keywords(&listing.description);
    let tag_keywords: HashSet<String> = listing
        .tags
        .iter()
        .flat_map(|tag| extract_keywords(tag))
        .collect();

    for query_token in query_tokens {
        let mut token_score = 0.0;

        // 제목 매칭 (가중치 높음)
        if title_keywords.contains(query_token) {
            token_score += 10.0;

            // 제목 전체 매칭 보너스
            if listing.title.to_lowercase().contains(query_token) {
                token_score += 5.0;
            }
        }

        // 설명 매칭
        if desc_keywords.contains(query_token) {
            token_score += 3.0;
        }

        // 태그 매칭 (정확도 높음)
        if tag_keywords.contains(query_token) {
            token_score += 8.0;
        }

        // 카테고리 매칭
        if listing.category.to_lowercase().contains(query_token) {
            token_score += 12.0;
        }

        total_score += token_score;
    }

    // 인기도 보너스 (조회수 + 즐겨찾기)
    let popularity_bonus =
        (listing.view_count as f64 * 0.01) + (listing.favorite_count as f64 * 0.1);

    total_score + popularity_bonus
}

/// 기본 필터 적용 (검색어 제외)
fn apply_basic_filters(listing: &Listing, request: &SearchListingsRequest) -> bool {
    // 삭제된 항목은 제외
    if listing.status == ListingStatus::Deleted {
        return false;
    }

    // 상태 필터
    if let Some(ref status) = request.status {
        if &listing.status != status {
            return false;
        }
    }

    // 카테고리 필터
    if let Some(ref category) = request.category {
        if &listing.category != category {
            return false;
        }
    }

    // 판매자 필터
    if let Some(ref seller) = request.seller {
        if &listing.seller != seller {
            return false;
        }
    }

    // 가격 필터
    if let Some(min_price) = request.min_price {
        if listing.price < min_price {
            return false;
        }
    }
    if let Some(max_price) = request.max_price {
        if listing.price > max_price {
            return false;
        }
    }

    // 통화 필터
    if let Some(ref currency) = request.currency {
        if &listing.currency != currency {
            return false;
        }
    }

    // 태그 필터 (모든 태그가 포함되어야 함)
    if let Some(ref tags) = request.tags {
        for tag in tags {
            if !listing
                .tags
                .iter()
                .any(|t| t.to_lowercase().contains(&tag.to_lowercase()))
            {
                return false;
            }
        }
    }

    true
}

/// 스마트 검색 (관련성 점수 기반)
pub fn smart_search_listings(request: &SearchListingsRequest) -> SearchResult {
    let all_listings: Vec<Listing> = LISTINGS.with(|storage_cell| {
        let storage_ref = storage_cell.borrow();
        if let Some(storage) = storage_ref.as_ref() {
            storage
                .iter()
                .map(|(_, listing)| listing)
                .filter(|listing| listing.status != ListingStatus::Deleted)
                .collect()
        } else {
            Vec::new()
        }
    });

    // 1단계: 기본 필터 적용 (가격, 카테고리, 상태 등)
    let mut filtered_listings: Vec<Listing> = all_listings
        .into_iter()
        .filter(|listing| apply_basic_filters(listing, request))
        .collect();

    // 2단계: 검색어가 있으면 관련성 점수 계산
    if let Some(ref query) = request.query {
        let query_tokens = tokenize_query(query);

        if !query_tokens.is_empty() {
            // 관련성 점수와 함께 저장
            let mut scored_listings: Vec<(Listing, f64)> = filtered_listings
                .into_iter()
                .map(|listing| {
                    let score = calculate_relevance_score(&listing, &query_tokens);
                    (listing, score)
                })
                .filter(|(_, score)| *score > 0.0) // 관련성 있는 것만
                .collect();

            // 관련성 점수순 정렬
            scored_listings
                .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            filtered_listings = scored_listings
                .into_iter()
                .map(|(listing, _)| listing)
                .collect();
        }
    } else {
        // 검색어가 없으면 기본 정렬
        sort_listings(&mut filtered_listings, &request.get_sort_by());
    }

    // 3단계: 페이징 적용
    let total_count = filtered_listings.len() as u64;
    let page = request.get_page();
    let page_size = request.get_page_size();
    let total_pages = (total_count + page_size - 1) / page_size;

    let start = (page * page_size) as usize;
    let end = ((page + 1) * page_size).min(total_count) as usize;

    let listings = if start < filtered_listings.len() {
        filtered_listings[start..end.min(filtered_listings.len())]
            .iter()
            .map(|listing| to_listing_summary(listing.clone()))
            .collect()
    } else {
        Vec::new()
    };

    SearchResult {
        listings,
        total_count,
        page,
        page_size,
        total_pages,
    }
}

// =====================
// 2) 판매글 관리
// =====================

/// 판매글 생성
pub fn create_listing(request: CreateListingRequest, seller: Principal) -> Result<u64, String> {
    // 판매글 ID 생성
    let listing_id = LISTING_COUNTER.with(|counter_cell| {
        let mut counter_ref = counter_cell.borrow_mut();
        if let Some(counter) = counter_ref.as_mut() {
            let current = counter.get();
            let next_id = current + 1;
            counter
                .set(next_id)
                .map_err(|e| format!("카운터 업데이트 실패: {:?}", e))?;
            Ok(next_id)
        } else {
            Err("Listing counter not initialized".to_string())
        }
    })?;

    let now = ic_cdk::api::time();
    let listing = Listing {
        id: listing_id,
        seller,
        title: request.title,
        description: request.description,
        price: request.price,
        currency: request.currency,
        data_ids: request.data_ids,
        category: request.category,
        tags: request.tags,
        preview_data: request.preview_data,
        status: ListingStatus::Active,
        created_at: now,
        updated_at: now,
        view_count: 0,
        favorite_count: 0,
    };

    // 판매글 저장
    LISTINGS.with(|storage_cell| {
        let mut storage_ref = storage_cell.borrow_mut();
        if let Some(storage) = storage_ref.as_mut() {
            storage.insert(listing_id, listing);
            Ok(())
        } else {
            Err("Listings storage not initialized".to_string())
        }
    })?;

    // 활동 로그 기록
    log_activity(
        ActivityType::ListingCreated,
        seller,
        Some(listing_id),
        "판매글 생성".to_string(),
    );

    Ok(listing_id)
}

/// 판매글 조회 (조회수 증가)
pub fn get_listing(listing_id: u64) -> Option<Listing> {
    let mut listing = LISTINGS.with(|storage| {
        let storage = storage.borrow();
        storage.as_ref()?.get(&listing_id)
    })?;

    // 조회수 증가
    listing.view_count += 1;
    listing.updated_at = ic_cdk::api::time();

    // 업데이트된 정보 저장
    LISTINGS.with(|storage| {
        let mut storage = storage.borrow_mut();
        if let Some(ref mut storage) = storage.as_mut() {
            storage.insert(listing_id, listing.clone());
        }
    });

    Some(listing)
}

/// 판매글 조회 (조회수 증가 없음)
pub fn get_listing_readonly(listing_id: u64) -> Option<Listing> {
    LISTINGS.with(|storage| {
        let storage = storage.borrow();
        storage.as_ref()?.get(&listing_id)
    })
}

/// 판매글 업데이트
pub fn update_listing(request: UpdateListingRequest, user: Principal) -> Result<(), String> {
    let mut listing = LISTINGS.with(|storage_cell| {
        let storage_ref = storage_cell.borrow();
        if let Some(storage) = storage_ref.as_ref() {
            storage
                .get(&request.listing_id)
                .ok_or_else(|| "판매글을 찾을 수 없습니다".to_string())
        } else {
            Err("Listings storage not initialized".to_string())
        }
    })?;

    // 권한 확인
    if listing.seller != user {
        return Err("판매글을 수정할 권한이 없습니다".to_string());
    }

    // 업데이트
    if let Some(title) = request.title {
        listing.title = title;
    }
    if let Some(description) = request.description {
        listing.description = description;
    }
    if let Some(price) = request.price {
        listing.price = price;
    }
    if let Some(currency) = request.currency {
        listing.currency = currency;
    }
    if let Some(category) = request.category {
        listing.category = category;
    }
    if let Some(tags) = request.tags {
        listing.tags = tags;
    }
    if let Some(preview_data) = request.preview_data {
        listing.preview_data = Some(preview_data);
    }
    if let Some(status) = request.status {
        listing.status = status;
    }

    listing.updated_at = ic_cdk::api::time();

    // 저장
    LISTINGS.with(|storage_cell| {
        let mut storage_ref = storage_cell.borrow_mut();
        if let Some(storage) = storage_ref.as_mut() {
            storage.insert(request.listing_id, listing);
            Ok(())
        } else {
            Err("Listings storage not initialized".to_string())
        }
    })?;

    // 활동 로그 기록
    log_activity(
        ActivityType::ListingUpdated,
        user,
        Some(request.listing_id),
        "판매글 수정".to_string(),
    );

    Ok(())
}

/// 판매글 삭제
pub fn delete_listing(listing_id: u64, user: Principal) -> Result<(), String> {
    let listing = LISTINGS.with(|storage_cell| {
        let storage_ref = storage_cell.borrow();
        if let Some(storage) = storage_ref.as_ref() {
            storage
                .get(&listing_id)
                .ok_or_else(|| "판매글을 찾을 수 없습니다".to_string())
        } else {
            Err("Listings storage not initialized".to_string())
        }
    })?;

    // 권한 확인
    if listing.seller != user {
        return Err("판매글을 삭제할 권한이 없습니다".to_string());
    }

    // 상태를 삭제됨으로 변경 (실제로는 삭제하지 않고 상태만 변경)
    let mut updated_listing = listing;
    updated_listing.status = ListingStatus::Deleted;
    updated_listing.updated_at = ic_cdk::api::time();

    LISTINGS.with(|storage_cell| {
        let mut storage_ref = storage_cell.borrow_mut();
        if let Some(storage) = storage_ref.as_mut() {
            storage.insert(listing_id, updated_listing);
            Ok(())
        } else {
            Err("Listings storage not initialized".to_string())
        }
    })?;

    // 활동 로그 기록
    log_activity(
        ActivityType::ListingDeleted,
        user,
        Some(listing_id),
        "판매글 삭제".to_string(),
    );

    Ok(())
}

/// 판매글 목록 조회 (기본)
pub fn list_listings(status: Option<ListingStatus>, limit: Option<u64>) -> Vec<ListingSummary> {
    LISTINGS.with(|storage| {
        let storage = storage.borrow();
        match storage.as_ref() {
            Some(storage) => {
                let mut listings: Vec<_> = storage
                    .iter()
                    .filter(|(_, listing)| {
                        if let Some(ref status) = status {
                            &listing.status == status
                        } else {
                            listing.status != ListingStatus::Deleted
                        }
                    })
                    .map(|(_, listing)| to_listing_summary(listing))
                    .collect();

                // 최신순 정렬
                listings.sort_by(|a, b| b.created_at.cmp(&a.created_at));

                // 제한 적용
                if let Some(limit) = limit {
                    listings.truncate(limit as usize);
                }

                listings
            }
            None => Vec::new(),
        }
    })
}

/// 판매자별 판매글 조회
pub fn get_listings_by_seller(seller: Principal) -> Vec<ListingSummary> {
    LISTINGS.with(|storage| {
        let storage = storage.borrow();
        match storage.as_ref() {
            Some(storage) => {
                let mut listings: Vec<_> = storage
                    .iter()
                    .filter(|(_, listing)| {
                        listing.seller == seller && listing.status != ListingStatus::Deleted
                    })
                    .map(|(_, listing)| to_listing_summary(listing))
                    .collect();

                listings.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                listings
            }
            None => Vec::new(),
        }
    })
}

// =====================
// 3) 기존 검색 함수 (스마트 검색으로 교체)
// =====================

/// 판매글 검색 (개선된 스마트 검색 사용)
pub fn search_listings(request: &SearchListingsRequest) -> SearchResult {
    smart_search_listings(request)
}

// 기존 apply_filters 함수는 apply_basic_filters로 대체됨

fn sort_listings(listings: &mut Vec<Listing>, sort_by: &SortBy) {
    match sort_by {
        SortBy::CreatedAtDesc => {
            listings.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        }
        SortBy::CreatedAtAsc => {
            listings.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        }
        SortBy::PriceDesc => {
            listings.sort_by(|a, b| b.price.cmp(&a.price));
        }
        SortBy::PriceAsc => {
            listings.sort_by(|a, b| a.price.cmp(&b.price));
        }
        SortBy::ViewCountDesc => {
            listings.sort_by(|a, b| b.view_count.cmp(&a.view_count));
        }
        SortBy::FavoriteCountDesc => {
            listings.sort_by(|a, b| b.favorite_count.cmp(&a.favorite_count));
        }
        SortBy::UpdatedAtDesc => {
            listings.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        }
    }
}

// =====================
// 4) 자동완성 및 제안 기능
// =====================

/// 검색어 자동완성 제안
pub fn get_search_suggestions(partial_query: &str, limit: usize) -> Vec<String> {
    let query_lower = partial_query.to_lowercase();
    let mut suggestions = HashSet::new();

    LISTINGS.with(|storage_cell| {
        let storage_ref = storage_cell.borrow();
        if let Some(storage) = storage_ref.as_ref() {
            for (_, listing) in storage.iter() {
                if listing.status == ListingStatus::Active {
                    // 제목에서 제안
                    let title_words = tokenize_query(&listing.title);
                    for word in title_words {
                        if word.starts_with(&query_lower) && word.len() > partial_query.len() {
                            suggestions.insert(word);
                        }
                    }

                    // 태그에서 제안
                    for tag in &listing.tags {
                        if tag.to_lowercase().starts_with(&query_lower) {
                            suggestions.insert(tag.clone());
                        }
                    }

                    // 카테고리에서 제안
                    if listing.category.to_lowercase().starts_with(&query_lower) {
                        suggestions.insert(listing.category.clone());
                    }
                }
            }
        }
    });

    let mut result: Vec<String> = suggestions.into_iter().collect();
    result.sort();
    result.truncate(limit);
    result
}

/// 인기 검색어 추출
pub fn get_trending_keywords(limit: usize) -> Vec<(String, u32)> {
    let mut keyword_counts: HashMap<String, u32> = HashMap::new();

    LISTINGS.with(|storage_cell| {
        let storage_ref = storage_cell.borrow();
        if let Some(storage) = storage_ref.as_ref() {
            for (_, listing) in storage.iter() {
                if listing.status == ListingStatus::Active {
                    // 제목에서 키워드 추출
                    let title_keywords = extract_keywords(&listing.title);
                    for keyword in title_keywords {
                        if keyword.len() >= 2 {
                            let count = keyword_counts.entry(keyword).or_insert(0);
                            *count += listing.view_count as u32;
                        }
                    }

                    // 태그에서 키워드 추출
                    for tag in &listing.tags {
                        let count = keyword_counts.entry(tag.clone()).or_insert(0);
                        *count += (listing.view_count + listing.favorite_count) as u32;
                    }
                }
            }
        }
    });

    let mut result: Vec<(String, u32)> = keyword_counts.into_iter().collect();
    result.sort_by(|a, b| b.1.cmp(&a.1)); // 빈도수 내림차순
    result.truncate(limit);
    result
}

// =====================
// 5) 검색 통계 및 분석 (타입은 marketplace_types.rs에 정의됨)
// =====================

/// 검색 결과에 대한 통계 계산은 marketplace.rs에서 처리

// =====================
// 4) 즐겨찾기 관리
// =====================

/// 즐겨찾기 추가
pub fn add_favorite(user: Principal, listing_id: u64) -> Result<(), String> {
    // 판매글 존재 확인
    let mut listing = LISTINGS.with(|storage_cell| {
        let storage_ref = storage_cell.borrow();
        if let Some(storage) = storage_ref.as_ref() {
            storage
                .get(&listing_id)
                .ok_or_else(|| "판매글을 찾을 수 없습니다".to_string())
        } else {
            Err("Listings storage not initialized".to_string())
        }
    })?;

    // 이미 즐겨찾기에 추가되어 있는지 확인
    let already_favorited = FAVORITES.with(|storage_cell| {
        let storage_ref = storage_cell.borrow();
        if let Some(storage) = storage_ref.as_ref() {
            storage
                .iter()
                .any(|(_, fav)| fav.user == user && fav.listing_id == listing_id)
        } else {
            false
        }
    });

    if already_favorited {
        return Err("이미 즐겨찾기에 추가된 판매글입니다".to_string());
    }

    // 즐겨찾기 ID 생성 (해시 기반으로 고유성 보장)
    let favorite_id = generate_unique_id("favorite", &format!("{}_{}", user.to_text(), listing_id));

    let favorite = Favorite {
        user,
        listing_id,
        created_at: ic_cdk::api::time(),
    };

    // 즐겨찾기 저장
    FAVORITES.with(|storage_cell| {
        let mut storage_ref = storage_cell.borrow_mut();
        if let Some(storage) = storage_ref.as_mut() {
            storage.insert(favorite_id, favorite);
            Ok(())
        } else {
            Err("Favorites storage not initialized".to_string())
        }
    })?;

    // 판매글의 즐겨찾기 수 증가
    listing.favorite_count += 1;
    listing.updated_at = ic_cdk::api::time();

    LISTINGS.with(|storage_cell| {
        let mut storage_ref = storage_cell.borrow_mut();
        if let Some(storage) = storage_ref.as_mut() {
            storage.insert(listing_id, listing);
        }
    });

    // 활동 로그 기록
    log_activity(
        ActivityType::ListingFavorited,
        user,
        Some(listing_id),
        "즐겨찾기 추가".to_string(),
    );

    Ok(())
}

/// 즐겨찾기 제거
pub fn remove_favorite(user: Principal, listing_id: u64) -> Result<(), String> {
    // 즐겨찾기 찾기
    let favorite_id = FAVORITES
        .with(|storage| {
            let storage = storage.borrow();
            match storage.as_ref() {
                Some(storage) => {
                    for (id, fav) in storage.iter() {
                        if fav.user == user && fav.listing_id == listing_id {
                            return Some(id);
                        }
                    }
                    None
                }
                None => None,
            }
        })
        .ok_or_else(|| "즐겨찾기를 찾을 수 없습니다".to_string())?;

    // 즐겨찾기 삭제
    FAVORITES.with(|storage| {
        let mut storage = storage.borrow_mut();
        if let Some(ref mut storage) = storage.as_mut() {
            storage.remove(&favorite_id);
        }
    });

    // 판매글의 즐겨찾기 수 감소
    if let Some(mut listing) = LISTINGS.with(|storage| {
        let storage = storage.borrow();
        storage.as_ref().and_then(|s| s.get(&listing_id))
    }) {
        listing.favorite_count = listing.favorite_count.saturating_sub(1);
        listing.updated_at = ic_cdk::api::time();

        LISTINGS.with(|storage| {
            let mut storage = storage.borrow_mut();
            if let Some(ref mut storage) = storage.as_mut() {
                storage.insert(listing_id, listing);
            }
        });
    }

    Ok(())
}

/// 사용자의 즐겨찾기 목록 조회
pub fn get_user_favorites(user: Principal) -> Vec<ListingSummary> {
    let favorite_listing_ids: Vec<u64> = FAVORITES.with(|storage| {
        let storage = storage.borrow();
        match storage.as_ref() {
            Some(storage) => storage
                .iter()
                .filter(|(_, fav)| fav.user == user)
                .map(|(_, fav)| fav.listing_id)
                .collect(),
            None => Vec::new(),
        }
    });

    LISTINGS.with(|storage| {
        let storage = storage.borrow();
        match storage.as_ref() {
            Some(storage) => favorite_listing_ids
                .into_iter()
                .filter_map(|id| storage.get(&id))
                .filter(|listing| listing.status != ListingStatus::Deleted)
                .map(to_listing_summary)
                .collect(),
            None => Vec::new(),
        }
    })
}

/// 즐겨찾기 여부 확인
pub fn is_favorited(user: Principal, listing_id: u64) -> bool {
    FAVORITES.with(|storage| {
        let storage = storage.borrow();
        match storage.as_ref() {
            Some(storage) => storage
                .iter()
                .any(|(_, fav)| fav.user == user && fav.listing_id == listing_id),
            None => false,
        }
    })
}

// =====================
// 5) 활동 로그 관리
// =====================

/// 활동 로그 기록
pub fn log_activity(
    activity_type: ActivityType,
    user: Principal,
    listing_id: Option<u64>,
    details: String,
) {
    // 활동 ID 생성 (해시 기반)
    let activity_data = format!(
        "{}_{}_{}_{}",
        user.to_text(),
        listing_id.unwrap_or(0),
        details,
        ic_cdk::api::time()
    );
    let activity_id = generate_unique_id("activity", &activity_data);

    let activity = ActivityLog {
        timestamp: ic_cdk::api::time(),
        activity_type,
        user,
        listing_id,
        details,
    };

    ACTIVITY_LOGS.with(|storage_cell| {
        let mut storage_ref = storage_cell.borrow_mut();
        if let Some(storage) = storage_ref.as_mut() {
            storage.insert(activity_id, activity);
        }
    });
}

/// 최근 활동 로그 조회
pub fn get_recent_activities(limit: u64) -> Vec<ActivityLog> {
    ACTIVITY_LOGS.with(|storage| {
        let storage = storage.borrow();
        match storage.as_ref() {
            Some(storage) => {
                let mut activities: Vec<_> = storage.iter().map(|(_, activity)| activity).collect();
                activities.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                activities.truncate(limit as usize);
                activities
            }
            None => Vec::new(),
        }
    })
}

// =====================
// 6) 통계 정보
// =====================

/// 마켓플레이스 통계 조회
pub fn get_marketplace_stats() -> MarketplaceStats {
    LISTINGS.with(|storage| {
        let storage = storage.borrow();
        match storage.as_ref() {
            Some(storage) => {
                let mut total_listings = 0u64;
                let mut active_listings = 0u64;
                let mut sold_listings = 0u64;
                let mut total_views = 0u64;
                let mut total_favorites = 0u64;
                let mut sellers = std::collections::HashSet::new();
                let mut categories = std::collections::HashMap::new();

                for (_, listing) in storage.iter() {
                    if listing.status == ListingStatus::Deleted {
                        continue;
                    }

                    total_listings += 1;
                    total_views += listing.view_count;
                    total_favorites += listing.favorite_count;
                    sellers.insert(listing.seller);

                    match listing.status {
                        ListingStatus::Active => active_listings += 1,
                        ListingStatus::Sold => sold_listings += 1,
                        _ => {}
                    }

                    // 카테고리별 통계
                    let category_stat = categories
                        .entry(listing.category.clone())
                        .or_insert((0u64, 0u64, 0u64));
                    category_stat.0 += 1; // count
                    category_stat.1 += listing.price; // total price
                }

                let category_stats: Vec<CategoryStats> = categories
                    .into_iter()
                    .map(|(category, (count, total_price, _))| CategoryStats {
                        category,
                        count,
                        avg_price: if count > 0 { total_price / count } else { 0 },
                    })
                    .collect();

                let recent_activity = get_recent_activities(10);

                MarketplaceStats {
                    total_listings,
                    active_listings,
                    sold_listings,
                    total_sellers: sellers.len() as u64,
                    total_views,
                    total_favorites,
                    categories: category_stats,
                    recent_activity,
                }
            }
            None => MarketplaceStats {
                total_listings: 0,
                active_listings: 0,
                sold_listings: 0,
                total_sellers: 0,
                total_views: 0,
                total_favorites: 0,
                categories: Vec::new(),
                recent_activity: Vec::new(),
            },
        }
    })
}

// =====================
// 7) 헬퍼 함수들
// =====================

/// Listing을 ListingSummary로 변환
fn to_listing_summary(listing: Listing) -> ListingSummary {
    // 설명을 200자로 제한
    let description = if listing.description.len() > 200 {
        format!("{}...", &listing.description[..197])
    } else {
        listing.description
    };

    ListingSummary {
        id: listing.id,
        seller: listing.seller,
        title: listing.title,
        description,
        price: listing.price,
        currency: listing.currency,
        category: listing.category,
        tags: listing.tags,
        status: listing.status,
        created_at: listing.created_at,
        updated_at: listing.updated_at,
        view_count: listing.view_count,
        favorite_count: listing.favorite_count,
        data_count: listing.data_ids.len() as u64,
    }
}

/// 판매글 상세 정보 가져오기 (데이터 정보 포함)
pub fn get_listing_detail(listing_id: u64) -> Option<ListingDetail> {
    let listing = get_listing(listing_id)?;

    // 데이터 정보 가져오기 (storage.rs의 새로운 공개 함수 사용)
    let data_info: Vec<crate::types::DataInfo> =
        crate::storage::get_multiple_data_info(&listing.data_ids);

    Some(ListingDetail { listing, data_info })
}

/// 카테고리별 판매글 수 조회
pub fn get_listings_count_by_category() -> Vec<(String, u64)> {
    LISTINGS.with(|storage| {
        let storage = storage.borrow();
        match storage.as_ref() {
            Some(storage) => {
                let mut categories = std::collections::HashMap::new();

                for (_, listing) in storage.iter() {
                    if listing.status == ListingStatus::Active {
                        *categories.entry(listing.category.clone()).or_insert(0) += 1;
                    }
                }

                let mut result: Vec<_> = categories.into_iter().collect();
                result.sort_by(|a, b| b.1.cmp(&a.1)); // 개수별 내림차순 정렬
                result
            }
            None => Vec::new(),
        }
    })
}

/// 인기 태그 조회
pub fn get_popular_tags(limit: u64) -> Vec<(String, u64)> {
    LISTINGS.with(|storage| {
        let storage = storage.borrow();
        match storage.as_ref() {
            Some(storage) => {
                let mut tags = std::collections::HashMap::new();

                for (_, listing) in storage.iter() {
                    if listing.status == ListingStatus::Active {
                        for tag in &listing.tags {
                            *tags.entry(tag.clone()).or_insert(0) += 1;
                        }
                    }
                }

                let mut result: Vec<_> = tags.into_iter().collect();
                result.sort_by(|a, b| b.1.cmp(&a.1)); // 사용횟수별 내림차순 정렬
                result.truncate(limit as usize);
                result
            }
            None => Vec::new(),
        }
    })
}
