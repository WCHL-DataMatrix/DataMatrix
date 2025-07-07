use serde_cbor::Value;
use std::collections::HashSet;

/// 데이터 검증: 비어있지 않은지, 중복 체크
pub fn validate_data(data: &[Value]) -> Result<(), String> {
    if data.is_empty() {
        return Err("업로드된 데이터가 비어 있습니다".into());
    }
    // 예: Value::Text 형식으로만 비교한다고 가정
    let mut seen = HashSet::new();
    for item in data {
        if let Value::Text(s) = item {
            if !seen.insert(s.clone()) {
                return Err(format!("중복된 데이터 발견: {}", s));
            }
        }
    }
    Ok(())
}
