use csv::ReaderBuilder;
use serde_cbor::value::{to_value, Value as CborValue};
use serde_json::Value as JsonValue;
use std::str;

// 업로드할 때 Blob(=Vec<u8>)와 mime_type(예: "application/json", "text/csv")를 함께 전달
pub fn upload_data(content: Vec<u8>, mime_type: &str) -> Result<Vec<CborValue>, String> {
    match mime_type {
        "application/json" => {
            // JSON 파싱 → CBOR Value 변환
            let json: JsonValue =
                serde_json::from_slice(&content).map_err(|e| format!("JSON 파싱 실패: {}", e))?;
            let cbor = to_value(json).map_err(|e| format!("CBOR 변환 실패: {}", e))?;
            // 만약 배열이 아닌 단일 값이라면 Vec로 래핑
            match cbor {
                CborValue::Array(arr) => Ok(arr),
                other => Ok(vec![other]),
            }
        }
        "text/csv" => {
            // CSV 파싱: 각 레코드를 Text 배열로 변환
            let mut rdr = ReaderBuilder::new()
                .has_headers(false)
                .from_reader(content.as_slice());
            let mut out = Vec::new();
            for result in rdr.records() {
                let record = result.map_err(|e| format!("CSV 레코드 오류: {}", e))?;
                let row = record
                    .iter()
                    .map(|s| CborValue::Text(s.to_string()))
                    .collect::<Vec<_>>();
                out.push(CborValue::Array(row));
            }
            if out.is_empty() {
                Err("CSV 데이터가 비어 있습니다".into())
            } else {
                Ok(out)
            }
        }
        // 기타 바이너리 포맷(EX: 이미지)은 우선 그대로 Blob으로 저장하거나 IPFS 등에 올림
        _ => Err(format!("지원하지 않는 MIME 타입: {}", mime_type)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::validate_data;

    #[test]
    fn test_json_upload_and_validate() {
        let raw = br#"[{"foo":"bar"},{"foo":"baz"}]"#.to_vec();
        let parsed = upload_data(raw, "application/json").unwrap();
        // 두 개의 CBOR blob 이 들어와야 함
        assert_eq!(parsed.len(), 2);
        // validate_data 에러 없어야 함
        validate_data(&parsed).unwrap();
    }

    #[test]
    fn test_csv_upload_and_validate() {
        let raw = b"a,b\nx,y\n".to_vec();
        let parsed = upload_data(raw, "text/csv").unwrap();
        assert_eq!(parsed.len(), 2);
        validate_data(&parsed).unwrap();
    }
}
