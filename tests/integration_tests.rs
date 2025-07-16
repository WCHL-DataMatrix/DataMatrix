// tests/integration_tests.rs

use backend::*;
use candid::Principal;

#[cfg(test)]
mod integration_tests {
    use super::*;

    fn create_test_json_data() -> Vec<u8> {
        br#"[
            {"name": "Alice", "age": 30, "city": "Seoul"},
            {"name": "Bob", "age": 25, "city": "Busan"}
        ]"#
        .to_vec()
    }

    fn create_test_csv_data() -> Vec<u8> {
        b"name,age,city\nAlice,30,Seoul\nBob,25,Busan\n".to_vec()
    }

    #[test]
    fn test_full_workflow() {
        // 1. JSON 데이터 업로드
        let json_req = UploadRequest {
            content: create_test_json_data(),
            mime_type: "application/json".to_string(),
        };

        let upload_result = upload(json_req).unwrap();
        assert_eq!(upload_result.data.len(), 2);

        // 2. 민팅 요청
        let mint_req = MintRequest {
            owner: Some(Principal::anonymous()),
            cid: "QmTest123".to_string(),
            metadata: upload_result.data,
        };

        let mint_response = request_mint(mint_req);
        assert!(mint_response.request_id > 0);

        // 3. 상태 확인
        let status = get_mint_status(mint_response.request_id);
        assert_eq!(status, Some(MintStatus::Pending));

        // 4. 통계 확인
        let stats = get_storage_stats();
        assert!(stats.total_uploads > 0);
        assert!(stats.total_mint_requests > 0);
    }

    #[test]
    fn test_data_persistence() {
        // 데이터 업로드
        let req = UploadRequest {
            content: create_test_json_data(),
            mime_type: "application/json".to_string(),
        };

        let result = upload(req).unwrap();
        let data_ids: Vec<u64> = result
            .data
            .iter()
            .map(|bytes| u64::from_be_bytes(bytes.try_into().unwrap()))
            .collect();

        // 데이터 조회
        for data_id in data_ids {
            let stored_data = get_uploaded_data(data_id);
            assert!(stored_data.is_some());
        }

        // 데이터 목록 조회
        let data_list = list_uploaded_data();
        assert!(!data_list.is_empty());
    }

    #[test]
    fn test_error_handling() {
        // 잘못된 JSON
        let invalid_req = UploadRequest {
            content: b"invalid json".to_vec(),
            mime_type: "application/json".to_string(),
        };
        assert!(upload(invalid_req).is_err());

        // 지원하지 않는 MIME 타입
        let unsupported_req = UploadRequest {
            content: b"some data".to_vec(),
            mime_type: "application/pdf".to_string(),
        };
        assert!(upload(unsupported_req).is_err());
    }

    #[test]
    fn test_mint_request_management() {
        // 여러 민팅 요청 생성
        let mut request_ids = Vec::new();

        for i in 0..5 {
            let mint_req = MintRequest {
                owner: Some(Principal::anonymous()),
                cid: format!("QmTest{}", i),
                metadata: vec![format!("data_{}", i).into_bytes()],
            };

            let response = request_mint(mint_req);
            request_ids.push(response.request_id);
        }

        // 모든 요청이 Pending 상태인지 확인
        for request_id in &request_ids {
            let status = get_mint_status(*request_id);
            assert_eq!(status, Some(MintStatus::Pending));
        }

        // 민팅 요청 목록 조회
        let mint_requests = list_mint_requests();
        assert!(mint_requests.len() >= 5);
    }
}
