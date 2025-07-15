// backend/src/lib.rs

// 매크로 및 타입 임포트
use candid::Principal;
use candid::{CandidType, Deserialize};
use ic_cdk::api::call::call;
use ic_cdk_macros::{init, query, update};
use std::time::Duration;

// 모듈 선언
mod nft;
mod upload;
mod validation;

// #derive: 해당 구조체는 RUST <-> Candid 간 형식 변환을 지원한다
// #update: 수정을 하겠다
// #query: 조회만 하겠다

use once_cell::sync::Lazy;
static WORKER_CANISTER_TEXT: &str = "bw4dl-smaaa-aaaaa-qaacq-cai";
static WORKER_CANISTER: Lazy<Principal> =
    Lazy::new(|| Principal::from_text(WORKER_CANISTER_TEXT).expect("잘못된 워커 canister ID"));

// =====================
// 1) Upload 인터페이스
// =====================

/// 업로드 요청: 바이너리 컨텐츠와 MIME 타입
#[derive(CandidType, Deserialize)]
pub struct UploadRequest {
    pub content: Vec<u8>,
    pub mime_type: String,
}

/// 업로드 응답: 각 레코드의 CBOR 직렬화 바이트 배열
#[derive(CandidType)]
pub struct UploadResponse {
    pub data: Vec<Vec<u8>>,
}

/// Upload 엔드포인트
#[update]
pub fn upload(req: UploadRequest) -> Result<UploadResponse, String> {
    // 1) 바이너리 → 파싱된 CBOR Value 벡터
    let parsed = upload::upload_data(req.content, &req.mime_type)?;
    // 2) 검증
    validation::validate_data(&parsed)?;
    // 3) 각 Value를 CBOR -> 바이트로 재직렬화
    let bytes = parsed
        .into_iter()
        .map(|v| serde_cbor::to_vec(&v).map_err(|e| format!("CBOR 직렬화 실패: {}", e)))
        // .map 자체가 Resut<Vec<u8>, _>의 return type, error의 경우 .map_err로 return을 string으로 변환
        // 따라서 여기까지의 반환 값은 Resut<Vec<u8>, String>
        .collect::<Result<Vec<_>, _>>()?;
    // _는 generic, 따라서 위 반환 값과 type이 동일
    // collect를 이용해서 모든 아이템이 Ok일 때, Vec<Vec<u8>>, 하나라도 Err일 때, Err(String type)를 즉시 반환
    Ok(UploadResponse { data: bytes })
}

// =====================
// 2) 비동기 민팅 인터페이스
// =====================

use nft::{MintRequest, MintStatus, RequestResponse};

/// 민팅 요청을 큐에 추가
#[update]
pub fn request_mint(req: MintRequest) -> RequestResponse {
    nft::request_mint_internal(req)
}

/// 민팅 요청 상태 조회
#[query]
pub fn get_mint_status(request_id: u64) -> Option<MintStatus> {
    nft::get_mint_status_internal(request_id)
}

/// 특정 토큰 정보 조회
#[query]
pub async fn get_token_info(token_id: u64) -> Option<nft::TokenInfo> {
    // worker.get_token_info(query) 호출
    let (info,): (Option<nft::TokenInfo>,) = call(*WORKER_CANISTER, "get_token_info", (token_id,))
        .await
        .unwrap_or_else(|(c, m)| panic!("worker query failed: {:?} {}", c, m));
    info
}

/// 전체 민팅된 토큰 ID 리스트 조회
#[query]
pub async fn list_tokens() -> Vec<u64> {
    // worker.list_tokens(query) 호출
    let (ids,): (Vec<u64>,) = call(*WORKER_CANISTER, "list_tokens", ())
        .await
        .unwrap_or_else(|(c, m)| panic!("worker query failed: {:?} {}", c, m));
    ids
}

// =====================
// 3) 초기화: 백그라운드 작업 예약
// =====================
use ic_cdk_timers::set_timer_interval;

#[init]
fn init() {
    // 10초마다 process_next_mint를 호출
    set_timer_interval(Duration::from_secs(10), || {
        nft::spawn_next_mint();
    });
}

// Candid 연동
use ic_cdk::export_candid;
export_candid!();

// =====================
// 4) 테스트
// =====================
#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;

    // 테스트용 더미 데이터
    fn create_test_json_data() -> Vec<u8> {
        br#"[
            {"name": "Alice", "age": 30, "city": "Seoul"},
            {"name": "Bob", "age": 25, "city": "Busan"},
            {"name": "Charlie", "age": 35, "city": "Incheon"}
        ]"#
        .to_vec()
    }

    fn create_test_csv_data() -> Vec<u8> {
        b"name,age,city\nAlice,30,Seoul\nBob,25,Busan\nCharlie,35,Incheon\n".to_vec()
    }

    #[test]
    fn test_upload_json_data() {
        let req = UploadRequest {
            content: create_test_json_data(),
            mime_type: "application/json".to_string(),
        };

        let result = upload(req);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.data.len(), 3); // 3개의 레코드
        println!("JSON 업로드 성공: {} 레코드", response.data.len());
    }

    #[test]
    fn test_upload_csv_data() {
        let req = UploadRequest {
            content: create_test_csv_data(),
            mime_type: "text/csv".to_string(),
        };

        let result = upload(req);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.data.len(), 4); // 헤더 + 3개의 데이터 레코드
        println!("CSV 업로드 성공: {} 레코드", response.data.len());
    }

    #[test]
    fn test_mint_request_flow() {
        // 1. 데이터 업로드 시뮬레이션
        let upload_req = UploadRequest {
            content: create_test_json_data(),
            mime_type: "application/json".to_string(),
        };

        let upload_result = upload(upload_req).unwrap();
        assert_eq!(upload_result.data.len(), 3);

        // 2. 각 업로드된 데이터에 대해 민팅 요청
        let test_owner = Principal::anonymous();

        for (i, data_blob) in upload_result.data.iter().enumerate() {
            let mint_req = MintRequest {
                owner: Some(test_owner),
                cid: format!("QmTest{}", i + 1), // IPFS CID 시뮬레이션
                metadata: vec![data_blob.clone()],
            };

            // 민팅 요청
            let request_response = request_mint(mint_req);
            println!("민팅 요청 ID: {}", request_response.request_id);

            // 요청 상태 확인
            let status = get_mint_status(request_response.request_id);
            assert!(status.is_some());
            assert_eq!(status.unwrap(), MintStatus::Pending);
        }
    }

    #[test]
    fn test_full_minting_workflow() {
        println!("=== 전체 민팅 워크플로우 테스트 ===");

        // 1. JSON 데이터 업로드
        println!("1. JSON 데이터 업로드 중...");
        let json_req = UploadRequest {
            content: create_test_json_data(),
            mime_type: "application/json".to_string(),
        };
        let json_result = upload(json_req).unwrap();
        println!("✓ JSON 업로드 완료: {} 레코드", json_result.data.len());

        // 2. CSV 데이터 업로드
        println!("2. CSV 데이터 업로드 중...");
        let csv_req = UploadRequest {
            content: create_test_csv_data(),
            mime_type: "text/csv".to_string(),
        };
        let csv_result = upload(csv_req).unwrap();
        println!("✓ CSV 업로드 완료: {} 레코드", csv_result.data.len());

        // 3. 민팅 요청 생성
        println!("3. 민팅 요청 생성 중...");
        let mut request_ids = Vec::new();

        // JSON 데이터 민팅
        for (i, data_blob) in json_result.data.iter().enumerate() {
            let mint_req = MintRequest {
                owner: None, // caller()를 사용
                cid: format!("QmJsonData{}", i + 1),
                metadata: vec![data_blob.clone()],
            };
            let response = request_mint(mint_req);
            request_ids.push(response.request_id);
            println!(
                "✓ JSON 데이터 {} 민팅 요청 ID: {}",
                i + 1,
                response.request_id
            );
        }

        // CSV 데이터 민팅
        for (i, data_blob) in csv_result.data.iter().enumerate() {
            let mint_req = MintRequest {
                owner: None,
                cid: format!("QmCsvData{}", i + 1),
                metadata: vec![data_blob.clone()],
            };
            let response = request_mint(mint_req);
            request_ids.push(response.request_id);
            println!(
                "✓ CSV 데이터 {} 민팅 요청 ID: {}",
                i + 1,
                response.request_id
            );
        }

        // 4. 모든 요청 상태 확인
        println!("4. 민팅 요청 상태 확인 중...");
        for request_id in request_ids {
            let status = get_mint_status(request_id);
            match status {
                Some(MintStatus::Pending) => println!("✓ 요청 {} 상태: 대기 중", request_id),
                Some(MintStatus::InProgress) => println!("✓ 요청 {} 상태: 진행 중", request_id),
                Some(MintStatus::Completed(token_id)) => {
                    println!("✓ 요청 {} 상태: 완료 (토큰 ID: {})", request_id, token_id)
                }
                Some(MintStatus::Failed(err)) => {
                    println!("✗ 요청 {} 상태: 실패 ({})", request_id, err)
                }
                None => println!("✗ 요청 {} 상태: 찾을 수 없음", request_id),
            }
        }

        println!("=== 테스트 완료 ===");
    }

    #[test]
    fn test_error_handling() {
        // 잘못된 JSON 데이터 테스트
        let invalid_json = UploadRequest {
            content: b"invalid json data".to_vec(),
            mime_type: "application/json".to_string(),
        };
        let result = upload(invalid_json);
        assert!(result.is_err());
        println!("✓ 잘못된 JSON 데이터 에러 처리 확인");

        // 지원하지 않는 MIME 타입 테스트
        let unsupported_mime = UploadRequest {
            content: b"some data".to_vec(),
            mime_type: "application/pdf".to_string(),
        };
        let result = upload(unsupported_mime);
        assert!(result.is_err());
        println!("✓ 지원하지 않는 MIME 타입 에러 처리 확인");

        // 빈 데이터 테스트
        let empty_data = UploadRequest {
            content: Vec::new(),
            mime_type: "application/json".to_string(),
        };
        let result = upload(empty_data);
        assert!(result.is_err());
        println!("✓ 빈 데이터 에러 처리 확인");
    }

    #[test]
    fn test_concurrent_minting_simulation() {
        println!("=== 동시 민팅 시뮬레이션 테스트 ===");

        let mut request_ids = Vec::new();

        // 10개의 민팅 요청을 동시에 생성
        for i in 0..10 {
            let mint_req = MintRequest {
                owner: None,
                cid: format!("QmConcurrentTest{}", i + 1),
                metadata: vec![format!("test_data_{}", i).into_bytes()],
            };

            let response = request_mint(mint_req);
            request_ids.push(response.request_id);
            println!("동시 민팅 요청 {} 생성: ID {}", i + 1, response.request_id);
        }

        // 모든 요청이 Pending 상태인지 확인
        for request_id in &request_ids {
            let status = get_mint_status(*request_id);
            assert!(matches!(status, Some(MintStatus::Pending)));
        }

        println!("✓ {} 개의 동시 민팅 요청이 큐에 추가됨", request_ids.len());
    }

    #[test]
    fn test_large_metadata_handling() {
        println!("=== 대용량 메타데이터 처리 테스트 ===");

        // 큰 JSON 데이터 생성 (1000개 레코드)
        let mut large_json = String::from("[");
        for i in 0..1000 {
            if i > 0 {
                large_json.push(',');
            }
            large_json.push_str(&format!(
                r#"{{"id": {}, "name": "User{}", "data": "large_data_chunk_{}_{}"}}"#,
                i,
                i,
                i,
                "x".repeat(100)
            ));
        }
        large_json.push(']');

        let large_req = UploadRequest {
            content: large_json.into_bytes(),
            mime_type: "application/json".to_string(),
        };

        let result = upload(large_req);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.data.len(), 1000);
        println!("✓ 대용량 데이터 처리 성공: {} 레코드", response.data.len());

        // 첫 번째 레코드로 민팅 테스트
        let mint_req = MintRequest {
            owner: None,
            cid: "QmLargeDataTest".to_string(),
            metadata: vec![response.data[0].clone()],
        };

        let mint_response = request_mint(mint_req);
        println!(
            "✓ 대용량 메타데이터 민팅 요청 ID: {}",
            mint_response.request_id
        );

        let status = get_mint_status(mint_response.request_id);
        assert!(matches!(status, Some(MintStatus::Pending)));
    }
}

// 통합 테스트 실행을 위한 헬퍼 함수들
#[cfg(test)]
mod integration_helpers {

    /// 테스트용 샘플 데이터 생성기
    pub struct TestDataGenerator;

    impl TestDataGenerator {
        pub fn generate_user_data(count: usize) -> Vec<u8> {
            let mut json = String::from("[");
            for i in 0..count {
                if i > 0 {
                    json.push(',');
                }
                json.push_str(&format!(
                    r#"{{"id": {}, "name": "User{}", "email": "user{}@example.com", "age": {}}}"#,
                    i,
                    i,
                    i,
                    20 + (i % 50)
                ));
            }
            json.push(']');
            json.into_bytes()
        }

        pub fn generate_product_data(count: usize) -> Vec<u8> {
            let mut csv = String::from("id,name,price,category\n");
            for i in 0..count {
                csv.push_str(&format!(
                    "{},Product{},{},Category{}\n",
                    i,
                    i,
                    10 + (i % 100),
                    i % 5
                ));
            }
            csv.into_bytes()
        }
    }
}
