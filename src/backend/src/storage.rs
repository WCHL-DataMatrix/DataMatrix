// backend/src/storage.rs

use crate::types::*;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl, StableBTreeMap, StableCell,
};
use serde_cbor::value::Value as CborValue;
use sha2::{Digest, Sha256};
use std::cell::RefCell;

// 메모리 관리
type Memory = VirtualMemory<DefaultMemoryImpl>;

// BACKEND 전용 메모리 관리자 (wallet과 완전 분리)
thread_local! {
    static BACKEND_MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

// Backend 전용 메모리 ID들 (0-9는 backend용)
fn get_uploaded_data_memory() -> Memory {
    BACKEND_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
}

fn get_mint_requests_memory() -> Memory {
    BACKEND_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
}

fn get_mint_status_memory() -> Memory {
    BACKEND_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
}

fn get_upload_counter_memory() -> Memory {
    BACKEND_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
}

fn get_mint_counter_memory() -> Memory {
    BACKEND_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4)))
}

fn get_data_hashes_memory() -> Memory {
    BACKEND_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5)))
}

fn get_minted_hashes_memory() -> Memory {
    BACKEND_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(6)))
}

// 데이터 해시를 위한 타입
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DataHash([u8; 32]);

impl ic_stable_structures::Storable for DataHash {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Borrowed(&self.0)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&bytes);
        DataHash(hash)
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 32,
            is_fixed_size: true,
        };
}

// 저장소들 - RefCell<StableBTreeMap> 직접 초기화 방식 사용
thread_local! {
    static UPLOADED_DATA: RefCell<StableBTreeMap<u64, DataBlob, Memory>> = RefCell::new(
        StableBTreeMap::init(
            BACKEND_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
        )
    );

    static MINT_REQUESTS: RefCell<StableBTreeMap<u64, MintRequestData, Memory>> = RefCell::new(
        StableBTreeMap::init(
            BACKEND_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
        )
    );

    static MINT_STATUS_MAP: RefCell<StableBTreeMap<u64, MintStatus, Memory>> = RefCell::new(
        StableBTreeMap::init(
            BACKEND_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
        )
    );

    static DATA_HASHES: RefCell<StableBTreeMap<DataHash, u64, Memory>> = RefCell::new(
        StableBTreeMap::init(
            BACKEND_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5)))
        )
    );

    static MINTED_HASHES: RefCell<StableBTreeMap<DataHash, u64, Memory>> = RefCell::new(
        StableBTreeMap::init(
            BACKEND_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(6)))
        )
    );
}

// 카운터들은 별도로 초기화
thread_local! {
    static UPLOAD_COUNTER: RefCell<Option<StableCell<u64, Memory>>> = const { RefCell::new(None) };
    static MINT_COUNTER: RefCell<Option<StableCell<u64, Memory>>> = const { RefCell::new(None) };
}

// =====================
// 1) 저장소 초기화
// =====================

pub fn init_storage() {
    ic_cdk::println!("Initializing backend storage...");

    // 업로드 카운터 초기화
    UPLOAD_COUNTER.with(|counter| {
        let mut counter_ref = counter.borrow_mut();
        if counter_ref.is_none() {
            // 매번 새로운 메모리 인스턴스 생성
            match StableCell::init(get_upload_counter_memory(), 0) {
                Ok(cell) => {
                    *counter_ref = Some(cell);
                    ic_cdk::println!("Upload counter initialized successfully");
                }
                Err(_) => {
                    // 이미 존재하는 경우 기존 값 로드
                    match StableCell::new(get_upload_counter_memory(), 0) {
                        Ok(cell) => {
                            *counter_ref = Some(cell);
                            ic_cdk::println!("Upload counter loaded from existing memory");
                        }
                        Err(e) => {
                            ic_cdk::println!("Failed to initialize upload counter: {:?}", e);
                        }
                    }
                }
            }
        }
    });

    // 민팅 카운터 초기화 - 컴파일 에러 수정
    MINT_COUNTER.with(|counter| {
        let mut counter_ref = counter.borrow_mut();
        if counter_ref.is_none() {
            // 매번 새로운 메모리 인스턴스 생성하여 소유권 문제 해결
            match StableCell::init(get_mint_counter_memory(), 0) {
                Ok(cell) => {
                    *counter_ref = Some(cell);
                    ic_cdk::println!("Mint counter initialized successfully with init()");
                }
                Err(_) => {
                    // init 실패시 new로 기존 값 로드 시도
                    match StableCell::new(get_mint_counter_memory(), 0) {
                        Ok(cell) => {
                            *counter_ref = Some(cell);
                            ic_cdk::println!("Mint counter loaded from existing memory with new()");
                        }
                        Err(e) => {
                            ic_cdk::println!("Mint counter both init and new failed: {:?}", e);

                            // 최후의 수단: 다른 메모리 ID로 시도
                            match StableCell::init(
                                BACKEND_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(7))),
                                0,
                            ) {
                                Ok(cell) => {
                                    *counter_ref = Some(cell);
                                    ic_cdk::println!(
                                        "Mint counter initialized with alternative memory ID 7"
                                    );
                                }
                                Err(e2) => {
                                    ic_cdk::println!(
                                        "All mint counter initialization attempts failed: {:?}",
                                        e2
                                    );
                                    // 패닉 대신 경고 로그만 출력
                                }
                            }
                        }
                    }
                }
            }
        } else {
            ic_cdk::println!("Mint counter already initialized");
        }
    });

    ic_cdk::println!("Backend storage initialization completed");
}

// =====================
// 2) 안전한 카운터 접근 함수들 - 컴파일 에러 수정
// =====================

/// 업로드 카운터에 안전하게 접근
fn with_upload_counter<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce(&mut StableCell<u64, Memory>) -> Result<R, String>,
{
    UPLOAD_COUNTER.with(|counter_cell| {
        let mut counter_ref = counter_cell.borrow_mut();

        if counter_ref.is_none() {
            ic_cdk::println!("Upload counter not initialized, attempting initialization...");

            match StableCell::init(get_upload_counter_memory(), 0) {
                Ok(cell) => *counter_ref = Some(cell),
                Err(_) => match StableCell::new(get_upload_counter_memory(), 0) {
                    Ok(cell) => *counter_ref = Some(cell),
                    Err(e) => return Err(format!("Failed to initialize upload counter: {:?}", e)),
                },
            }
        }

        match counter_ref.as_mut() {
            Some(counter) => f(counter),
            None => Err("Upload counter still not available".to_string()),
        }
    })
}

/// 민팅 카운터에 안전하게 접근
fn with_mint_counter<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce(&mut StableCell<u64, Memory>) -> Result<R, String>,
{
    MINT_COUNTER.with(|counter_cell| {
        let mut counter_ref = counter_cell.borrow_mut();

        if counter_ref.is_none() {
            ic_cdk::println!("Mint counter not initialized, attempting re-initialization...");

            // 여러 방법으로 시도
            let mut initialized = false;

            // 방법 1: init
            if let Ok(cell) = StableCell::init(get_mint_counter_memory(), 0) {
                *counter_ref = Some(cell);
                initialized = true;
                ic_cdk::println!("Mint counter re-initialized with init()");
            }
            // 방법 2: new
            else if let Ok(cell) = StableCell::new(get_mint_counter_memory(), 0) {
                *counter_ref = Some(cell);
                initialized = true;
                ic_cdk::println!("Mint counter re-initialized with new()");
            }
            // 방법 3: 대체 메모리
            else if let Ok(cell) = StableCell::init(
                BACKEND_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(7))),
                0,
            ) {
                *counter_ref = Some(cell);
                initialized = true;
                ic_cdk::println!("Mint counter initialized with alternative memory");
            }

            if !initialized {
                return Err("All mint counter initialization methods failed".to_string());
            }
        }

        match counter_ref.as_mut() {
            Some(counter) => f(counter),
            None => {
                Err("Mint counter still not available after initialization attempt".to_string())
            }
        }
    })
}

// =====================
// 3) 해시 관련 함수
// =====================

/// 데이터의 해시 계산
fn calculate_data_hash(data: &[u8]) -> DataHash {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    DataHash(hash)
}

/// 업로드된 데이터와 중복 검사
pub fn check_data_exists(data: &[u8]) -> Option<u64> {
    let hash = calculate_data_hash(data);
    DATA_HASHES.with(|hashes| hashes.borrow().get(&hash))
}

/// 이미 민팅된 데이터인지 검사
pub fn check_data_minted(data: &[u8]) -> bool {
    let hash = calculate_data_hash(data);
    MINTED_HASHES.with(|hashes| hashes.borrow().contains_key(&hash))
}

/// 여러 데이터의 중복 검사 (업로드 및 민팅 여부)
pub fn check_multiple_data_status(data_list: &[Vec<u8>]) -> Vec<(Option<u64>, bool)> {
    data_list
        .iter()
        .map(|data| {
            let exists = check_data_exists(data);
            let minted = check_data_minted(data);
            (exists, minted)
        })
        .collect()
}

// =====================
// 4) 업로드 데이터 관리
// =====================

/// 업로드 데이터 저장 (중복 검사 포함)
pub fn store_upload_data(parsed_data: Vec<CborValue>, mime_type: &str) -> Result<Vec<u64>, String> {
    let mut data_ids = Vec::new();
    let timestamp = ic_cdk::api::time();

    for value in parsed_data {
        let bytes = serde_cbor::to_vec(&value).map_err(|e| format!("CBOR 직렬화 실패: {}", e))?;

        // 중복 검사
        let hash = calculate_data_hash(&bytes);

        // 이미 존재하는 데이터인지 확인
        if let Some(existing_id) = DATA_HASHES.with(|hashes| hashes.borrow().get(&hash)) {
            // 이미 민팅되었는지 확인
            if MINTED_HASHES.with(|hashes| hashes.borrow().contains_key(&hash)) {
                return Err(format!(
                    "데이터가 이미 민팅되었습니다. (데이터 ID: {})",
                    existing_id
                ));
            }
            // 민팅되지 않았다면 기존 ID 반환
            data_ids.push(existing_id);
            continue;
        }

        // 데이터 ID 생성
        let data_id = with_upload_counter(|counter| {
            let current = *counter.get();
            let next_id = current + 1;
            counter
                .set(next_id)
                .map_err(|e| format!("카운터 업데이트 실패: {:?}", e))?;
            Ok(next_id)
        })?;

        // 데이터 저장
        let data_blob = DataBlob {
            data: bytes,
            mime_type: mime_type.to_string(),
            timestamp,
        };

        UPLOADED_DATA.with(|storage| {
            storage.borrow_mut().insert(data_id, data_blob);
        });

        // 해시 저장
        DATA_HASHES.with(|hashes| {
            hashes.borrow_mut().insert(hash, data_id);
        });

        data_ids.push(data_id);
    }

    Ok(data_ids)
}

/// 업로드 데이터 조회
pub fn get_uploaded_data(data_id: u64) -> Option<Vec<u8>> {
    UPLOADED_DATA.with(|storage| storage.borrow().get(&data_id).map(|blob| blob.data))
}

/// 업로드 데이터 목록 조회
pub fn list_uploaded_data() -> Vec<DataInfo> {
    UPLOADED_DATA.with(|storage| {
        storage
            .borrow()
            .iter()
            .map(|(id, blob)| DataInfo {
                id,
                mime_type: blob.mime_type.clone(),
                timestamp: blob.timestamp,
                size: blob.data.len() as u64,
            })
            .collect()
    })
}

/// 업로드 데이터 삭제
pub fn delete_uploaded_data(data_id: u64) -> Result<String, String> {
    // 먼저 데이터를 가져와서 해시 계산
    let data_hash = UPLOADED_DATA.with(|storage| {
        storage
            .borrow()
            .get(&data_id)
            .map(|blob| calculate_data_hash(&blob.data))
    });

    // 민팅된 데이터인지 확인
    if let Some(hash) = &data_hash {
        if MINTED_HASHES.with(|hashes| hashes.borrow().contains_key(hash)) {
            return Err("민팅된 데이터는 삭제할 수 없습니다".to_string());
        }
    }

    // 데이터 삭제
    UPLOADED_DATA.with(|storage| {
        match storage.borrow_mut().remove(&data_id) {
            Some(_) => {
                // 해시 매핑도 삭제
                if let Some(hash) = data_hash {
                    DATA_HASHES.with(|hashes| {
                        hashes.borrow_mut().remove(&hash);
                    });
                }
                Ok(format!("데이터 ID {} 삭제 완료", data_id))
            }
            None => Err(format!("데이터 ID {}를 찾을 수 없습니다", data_id)),
        }
    })
}

// =====================
// 5) 민팅 요청 관리
// =====================

/// 민팅 요청 저장
pub fn store_mint_request(request: MintRequest) -> u64 {
    let request_id = with_mint_counter(|counter| {
        let current = *counter.get();
        let next_id = current + 1;
        counter
            .set(next_id)
            .map_err(|e| format!("Failed to increment counter: {:?}", e))?;
        Ok(next_id)
    })
    .unwrap_or_else(|e| {
        ic_cdk::println!("Error accessing mint counter: {}", e);
        // 긴급 대안: 현재 시간을 기반으로 ID 생성
        let timestamp = ic_cdk::api::time();
        timestamp % 1_000_000
    });

    let request_data = MintRequestData {
        request,
        timestamp: ic_cdk::api::time(),
    };

    // 민팅 요청 저장
    MINT_REQUESTS.with(|requests| {
        requests.borrow_mut().insert(request_id, request_data);
    });

    // 초기 상태 설정
    MINT_STATUS_MAP.with(|status_map| {
        status_map
            .borrow_mut()
            .insert(request_id, MintStatus::Pending);
    });

    request_id
}

/// 민팅 요청 조회
pub fn get_mint_request(request_id: u64) -> Option<MintRequest> {
    MINT_REQUESTS.with(|requests| {
        requests
            .borrow()
            .get(&request_id)
            .map(|data| data.request.clone())
    })
}

/// 민팅 상태 조회
pub fn get_mint_status(request_id: u64) -> Option<MintStatus> {
    MINT_STATUS_MAP.with(|status_map| status_map.borrow().get(&request_id))
}

/// 민팅 상태 업데이트 (민팅 완료 시 해시 기록)
pub fn update_mint_status(request_id: u64, new_status: MintStatus) -> Result<(), String> {
    // 민팅이 완료된 경우
    if let MintStatus::Completed(_) = &new_status {
        // 해당 요청의 메타데이터를 가져와서 민팅된 것으로 표시
        if let Some(request) = get_mint_request(request_id) {
            for metadata in &request.metadata {
                let hash = calculate_data_hash(metadata);
                MINTED_HASHES.with(|hashes| {
                    hashes.borrow_mut().insert(hash, request_id);
                });
            }
        }
    }

    MINT_STATUS_MAP.with(|status_map| {
        status_map.borrow_mut().insert(request_id, new_status);
        Ok(())
    })
}

/// 민팅 요청 목록 조회
pub fn list_mint_requests() -> Vec<MintRequestInfo> {
    MINT_REQUESTS.with(|requests| {
        MINT_STATUS_MAP.with(|status_map| {
            let requests_ref = requests.borrow();
            let status_map_ref = status_map.borrow();

            requests_ref
                .iter()
                .map(|(id, data)| {
                    let status = status_map_ref.get(&id).unwrap_or(MintStatus::Pending);
                    MintRequestInfo {
                        request_id: id,
                        owner: data.request.owner,
                        cid: data.request.cid.clone(),
                        status,
                        timestamp: data.timestamp,
                    }
                })
                .collect()
        })
    })
}

/// 다음 처리할 민팅 요청 조회
pub fn get_next_pending_mint() -> Option<(u64, MintRequest)> {
    MINT_REQUESTS.with(|requests| {
        MINT_STATUS_MAP.with(|status_map| {
            let requests_ref = requests.borrow();
            let status_map_ref = status_map.borrow();

            for (request_id, data) in requests_ref.iter() {
                if let Some(status) = status_map_ref.get(&request_id) {
                    if matches!(status, MintStatus::Pending) {
                        return Some((request_id, data.request.clone()));
                    }
                }
            }
            None
        })
    })
}

// =====================
// 6) 통계 정보
// =====================

/// 저장소 통계 조회
pub fn get_storage_stats() -> StorageStats {
    let total_uploads: u64 =
        UPLOAD_COUNTER.with(|counter| counter.borrow().as_ref().map_or(0, |c| *c.get()));

    let total_mint_requests: u64 =
        MINT_COUNTER.with(|counter| counter.borrow().as_ref().map_or(0, |c| *c.get()));

    let (pending_mints, completed_mints, failed_mints): (u64, u64, u64) =
        MINT_STATUS_MAP.with(|status_map| {
            let mut pending = 0u64;
            let mut completed = 0u64;
            let mut failed = 0u64;

            for (_, status) in status_map.borrow().iter() {
                match status {
                    MintStatus::Pending | MintStatus::InProgress => pending += 1,
                    MintStatus::Completed(_) => completed += 1,
                    MintStatus::Failed(_) => failed += 1,
                }
            }

            (pending, completed, failed)
        });

    let storage_size: u64 = UPLOADED_DATA.with(|storage| {
        storage
            .borrow()
            .iter()
            .map(|(_, blob)| blob.data.len() as u64)
            .sum::<u64>()
    });

    StorageStats {
        total_uploads,
        total_mint_requests,
        pending_mints,
        completed_mints,
        failed_mints,
        storage_size,
    }
}

// =====================
// 7) 추가 헬퍼 함수들
// =====================

/// 특정 데이터 ID의 정보 조회
pub fn get_uploaded_data_info(data_id: u64) -> Option<crate::types::DataInfo> {
    UPLOADED_DATA.with(|storage| {
        storage
            .borrow()
            .get(&data_id)
            .map(|blob| crate::types::DataInfo {
                id: data_id,
                mime_type: blob.mime_type.clone(),
                timestamp: blob.timestamp,
                size: blob.data.len() as u64,
            })
    })
}

/// 여러 데이터 ID의 정보를 한번에 조회
pub fn get_multiple_data_info(data_ids: &[u64]) -> Vec<crate::types::DataInfo> {
    data_ids
        .iter()
        .filter_map(|&data_id| get_uploaded_data_info(data_id))
        .collect()
}

/// 데이터 ID가 존재하는지 확인
pub fn data_id_exists(data_id: u64) -> bool {
    UPLOADED_DATA.with(|storage| storage.borrow().contains_key(&data_id))
}

/// 여러 데이터 ID가 모두 존재하는지 확인
pub fn validate_data_ids_exist(data_ids: &[u64]) -> Result<(), String> {
    for &data_id in data_ids {
        if !data_id_exists(data_id) {
            return Err(format!("데이터 ID {}를 찾을 수 없습니다", data_id));
        }
    }
    Ok(())
}
