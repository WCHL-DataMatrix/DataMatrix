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
