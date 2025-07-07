// src/backend/src/lib.rs

// 1) 표준 매크로·타입 임포트
use crate::nft_impl::Dip721Impl;
use dip721_rs::{GenericValue, TokenIdentifier, TokenMetadata};
use ic_cdk::storage;
use ic_cdk_macros::{query, update};
use std::cell::RefCell;
use std::thread_local;

// 2) thread_local! + RefCell 으로 전역 상태 선언
storage::thread_local! {
    static NFT: RefCell<Dip721Impl> = RefCell::new(Dip721Impl::new());
}

#[update]
pub fn mint_data_nft(data: Vec<GenericValue>) -> TokenIdentifier {
    // RefCell.borrow_mut() 로 가변 참조 얻은 뒤 mint 호출
    NFT.with(|cell| cell.borrow_mut().mint(data))
}

#[query]
pub fn get_token_metadata(id: TokenIdentifier) -> Option<TokenMetadata> {
    // RefCell.borrow() 로 불변 참조 얻은 뒤 token_metadata 호출
    NFT.with(|cell| cell.borrow().token_metadata(&id).cloned())
}
