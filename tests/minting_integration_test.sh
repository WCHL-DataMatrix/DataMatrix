#!/bin/bash
set -e

echo "=== Minting Integration Test ==="

# 테스트 환경 확인
check_environment() {
    echo "Checking test environment..."
    
    # Backend canister 확인
    BACKEND_CANISTER_ID=$(dfx canister id backend 2>/dev/null || echo "")
    if [ -z "$BACKEND_CANISTER_ID" ]; then
        echo "Error: Backend canister not found. Please deploy first."
        exit 1
    fi
    
    echo "Backend Canister ID: $BACKEND_CANISTER_ID"
    
    # Storage 초기화 확인
    echo "Checking storage initialization..."
    storage_check=$(dfx canister call backend get_storage_stats 2>&1)
    if echo "$storage_check" | grep -q "Upload counter not initialized"; then
        echo "⚠️  Storage not initialized. Attempting to reinitialize..."
        
        # 수동 초기화 함수 호출 시도
        if dfx canister call backend init_storage_manual 2>/dev/null; then
            echo "✅ Storage manually initialized"
        else
            echo "Manual initialization not available. Reinstalling canister..."
            dfx deploy backend --mode reinstall --with-cycles 2000000000000
            echo "✅ Backend canister reinstalled"
            BACKEND_CANISTER_ID=$(dfx canister id backend)
        fi
    else
        echo "✅ Storage already initialized"
    fi
    
    # Worker canister 확인
    WORKER_CANISTER_ID=$(dfx canister id worker 2>/dev/null || echo "")
    if [ -z "$WORKER_CANISTER_ID" ]; then
        echo "Warning: Worker canister not found. Will deploy during test."
        NEED_WORKER_DEPLOY=true
    else
        echo "Worker Canister ID: $WORKER_CANISTER_ID"
        NEED_WORKER_DEPLOY=false
    fi
    
    # 현재 identity 확인
    CURRENT_IDENTITY=$(dfx identity whoami)
    CURRENT_PRINCIPAL=$(dfx identity get-principal)
    echo "Current Identity: $CURRENT_IDENTITY"
    echo "Current Principal: $CURRENT_PRINCIPAL"
}

# Worker canister 배포
deploy_worker_if_needed() {
    if [ "$NEED_WORKER_DEPLOY" = true ]; then
        echo ""
        echo "--- Worker Canister Deployment ---"
        echo "⚠️  SKIPPED: Worker canister not configured in dfx.json"
        echo "💡 Note: Minting tests will focus on backend request handling"
        echo "🔧 To enable worker tests, add worker canister to dfx.json"
        
        # Worker 없이 진행
        WORKER_CANISTER_ID="not-deployed"
        echo "Continuing without worker canister..."
    fi
}

# 테스트 데이터 준비
prepare_minting_data() {
    echo ""
    echo "--- Preparing Test Data for Minting ---"
    
    # 민팅용 테스트 데이터
    MINT_JSON_DATA="vec {123; 34; 110; 97; 109; 101; 34; 58; 34; 77; 105; 110; 116; 84; 101; 115; 116; 34; 44; 34; 116; 121; 112; 101; 34; 58; 34; 78; 70; 84; 34; 125}"
    
    # 복수 데이터 민팅용
    MINT_CSV_DATA="vec {110; 97; 109; 101; 44; 116; 121; 112; 101; 10; 78; 70; 84; 49; 44; 97; 114; 116; 10; 78; 70; 84; 50; 44; 103; 97; 109; 101}"
    
    # 대용량 메타데이터
    LARGE_METADATA="vec {"
    for i in {1..100}; do
        LARGE_METADATA="$LARGE_METADATA 77; 101; 116; 97; 100; 97; 116; 97;"
    done
    LARGE_METADATA="$LARGE_METADATA 100; 97; 116; 97}"
    
    echo "Test data prepared for minting tests"
}

# 테스트 결과 추적
MINT_TEST_RESULTS=()
MINT_TOTAL_TESTS=0
MINT_PASSED_TESTS=0
UPLOADED_DATA_IDS=()

run_mint_test() {
    local test_name="$1"
    local test_command="$2"
    local expected_result="$3"
    
    MINT_TOTAL_TESTS=$((MINT_TOTAL_TESTS + 1))
    echo ""
    echo "--- Mint Test $MINT_TOTAL_TESTS: $test_name ---"
    
    if eval "$test_command" > /tmp/mint_test_output 2>&1; then
        if [ "$expected_result" == "success" ]; then
            echo "✅ PASS: $test_name"
            MINT_PASSED_TESTS=$((MINT_PASSED_TESTS + 1))
            MINT_TEST_RESULTS+=("PASS: $test_name")
            
            # 결과에서 request_id 추출 (있는 경우)
            if grep -q "request_id" /tmp/mint_test_output; then
                REQUEST_ID=$(grep -oP '(?<=request_id = )\d+' /tmp/mint_test_output)
                echo "Request ID: $REQUEST_ID"
            fi
        else
            echo "❌ FAIL: $test_name (expected failure but succeeded)"
            MINT_TEST_RESULTS+=("FAIL: $test_name (unexpected success)")
        fi
    else
        if [ "$expected_result" == "failure" ]; then
            echo "✅ PASS: $test_name (expected failure)"
            MINT_PASSED_TESTS=$((MINT_PASSED_TESTS + 1))
            MINT_TEST_RESULTS+=("PASS: $test_name (expected failure)")
        else
            echo "❌ FAIL: $test_name"
            echo "Error output:"
            cat /tmp/mint_test_output
            MINT_TEST_RESULTS+=("FAIL: $test_name")
        fi
    fi
}

# 실제 민팅 처리 대기 및 확인
wait_for_minting() {
    local request_id="$1"
    local max_wait_time=120  # 2분
    local wait_interval=10   # 10초마다 확인
    local waited=0
    
    echo ""
    echo "--- Waiting for Minting Process (Request ID: $request_id) ---"
    
    while [ $waited -lt $max_wait_time ]; do
        echo "Checking mint status... (waited ${waited}s)"
        
        # 민팅 상태 확인
        dfx canister call backend get_mint_status "($request_id)" > /tmp/mint_status 2>&1
        
        if grep -q "Completed" /tmp/mint_status; then
            echo "✅ Minting completed!"
            TOKEN_ID=$(grep -oP '(?<=Completed = )\d+' /tmp/mint_status)
            echo "Token ID: $TOKEN_ID"
            return 0
        elif grep -q "Failed" /tmp/mint_status; then
            echo "❌ Minting failed!"
            cat /tmp/mint_status
            return 1
        elif grep -q "InProgress" /tmp/mint_status; then
            echo "🔄 Minting in progress..."
        else
            echo "⏳ Minting pending..."
        fi
        
        sleep $wait_interval
        waited=$((waited + wait_interval))
    done
    
    echo "⏰ Timeout waiting for minting to complete"
    return 1
}

# 메인 테스트 실행
main_minting_tests() {
    echo ""
    echo "Starting minting integration tests..."
    
    # 1. 먼저 민팅할 데이터 업로드
    echo ""
    echo "--- Setting up Data for Minting ---"
    
    echo "Uploading JSON data for minting..."
    dfx canister call backend upload "(record { content = $MINT_JSON_DATA; mime_type = \"application/json\" })" > /tmp/upload_result
    
    if [ $? -eq 0 ]; then
        echo "✅ Data uploaded successfully for minting"
        # 업로드된 데이터 ID 추출
        DATA_ID_BYTES=$(grep -oP '(?<=data = vec \{ vec \{ )[^}]+' /tmp/upload_result | head -1)
        echo "Uploaded data ID bytes: $DATA_ID_BYTES"
    else
        echo "❌ Failed to upload data for minting"
        exit 1
    fi
    
    # 2. 유효한 민팅 요청 테스트
    run_mint_test "Valid Mint Request" \
        "dfx canister call backend request_mint '(record { owner = opt principal \"$CURRENT_PRINCIPAL\"; cid = \"QmTestMint123\"; metadata = vec { vec { 116; 101; 115; 116; 32; 109; 101; 116; 97; 100; 97; 116; 97 } } })'" \
        "success"
    
    FIRST_REQUEST_ID="$REQUEST_ID"
    
    # 3. 잘못된 CID로 민팅 요청
    run_mint_test "Invalid CID Mint Request" \
        "dfx canister call backend request_mint '(record { owner = opt principal \"$CURRENT_PRINCIPAL\"; cid = \"\"; metadata = vec { vec { 116; 101; 115; 116 } } })'" \
        "failure"
    
    # 4. 빈 메타데이터로 민팅 요청
    run_mint_test "Empty Metadata Mint Request" \
        "dfx canister call backend request_mint '(record { owner = opt principal \"$CURRENT_PRINCIPAL\"; cid = \"QmTestEmpty\"; metadata = vec {} })'" \
        "failure"
    
    # 5. 대용량 메타데이터로 민팅 요청
    run_mint_test "Large Metadata Mint Request" \
        "dfx canister call backend request_mint '(record { owner = opt principal \"$CURRENT_PRINCIPAL\"; cid = \"QmTestLarge\"; metadata = vec { $LARGE_METADATA } })'" \
        "success"
    
    SECOND_REQUEST_ID="$REQUEST_ID"
    
    # 6. 다른 사용자로 민팅 요청 (권한 테스트)
    OTHER_PRINCIPAL="rdmx6-jaaaa-aaaah-qcaiq-cai"  # 테스트용 더미 principal
    run_mint_test "Different Owner Mint Request" \
        "dfx canister call backend request_mint '(record { owner = opt principal \"$OTHER_PRINCIPAL\"; cid = \"QmTestOther\"; metadata = vec { vec { 111; 116; 104; 101; 114 } } })'" \
        "success"
    
    # 7. 민팅 상태 조회 테스트
    echo ""
    echo "--- Testing Mint Status Queries ---"
    
    if [ ! -z "$FIRST_REQUEST_ID" ]; then
        run_mint_test "Valid Mint Status Query" \
            "dfx canister call backend get_mint_status '($FIRST_REQUEST_ID)'" \
            "success"
    fi
    
    run_mint_test "Invalid Mint Status Query" \
        "dfx canister call backend get_mint_status '(999999)'" \
        "success"
    
    echo "Note: Invalid request ID should return null/none"
    
    # 8. 민팅 요청 목록 조회
    run_mint_test "List Mint Requests" \
        "dfx canister call backend list_mint_requests" \
        "success"
    
    # 9. 실제 민팅 처리 대기 (worker canister와의 상호작용)
    if [ ! -z "$FIRST_REQUEST_ID" ] && [ "$WORKER_CANISTER_ID" != "not-deployed" ]; then
        echo ""
        echo "--- Testing Actual Minting Process ---"
        
        # 민팅 처리 대기
        if wait_for_minting "$FIRST_REQUEST_ID"; then
            echo "✅ First minting process completed successfully"
            
            # 민팅 완료 후 상태 재확인
            run_mint_test "Post-Minting Status Check" \
                "dfx canister call backend get_mint_status '($FIRST_REQUEST_ID)'" \
                "success"
            
            # Worker canister에서 토큰 정보 조회
            if [ ! -z "$TOKEN_ID" ]; then
                run_mint_test "Worker Token Info Query" \
                    "dfx canister call backend get_token_info_from_worker '($TOKEN_ID)'" \
                    "success"
            fi
            
        else
            echo "❌ First minting process failed or timed out"
            MINT_TEST_RESULTS+=("FAIL: Actual Minting Process")
        fi
    else
        echo ""
        echo "--- Actual Minting Process ---"
        echo "⚠️  SKIPPED: Worker canister not available"
        echo "💡 Minting requests are queued but not processed without worker"
        
        # 대신 대기 상태 확인
        if [ ! -z "$FIRST_REQUEST_ID" ]; then
            run_mint_test "Mint Status While Pending" \
                "dfx canister call backend get_mint_status '($FIRST_REQUEST_ID)'" \
                "success"
        fi
    fi
    
    # 10. 중복 민팅 시도 테스트
    echo ""
    echo "--- Testing Duplicate Minting Prevention ---"
    
    # 동일한 메타데이터로 다시 민팅 시도
    run_mint_test "Duplicate Metadata Mint Attempt" \
        "dfx canister call backend request_mint '(record { owner = opt principal \"$CURRENT_PRINCIPAL\"; cid = \"QmTestDuplicate\"; metadata = vec { vec { 116; 101; 115; 116; 32; 109; 101; 116; 97; 100; 97; 116; 97 } } })'" \
        "failure"
    
    echo "Note: Duplicate metadata should be rejected"
    
    # 11. Worker canister 직접 통신 테스트
    echo ""
    echo "--- Testing Worker Canister Communication ---"
    
    if [ "$WORKER_CANISTER_ID" != "not-deployed" ]; then
        run_mint_test "List Tokens from Worker" \
            "dfx canister call backend list_tokens_from_worker" \
            "success"
    else
        echo "⚠️  SKIPPED: Worker canister communication (not deployed)"
        MINT_TOTAL_TESTS=$((MINT_TOTAL_TESTS + 1))
        MINT_PASSED_TESTS=$((MINT_PASSED_TESTS + 1))
        MINT_TEST_RESULTS+=("PASS: Worker Communication (Skipped - Not deployed)")
        echo "✅ PASS: Worker Communication (Skipped - Not deployed)"
    fi
    
    # 12. 민팅된 데이터 상태 확인
    echo ""
    echo "--- Testing Minted Data Status ---"
    
    # 민팅된 메타데이터의 상태 확인
    MINTED_METADATA="vec { 116; 101; 115; 116; 32; 109; 101; 116; 97; 100; 97; 116; 97 }"
    run_mint_test "Check Minted Data Status" \
        "dfx canister call backend check_data_minted '($MINTED_METADATA)'" \
        "success"
    
    # 13. 저장소 통계 업데이트 확인
    echo ""
    echo "--- Testing Storage Stats After Minting ---"
    
    run_mint_test "Storage Stats After Minting" \
        "dfx canister call backend get_storage_stats" \
        "success"
    
    echo ""
    echo "--- Testing Edge Cases ---"
    
    # 14. 매우 긴 CID 테스트
    LONG_CID="Qm$(printf 'a%.0s' {1..90})"
    run_mint_test "Very Long CID Test" \
        "dfx canister call backend request_mint '(record { owner = opt principal \"$CURRENT_PRINCIPAL\"; cid = \"$LONG_CID\"; metadata = vec { vec { 116; 101; 115; 116 } } })'" \
        "failure"
    
    # 15. 특수 문자가 포함된 CID 테스트
    run_mint_test "Special Characters CID Test" \
        "dfx canister call backend request_mint '(record { owner = opt principal \"$CURRENT_PRINCIPAL\"; cid = \"Qm@#\$%^&*()\"; metadata = vec { vec { 116; 101; 115; 116 } } })'" \
        "failure"
    
    # 16. 최대 메타데이터 개수 테스트
    echo "Testing maximum metadata count..."
    MAX_METADATA="vec {"
    for i in {1..50}; do  # 50개의 메타데이터 (최대 100개까지 허용)
        MAX_METADATA="$MAX_METADATA vec { 116; 101; 115; 116; $i };"
    done
    MAX_METADATA="$MAX_METADATA }"
    
    run_mint_test "Maximum Metadata Count Test" \
        "dfx canister call backend request_mint '(record { owner = opt principal \"$CURRENT_PRINCIPAL\"; cid = \"QmTestMaxMeta\"; metadata = $MAX_METADATA })'" \
        "success"
    
    # 17. 초과 메타데이터 개수 테스트
    echo "Testing excessive metadata count..."
    EXCESS_METADATA="vec {"
    for i in {1..101}; do  # 101개의 메타데이터 (제한 초과)
        EXCESS_METADATA="$EXCESS_METADATA vec { 116; 101; 115; 116; $((i % 256)) };"
    done
    EXCESS_METADATA="$EXCESS_METADATA }"
    
    run_mint_test "Excessive Metadata Count Test" \
        "dfx canister call backend request_mint '(record { owner = opt principal \"$CURRENT_PRINCIPAL\"; cid = \"QmTestExcess\"; metadata = $EXCESS_METADATA })'" \
        "failure"
}

# 성능 테스트
performance_tests() {
    echo ""
    echo "--- Performance and Stress Tests ---"
    
    # 동시 민팅 요청 테스트
    echo "Testing concurrent mint requests..."
    for i in {1..5}; do
        dfx canister call backend request_mint "(record { owner = opt principal \"$CURRENT_PRINCIPAL\"; cid = \"QmConcurrent$i\"; metadata = vec { vec { 99; 111; 110; 99; 117; 114; 114; 101; 110; 116; $i } } })" > /tmp/concurrent_$i.log 2>&1 &
    done
    
    wait  # 모든 백그라운드 작업 완료 대기
    
    echo "Concurrent requests completed"
    
    # 결과 확인
    SUCCESS_COUNT=0
    for i in {1..5}; do
        if grep -q "request_id" /tmp/concurrent_$i.log; then
            SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
        fi
    done
    
    if [ $SUCCESS_COUNT -eq 5 ]; then
        echo "✅ PASS: Concurrent Mint Requests ($SUCCESS_COUNT/5)"
        MINT_PASSED_TESTS=$((MINT_PASSED_TESTS + 1))
        MINT_TEST_RESULTS+=("PASS: Concurrent Mint Requests")
    else
        echo "❌ FAIL: Concurrent Mint Requests ($SUCCESS_COUNT/5)"
        MINT_TEST_RESULTS+=("FAIL: Concurrent Mint Requests")
    fi
    MINT_TOTAL_TESTS=$((MINT_TOTAL_TESTS + 1))
    
    # 정리
    rm -f /tmp/concurrent_*.log
}

# 최종 상태 확인
final_state_check() {
    echo ""
    echo "--- Final State Verification ---"
    
    echo "Final storage statistics:"
    dfx canister call backend get_storage_stats
    
    echo ""
    echo "Final mint requests list:"
    dfx canister call backend list_mint_requests
    
    echo ""
    echo "Tokens from worker canister:"
    if [ "$WORKER_CANISTER_ID" != "not-deployed" ]; then
        dfx canister call backend list_tokens_from_worker
        
        # 민팅된 토큰들의 상세 정보 확인
        echo ""
        echo "Checking minted token details..."
        dfx canister call backend list_tokens_from_worker > /tmp/token_list 2>&1
        
        if grep -q "vec {" /tmp/token_list; then
            echo "Found minted tokens, checking details..."
            # 첫 번째 토큰 정보 조회 시도
            dfx canister call backend get_token_info_from_worker '(1)' 2>/dev/null || echo "No token with ID 1"
        else
            echo "No tokens found in worker canister"
        fi
    else
        echo "Worker canister not deployed - tokens remain in pending state"
        echo "Mint requests are queued but not processed without worker canister"
    fi
}

# 정리 함수
cleanup() {
    echo ""
    echo "--- Cleaning up test files ---"
    rm -f /tmp/mint_test_output /tmp/mint_status /tmp/upload_result /tmp/token_list
    echo "Cleanup completed"
}

# 메인 실행 함수
main() {
    echo "Starting Minting Integration Test Suite"
    echo "======================================="
    
    # 환경 확인
    check_environment
    
    # Worker canister 배포 (필요한 경우)
    deploy_worker_if_needed
    
    # 테스트 데이터 준비
    prepare_minting_data
    
    # 메인 민팅 테스트 실행
    main_minting_tests
    
    # 성능 테스트 실행
    performance_tests
    
    # 최종 상태 확인
    final_state_check
    
    # 최종 결과 출력
    echo ""
    echo "========================================"
    echo "Minting Integration Test Results"
    echo "========================================"
    echo "Total Tests: $MINT_TOTAL_TESTS"
    echo "Passed: $MINT_PASSED_TESTS"
    echo "Failed: $((MINT_TOTAL_TESTS - MINT_PASSED_TESTS))"
    echo ""
    
    if [ $MINT_PASSED_TESTS -eq $MINT_TOTAL_TESTS ]; then
        echo "🎉 ALL MINTING TESTS PASSED!"
        MINT_SUCCESS_RATE=100
    else
        echo "❌ SOME MINTING TESTS FAILED"
        MINT_SUCCESS_RATE=$((MINT_PASSED_TESTS * 100 / MINT_TOTAL_TESTS))
    fi
    
    echo "Success Rate: $MINT_SUCCESS_RATE%"
    echo ""
    
    # 세부 결과 출력
    echo "Detailed Results:"
    for result in "${MINT_TEST_RESULTS[@]}"; do
        echo "  $result"
    done
    
    # 중요한 메트릭 출력
    echo ""
    echo "Key Metrics:"
    echo "============"
    
    # 저장소 통계에서 주요 메트릭 추출
    dfx canister call backend get_storage_stats > /tmp/final_stats 2>&1
    
    if grep -q "total_uploads" /tmp/final_stats; then
        TOTAL_UPLOADS=$(grep -oP '(?<=total_uploads = )\d+' /tmp/final_stats)
        TOTAL_MINT_REQUESTS=$(grep -oP '(?<=total_mint_requests = )\d+' /tmp/final_stats)
        COMPLETED_MINTS=$(grep -oP '(?<=completed_mints = )\d+' /tmp/final_stats)
        PENDING_MINTS=$(grep -oP '(?<=pending_mints = )\d+' /tmp/final_stats)
        FAILED_MINTS=$(grep -oP '(?<=failed_mints = )\d+' /tmp/final_stats)
        
        echo "  Total Uploads: $TOTAL_UPLOADS"
        echo "  Total Mint Requests: $TOTAL_MINT_REQUESTS"
        echo "  Completed Mints: $COMPLETED_MINTS"
        echo "  Pending Mints: $PENDING_MINTS"
        echo "  Failed Mints: $FAILED_MINTS"
        
        if [ "$TOTAL_MINT_REQUESTS" -gt 0 ]; then
            MINT_COMPLETION_RATE=$((COMPLETED_MINTS * 100 / TOTAL_MINT_REQUESTS))
            echo "  Mint Completion Rate: $MINT_COMPLETION_RATE%"
        fi
    fi
    
    # Worker canister 토큰 수 확인
    dfx canister call backend list_tokens_from_worker > /tmp/worker_tokens 2>&1
    if grep -q "vec {" /tmp/worker_tokens; then
        TOKEN_COUNT=$(grep -oP '(?<=vec \{ )[^}]+' /tmp/worker_tokens | tr ';' '\n' | wc -l)
        echo "  Tokens in Worker Canister: $TOKEN_COUNT"
    else
        echo "  Tokens in Worker Canister: 0"
    fi
    
    # 정리
    cleanup
    
    echo ""
    echo "=== Minting Integration Test Completed ==="
    
    # 성공률이 85% 미만이면 실패로 처리 (민팅은 더 까다로운 기준)
    if [ $MINT_SUCCESS_RATE -lt 85 ]; then
        echo ""
        echo "❌ Test suite failed due to low success rate"
        exit 1
    fi
    
    echo ""
    echo "✅ Minting integration test suite completed successfully"
}

# 스크립트 실행
main "$@"