#!/bin/bash
set -e

echo "=== IC Environment Integration Test ==="

# # 1. IC 환경 시작
# echo "Starting IC environment..."
# dfx start --clean --background

# # 2. Backend 배포
# echo "Deploying backend..."
# dfx deploy backend

# 3. 간단한 JSON 테스트
echo "Testing simple JSON upload..."
dfx canister call backend upload '(record {
  content = vec {123; 34; 110; 97; 109; 101; 34; 58; 34; 74; 111; 104; 110; 34; 125};
  mime_type = "application/json"
})'

# 4. CSV 테스트
echo "Testing CSV upload..."
dfx canister call backend upload '(record {
  content = vec {110; 97; 109; 101; 44; 97; 103; 101; 10; 74; 111; 104; 110; 44; 50; 56; 10};
  mime_type = "text/csv"
})'

# 5. 민팅 테스트
echo "Testing minting..."
dfx canister call backend request_mint '(record {
  owner = opt principal "'$(dfx identity get-principal)'";
  cid = "QmTest123";
  metadata = vec {vec {116; 101; 115; 116}}
})'

# 6. 상태 확인
echo "Checking results..."
dfx canister call backend get_storage_stats
dfx canister call backend list_uploaded_data
dfx canister call backend get_mint_status '(1)'

echo "=== Test Completed ==="