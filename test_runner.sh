#!/bin/bash
# test_runner.sh - DataMatrix 프로젝트 테스트 실행 스크립트

set -e

echo "=========================================="
echo "DataMatrix 프로젝트 테스트 실행"
echo "=========================================="

# 프로젝트 루트로 이동
cd "$(dirname "$0")"

# 색상 정의
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 로그 함수
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 1. 환경 확인
log_info "환경 확인 중..."
if ! command -v cargo &> /dev/null; then
    log_error "Cargo가 설치되어 있지 않습니다."
    exit 1
fi

if ! command -v dfx &> /dev/null; then
    log_warn "DFX가 설치되어 있지 않습니다. IC 관련 기능은 제한될 수 있습니다."
fi

# 2. 의존성 확인
log_info "의존성 확인 중..."
cd src/backend
cargo check

# 3. 단위 테스트 실행
log_info "단위 테스트 실행 중..."
echo "----------------------------------------"
echo "모듈별 단위 테스트"
echo "----------------------------------------"

# validation 모듈 테스트
log_info "Validation 모듈 테스트..."
cargo test validation::tests --lib -- --nocapture

# upload 모듈 테스트
log_info "Upload 모듈 테스트..."
cargo test upload::tests --lib -- --nocapture

# 4. 통합 테스트 실행
log_info "통합 테스트 실행 중..."
echo "----------------------------------------"
echo "통합 테스트"
echo "----------------------------------------"

# 개별 테스트 그룹 실행
log_info "1. 데이터 업로드 & 검증 테스트"
cargo test test_successful_json_upload_and_validation --test integration_tests -- --nocapture
cargo test test_successful_csv_upload_and_validation --test integration_tests -- --nocapture
cargo test test_upload_validation_failures --test integration_tests -- --nocapture

log_info "2. 민팅 프로세스 테스트"
cargo test test_independent_minting_process --test integration_tests -- --nocapture
cargo test test_mint_request_validation --test integration_tests -- --nocapture

log_info "3. 전체 워크플로우 테스트"
cargo test test_complete_data_lifecycle --test integration_tests -- --nocapture

log_info "4. 성능 & 스트레스 테스트"
cargo test test_multiple_concurrent_uploads --test integration_tests -- --nocapture
cargo test test_storage_management --test integration_tests -- --nocapture

log_info "5. 에러 복구 테스트"
cargo test test_error_recovery_scenarios --test integration_tests -- --nocapture
cargo test test_system_functionality --test integration_tests -- --nocapture

# 5. 전체 테스트 요약 실행
log_info "전체 테스트 요약 실행..."
echo "----------------------------------------"
echo "전체 테스트 요약"
echo "----------------------------------------"
cargo test --test integration_tests -- --nocapture

# 6. 테스트 커버리지 체크 (옵션)
if command -v cargo-tarpaulin &> /dev/null; then
    log_info "테스트 커버리지 측정 중..."
    cargo tarpaulin --out Html --output-dir coverage
    log_info "커버리지 리포트가 coverage/tarpaulin-report.html에 생성되었습니다."
else
    log_warn "cargo-tarpaulin이 설치되어 있지 않습니다. 커버리지 측정을 건너뜁니다."
    log_info "설치하려면: cargo install cargo-tarpaulin"
fi

echo "=========================================="
log_info "모든 테스트 완료!"
echo "=========================================="

# 테스트 결과 요약
echo ""
echo "테스트 결과 요약:"
echo "- ✅ 데이터 업로드 & 검증 기능"
echo "- ✅ 민팅 프로세스 (독립적 canister 생성)"
echo "- ✅ 전체 데이터 라이프사이클"
echo "- ✅ 에러 처리 및 복구"
echo ""
log_info "DataMatrix 프로젝트가 프로덕션 준비 상태입니다!"