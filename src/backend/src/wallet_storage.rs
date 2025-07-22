// src/backend/src/wallet_storage.rs

use crate::wallet_types::*;
use candid::Principal;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl, StableBTreeMap, StableCell,
};
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::collections::HashMap;

type Memory = VirtualMemory<DefaultMemoryImpl>;

// 지갑 전용 메모리 관리자
thread_local! {
    static WALLET_MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

// 메모리 헬퍼 함수들
fn get_wallets_memory() -> Memory {
    WALLET_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(10)))
}

fn get_transactions_memory() -> Memory {
    WALLET_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(11)))
}

fn get_trade_offers_memory() -> Memory {
    WALLET_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(12)))
}

fn get_wallet_counter_memory() -> Memory {
    WALLET_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(13)))
}

fn get_transaction_counter_memory() -> Memory {
    WALLET_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(14)))
}

fn get_offer_counter_memory() -> Memory {
    WALLET_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(15)))
}

fn get_username_index_memory() -> Memory {
    WALLET_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(16)))
}

// 저장소들
thread_local! {
    static WALLETS: RefCell<Option<StableBTreeMap<Principal, Wallet, Memory>>> = const { RefCell::new(None) };
    static WALLET_TRANSACTIONS: RefCell<Option<StableBTreeMap<u64, WalletTransaction, Memory>>> = const { RefCell::new(None) };
    static TRADE_OFFERS: RefCell<Option<StableBTreeMap<u64, TradeOffer, Memory>>> = const { RefCell::new(None) };
    static TRANSACTION_COUNTER: RefCell<Option<StableCell<u64, Memory>>> = const { RefCell::new(None) };
    static OFFER_COUNTER: RefCell<Option<StableCell<u64, Memory>>> = const { RefCell::new(None) };
    static USERNAME_INDEX: RefCell<Option<StableBTreeMap<String, Principal, Memory>>> = const { RefCell::new(None) };
}

// =====================
// 1) 저장소 초기화
// =====================

pub fn init_wallet_storage() {
    // 지갑 저장소 초기화
    WALLETS.with(|storage| {
        let mut storage = storage.borrow_mut();
        if storage.is_none() {
            *storage = Some(StableBTreeMap::init(get_wallets_memory()));
        }
    });

    // 거래 기록 저장소 초기화
    WALLET_TRANSACTIONS.with(|storage| {
        let mut storage = storage.borrow_mut();
        if storage.is_none() {
            *storage = Some(StableBTreeMap::init(get_transactions_memory()));
        }
    });

    // 거래 제안 저장소 초기화
    TRADE_OFFERS.with(|storage| {
        let mut storage = storage.borrow_mut();
        if storage.is_none() {
            *storage = Some(StableBTreeMap::init(get_trade_offers_memory()));
        }
    });

    // 거래 카운터 초기화
    TRANSACTION_COUNTER.with(|counter| {
        let mut counter = counter.borrow_mut();
        if counter.is_none() {
            *counter = Some(
                StableCell::init(get_transaction_counter_memory(), 0)
                    .expect("Failed to initialize transaction counter"),
            );
        }
    });

    // 제안 카운터 초기화
    OFFER_COUNTER.with(|counter| {
        let mut counter = counter.borrow_mut();
        if counter.is_none() {
            *counter = Some(
                StableCell::init(get_offer_counter_memory(), 0)
                    .expect("Failed to initialize offer counter"),
            );
        }
    });

    // 사용자명 인덱스 초기화
    USERNAME_INDEX.with(|index| {
        let mut index = index.borrow_mut();
        if index.is_none() {
            *index = Some(StableBTreeMap::init(get_username_index_memory()));
        }
    });

    ic_cdk::println!("Wallet storage initialized");
}

// =====================
// 2) 지갑 관리
// =====================

/// 지갑 생성
pub fn create_wallet(
    owner: Principal,
    request: CreateWalletRequest,
) -> Result<Wallet, String> {
    // 이미 지갑이 있는지 확인
    if wallet_exists(owner) {
        return Err("이미 지갑이 존재합니다".to_string());
    }

    // 사용자명 중복 확인
    if let Some(ref username) = request.username {
        if is_username_taken(username) {
            return Err("이미 사용 중인 사용자명입니다".to_string());
        }
    }

    let now = ic_cdk::api::time();
    let profile = UserProfile {
        username: request.username.clone(),
        display_name: request.display_name,
        avatar_url: request.avatar_url,
        bio: request.bio,
        email: request.email,
        verified: false,
        join_date: now,
    };

    let wallet = Wallet {
        owner,
        balance: 0, // 초기 잔액 0
        created_at: now,
        updated_at: now,
        profile,
    };

    // 지갑 저장
    WALLETS.with(|storage_cell| {
        let mut storage_ref = storage_cell.borrow_mut();
        if let Some(storage) = storage_ref.as_mut() {
            storage.insert(owner, wallet.clone());
            Ok(())
        } else {
            Err("지갑 저장소가 초기화되지 않았습니다".to_string())
        }
    })?;

    // 사용자명 인덱스 업데이트
    if let Some(ref username) = request.username {
        USERNAME_INDEX.with(|index_cell| {
            let mut index_ref = index_cell.borrow_mut();
            if let Some(index) = index_ref.as_mut() {
                index.insert(username.clone(), owner);
            }
        });
    }

    Ok(wallet)
}

/// 지갑 조회
pub fn get_wallet(owner: Principal) -> Option<Wallet> {
    WALLETS.with(|storage| {
        let storage = storage.borrow();
        storage.as_ref()?.get(&owner)
    })
}

/// 지갑 존재 여부 확인
pub fn wallet_exists(owner: Principal) -> bool {
    WALLETS.with(|storage| {
        let storage = storage.borrow();
        storage.as_ref().map_or(false, |s| s.contains_key(&owner))
    })
}

/// 사용자명으로 지갑 조회
pub fn get_wallet_by_username(username: &str) -> Option<Wallet> {
    let owner = USERNAME_INDEX.with(|index| {
        let index = index.borrow();
        index.as_ref()?.get(username)
    })?;

    get_wallet(owner)
}

/// 사용자명 사용 여부 확인
pub fn is_username_taken(username: &str) -> bool {
    USERNAME_INDEX.with(|index| {
        let index = index.borrow();
        index.as_ref().map_or(false, |i| i.contains_key(username))
    })
}

/// 프로필 업데이트
pub fn update_profile(
    owner: Principal,
    request: UpdateProfileRequest,
) -> Result<(), String> {
    let mut wallet = get_wallet(owner)
        .ok_or_else(|| "지갑을 찾을 수 없습니다".to_string())?;

    // 사용자명 변경 시 중복 확인
    if let Some(ref new_username) = request.username {
        if let Some(ref old_username) = wallet.profile.username {
            if new_username != old_username && is_username_taken(new_username) {
                return Err("이미 사용 중인 사용자명입니다".to_string());
            }
        } else if is_username_taken(new_username) {
            return Err("이미 사용 중인 사용자명입니다".to_string());
        }
    }

    // 기존 사용자명 인덱스 제거
    if let Some(ref old_username) = wallet.profile.username {
        USERNAME_INDEX.with(|index_cell| {
            let mut index_ref = index_cell.borrow_mut();
            if let Some(index) = index_ref.as_mut() {
                index.remove(old_username);
            }
        });
    }

    // 프로필 업데이트
    if let Some(username) = request.username {
        wallet.profile.username = Some(username.clone());
        // 새 사용자명 인덱스 추가
        USERNAME_INDEX.with(|index_cell| {
            let mut index_ref = index_cell.borrow_mut();
            if let Some(index) = index_ref.as_mut() {
                index.insert(username, owner);
            }
        });
    }

    if let Some(display_name) = request.display_name {
        wallet.profile.display_name = Some(display_name);
    }

    if let Some(avatar_url) = request.avatar_url {
        wallet.profile.avatar_url = Some(avatar_url);
    }

    if let Some(bio) = request.bio {
        wallet.profile.bio = Some(bio);
    }

    if let Some(email) = request.email {
        wallet.profile.email = Some(email);
    }

    wallet.updated_at = ic_cdk::api::time();

    // 업데이트된 지갑 저장
    WALLETS.with(|storage_cell| {
        let mut storage_ref = storage_cell.borrow_mut();
        if let Some(storage) = storage_ref.as_mut() {
            storage.insert(owner, wallet);
            Ok(())
        } else {
            Err("지갑 저장소가 초기화되지 않았습니다".to_string())
        }
    })
}

// =====================
// 3) 잔액 관리
// =====================

/// 잔액 추가 (입금)
pub fn add_balance(owner: Principal, amount: u64, description: String) -> Result<(), String> {
    let mut wallet = get_wallet(owner)
        .ok_or_else(|| "지갑을 찾을 수 없습니다".to_string())?;

    wallet.balance += amount;
    wallet.updated_at = ic_cdk::api::time();

    // 지갑 업데이트
    WALLETS.with(|storage_cell| {
        let mut storage_ref = storage_cell.borrow_mut();
        if let Some(storage) = storage_ref.as_mut() {
            storage.insert(owner, wallet);
        }
    });

    // 거래 기록 추가
    create_transaction(
        Principal::management_canister(), // 시스템에서 입금
        owner,
        amount,
        TransactionType::Deposit,
        description,
        TransactionStatus::Completed,
    )?;

    Ok(())
}

/// 잔액 차감 (출금/결제)
pub fn deduct_balance(owner: Principal, amount: u64, description: String) -> Result<(), String> {
    let mut wallet = get_wallet(owner)
        .ok_or_else(|| "지갑을 찾을 수 없습니다".to_string())?;

    if wallet.balance < amount {
        return Err("잔액이 부족합니다".to_string());
    }

    wallet.balance -= amount;
    wallet.updated_at = ic_cdk::api::time();

    // 지갑 업데이트
    WALLETS.with(|storage_cell| {
        let mut storage_ref = storage_cell.borrow_mut();
        if let Some(storage) = storage_ref.as_mut() {
            storage.insert(owner, wallet);
        }
    });

    // 거래 기록 추가
    create_transaction(
        owner,
        Principal::management_canister(), // 시스템으로 출금
        amount,
        TransactionType::Withdrawal,
        description,
        TransactionStatus::Completed,
    )?;

    Ok(())
}

/// 잔액 전송
pub fn transfer_balance(
    from: Principal,
    to: Principal,
    amount: u64,
    description: String,
) -> Result<(), String> {
    // 송신자 잔액 확인 및 차감
    let mut from_wallet = get_wallet(from)
        .ok_or_else(|| "송신자 지갑을 찾을 수 없습니다".to_string())?;

    if from_wallet.balance < amount {
        return Err("잔액이 부족합니다".to_string());
    }

    // 수신자 지갑 확인
    let mut to_wallet = get_wallet(to)
        .ok_or_else(|| "수신자 지갑을 찾을 수 없습니다".to_string())?;

    // 잔액 이동
    from_wallet.balance -= amount;
    to_wallet.balance += amount;

    let now = ic_cdk::api::time();
    from_wallet.updated_at = now;
    to_wallet.updated_at = now;

    // 지갑 업데이트
    WALLETS.with(|storage_cell| {
        let mut storage_ref = storage_cell.borrow_mut();
        if let Some(storage) = storage_ref.as_mut() {
            storage.insert(from, from_wallet);
            storage.insert(to, to_wallet);
        }
    });

    // 거래 기록 추가
    create_transaction(
        from,
        to,
        amount,
        TransactionType::Transfer,
        description,
        TransactionStatus::Completed,
    )?;

    Ok(())
}

// =====================
// 4) 거래 기록 관리
// =====================

/// 거래 기록 생성
pub fn create_transaction(
    from: Principal,
    to: Principal,
    amount: u64,
    transaction_type: TransactionType,
    description: String,
    status: TransactionStatus,
) -> Result<u64, String> {
    let transaction_id = TRANSACTION_COUNTER.with(|counter_cell| {
        let mut counter_ref = counter_cell.borrow_mut();
        if let Some(counter) = counter_ref.as_mut() {
            let current = counter.get();
            let next_id = current + 1;
            counter
                .set(next_id)
                .map_err(|e| format!("거래 카운터 업데이트 실패: {:?}", e))?;
            Ok(next_id)
        } else {
            Err("거래 카운터가 초기화되지 않았습니다".to_string())
        }
    })?;

    let transaction = WalletTransaction {
        id: transaction_id,
        from,
        to,
        amount,
        transaction_type,
        description,
        timestamp: ic_cdk::api::time(),
        status,
    };

// src/backend/src/wallet_storage.rs 계속...

    WALLET_TRANSACTIONS.with(|storage_cell| {
        let mut storage_ref = storage_cell.borrow_mut();
        if let Some(storage) = storage_ref.as_mut() {
            storage.insert(transaction_id, transaction);
            Ok(())
        } else {
            Err("거래 기록 저장소가 초기화되지 않았습니다".to_string())
        }
    })?;

    Ok(transaction_id)
}

/// 거래 기록 조회
pub fn get_transaction(transaction_id: u64) -> Option<WalletTransaction> {
    WALLET_TRANSACTIONS.with(|storage| {
        let storage = storage.borrow();
        storage.as_ref()?.get(&transaction_id)
    })
}

/// 사용자별 거래 기록 조회
pub fn get_user_transactions(
    user: Principal,
    limit: Option<u64>,
    offset: Option<u64>,
) -> Vec<WalletTransaction> {
    WALLET_TRANSACTIONS.with(|storage| {
        let storage = storage.borrow();
        if let Some(storage) = storage.as_ref() {
            let mut transactions: Vec<WalletTransaction> = storage
                .iter()
                .filter_map(|(_, transaction)| {
                    if transaction.from == user || transaction.to == user {
                        Some(transaction)
                    } else {
                        None
                    }
                })
                .collect();

            // 최신순으로 정렬
            transactions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

            // 페이지네이션 적용
            let start = offset.unwrap_or(0) as usize;
            let end = if let Some(limit) = limit {
                std::cmp::min(start + limit as usize, transactions.len())
            } else {
                transactions.len()
            };

            if start < transactions.len() {
                transactions[start..end].to_vec()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    })
}

/// 거래 상태 업데이트
pub fn update_transaction_status(
    transaction_id: u64,
    status: TransactionStatus,
) -> Result<(), String> {
    WALLET_TRANSACTIONS.with(|storage_cell| {
        let mut storage_ref = storage_cell.borrow_mut();
        if let Some(storage) = storage_ref.as_mut() {
            if let Some(mut transaction) = storage.get(&transaction_id) {
                transaction.status = status;
                storage.insert(transaction_id, transaction);
                Ok(())
            } else {
                Err("거래 기록을 찾을 수 없습니다".to_string())
            }
        } else {
            Err("거래 기록 저장소가 초기화되지 않았습니다".to_string())
        }
    })
}

// =====================
// 5) 거래 제안 관리
// =====================

/// 거래 제안 생성
pub fn create_trade_offer(request: CreateTradeOfferRequest) -> Result<u64, String> {
    // 제안자 지갑 확인
    let proposer_wallet = get_wallet(request.proposer)
        .ok_or_else(|| "제안자 지갑을 찾을 수 없습니다".to_string())?;

    // 제안 금액이 잔액보다 큰 경우 확인
    if proposer_wallet.balance < request.offered_amount {
        return Err("제안 금액이 잔액을 초과합니다".to_string());
    }

    let offer_id = OFFER_COUNTER.with(|counter_cell| {
        let mut counter_ref = counter_cell.borrow_mut();
        if let Some(counter) = counter_ref.as_mut() {
            let current = counter.get();
            let next_id = current + 1;
            counter
                .set(next_id)
                .map_err(|e| format!("제안 카운터 업데이트 실패: {:?}", e))?;
            Ok(next_id)
        } else {
            Err("제안 카운터가 초기화되지 않았습니다".to_string())
        }
    })?;

    let trade_offer = TradeOffer {
        id: offer_id,
        proposer: request.proposer,
        target: request.target,
        nft_id: request.nft_id,
        offered_amount: request.offered_amount,
        message: request.message,
        expires_at: request.expires_at,
        created_at: ic_cdk::api::time(),
        status: TradeOfferStatus::Pending,
    };

    TRADE_OFFERS.with(|storage_cell| {
        let mut storage_ref = storage_cell.borrow_mut();
        if let Some(storage) = storage_ref.as_mut() {
            storage.insert(offer_id, trade_offer);
            Ok(())
        } else {
            Err("거래 제안 저장소가 초기화되지 않았습니다".to_string())
        }
    })?;

    Ok(offer_id)
}

/// 거래 제안 조회
pub fn get_trade_offer(offer_id: u64) -> Option<TradeOffer> {
    TRADE_OFFERS.with(|storage| {
        let storage = storage.borrow();
        storage.as_ref()?.get(&offer_id)
    })
}

/// 사용자별 받은 거래 제안 조회
pub fn get_received_trade_offers(user: Principal) -> Vec<TradeOffer> {
    TRADE_OFFERS.with(|storage| {
        let storage = storage.borrow();
        if let Some(storage) = storage.as_ref() {
            storage
                .iter()
                .filter_map(|(_, offer)| {
                    if offer.target == Some(user) && offer.status == TradeOfferStatus::Pending {
                        Some(offer)
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    })
}

/// 사용자별 보낸 거래 제안 조회
pub fn get_sent_trade_offers(user: Principal) -> Vec<TradeOffer> {
    TRADE_OFFERS.with(|storage| {
        let storage = storage.borrow();
        if let Some(storage) = storage.as_ref() {
            storage
                .iter()
                .filter_map(|(_, offer)| {
                    if offer.proposer == user {
                        Some(offer)
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    })
}

/// 거래 제안 수락
pub fn accept_trade_offer(offer_id: u64, acceptor: Principal) -> Result<(), String> {
    let mut offer = get_trade_offer(offer_id)
        .ok_or_else(|| "거래 제안을 찾을 수 없습니다".to_string())?;

    // 권한 확인
    if Some(acceptor) != offer.target {
        return Err("거래 제안을 수락할 권한이 없습니다".to_string());
    }

    // 제안 상태 확인
    if offer.status != TradeOfferStatus::Pending {
        return Err("이미 처리된 거래 제안입니다".to_string());
    }

    // 만료 시간 확인
    let now = ic_cdk::api::time();
    if let Some(expires_at) = offer.expires_at {
        if now > expires_at {
            offer.status = TradeOfferStatus::Expired;
            update_trade_offer_status(offer_id, TradeOfferStatus::Expired)?;
            return Err("만료된 거래 제안입니다".to_string());
        }
    }

    // 제안자의 잔액 재확인
    let proposer_wallet = get_wallet(offer.proposer)
        .ok_or_else(|| "제안자 지갑을 찾을 수 없습니다".to_string())?;

    if proposer_wallet.balance < offer.offered_amount {
        offer.status = TradeOfferStatus::Rejected;
        update_trade_offer_status(offer_id, TradeOfferStatus::Rejected)?;
        return Err("제안자의 잔액이 부족합니다".to_string());
    }

    // 거래 실행
    transfer_balance(
        offer.proposer,
        acceptor,
        offer.offered_amount,
        format!("NFT 거래 제안 #{} 수락", offer_id),
    )?;

    // 제안 상태 업데이트
    offer.status = TradeOfferStatus::Accepted;
    update_trade_offer_status(offer_id, TradeOfferStatus::Accepted)?;

    Ok(())
}

/// 거래 제안 거부
pub fn reject_trade_offer(offer_id: u64, rejector: Principal) -> Result<(), String> {
    let offer = get_trade_offer(offer_id)
        .ok_or_else(|| "거래 제안을 찾을 수 없습니다".to_string())?;

    // 권한 확인
    if Some(rejector) != offer.target {
        return Err("거래 제안을 거부할 권한이 없습니다".to_string());
    }

    // 제안 상태 확인
    if offer.status != TradeOfferStatus::Pending {
        return Err("이미 처리된 거래 제안입니다".to_string());
    }

    // 제안 상태 업데이트
    update_trade_offer_status(offer_id, TradeOfferStatus::Rejected)?;

    Ok(())
}

/// 거래 제안 취소
pub fn cancel_trade_offer(offer_id: u64, canceller: Principal) -> Result<(), String> {
    let offer = get_trade_offer(offer_id)
        .ok_or_else(|| "거래 제안을 찾을 수 없습니다".to_string())?;

    // 권한 확인
    if canceller != offer.proposer {
        return Err("거래 제안을 취소할 권한이 없습니다".to_string());
    }

    // 제안 상태 확인
    if offer.status != TradeOfferStatus::Pending {
        return Err("이미 처리된 거래 제안입니다".to_string());
    }

    // 제안 상태 업데이트
    update_trade_offer_status(offer_id, TradeOfferStatus::Cancelled)?;

    Ok(())
}

/// 거래 제안 상태 업데이트
pub fn update_trade_offer_status(
    offer_id: u64,
    status: TradeOfferStatus,
) -> Result<(), String> {
    TRADE_OFFERS.with(|storage_cell| {
        let mut storage_ref = storage_cell.borrow_mut();
        if let Some(storage) = storage_ref.as_mut() {
            if let Some(mut offer) = storage.get(&offer_id) {
                offer.status = status;
                storage.insert(offer_id, offer);
                Ok(())
            } else {
                Err("거래 제안을 찾을 수 없습니다".to_string())
            }
        } else {
            Err("거래 제안 저장소가 초기화되지 않았습니다".to_string())
        }
    })
}

/// 만료된 거래 제안 정리
pub fn cleanup_expired_offers() -> u64 {
    let now = ic_cdk::api::time();
    let mut cleaned_count = 0u64;

    TRADE_OFFERS.with(|storage_cell| {
        let mut storage_ref = storage_cell.borrow_mut();
        if let Some(storage) = storage_ref.as_mut() {
            let expired_offers: Vec<u64> = storage
                .iter()
                .filter_map(|(id, offer)| {
                    if offer.status == TradeOfferStatus::Pending {
                        if let Some(expires_at) = offer.expires_at {
                            if now > expires_at {
                                return Some(id);
                            }
                        }
                    }
                    None
                })
                .collect();

            for offer_id in expired_offers {
                if let Some(mut offer) = storage.get(&offer_id) {
                    offer.status = TradeOfferStatus::Expired;
                    storage.insert(offer_id, offer);
                    cleaned_count += 1;
                }
            }
        }
    });

    cleaned_count
}

// =====================
// 6) 통계 및 유틸리티
// =====================

/// 전체 지갑 수 조회
pub fn get_total_wallets() -> u64 {
    WALLETS.with(|storage| {
        let storage = storage.borrow();
        storage.as_ref().map_or(0, |s| s.len())
    })
}

/// 전체 거래 수 조회
pub fn get_total_transactions() -> u64 {
    TRANSACTION_COUNTER.with(|counter| {
        let counter = counter.borrow();
        counter.as_ref().map_or(0, |c| c.get())
    })
}

/// 전체 거래 제안 수 조회
pub fn get_total_trade_offers() -> u64 {
    OFFER_COUNTER.with(|counter| {
        let counter = counter.borrow();
        counter.as_ref().map_or(0, |c| c.get())
    })
}

/// 시스템 총 잔액 조회
pub fn get_total_system_balance() -> u64 {
    WALLETS.with(|storage| {
        let storage = storage.borrow();
        if let Some(storage) = storage.as_ref() {
            storage.iter().map(|(_, wallet)| wallet.balance).sum()
        } else {
            0
        }
    })
}

/// 사용자 통계 조회
pub fn get_user_stats(user: Principal) -> Option<UserStats> {
    let wallet = get_wallet(user)?;
    
    let transactions = get_user_transactions(user, None, None);
    let sent_count = transactions.iter()
        .filter(|tx| tx.from == user && tx.transaction_type == TransactionType::Transfer)
        .count() as u64;
    let received_count = transactions.iter()
        .filter(|tx| tx.to == user && tx.transaction_type == TransactionType::Transfer)
        .count() as u64;
    let total_sent = transactions.iter()
        .filter(|tx| tx.from == user && tx.transaction_type == TransactionType::Transfer)
        .map(|tx| tx.amount)
        .sum();
    let total_received = transactions.iter()
        .filter(|tx| tx.to == user && tx.transaction_type == TransactionType::Transfer)
        .map(|tx| tx.amount)
        .sum();

    let sent_offers = get_sent_trade_offers(user);
    let received_offers = get_received_trade_offers(user);

    Some(UserStats {
        total_balance: wallet.balance,
        transactions_sent: sent_count,
        transactions_received: received_count,
        total_amount_sent: total_sent,
        total_amount_received: total_received,
        trade_offers_sent: sent_offers.len() as u64,
        trade_offers_received: received_offers.len() as u64,
        join_date: wallet.created_at,
    })
}

/// 지갑 백업 데이터 생성
pub fn export_wallet_data(owner: Principal) -> Option<WalletBackup> {
    let wallet = get_wallet(owner)?;
    let transactions = get_user_transactions(owner, None, None);
    let sent_offers = get_sent_trade_offers(owner);
    let received_offers = get_received_trade_offers(owner);

    Some(WalletBackup {
        wallet,
        transactions,
        sent_trade_offers: sent_offers,
        received_trade_offers: received_offers,
        exported_at: ic_cdk::api::time(),
    })
}

/// 데이터 무결성 검증
pub fn verify_data_integrity() -> IntegrityReport {
    let mut report = IntegrityReport {
        wallet_count: 0,
        transaction_count: 0,
        offer_count: 0,
        total_balance: 0,
        orphaned_transactions: Vec::new(),
        balance_mismatches: Vec::new(),
        is_valid: true,
    };

    // 지갑 데이터 검증
    WALLETS.with(|storage| {
        let storage = storage.borrow();
        if let Some(storage) = storage.as_ref() {
            report.wallet_count = storage.len();
            report.total_balance = storage.iter().map(|(_, wallet)| wallet.balance).sum();
        }
    });

    // 거래 기록 검증
    WALLET_TRANSACTIONS.with(|storage| {
        let storage = storage.borrow();
        if let Some(storage) = storage.as_ref() {
            report.transaction_count = storage.len();
            
            for (_, transaction) in storage.iter() {
                // 관련 지갑 존재 확인
                let from_exists = transaction.from == Principal::management_canister() 
                    || wallet_exists(transaction.from);
                let to_exists = transaction.to == Principal::management_canister() 
                    || wallet_exists(transaction.to);
                
                if !from_exists || !to_exists {
                    report.orphaned_transactions.push(transaction.id);
                    report.is_valid = false;
                }
            }
        }
    });

    // 거래 제안 검증
    TRADE_OFFERS.with(|storage| {
        let storage = storage.borrow();
        if let Some(storage) = storage.as_ref() {
            report.offer_count = storage.len();
        }
    });

    report
}

// =====================
// 7) 정리 및 유지보수
// =====================

/// 시스템 정리 작업 실행
pub fn perform_system_cleanup() -> CleanupResult {
    let expired_offers = cleanup_expired_offers();
    
    // 추가 정리 작업들...
    let cleanup_time = ic_cdk::api::time();
    
    CleanupResult {
        expired_offers_cleaned: expired_offers,
        cleanup_timestamp: cleanup_time,
    }
}

/// 메모리 사용량 통계
pub fn get_memory_stats() -> MemoryStats {
    MemoryStats {
        wallets_memory: get_wallets_memory().size(),
        transactions_memory: get_transactions_memory().size(),
        trade_offers_memory: get_trade_offers_memory().size(),
        total_memory: get_wallets_memory().size() 
            + get_transactions_memory().size() 
            + get_trade_offers_memory().size(),
    }
}