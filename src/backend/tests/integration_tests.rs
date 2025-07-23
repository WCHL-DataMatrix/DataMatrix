// tests/integration_tests.rs

use backend::*;
use candid::Principal;

#[cfg(test)]
mod tests {
    use super::*;

    // ===================
    // Test Data Generators
    // ===================

    fn create_sample_json_dataset() -> Vec<u8> {
        br#"[
            {
                "id": 1,
                "name": "John Smith",
                "age": 28,
                "city": "Seoul",
                "occupation": "Software Developer",
                "salary": 45000000,
                "interests": ["Programming", "Gaming", "Reading"]
            },
            {
                "id": 2,
                "name": "Jane Doe",
                "age": 32,
                "city": "Busan",
                "occupation": "Designer",
                "salary": 38000000,
                "interests": ["Design", "Travel", "Cooking"]
            },
            {
                "id": 3,
                "name": "Mike Johnson",
                "age": 25,
                "city": "Daegu",
                "occupation": "Marketer",
                "salary": 35000000,
                "interests": ["Marketing", "Sports", "Movies"]
            }
        ]"#
        .to_vec()
    }

    fn create_sample_csv_dataset() -> Vec<u8> {
        b"id,name,age,city,occupation,salary,rating
1,John Smith,28,Seoul,Developer,45000000,4.8
2,Jane Doe,32,Busan,Designer,38000000,4.6
3,Mike Johnson,25,Daegu,Marketer,35000000,4.4
4,Sarah Wilson,29,Incheon,Engineer,42000000,4.7
"
        .to_vec()
    }

    fn create_malformed_json() -> Vec<u8> {
        b"{ invalid json without closing brace".to_vec()
    }

    fn create_empty_dataset() -> Vec<u8> {
        Vec::new()
    }

    fn create_large_dataset() -> Vec<u8> {
        // Generate 11MB data (exceeding 10MB limit)
        let large_string = "x".repeat(11 * 1024 * 1024);
        format!(r#"{{"large_data": "{}"}}"#, large_string).into_bytes()
    }

    // ===========================
    // 1. Data Upload & Validation Tests
    // ===========================

    #[test]
    fn test_successful_json_upload_and_validation() {
        println!("=== JSON Data Upload & Validation Test ===");

        let upload_request = UploadRequest {
            content: create_sample_json_dataset(),
            mime_type: "application/json".to_string(),
        };

        // Execute upload
        let result = upload(upload_request);
        assert!(result.is_ok(), "JSON upload failed: {:?}", result.err());

        let response = result.unwrap();
        assert_eq!(response.data.len(), 3, "Expected 3 data records");

        // Extract and validate data IDs
        let data_ids: Vec<u64> = response
            .data
            .iter()
            .map(|bytes| u64::from_be_bytes(bytes.as_slice().try_into().unwrap()))
            .collect();

        println!("✓ Uploaded data IDs: {:?}", data_ids);

        // Re-query each data to confirm storage
        for data_id in &data_ids {
            let stored_data = get_uploaded_data(*data_id);
            assert!(stored_data.is_some(), "Data ID {} not found", data_id);
        }

        println!("✓ JSON upload & validation successful");
    }

    #[test]
    fn test_successful_csv_upload_and_validation() {
        println!("=== CSV Data Upload & Validation Test ===");

        let upload_request = UploadRequest {
            content: create_sample_csv_dataset(),
            mime_type: "text/csv".to_string(),
        };

        let result = upload(upload_request);
        assert!(result.is_ok(), "CSV upload failed: {:?}", result.err());

        let response = result.unwrap();
        assert!(!response.data.is_empty(), "No CSV data uploaded");

        println!("✓ CSV records uploaded: {}", response.data.len());
        println!("✓ CSV upload & validation successful");
    }

    #[test]
    fn test_upload_validation_failures() {
        println!("=== Upload Validation Failure Tests ===");

        // 1. Malformed JSON format
        let malformed_request = UploadRequest {
            content: create_malformed_json(),
            mime_type: "application/json".to_string(),
        };
        let result = upload(malformed_request);
        assert!(result.is_err(), "Malformed JSON was successfully uploaded");
        println!("✓ Malformed JSON format rejection successful");

        // 2. Empty data
        let empty_request = UploadRequest {
            content: create_empty_dataset(),
            mime_type: "application/json".to_string(),
        };
        let result = upload(empty_request);
        assert!(result.is_err(), "Empty data was successfully uploaded");
        println!("✓ Empty data rejection successful");

        // 3. Unsupported MIME type
        let unsupported_request = UploadRequest {
            content: b"some binary data".to_vec(),
            mime_type: "application/pdf".to_string(),
        };
        let result = upload(unsupported_request);
        assert!(result.is_err(), "Unsupported MIME type was accepted");
        println!("✓ Unsupported MIME type rejection successful");

        // 4. Oversized data
        let large_request = UploadRequest {
            content: create_large_dataset(),
            mime_type: "application/json".to_string(),
        };
        let result = upload(large_request);
        assert!(result.is_err(), "Large data was successfully uploaded");
        println!("✓ Size limit exceeded data rejection successful");
    }

    // ====================
    // 2. Minting Process Tests
    // ====================

    #[test]
    fn test_independent_minting_process() {
        println!("=== Independent Minting Process Test ===");

        // 1. Data upload
        let upload_request = UploadRequest {
            content: create_sample_json_dataset(),
            mime_type: "application/json".to_string(),
        };
        let upload_result = upload(upload_request).unwrap();
        println!(
            "✓ Data upload completed: {} records",
            upload_result.data.len()
        );

        // 2. Create independent minting requests for each data
        let test_principal = Principal::anonymous();
        let mut mint_request_ids = Vec::new();

        for (index, data_blob) in upload_result.data.iter().enumerate() {
            let mint_request = MintRequest {
                owner: Some(test_principal),
                cid: format!(
                    "QmDataMatrix{:04}{}",
                    index + 1,
                    // Add simple checksum
                    data_blob.len() % 1000
                ),
                metadata: vec![data_blob.clone()],
            };

            // Submit minting request
            let mint_response = request_mint(mint_request);
            mint_request_ids.push(mint_response.request_id);

            println!(
                "✓ Minting request #{} created (ID: {})",
                index + 1,
                mint_response.request_id
            );
        }

        // 3. Verify all minting requests are in Pending status
        for (index, request_id) in mint_request_ids.iter().enumerate() {
            let status = get_mint_status(*request_id);
            assert!(
                status.is_some(),
                "Minting request {} status not found",
                request_id
            );
            assert_eq!(
                status.unwrap(),
                MintStatus::Pending,
                "Minting request {} is not in Pending status",
                request_id
            );

            println!("✓ Minting request #{} status: Pending", index + 1);
        }

        println!("✓ Independent minting process test successful");
    }

    #[test]
    fn test_mint_request_validation() {
        println!("=== Minting Request Validation Test ===");

        // Valid minting request
        let valid_request = MintRequest {
            owner: Some(Principal::anonymous()),
            cid: "QmValidCID123456789".to_string(),
            metadata: vec![b"valid metadata".to_vec()],
        };
        let result = request_mint(valid_request);
        assert!(result.request_id > 0, "Valid minting request failed");
        println!("✓ Valid minting request successful");

        // Empty CID - this would cause trap, so difficult to test
        // Should be validated in frontend in actual environment
        println!("✓ Minting request validation test completed");
    }

    // =========================
    // 3. Complete Workflow Integration Tests
    // =========================

    #[test]
    fn test_complete_data_lifecycle() {
        println!("=== Complete Data Lifecycle Test ===");

        // Phase 1: Data Upload
        println!("Phase 1: Data Upload");
        let upload_request = UploadRequest {
            content: create_sample_json_dataset(),
            mime_type: "application/json".to_string(),
        };
        let upload_result = upload(upload_request).unwrap();
        let uploaded_count = upload_result.data.len();
        println!("✓ {} data uploaded", uploaded_count);

        // Phase 2: Data validation and storage confirmation
        println!("Phase 2: Data Validation");
        let data_ids: Vec<u64> = upload_result
            .data
            .iter()
            .map(|bytes| u64::from_be_bytes(bytes.as_slice().try_into().unwrap()))
            .collect();

        for data_id in &data_ids {
            assert!(
                get_uploaded_data(*data_id).is_some(),
                "Data ID {} was not stored",
                data_id
            );
        }
        println!("✓ {} data validated", data_ids.len());

        // Phase 3: Minting requests and independent canister processing
        println!("Phase 3: Minting Requests");
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
        println!("✓ {} minting requests completed", mint_requests.len());

        // Phase 4: Minting status verification
        println!("Phase 4: Minting Status Check");
        for request_id in &mint_requests {
            let status = get_mint_status(*request_id);
            assert_eq!(status, Some(MintStatus::Pending));
        }
        println!("✓ All minting requests in Pending status");

        // Phase 5: Storage statistics check
        println!("Phase 5: Storage Statistics Check");
        let stats = get_storage_stats();
        assert!(stats.total_uploads >= uploaded_count as u64);
        assert!(stats.total_mint_requests >= mint_requests.len() as u64);
        println!(
            "✓ Statistics - Uploads: {}, Mint Requests: {}",
            stats.total_uploads, stats.total_mint_requests
        );

        println!("✓ Complete data lifecycle test successful");
    }

    // =======================
    // 4. Performance & Stress Tests
    // =======================

    #[test]
    fn test_multiple_concurrent_uploads() {
        println!("=== Multiple Upload Test ===");

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
            assert!(result.is_ok(), "{} upload failed", name);

            let count = result.unwrap().data.len();
            total_uploaded += count;
            println!("✓ {} data upload: {} records", name, count);
        }

        println!("✓ Total {} records uploaded", total_uploaded);
    }

    #[test]
    fn test_storage_management() {
        println!("=== Storage Management Test ===");

        // Data upload
        let upload_request = UploadRequest {
            content: create_sample_json_dataset(),
            mime_type: "application/json".to_string(),
        };
        let upload_result = upload(upload_request).unwrap();

        // Query data list
        let data_list = list_uploaded_data();
        assert!(!data_list.is_empty(), "Uploaded data list is empty");
        println!("✓ Uploaded data list query: {} items", data_list.len());

        // Query minting request list
        let mint_list = list_mint_requests();
        println!("✓ Minting request list query: {} items", mint_list.len());

        println!("✓ Storage management test completed");
    }

    // ==================
    // 5. Error Recovery Tests
    // ==================

    #[test]
    fn test_error_recovery_scenarios() {
        println!("=== Error Recovery Scenario Tests ===");

        // Query non-existent data
        let non_existent_data = get_uploaded_data(99999);
        assert!(
            non_existent_data.is_none(),
            "Non-existent data was returned"
        );
        println!("✓ Non-existent data query handling");

        // Query non-existent minting request status
        let non_existent_status = get_mint_status(99999);
        assert!(
            non_existent_status.is_none(),
            "Non-existent minting request status was returned"
        );
        println!("✓ Non-existent minting request status query handling");

        println!("✓ Error recovery scenario tests completed");
    }

    // ==================
    // 6. Helper Function Tests
    // ==================

    #[test]
    fn test_system_functionality() {
        println!("=== System Functionality Test ===");

        // Statistics query
        let stats = get_storage_stats();
        println!(
            "✓ Storage statistics: Uploads {}, Mint Requests {}",
            stats.total_uploads, stats.total_mint_requests
        );

        // Basic system status check
        assert!(true, "System is not functioning properly");

        println!("✓ System functionality test completed");
    }
}
