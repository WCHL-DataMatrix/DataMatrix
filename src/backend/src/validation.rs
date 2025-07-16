use serde_cbor::value::Value as CborValue;

/// 데이터 검증: 비어있지 않은지, 중복 체크
pub fn validate_data(data: &[CborValue]) -> Result<(), String> {
    if data.is_empty() {
        return Err("업로드된 데이터가 비어 있습니다".into());
    }

    // 현재 등록된 데이터 중 동일 데이터가 존재하는지 확인
    // 아직 저장된 데이터가 없기에 pass

    Ok(()) // ()은 unit 타입으로 void 느낌
}

// // backend/src/validation.rs

// use serde_cbor::value::Value as CborValue;
// use std::collections::HashSet;

// /// 업로드된 데이터 검증
// pub fn validate_data(data: &[CborValue]) -> Result<(), String> {
//     // 1. 빈 데이터 체크
//     if data.is_empty() {
//         return Err("업로드된 데이터가 비어 있습니다".to_string());
//     }

//     // 2. 데이터 개수 제한
//     if data.len() > 10000 {
//         return Err(format!(
//             "데이터 레코드 수가 너무 많습니다. 최대 10,000개, 현재 {}개",
//             data.len()
//         ));
//     }

//     // 3. 각 레코드 검증
//     for (index, record) in data.iter().enumerate() {
//         validate_record(record, index)?;
//     }

//     // 4. 중복 데이터 검증
//     validate_duplicates(data)?;

//     Ok(())
// }

// /// 개별 레코드 검증
// fn validate_record(record: &CborValue, index: usize) -> Result<(), String> {
//     // 레코드 크기 검증 (1MB 제한)
//     let serialized = serde_cbor::to_vec(record)
//         .map_err(|e| format!("레코드 {} 직렬화 실패: {}", index, e))?;

//     if serialized.len() > 1024 * 1024 {
//         return Err(format!(
//             "레코드 {}의 크기가 너무 큽니다. 최대 1MB, 현재 {}바이트",
//             index,
//             serialized.len()
//         ));
//     }

//     // 텍스트 길이 검증
//     validate_text_fields(record, index)?;

//     Ok(())
// }

// /// 텍스트 필드 검증
// fn validate_text_fields(value: &CborValue, index: usize) -> Result<(), String> {
//     match value {
//         CborValue::Text(text) => {
//             if text.len() > 10000 {
//                 return Err(format!(
//                     "레코드 {}의 텍스트 필드가 너무 깁니다. 최대 10,000자, 현재 {}자",
//                     index,
//                     text.len()
//                 ));
//             }
//         }
//         CborValue::Array(arr) => {
//             for item in arr {
//                 validate_text_fields(item, index)?;
//             }
//         }
//         CborValue::Map(map) => {
//             for (key, val) in map {
//                 validate_text_fields(key, index)?;
//                 validate_text_fields(val, index)?;
//             }
//         }
//         _ => {} // 다른 타입은 검증하지 않음
//     }
//     Ok(())
// }

// /// 중복 데이터 검증
// fn validate_duplicates(data: &[CborValue]) -> Result<(), String> {
//     let mut seen = HashSet::new();
//     let mut duplicates = Vec::new();

//     for (index, record) in data.iter().enumerate() {
//         if let Ok(bytes) = serde_cbor::to_vec(record) {
//             if !seen.insert(bytes) {
//                 duplicates.push(index);
//             }
//         }
//     }

//     if !duplicates.is_empty() {
//         return Err(format!(
//             "중복된 데이터가 발견되었습니다. 인덱스: {:?}",
//             duplicates
//         ));
//     }

//     Ok(())
// }

// /// 민팅 요청 검증
// pub fn validate_mint_request(cid: &str, metadata: &[Vec<u8>]) -> Result<(), String> {
//     // 1. CID 형식 검증
//     if cid.is_empty() {
//         return Err("CID가 비어 있습니다".to_string());
//     }

//     if cid.len() > 100 {
//         return Err(format!(
//             "CID가 너무 깁니다. 최대 100자, 현재 {}자",
//             cid.len()
//         ));
//     }

//     // 2. 메타데이터 검증
//     if metadata.is_empty() {
//         return Err("메타데이터가 비어 있습니다".to_string());
//     }

//     if metadata.len() > 100 {
//         return Err(format!(
//             "메타데이터 개수가 너무 많습니다. 최대 100개, 현재 {}개",
//             metadata.len()
//         ));
//     }

//     // 3. 각 메타데이터 크기 검증
//     for (index, data) in metadata.iter().enumerate() {
//         if data.len() > 1024 * 1024 {
//             return Err(format!(
//                 "메타데이터 {}의 크기가 너무 큽니다. 최대 1MB, 현재 {}바이트",
//                 index,
//                 data.len()
//             ));
//         }
//     }

//     Ok(())
// }

// /// 데이터 무결성 검증
// pub fn validate_data_integrity(data: &[u8]) -> Result<(), String> {
//     // CBOR 형식 검증
//     match serde_cbor::from_slice::<CborValue>(data) {
//         Ok(_) => Ok(()),
//         Err(e) => Err(format!("데이터 무결성 검증 실패: {}", e)),
//     }
// }

// /// 사용자 권한 검증
// pub fn validate_user_permission(owner: Option<candid::Principal>) -> Result<(), String> {
//     if owner.is_none() {
//         // caller() 사용 시 추가 검증 로직
//         let caller = ic_cdk::caller();
//         if caller == candid::Principal::anonymous() {
//             return Err("익명 사용자는 민팅을 요청할 수 없습니다".to_string());
//         }
//     }
//     Ok(())
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use serde_cbor::value::Value as CborValue;

//     fn create_test_data() -> Vec<CborValue> {
//         vec![
//             CborValue::Text("Hello".to_string()),
//             CborValue::Integer(42),
//             CborValue::Array(vec![
//                 CborValue::Text("World".to_string()),
//                 CborValue::Bool(true),
//             ]),
//         ]
//     }

//     #[test]
//     fn test_valid_data() {
//         let data = create_test_data();
//         assert!(validate_data(&data).is_ok());
//     }

//     #[test]
//     fn test_empty_data() {
//         let data = vec![];
//         assert!(validate_data(&data).is_err());
//     }

//     #[test]
//     fn test_too_many_records() {
//         let data = vec![CborValue::Text("test".to_string()); 10001];
//         assert!(validate_data(&data).is_err());
//     }

//     #[test]
//     fn test_duplicate_data() {
//         let data = vec![
//             CborValue::Text("same".to_string()),
//             CborValue::Text("same".to_string()),
//         ];
//         assert!(validate_data(&data).is_err());
//     }

//     #[test]
//     fn test_long_text() {
//         let long_text = "a".repeat(10001);
//         let data = vec![CborValue::Text(long_text)];
//         assert!(validate_data(&data).is_err());
//     }

//     #[test]
//     fn test_mint_request_validation() {
//         let cid = "QmTest123";
//         let metadata = vec![b"test".to_vec()];
//         assert!(validate_mint_request(cid, &metadata).is_ok());
//     }

//     #[test]
//     fn test_empty_cid() {
//         let cid = "";
//         let metadata = vec![b"test".to_vec()];
//         assert!(validate_mint_request(cid, &metadata).is_err());
//     }

//     #[test]
//     fn test_empty_metadata() {
//         let cid = "QmTest123";
//         let metadata = vec![];
//         assert!(validate_mint_request(cid, &metadata).is_err());
//     }

//     #[test]
//     fn test_data_integrity() {
//         let valid_cbor = serde_cbor::to_vec(&CborValue::Text("test".to_string())).unwrap();
//         assert!(validate_data_integrity(&valid_cbor).is_ok());

//         let invalid_cbor = vec![0xFF, 0xFF, 0xFF];
//         assert!(validate_data_integrity(&invalid_cbor).is_err());
//     }
// }
