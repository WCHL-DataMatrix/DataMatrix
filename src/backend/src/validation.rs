// backend/src/validation.rs

use crate::storage;
use serde_cbor::value::Value as CborValue;
use std::collections::HashSet;

/// 업로드된 데이터 검증
pub fn validate_data(data: &[CborValue]) -> Result<(), String> {
    // 1. 빈 데이터 체크
    if data.is_empty() {
        return Err("업로드된 데이터가 비어 있습니다".to_string());
    }

    // 2. 데이터 개수 제한
    if data.len() > 10000 {
        return Err(format!(
            "데이터 레코드 수가 너무 많습니다. 최대 10,000개, 현재 {}개",
            data.len()
        ));
    }

    // 3. 각 레코드 검증
    for (index, record) in data.iter().enumerate() {
        validate_record(record, index)?;
    }

    // 4. 업로드 데이터 내 중복 검증
    validate_duplicates(data)?;

    // 5. 기존 민팅된 데이터와의 중복 검증
    validate_against_minted_data(data)?;

    Ok(())
}

/// 개별 레코드 검증
fn validate_record(record: &CborValue, index: usize) -> Result<(), String> {
    // 레코드 크기 검증 (1MB 제한)
    let serialized =
        serde_cbor::to_vec(record).map_err(|e| format!("레코드 {} 직렬화 실패: {}", index, e))?;

    if serialized.len() > 1024 * 1024 {
        return Err(format!(
            "레코드 {}의 크기가 너무 큽니다. 최대 1MB, 현재 {}바이트",
            index,
            serialized.len()
        ));
    }

    // 텍스트 길이 검증
    validate_text_fields(record, index)?;

    Ok(())
}

/// 텍스트 필드 검증
fn validate_text_fields(value: &CborValue, index: usize) -> Result<(), String> {
    match value {
        CborValue::Text(text) => {
            if text.len() > 10000 {
                return Err(format!(
                    "레코드 {}의 텍스트 필드가 너무 깁니다. 최대 10,000자, 현재 {}자",
                    index,
                    text.len()
                ));
            }
        }
        CborValue::Array(arr) => {
            for item in arr {
                validate_text_fields(item, index)?;
            }
        }
        CborValue::Map(map) => {
            for (key, val) in map {
                validate_text_fields(key, index)?;
                validate_text_fields(val, index)?;
            }
        }
        _ => {} // 다른 타입은 검증하지 않음
    }
    Ok(())
}

/// 중복 데이터 검증
fn validate_duplicates(data: &[CborValue]) -> Result<(), String> {
    let mut seen = HashSet::new();
    let mut duplicates = Vec::new();

    for (index, record) in data.iter().enumerate() {
        if let Ok(bytes) = serde_cbor::to_vec(record) {
            if !seen.insert(bytes) {
                duplicates.push(index);
            }
        }
    }

    if !duplicates.is_empty() {
        return Err(format!(
            "중복된 데이터가 발견되었습니다. 인덱스: {:?}",
            duplicates
        ));
    }

    Ok(())
}

/// 기존 민팅된 데이터와의 중복 검증
fn validate_against_minted_data(data: &[CborValue]) -> Result<(), String> {
    let mut minted_indices = Vec::new();
    let mut existing_indices = Vec::new();

    // 각 데이터를 직렬화하여 중복 검사
    for (index, record) in data.iter().enumerate() {
        if let Ok(bytes) = serde_cbor::to_vec(record) {
            // storage에서 중복 검사
            let status = storage::check_multiple_data_status(&[bytes]);
            if let Some((existing_id, is_minted)) = status.first() {
                if *is_minted {
                    minted_indices.push(index);
                } else if existing_id.is_some() {
                    existing_indices.push((index, existing_id.unwrap()));
                }
            }
        }
    }

    // 이미 민팅된 데이터가 있으면 에러
    if !minted_indices.is_empty() {
        return Err(format!(
            "이미 민팅된 데이터가 포함되어 있습니다. 인덱스: {:?}",
            minted_indices
        ));
    }

    // 이미 업로드되었지만 민팅되지 않은 데이터는 경고만
    if !existing_indices.is_empty() {
        ic_cdk::println!(
            "이미 업로드된 데이터가 포함되어 있습니다 (민팅은 가능): {:?}",
            existing_indices
        );
    }

    Ok(())
}

/// 민팅 요청 검증 - 강화된 버전
pub fn validate_mint_request(cid: &str, metadata: &[Vec<u8>]) -> Result<(), String> {
    // 1. CID 형식 검증 - 강화
    if cid.trim().is_empty() {
        return Err("CID가 비어 있습니다".to_string());
    }

    if cid.len() > 100 {
        return Err(format!(
            "CID가 너무 깁니다. 최대 100자, 현재 {}자",
            cid.len()
        ));
    }

    // CID 형식 검증 강화
    if !is_valid_cid(cid) {
        return Err("유효하지 않은 CID 형식입니다".to_string());
    }

    // 2. 메타데이터 검증 - 강화
    if metadata.is_empty() {
        return Err("메타데이터가 비어 있습니다".to_string());
    }

    if metadata.len() > 100 {
        return Err(format!(
            "메타데이터 개수가 너무 많습니다. 최대 100개, 현재 {}개",
            metadata.len()
        ));
    }

    // 3. 각 메타데이터 크기 및 내용 검증
    for (index, data) in metadata.iter().enumerate() {
        if data.is_empty() {
            return Err(format!("메타데이터 {}가 비어 있습니다", index));
        }

        if data.len() > 1024 * 1024 {
            return Err(format!(
                "메타데이터 {}의 크기가 너무 큽니다. 최대 1MB, 현재 {}바이트",
                index,
                data.len()
            ));
        }

        // 메타데이터 내용 검증
        validate_metadata_content(data, index)?;
    }

    // 4. 이미 민팅된 데이터인지 확인
    let mut already_minted = Vec::new();
    for (index, data) in metadata.iter().enumerate() {
        if storage::check_data_minted(data) {
            already_minted.push(index);
        }
    }

    if !already_minted.is_empty() {
        return Err(format!(
            "이미 민팅된 메타데이터가 포함되어 있습니다. 인덱스: {:?}",
            already_minted
        ));
    }

    Ok(())
}

/// CID 유효성 검증
fn is_valid_cid(cid: &str) -> bool {
    // 1. 길이 검증
    if cid.len() < 10 || cid.len() > 100 {
        return false;
    }

    // 2. Qm으로 시작하는지 확인
    if !cid.starts_with("Qm") {
        return false;
    }

    // 3. 영숫자만 포함하는지 확인
    if !cid.chars().all(|c| c.is_alphanumeric()) {
        return false;
    }

    // 4. 최소 길이 확인 (실제 IPFS CID는 보통 46자 이상)
    if cid.len() < 46 {
        return false;
    }

    true
}

/// 메타데이터 내용 검증
fn validate_metadata_content(data: &[u8], index: usize) -> Result<(), String> {
    // 최소 10바이트 요구
    if data.len() < 10 {
        return Err(format!(
            "메타데이터 {}의 내용이 너무 짧습니다. 최소 10바이트 필요",
            index
        ));
    }

    // 최대 1MB 제한
    if data.len() > 1024 * 1024 {
        return Err(format!(
            "메타데이터 {}의 크기가 너무 큽니다. 최대 1MB, 현재 {}바이트",
            index,
            data.len()
        ));
    }

    Ok(())
}

/// 데이터 무결성 검증
pub fn validate_data_integrity(data: &[u8]) -> Result<(), String> {
    // CBOR 형식 검증
    match serde_cbor::from_slice::<CborValue>(data) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("데이터 무결성 검증 실패: {}", e)),
    }
}

/// 사용자 권한 검증 - 강화
pub fn validate_user_permission(owner: Option<candid::Principal>) -> Result<(), String> {
    if owner.is_none() {
        // caller() 사용 시 추가 검증 로직
        let caller = ic_cdk::caller();
        if caller == candid::Principal::anonymous() {
            return Err("익명 사용자는 민팅을 요청할 수 없습니다".to_string());
        }
    } else {
        // 명시적으로 지정된 owner가 있는 경우 검증
        let owner_principal = owner.unwrap();
        if owner_principal == candid::Principal::anonymous() {
            return Err("익명 사용자는 민팅을 요청할 수 없습니다".to_string());
        }
    }
    Ok(())
}

/// 데이터 크기 검증 - 강화
pub fn validate_data_size(content: &[u8], max_size: usize) -> Result<(), String> {
    if content.is_empty() {
        return Err("업로드된 데이터가 비어 있습니다".to_string());
    }

    if content.len() > max_size {
        return Err(format!(
            "데이터 크기가 너무 큽니다. 최대 {}바이트, 현재 {}바이트",
            max_size,
            content.len()
        ));
    }
    Ok(())
}

/// 지원되는 MIME 타입 검증 - 강화
pub fn validate_mime_type(mime_type: &str) -> Result<(), String> {
    if mime_type.trim().is_empty() {
        return Err("MIME 타입이 비어 있습니다".to_string());
    }

    match mime_type {
        "application/json" | "text/csv" => Ok(()),
        _ => Err(format!("지원하지 않는 MIME 타입: {}", mime_type)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data() -> Vec<CborValue> {
        vec![
            CborValue::Text("Hello".to_string()),
            CborValue::Integer(42),
            CborValue::Array(vec![
                CborValue::Text("World".to_string()),
                CborValue::Bool(true),
            ]),
        ]
    }

    #[test]
    fn test_valid_data() {
        let data = create_test_data();
        assert!(validate_data(&data).is_ok());
    }

    #[test]
    fn test_empty_data() {
        let data = vec![];
        assert!(validate_data(&data).is_err());
    }

    #[test]
    fn test_cid_validation() {
        assert!(is_valid_cid("QmTest123"));
        assert!(!is_valid_cid(""));
        assert!(!is_valid_cid("Qm@#$%"));
        assert!(!is_valid_cid("invalid"));
    }

    #[test]
    fn test_mint_request_validation() {
        let cid = "QmTest123";
        let metadata = vec![b"test".to_vec()];
        assert!(validate_mint_request(cid, &metadata).is_err()); // 메타데이터가 너무 짧음

        let longer_metadata = vec![b"test_metadata_longer".to_vec()];
        assert!(validate_mint_request(cid, &longer_metadata).is_ok());
    }

    #[test]
    fn test_empty_cid() {
        let cid = "";
        let metadata = vec![b"test_metadata_longer".to_vec()];
        assert!(validate_mint_request(cid, &metadata).is_err());
    }

    #[test]
    fn test_empty_metadata() {
        let cid = "QmTest123";
        let metadata = vec![];
        assert!(validate_mint_request(cid, &metadata).is_err());
    }

    #[test]
    fn test_mime_type_validation() {
        assert!(validate_mime_type("application/json").is_ok());
        assert!(validate_mime_type("text/csv").is_ok());
        assert!(validate_mime_type("application/pdf").is_err());
        assert!(validate_mime_type("").is_err());
    }

    #[test]
    fn test_data_size_validation() {
        let small_data = vec![0u8; 100];
        let large_data = vec![0u8; 1000];

        assert!(validate_data_size(&small_data, 500).is_ok());
        assert!(validate_data_size(&large_data, 500).is_err());
        assert!(validate_data_size(&[], 500).is_err());
    }
}
