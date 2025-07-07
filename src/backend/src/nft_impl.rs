use dip721_rs::{
    Dip721,       // 트레잇
    GenericValue, // 메타데이터용 값 타입
    Metadata,     // 기본 제공 Metadata 구현체
    NftError,
    TokenIdentifier,
    TokenMetadata,
};

/// DIP721 상태를 보관하는 구조체
pub struct Dip721Impl {
    inner: Metadata<GenericValue>,
}

impl Dip721Impl {
    pub fn new() -> Self {
        Self {
            // Metadata::default() 로 내부 상태 초기화
            inner: Metadata::default(),
        }
    }
}

// async-trait은 필요 없고, 동기 트레잇입니다
impl Dip721 for Dip721Impl {
    /// 메타데이터 타입
    type Metadata = GenericValue;
    /// 에러 타입
    type Error = NftError;
    /// 토큰 식별자
    type TokenIdentifier = TokenIdentifier;
    /// 토큰 메타데이터
    type TokenMetadata = TokenMetadata;

    fn mint(&mut self, metadata: Vec<Self::Metadata>) -> Self::TokenIdentifier {
        // 실제 mint 로직은 inner에 위임
        self.inner.mint(metadata)
    }

    fn token_metadata(&self, id: &Self::TokenIdentifier) -> Option<&Self::TokenMetadata> {
        self.inner.token_metadata(id)
    }

    // Dip721 트레잇이 요구하는 나머지 메서드들도 동일하게 inner로 위임해 구현하세요.
}
