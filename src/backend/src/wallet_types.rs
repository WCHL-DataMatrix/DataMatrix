// src/backend/src/wallet_types.rs

use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::Storable;
use serde::Serialize;
use std::borrow::Cow;

// =====================
// 1) 지갑 관련 타입
// =====================

/// 사용자 지갑 정보
#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct Wallet {
    pub owner: Principal,
    pub balance: u64,           // ICP 잔액 (e8s 단위)
    pub created_at: u64,
    pub updated_at: u64,
    pub profile: UserProfile,
}

/// 사용자 프로필 정보
#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct UserProfile {
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub email: Option<String>,
    pub verified: bool,
    pub join_date: u64,
}

/// 지갑 생성 요청
#[derive(CandidType, Deserialize)]
pub struct CreateWalletRequest {
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub email: Option<String>,
}

/// 프로필 업데이트 요청
#[derive(CandidType, Deserialize)]
pub struct UpdateProfileRequest {
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub email: Option<String>,
}

/// 지갑 거래 기록
#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct WalletTransaction {
    pub id: u64,
    pub from: Principal,
    pub to: Principal,
    pub amount: u64,
    pub transaction_type: TransactionType,
    pub description: String,
    pub timestamp: u64,
    pub status: TransactionStatus,
}

/// 거래 타입
#[derive(CandidType, Deserialize, Serialize, Clone)]
pub enum TransactionType {
    Deposit,      // 입금
    Withdrawal,   // 출금
    Transfer,     // 전송
    Purchase,     // NFT 구매
    Sale,         // NFT 판매
    Mint,         // NFT 민팅
    Commission,   // 수수료
}

/// 거래 상태
#[derive(CandidType, Deserialize, Serialize, Clone)]
pub enum TransactionStatus {
    Pending,
    Completed,
    Failed,
    Cancelled,
}

// =====================
// 2) 거래 관련 타입
// =====================

/// NFT 거래 제안
#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct TradeOffer {
    pub id: u64,
    pub listing_id: u64,
    pub buyer: Principal,
    pub seller: Principal,
    pub price: u64,
    pub currency: String,
    pub offer_type: OfferType,
    pub expires_at: Option<u64>,
    pub created_at: u64,
    pub status: OfferStatus,
    pub message: Option<String>,
}

/// 제안 타입
#[derive(CandidType, Deserialize, Serialize, Clone)]
pub enum OfferType {
    DirectPurchase,  // 즉시 구매
    Bid,            // 경매 입찰
    CounterOffer,   // 재제안
}

/// 제안 상태
#[derive(CandidType, Deserialize, Serialize, Clone)]
pub enum OfferStatus {
    Pending,
    Accepted,
    Rejected,
    Expired,
    Withdrawn,
}

/// 거래 생성 요청
#[derive(CandidType, Deserialize)]
pub struct CreateTradeOfferRequest {
    pub listing_id: u64,
    pub price: u64,
    pub currency: String,
    pub offer_type: OfferType,
    pub expires_in_hours: Option<u64>,
    pub message: Option<String>,
}

/// 거래 응답 요청
#[derive(CandidType, Deserialize)]
pub struct RespondToOfferRequest {
    pub offer_id: u64,
    pub response: OfferResponse,
    pub counter_price: Option<u64>,
    pub message: Option<String>,
}

/// 제안 응답 타입
#[derive(CandidType, Deserialize)]
pub enum OfferResponse {
    Accept,
    Reject,
    CounterOffer,
}

/// 거래 실행 요청
#[derive(CandidType, Deserialize)]
pub struct ExecuteTradeRequest {
    pub offer_id: u64,
}

/// 거래 실행 결과
#[derive(CandidType, Deserialize)]
pub struct TradeExecutionResult {
    pub success: bool,
    pub transaction_id: u64,
    pub new_owner: Principal,
    pub price_paid: u64,
    pub commission_fee: u64,
    pub message: String,
}

// =====================
// 3) 응답 타입
// =====================

/// 지갑 정보 응답
#[derive(CandidType, Deserialize)]
pub struct WalletResponse {
    pub wallet: Wallet,
    pub recent_transactions: Vec<WalletTransaction>,
}

/// 거래 제안 목록 응답
#[derive(CandidType, Deserialize)]
pub struct TradeOffersResponse {
    pub offers: Vec<TradeOffer>,
    pub total_count: u64,
}

/// 일반 성공 응답
#[derive(CandidType, Deserialize)]
pub struct WalletSuccessResponse {
    pub message: String,
}

// =====================
// 4) Storable 구현
// =====================

impl Storable for Wallet {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(serde_cbor::to_vec(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        serde_cbor::from_slice(&bytes).unwrap()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

impl Storable for WalletTransaction {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(serde_cbor::to_vec(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        serde_cbor::from_slice(&bytes).unwrap()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

impl Storable for TradeOffer {
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
// 5) 검증 함수들
// =====================

impl CreateWalletRequest {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(ref username) = self.username {
            if username.trim().is_empty() {
                return Err("사용자명은 비어있을 수 없습니다".to_string());
            }
            if username.len() > 50 {
                return Err("사용자명은 50자를 초과할 수 없습니다".to_string());
            }
            // 영숫자와 언더스코어만 허용
            if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Err("사용자명은 영숫자와 언더스코어만 사용할 수 있습니다".to_string());
            }
        }

        if let Some(ref display_name) = self.display_name {
            if display_name.len() > 100 {
                return Err("표시명은 100자를 초과할 수 없습니다".to_string());
            }
        }

        if let Some(ref bio) = self.bio {
            if bio.len() > 500 {
                return Err("소개글은 500자를 초과할 수 없습니다".to_string());
            }
        }

        if let Some(ref email) = self.email {
            if !is_valid_email(email) {
                return Err("유효하지 않은 이메일 형식입니다".to_string());
            }
        }

        Ok(())
    }
}

impl UpdateProfileRequest {
    pub fn validate(&self) -> Result<(), String> {
        let temp_request = CreateWalletRequest {
            username: self.username.clone(),
            display_name: self.display_name.clone(),
            avatar_url: self.avatar_url.clone(),
            bio: self.bio.clone(),
            email: self.email.clone(),
        };
        temp_request.validate()
    }
}

impl CreateTradeOfferRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.price == 0 {
            return Err("제안 금액은 0보다 커야 합니다".to_string());
        }

        if self.currency.trim().is_empty() {
            return Err("통화는 필수입니다".to_string());
        }

        if !["ICP", "USD"].contains(&self.currency.as_str()) {
            return Err("지원되지 않는 통화입니다".to_string());
        }

        if let Some(hours) = self.expires_in_hours {
            if hours == 0 || hours > 24 * 30 {  // 최대 30일
                return Err("만료 시간은 1시간에서 30일 사이여야 합니다".to_string());
            }
        }

        if let Some(ref message) = self.message {
            if message.len() > 500 {
                return Err("메시지는 500자를 초과할 수 없습니다".to_string());
            }
        }

        Ok(())
    }
}

// =====================
// 6) 유틸리티 함수
// =====================

/// 이메일 형식 검증 (간단한 정규식)
fn is_valid_email(email: &str) -> bool {
    let email_regex = regex::Regex::new(r"^[^@]+@[^@]+\.[^@]+$").unwrap();
    email_regex.is_match(email)
}

/// ICP 잔액을 사람이 읽기 쉬운 형태로 변환
pub fn format_icp_balance(balance_e8s: u64) -> String {
    let icp_amount = balance_e8s as f64 / 100_000_000.0;
    format!("{:.8} ICP", icp_amount)
}

/// 사람이 읽기 쉬운 형태의 ICP를 e8s로 변환
pub fn parse_icp_amount(icp_str: &str) -> Result<u64, String> {
    let amount: f64 = icp_str.parse()
        .map_err(|_| "유효하지 않은 ICP 금액입니다".to_string())?;
    
    if amount < 0.0 {
        return Err("음수 금액은 허용되지 않습니다".to_string());
    }
    
    Ok((amount * 100_000_000.0) as u64)
}

/// 거래 수수료 계산 (2.5%)
pub fn calculate_commission_fee(amount: u64) -> u64 {
    (amount as f64 * 0.025) as u64
}

/// Principal을 짧은 형태로 표시
pub fn format_principal(principal: &Principal) -> String {
    let text = principal.to_text();
    if text.len() > 12 {
        format!("{}...{}", &text[..6], &text[text.len()-6..])
    } else {
        text
    }
}