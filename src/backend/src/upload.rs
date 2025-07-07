use serde_cbor::Value;

/// 데이터 업로드: 예시로 CBOR Value 리스트를 받는다
pub fn upload_data(raw: Vec<Value>) -> Vec<Value> {
    // 여기서 외부 스토리지(IPFS 등)에 올리거나,
    // 단순히 메모리에 저장하는 로직을 구현하세요.
    raw
}
