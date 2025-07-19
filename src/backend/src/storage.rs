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

// 전역 메모리 관리자
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

// 저장소 초기화를 위한 헬퍼 함수들
fn get_uploaded_data_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
}

fn get_mint_requests_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
}

fn get_mint_status_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
}

fn get_upload_counter_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
}

fn get_mint_counter_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4)))
}

fn get_data_hashes_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5)))
}

fn get_minted_hashes_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(6)))
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

// 저장소
thread_local! {
    static UPLOADED_DATA: RefCell<Option<StableBTreeMap<u64, DataBlob, Memory>>> = const { RefCell::new(None) };
    static MINT_REQUESTS: RefCell<Option<StableBTreeMap<u64, MintRequestData, Memory>>> = const { RefCell::new(None) };
    static MINT_STATUS_MAP: RefCell<Option<StableBTreeMap<u64, MintStatus, Memory>>> = const { RefCell::new(None) };
    static UPLOAD_COUNTER: RefCell<Option<StableCell<u64, Memory>>> = const { RefCell::new(None) };
    static MINT_COUNTER: RefCell<Option<StableCell<u64, Memory>>> = const { RefCell::new(None) };

    // 새로 추가: 데이터 해시 -> 데이터 ID 매핑
    static DATA_HASHES: RefCell<Option<StableBTreeMap<DataHash, u64, Memory>>> = const { RefCell::new(None) };
    // 새로 추가: 민팅된 데이터 해시 집합
    static MINTED_HASHES: RefCell<Option<StableBTreeMap<DataHash, u64, Memory>>> = const { RefCell::new(None) };
}

// =====================
// 1) 저장소 초기화
// =====================

pub fn init_storage() {
    // 업로드 데이터 저장소 초기화
    UPLOADED_DATA.with(|storage| {
        let mut storage = storage.borrow_mut();
        if storage.is_none() {
            *storage = Some(StableBTreeMap::init(get_uploaded_data_memory()));
        }
    });

    // 민팅 요청 저장소 초기화
    MINT_REQUESTS.with(|storage| {
        let mut storage = storage.borrow_mut();
        if storage.is_none() {
            *storage = Some(StableBTreeMap::init(get_mint_requests_memory()));
        }
    });

    // 민팅 상태 저장소 초기화
    MINT_STATUS_MAP.with(|storage| {
        let mut storage = storage.borrow_mut();
        if storage.is_none() {
            *storage = Some(StableBTreeMap::init(get_mint_status_memory()));
        }
    });

    // 업로드 카운터 초기화
    UPLOAD_COUNTER.with(|counter| {
        let mut counter = counter.borrow_mut();
        if counter.is_none() {
            *counter = Some(
                StableCell::init(get_upload_counter_memory(), 0)
                    .expect("Failed to initialize upload counter"),
            );
        }
    });

    // 민팅 카운터 초기화
    MINT_COUNTER.with(|counter| {
        let mut counter = counter.borrow_mut();
        if counter.is_none() {
            *counter = Some(
                StableCell::init(get_mint_counter_memory(), 0)
                    .expect("Failed to initialize mint counter"),
            );
        }
    });

    // 데이터 해시 저장소 초기화
    DATA_HASHES.with(|hashes| {
        let mut hashes = hashes.borrow_mut();
        if hashes.is_none() {
            *hashes = Some(StableBTreeMap::init(get_data_hashes_memory()));
        }
    });

    // 민팅된 해시 저장소 초기화
    MINTED_HASHES.with(|hashes| {
        let mut hashes = hashes.borrow_mut();
        if hashes.is_none() {
            *hashes = Some(StableBTreeMap::init(get_minted_hashes_memory()));
        }
    });

    ic_cdk::println!("Storage initialized");
}

// =====================
// 2) 해시 관련 함수
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

    DATA_HASHES.with(|hashes| {
        let hashes = hashes.borrow();
        hashes.as_ref()?.get(&hash)
    })
}

/// 이미 민팅된 데이터인지 검사
pub fn check_data_minted(data: &[u8]) -> bool {
    let hash = calculate_data_hash(data);

    MINTED_HASHES.with(|hashes| {
        let hashes = hashes.borrow();
        hashes.as_ref().map_or(false, |h| h.contains_key(&hash))
    })
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
// 3) 업로드 데이터 관리
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
        if let Some(existing_id) = DATA_HASHES.with(|hashes| {
            let hashes = hashes.borrow();
            hashes.as_ref().and_then(|h| h.get(&hash))
        }) {
            // 이미 민팅되었는지 확인
            if MINTED_HASHES.with(|hashes| {
                let hashes = hashes.borrow();
                hashes.as_ref().map_or(false, |h| h.contains_key(&hash))
            }) {
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
        let data_id = UPLOAD_COUNTER.with(|counter| {
            let mut counter = counter.borrow_mut();
            if let Some(ref mut counter) = counter.as_mut() {
                let current = counter.get();
                let next_id = current + 1;
                counter
                    .set(next_id)
                    .map_err(|e| format!("카운터 업데이트 실패: {:?}", e))?;
                Ok(next_id)
            } else {
                Err("Upload counter not initialized".to_string())
            }
        })?;

        // 데이터 저장
        let data_blob = DataBlob {
            data: bytes,
            mime_type: mime_type.to_string(),
            timestamp,
        };

        UPLOADED_DATA.with(|storage| {
            let mut storage = storage.borrow_mut();
            if let Some(ref mut storage) = storage.as_mut() {
                storage.insert(data_id, data_blob);
                Ok(())
            } else {
                Err("Upload data storage not initialized".to_string())
            }
        })?;

        // 해시 저장
        DATA_HASHES.with(|hashes| {
            let mut hashes = hashes.borrow_mut();
            if let Some(ref mut hashes) = hashes.as_mut() {
                hashes.insert(hash, data_id);
                Ok(())
            } else {
                Err("Data hashes storage not initialized".to_string())
            }
        })?;

        data_ids.push(data_id);
    }

    Ok(data_ids)
}

/// 업로드 데이터 조회
pub fn get_uploaded_data(data_id: u64) -> Option<Vec<u8>> {
    UPLOADED_DATA.with(|storage| {
        let storage = storage.borrow();
        storage.as_ref()?.get(&data_id).map(|blob| blob.data)
    })
}

/// 업로드 데이터 목록 조회
pub fn list_uploaded_data() -> Vec<DataInfo> {
    UPLOADED_DATA.with(|storage| {
        let storage = storage.borrow();
        match storage.as_ref() {
            Some(storage) => storage
                .iter()
                .map(|(id, blob)| DataInfo {
                    id,
                    mime_type: blob.mime_type.clone(),
                    timestamp: blob.timestamp,
                    size: blob.data.len() as u64,
                })
                .collect(),
            None => Vec::new(),
        }
    })
}

/// 업로드 데이터 삭제
pub fn delete_uploaded_data(data_id: u64) -> Result<String, String> {
    // 먼저 데이터를 가져와서 해시 계산
    let data_hash = UPLOADED_DATA.with(|storage| {
        let storage = storage.borrow();
        storage
            .as_ref()
            .and_then(|s| s.get(&data_id))
            .map(|blob| calculate_data_hash(&blob.data))
    });

    // 민팅된 데이터인지 확인
    if let Some(hash) = &data_hash {
        if MINTED_HASHES.with(|hashes| {
            let hashes = hashes.borrow();
            hashes.as_ref().map_or(false, |h| h.contains_key(hash))
        }) {
            return Err("민팅된 데이터는 삭제할 수 없습니다".to_string());
        }
    }

    // 데이터 삭제
    UPLOADED_DATA.with(|storage| {
        let mut storage = storage.borrow_mut();
        match storage.as_mut() {
            Some(storage) => match storage.remove(&data_id) {
                Some(_) => {
                    // 해시 매핑도 삭제
                    if let Some(hash) = data_hash {
                        DATA_HASHES.with(|hashes| {
                            let mut hashes = hashes.borrow_mut();
                            if let Some(ref mut hashes) = hashes.as_mut() {
                                hashes.remove(&hash);
                            }
                        });
                    }
                    Ok(format!("데이터 ID {} 삭제 완료", data_id))
                }
                None => Err(format!("데이터 ID {}를 찾을 수 없습니다", data_id)),
            },
            None => Err("Storage not initialized".to_string()),
        }
    })
}

// =====================
// 4) 민팅 요청 관리
// =====================

/// 민팅 요청 저장
pub fn store_mint_request(request: MintRequest) -> u64 {
    let request_id = MINT_COUNTER.with(|counter| {
        let mut counter = counter.borrow_mut();
        if let Some(ref mut counter) = counter.as_mut() {
            let current = counter.get();
            let next_id = current + 1;
            counter.set(next_id).expect("Failed to increment counter");
            next_id
        } else {
            panic!("Mint counter not initialized");
        }
    });

    let request_data = MintRequestData {
        request,
        timestamp: ic_cdk::api::time(),
    };

    // 민팅 요청 저장
    MINT_REQUESTS.with(|requests| {
        let mut requests = requests.borrow_mut();
        if let Some(ref mut requests) = requests.as_mut() {
            requests.insert(request_id, request_data);
        }
    });

    // 초기 상태 설정
    MINT_STATUS_MAP.with(|status_map| {
        let mut status_map = status_map.borrow_mut();
        if let Some(ref mut status_map) = status_map.as_mut() {
            status_map.insert(request_id, MintStatus::Pending);
        }
    });

    request_id
}

/// 민팅 요청 조회
pub fn get_mint_request(request_id: u64) -> Option<MintRequest> {
    MINT_REQUESTS.with(|requests| {
        let requests = requests.borrow();
        requests
            .as_ref()?
            .get(&request_id)
            .map(|data| data.request.clone())
    })
}

/// 민팅 상태 조회
pub fn get_mint_status(request_id: u64) -> Option<MintStatus> {
    MINT_STATUS_MAP.with(|status_map| {
        let status_map = status_map.borrow();
        status_map.as_ref()?.get(&request_id)
    })
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
                    let mut hashes = hashes.borrow_mut();
                    if let Some(ref mut hashes) = hashes.as_mut() {
                        hashes.insert(hash, request_id);
                    }
                });
            }
        }
    }

    MINT_STATUS_MAP.with(|status_map| {
        let mut status_map = status_map.borrow_mut();
        match status_map.as_mut() {
            Some(status_map) => {
                status_map.insert(request_id, new_status);
                Ok(())
            }
            None => Err("Status map not initialized".to_string()),
        }
    })
}

/// 민팅 요청 목록 조회
pub fn list_mint_requests() -> Vec<MintRequestInfo> {
    MINT_REQUESTS.with(|requests| {
        MINT_STATUS_MAP.with(|status_map| {
            let requests = requests.borrow();
            let status_map = status_map.borrow();

            match (requests.as_ref(), status_map.as_ref()) {
                (Some(requests), Some(status_map)) => requests
                    .iter()
                    .map(|(id, data)| {
                        let status = status_map.get(&id).unwrap_or(MintStatus::Pending);
                        MintRequestInfo {
                            request_id: id,
                            owner: data.request.owner,
                            cid: data.request.cid.clone(),
                            status,
                            timestamp: data.timestamp,
                        }
                    })
                    .collect(),
                _ => Vec::new(),
            }
        })
    })
}

/// 다음 처리할 민팅 요청 조회
pub fn get_next_pending_mint() -> Option<(u64, MintRequest)> {
    MINT_REQUESTS.with(|requests| {
        MINT_STATUS_MAP.with(|status_map| {
            let requests = requests.borrow();
            let status_map = status_map.borrow();

            match (requests.as_ref(), status_map.as_ref()) {
                (Some(requests), Some(status_map)) => {
                    for (request_id, data) in requests.iter() {
                        if let Some(status) = status_map.get(&request_id) {
                            if matches!(status, MintStatus::Pending) {
                                return Some((request_id, data.request.clone()));
                            }
                        }
                    }
                    None
                }
                _ => None,
            }
        })
    })
}

// =====================
// 5) 통계 정보
// =====================

/// 저장소 통계 조회
pub fn get_storage_stats() -> StorageStats {
    let total_uploads: u64 =
        UPLOAD_COUNTER.with(|counter| counter.borrow().as_ref().map_or(0, |c| *c.get()));

    let total_mint_requests: u64 =
        MINT_COUNTER.with(|counter| counter.borrow().as_ref().map_or(0, |c| *c.get()));

    let (pending_mints, completed_mints, failed_mints): (u64, u64, u64) =
        MINT_STATUS_MAP.with(|status_map| match status_map.borrow().as_ref() {
            Some(status_map) => {
                let mut pending = 0u64;
                let mut completed = 0u64;
                let mut failed = 0u64;

                for (_, status) in status_map.iter() {
                    match status {
                        MintStatus::Pending | MintStatus::InProgress => pending += 1,
                        MintStatus::Completed(_) => completed += 1,
                        MintStatus::Failed(_) => failed += 1,
                    }
                }

                (pending, completed, failed)
            }
            None => (0u64, 0u64, 0u64),
        });

    let storage_size: u64 = UPLOADED_DATA.with(|storage| {
        storage.borrow().as_ref().map_or(0, |s| {
            s.iter()
                .map(|(_, blob)| blob.data.len() as u64)
                .sum::<u64>()
        })
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
