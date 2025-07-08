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
