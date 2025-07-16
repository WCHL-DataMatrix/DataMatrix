// backend/src/upload.rs

use csv::ReaderBuilder;
use serde_cbor::value::{to_value, Value as CborValue};
use serde_json::Value as JsonValue;
use std::str;

/// 업로드 데이터 파싱
pub fn upload_data(content: Vec<u8>, mime_type: &str) -> Result<Vec<CborValue>, String> {
    // 빈 데이터 체크
    if content.is_empty() {
        return Err("업로드된 데이터가 비어 있습니다".to_string());
    }

    match mime_type {
        "application/json" => parse_json_data(content),
        "text/csv" => parse_csv_data(content),
        _ => Err(format!("지원하지 않는 MIME 타입: {}", mime_type)),
    }
}

/// JSON 데이터 파싱
fn parse_json_data(content: Vec<u8>) -> Result<Vec<CborValue>, String> {
    let json: JsonValue =
        serde_json::from_slice(&content).map_err(|e| format!("JSON 파싱 실패: {}", e))?;

    let cbor = to_value(json).map_err(|e| format!("CBOR 변환 실패: {}", e))?;

    // 배열이면 그대로 반환, 단일 값이면 배열로 래핑
    match cbor {
        CborValue::Array(arr) => Ok(arr),
        other => Ok(vec![other]),
    }
}

/// CSV 데이터 파싱
fn parse_csv_data(content: Vec<u8>) -> Result<Vec<CborValue>, String> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .from_reader(content.as_slice());

    let mut results = Vec::new();

    for result in rdr.records() {
        let record = result.map_err(|e| format!("CSV 레코드 파싱 실패: {}", e))?;

        let row: Vec<CborValue> = record
            .iter()
            .map(|field| CborValue::Text(field.to_string()))
            .collect();

        results.push(CborValue::Array(row));
    }

    if results.is_empty() {
        return Err("CSV 데이터가 비어 있습니다".to_string());
    }

    Ok(results)
}

/// 데이터 크기 검증
pub fn validate_data_size(content: &[u8], max_size: usize) -> Result<(), String> {
    if content.len() > max_size {
        return Err(format!(
            "데이터 크기가 너무 큽니다. 최대 {}바이트, 현재 {}바이트",
            max_size,
            content.len()
        ));
    }
    Ok(())
}

/// 지원되는 MIME 타입 검증
pub fn validate_mime_type(mime_type: &str) -> Result<(), String> {
    match mime_type {
        "application/json" | "text/csv" => Ok(()),
        _ => Err(format!("지원하지 않는 MIME 타입: {}", mime_type)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::validate_data;

    #[test]
    fn test_json_upload_single_object() {
        let raw = br#"{"name": "Alice", "age": 30}"#.to_vec();
        let parsed = upload_data(raw, "application/json").unwrap();
        assert_eq!(parsed.len(), 1);
        validate_data(&parsed).unwrap();
    }

    #[test]
    fn test_json_upload_array() {
        let raw = br#"[{"foo":"bar"},{"foo":"baz"}]"#.to_vec();
        let parsed = upload_data(raw, "application/json").unwrap();
        assert_eq!(parsed.len(), 2);
        validate_data(&parsed).unwrap();
    }

    #[test]
    fn test_csv_upload() {
        let raw = b"name,age\nAlice,30\nBob,25\n".to_vec();
        let parsed = upload_data(raw, "text/csv").unwrap();
        assert_eq!(parsed.len(), 3); // 헤더 + 2개 데이터
        validate_data(&parsed).unwrap();
    }

    #[test]
    fn test_empty_data() {
        let raw = Vec::new();
        let result = upload_data(raw, "application/json");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_json() {
        let raw = b"invalid json".to_vec();
        let result = upload_data(raw, "application/json");
        assert!(result.is_err());
    }

    #[test]
    fn test_unsupported_mime_type() {
        let raw = b"some data".to_vec();
        let result = upload_data(raw, "application/pdf");
        assert!(result.is_err());
    }

    #[test]
    fn test_mime_type_validation() {
        assert!(validate_mime_type("application/json").is_ok());
        assert!(validate_mime_type("text/csv").is_ok());
        assert!(validate_mime_type("application/pdf").is_err());
    }

    #[test]
    fn test_data_size_validation() {
        let small_data = vec![0u8; 100];
        let large_data = vec![0u8; 1000];

        assert!(validate_data_size(&small_data, 500).is_ok());
        assert!(validate_data_size(&large_data, 500).is_err());
    }
}
