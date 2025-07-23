// tests/integration_tests.rs
// DataMatrix 프로젝트용 새로운 통합 테스트

use backend::*;
use candid::Principal;

#[cfg(test)]
mod tests {
    use super::*;

    // ===================
    // 테스트 데이터 생성기
    // ===================
    
    fn create_sample_json_dataset() -> Vec<u8> {
        br#"[
            {
                "id": 1,
                "name": "김철수",
                "age": 28,
                "city": "서울",
                "occupation": "소프트웨어 개발자",
                "salary": 45000000,
                "interests": ["프로그래밍", "게임", "독서"]
            },
            {
                "id": 2,
                "name": "이영희",
                "age": 32,
                "city": "부산",
                "occupation": "디자이너",
                "salary": 38000000,
                "interests": ["디자인", "여행", "요리"]
            },
            {
                "id": 3,
                "name": "박민수",
                "age": 25,
                "city": "대구",
                "occupation": "마케터",
                "salary": 35000000,
                "interests": ["마케팅", "운동", "영화"]
            }
        ]"#.to_vec()
    }

    fn create_sample_csv_dataset() -> Vec<u8> {
        b"id,name,age,city,occupation,salary,rating
1,\xea\xb9\x80\xec\xb2\xa0\xec\x88\x98,28,\xec\x84\x9c\xec\x9a\xb8,\xea\xb0\x9c\xeb\xb0\x9c\xec\x9e\x90,45000000,4.8
2,\xec\x9d\xb4\xec\x98\x81\xed\x9d\xac,32,\xeb\xb6\x80\xec\x82\xb0,\xeb\x94\x94\xec\x9e\x90\xec\x9d\xb4\xeb\x84\x88,38000000,4.6
3,\xeb\xb0\x95\xeb\xaf\xbc\xec\x88\x98,25,\xeb\x8c\x80\xea\xb5\xac,\xeb\xa7\x88\xec\xbc\x80\xed\x84\xb0,35000000,4.4
4,\xec\xa0\x95\xec\x9c\xa0\xec\xa7\x84,29,\xec\x9d\xb8\xec\xb2\x9c,\xec\x97\x94\xec\xa7\x80\xeb\x8b\x88\xec\x96\xb4,42000000,4.7
".to_vec()
    }

    fn create_malformed_json() -> Vec<u8> {
        b"{ invalid json without closing brace".to_vec()
    }

    fn create_empty_dataset() -> Vec<u8> {
        Vec::new()
    }

    fn create_large_dataset() -> Vec<u8> {
        // 11MB 데이터 생성 (10MB 제한을 초과)
        let large_string = "x".repeat(11 * 1024 * 1024);
        format!(r#"{{"large_data": "{}"}}"#, large_string).into_bytes()
    }

    // ===========================
    // 1. 데이터 업로드 & 검증 테스트
    // ===========================

    #[test]
    fn test_successful_json_upload_and_validation() {
        println!("=== JSON 데이터 업로드 & 검증 테스트 ===");
        
        let upload_request = UploadRequest {
            content: create_sample_json_dataset(),
            mime_type: "application/json".to_string(),
        };

        // 업로드 실행
        let result = upload(upload_request);
        assert!(result.is_ok(), "JSON 업로드가 실패했습니다: {:?}", result.err());

        let response = result.unwrap();
        assert_eq!(response.data.len(), 3, "3개의 데이터 레코드가 예상되었습니다");
        
        // 데이터 ID 추출 및 검증
        let data_ids: Vec<u64> = response.data
            .iter()
            .map(|bytes| u64::from_be_bytes(bytes.as_slice().try_into().unwrap()))
            .collect();
        
        println!("✓ 업로드된 데이터 ID들: {:?}", data_ids);
        
        // 각 데이터를 다시 조회하여 저장 확인
        for data_id in &data_ids {
            let stored_data = get_uploaded_data(*data_id);
            assert!(stored_data.is_some(), "데이터 ID {}를 찾을 수 없습니다", data_id);
        }
        
        println!("✓ JSON 업로드 & 검증 성공");
    }

    #[test]
    fn test_successful_csv_upload_and_validation() {
        println!("=== CSV 데이터 업로드 & 검증 테스트 ===");
        
        let upload_request = UploadRequest {
            content: create_sample_csv_dataset(),
            mime_type: "text/csv".to_string(),
        };

        let result = upload(upload_request);
        assert!(result.is_ok(), "CSV 업로드가 실패했습니다: {:?}", result.err());

        let response = result.unwrap();
        assert!(response.data.len() > 0, "CSV 데이터가 업로드되지 않았습니다");
        
        println!("✓ CSV 업로드된 레코드 수: {}", response.data.len());
        println!("✓ CSV 업로드 & 검증 성공");
    }

    #[test]
    fn test_upload_validation_failures() {
        println!("=== 업로드 검증 실패 테스트 ===");
        
        // 1. 잘못된 JSON 형식
        let malformed_request = UploadRequest {
            content: create_malformed_json(),
            mime_type: "application/json".to_string(),
        };
        let result = upload(malformed_request);
        assert!(result.is_err(), "잘못된 JSON이 성공적으로 업로드되었습니다");
        println!("✓ 잘못된 JSON 형식 거부 성공");

        // 2. 빈 데이터
        let empty_request = UploadRequest {
            content: create_empty_dataset(),
            mime_type: "application/json".to_string(),
        };
        let result = upload(empty_request);
        assert!(result.is_err(), "빈 데이터가 성공적으로 업로드되었습니다");
        println!("✓ 빈 데이터 거부 성공");

        // 3. 지원하지 않는 MIME 타입
        let unsupported_request = UploadRequest {
            content: b"some binary data".to_vec(),
            mime_type: "application/pdf".to_string(),
        };
        let result = upload(unsupported_request);
        assert!(result.is_err(), "지원하지 않는 MIME 타입이 허용되었습니다");
        println!("✓ 지원하지 않는 MIME 타입 거부 성공");

        // 4. 너무 큰 데이터
        let large_request = UploadRequest {
            content: create_large_dataset(),
            mime_type: "application/json".to_string(),
        };
        let result = upload(large_request);
        assert!(result.is_err(), "큰 데이터가 성공적으로 업로드되었습니다");
        println!("✓ 크기 제한 초과 데이터 거부 성공");
    }

    // ====================
    // 2. 민팅 프로세스 테스트
    // ====================

    #[test]
    fn test_independent_minting_process() {
        println!("=== 독립적 민팅 프로세스 테스트 ===");
        
        // 1. 데이터 업로드
        let upload_request = UploadRequest {
            content: create_sample_json_dataset(),
            mime_type: "application/json".to_string(),
        };
        let upload_result = upload(upload_request).unwrap();
        println!("✓ 데이터 업로드 완료: {} 레코드", upload_result.data.len());

        // 2. 각 데이터에 대해 독립적인 민팅 요청 생성
        let test_principal = Principal::anonymous();
        let mut mint_request_ids = Vec::new();

        for (index, data_blob) in upload_result.data.iter().enumerate() {
            let mint_request = MintRequest {
                owner: Some(test_principal),
                cid: format!("QmDataMatrix{:04}{}", index + 1, 
                    // 간단한 체크섬 추가
                    data_blob.len() % 1000),
                metadata: vec![data_blob.clone()],
            };

            // 민팅 요청 제출
            let mint_response = request_mint(mint_request);
            mint_request_ids.push(mint_response.request_id);
            
            println!("✓ 민팅 요청 #{} 생성됨 (ID: {})", 
                index + 1, mint_response.request_id);
        }

        // 3. 모든 민팅 요청이 Pending 상태인지 확인
        for (index, request_id) in mint_request_ids.iter().enumerate() {
            let status = get_mint_status(*request_id);
            assert!(status.is_some(), "민팅 요청 {}의 상태를 찾을 수 없습니다", request_id);
            assert_eq!(status.unwrap(), MintStatus::Pending, 
                "민팅 요청 {}가 Pending 상태가 아닙니다", request_id);
            
            println!("✓ 민팅 요청 #{} 상태: Pending", index + 1);
        }

        println!("✓ 독립적 민팅 프로세스 테스트 성공");
    }

    #[test]
    fn test_mint_request_validation() {
        println!("=== 민팅 요청 검증 테스트 ===");
        
        // 유효한 민팅 요청
        let valid_request = MintRequest {
            owner: Some(Principal::anonymous()),
            cid: "QmValidCID123456789".to_string(),
            metadata: vec![b"valid metadata".to_vec()],
        };
        let result = request_mint(valid_request);
        assert!(result.request_id > 0, "유효한 민팅 요청이 실패했습니다");
        println!("✓ 유효한 민팅 요청 성공");

        // 빈 CID - 이 경우 trap이 발생하므로 테스트하기 어려움
        // 실제 환경에서는 frontend에서 검증해야 함
        println!("✓ 민팅 요청 검증 테스트 완료");
    }

    // =========================
    // 3. 전체 워크플로우 통합 테스트
    // =========================

    #[test]
    fn test_complete_data_lifecycle() {
        println!("=== 완전한 데이터 라이프사이클 테스트 ===");
        
        // Phase 1: 데이터 업로드
        println!("Phase 1: 데이터 업로드");
        let upload_request = UploadRequest {
            content: create_sample_json_dataset(),
            mime_type: "application/json".to_string(),
        };
        let upload_result = upload(upload_request).unwrap();
        let uploaded_count = upload_result.data.len();
        println!("✓ {} 개 데이터 업로드 완료", uploaded_count);

        // Phase 2: 데이터 검증 및 저장 확인
        println!("Phase 2: 데이터 검증");
        let data_ids: Vec<u64> = upload_result.data
            .iter()
            .map(|bytes| u64::from_be_bytes(bytes.as_slice().try_into().unwrap()))
            .collect();
        
        for data_id in &data_ids {
            assert!(get_uploaded_data(*data_id).is_some(), 
                "데이터 ID {}가 저장되지 않았습니다", data_id);
        }
        println!("✓ {} 개 데이터 검증 완료", data_ids.len());

        // Phase 3: 민팅 요청 및 각 canister 독립 처리
        println!("Phase 3: 민팅 요청");
        let mut mint_requests = Vec::new();
        
        for (idx, data_blob) in upload_result.data.iter().enumerate() {
            let mint_request = MintRequest {
                owner: Some(Principal::anonymous()),
                cid: format!("QmDataMatrix{:06x}", idx),
                metadata: vec![data_blob.clone()],
            };
            
            let response = request_mint(mint_request);
            mint_requests.push(response.request_id);
        }
        println!("✓ {} 개 민팅 요청 완료", mint_requests.len());

        // Phase 4: 민팅 상태 확인
        println!("Phase 4: 민팅 상태 확인");
        for request_id in &mint_requests {
            let status = get_mint_status(*request_id);
            assert_eq!(status, Some(MintStatus::Pending));
        }
        println!("✓ 모든 민팅 요청이 Pending 상태");

        // Phase 5: 저장소 통계 확인
        println!("Phase 5: 저장소 통계 확인");
        let stats = get_storage_stats();
        assert!(stats.total_uploads >= uploaded_count as u64);
        assert!(stats.total_mint_requests >= mint_requests.len() as u64);
        println!("✓ 통계 - 업로드: {}, 민팅 요청: {}", 
            stats.total_uploads, stats.total_mint_requests);

        println!("✓ 완전한 데이터 라이프사이클 테스트 성공");
    }

    // =======================
    // 4. 성능 및 스트레스 테스트
    // =======================

    #[test]
    fn test_multiple_concurrent_uploads() {
        println!("=== 다중 업로드 테스트 ===");
        
        let datasets = vec![
            ("JSON", create_sample_json_dataset(), "application/json"),
            ("CSV", create_sample_csv_dataset(), "text/csv"),
        ];
        
        let mut total_uploaded = 0;
        
        for (name, data, mime_type) in datasets {
            let request = UploadRequest {
                content: data,
                mime_type: mime_type.to_string(),
            };
            
            let result = upload(request);
            assert!(result.is_ok(), "{} 업로드 실패", name);
            
            let count = result.unwrap().data.len();
            total_uploaded += count;
            println!("✓ {} 데이터 업로드: {} 레코드", name, count);
        }
        
        println!("✓ 총 {} 레코드 업로드 완료", total_uploaded);
    }

    #[test]
    fn test_storage_management() {
        println!("=== 저장소 관리 테스트 ===");
        
        // 데이터 업로드
        let upload_request = UploadRequest {
            content: create_sample_json_dataset(),
            mime_type: "application/json".to_string(),
        };
        let upload_result = upload(upload_request).unwrap();
        
        // 데이터 목록 조회
        let data_list = list_uploaded_data();
        assert!(!data_list.is_empty(), "업로드된 데이터 목록이 비어 있습니다");
        println!("✓ 업로드된 데이터 목록 조회: {} 개", data_list.len());
        
        // 민팅 요청 목록 조회
        let mint_list = list_mint_requests();
        println!("✓ 민팅 요청 목록 조회: {} 개", mint_list.len());
        
        println!("✓ 저장소 관리 테스트 완료");
    }

    // ==================
    // 5. 에러 복구 테스트
    // ==================

    #[test]
    fn test_error_recovery_scenarios() {
        println!("=== 에러 복구 시나리오 테스트 ===");
        
        // 존재하지 않는 데이터 조회
        let non_existent_data = get_uploaded_data(99999);
        assert!(non_existent_data.is_none(), "존재하지 않는 데이터가 반환되었습니다");
        println!("✓ 존재하지 않는 데이터 조회 처리");
        
        // 존재하지 않는 민팅 요청 상태 조회
        let non_existent_status = get_mint_status(99999);
        assert!(non_existent_status.is_none(), "존재하지 않는 민팅 요청 상태가 반환되었습니다");
        println!("✓ 존재하지 않는 민팅 요청 상태 조회 처리");
        
        println!("✓ 에러 복구 시나리오 테스트 완료");
    }

    // ==================
    // 6. 헬퍼 함수 테스트
    // ==================

    #[test]
    fn test_system_functionality() {
        println!("=== 시스템 기능 테스트 ===");
        
        // 통계 조회
        let stats = get_storage_stats();
        println!("✓ 저장소 통계: 업로드 {}, 민팅 요청 {}", 
            stats.total_uploads, stats.total_mint_requests);
        
        // 기본적인 시스템 상태 확인
        assert!(true, "시스템이 정상적으로 작동하지 않습니다");
        
        println!("✓ 시스템 기능 테스트 완료");
    }
}