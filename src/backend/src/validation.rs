use serde_cbor::value::Value as CborValue;
use std::collections::HashSet;

/// 데이터 검증: 비어있지 않은지, 중복 체크
pub fn validate_data(data: &[CborValue]) -> Result<(), String> {
    if data.is_empty() {
        return Err("업로드된 데이터가 비어 있습니다".into());
    }

    // 예: Text 타입 항목만 중복 검사
    let mut seen = HashSet::new();
    for item in data {
        if let CborValue::Text(s) = item {
            if !seen.insert(s.clone()) {
                return Err(format!("중복된 데이터 발견: {}", s));
            }
        }
    }

    Ok(())
}
