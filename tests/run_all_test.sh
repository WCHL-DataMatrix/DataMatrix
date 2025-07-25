#!/bin/bash
set -e

echo "================================================================"
echo "IC Data Marketplace - Complete Integration Test Suite"
echo "================================================================"

# 색상 정의
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 로그 함수
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 환경 변수 설정
TEST_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$TEST_DIR")"
TEST_RESULTS_DIR="$TEST_DIR/results"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# 결과 디렉토리 생성
mkdir -p "$TEST_RESULTS_DIR"

# 전역 변수
TOTAL_TEST_SUITES=0
PASSED_TEST_SUITES=0
OVERALL_START_TIME=$(date +%s)

# IC 환경 관리 함수
start_ic_environment() {
    log_info "Starting IC local environment..."
    
    # 기존 IC 환경이 실행 중인지 확인
    if dfx ping local >/dev/null 2>&1; then
        log_warning "IC environment already running"
        return 0
    fi
    
    # IC 환경 시작
    dfx start --clean --background
    
    # IC 환경이 준비될 때까지 대기
    local max_wait=60
    local waited=0
    
    while [ $waited -lt $max_wait ]; do
        if dfx ping local >/dev/null 2>&1; then
            log_success "IC environment started successfully"
            return 0
        fi
        sleep 2
        waited=$((waited + 2))
    done
    
    log_error "Failed to start IC environment"
    return 1
}

stop_ic_environment() {
    log_info "Stopping IC environment..."
    dfx stop
    log_success "IC environment stopped"
}

# 프로젝트 빌드 및 배포
build_and_deploy() {
    log_info "Building and deploying canisters..."
    
    cd "$PROJECT_ROOT"
    
    # Backend canister 배포
    log_info "Deploying backend canister..."
    if dfx deploy backend --with-cycles 2000000000000; then
        BACKEND_CANISTER_ID=$(dfx canister id backend)
        log_success "Backend canister deployed: $BACKEND_CANISTER_ID"
    else
        log_error "Failed to deploy backend canister"
        return 1
    fi
    
    # Worker canister는 나중에 민팅 테스트에서 배포
    log_info "Build and deployment phase completed"
    return 0
}

# 개별 테스트 스위트 실행
run_test_suite() {
    local suite_name="$1"
    local test_script="$2"
    local log_file="$TEST_RESULTS_DIR/${suite_name}_${TIMESTAMP}.log"
    
    TOTAL_TEST_SUITES=$((TOTAL_TEST_SUITES + 1))
    
    log_info "Running test suite: $suite_name"
    log_info "Log file: $log_file"
    
    local start_time=$(date +%s)
    
    # 테스트 실행
    if bash "$test_script" > "$log_file" 2>&1; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        
        log_success "$suite_name completed successfully (${duration}s)"
        PASSED_TEST_SUITES=$((PASSED_TEST_SUITES + 1))
        
        # 성공한 테스트의 요약 정보 출력
        echo ""
        echo "--- $suite_name Summary ---"
        tail -n 20 "$log_file" | grep -E "(Test Results|Success Rate|Total Tests|Passed|Failed)" || true
        echo ""
        
        return 0
    else
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        
        log_error "$suite_name failed (${duration}s)"
        
        # 실패한 테스트의 에러 정보 출력
        echo ""
        echo "--- $suite_name Error Summary ---"
        tail -n 30 "$log_file" | grep -E "(FAIL|ERROR|Error|Failed)" || true
        echo ""
        
        return 1
    fi
}

# 테스트 후 정리
cleanup_test_environment() {
    log_info "Cleaning up test environment..."
    
    # 임시 파일 정리
    rm -f /tmp/test_output /tmp/mint_test_output /tmp/mint_status
    rm -f /tmp/upload_result /tmp/token_list /tmp/final_stats
    rm -f /tmp/worker_tokens /tmp/first_upload
    
    log_success "Test environment cleaned up"
}

# 테스트 결과 분석 및 리포트 생성
generate_test_report() {
    local report_file="$TEST_RESULTS_DIR/test_report_${TIMESTAMP}.md"
    
    log_info "Generating test report: $report_file"
    
    cat > "$report_file" << EOF
# IC Data Marketplace - Test Report

**Generated:** $(date)  
**Test Suite:** Complete Integration Test  
**Environment:** Local IC Network  

## Overall Results

- **Total Test Suites:** $TOTAL_TEST_SUITES
- **Passed Test Suites:** $PASSED_TEST_SUITES
- **Failed Test Suites:** $((TOTAL_TEST_SUITES - PASSED_TEST_SUITES))
- **Success Rate:** $((PASSED_TEST_SUITES * 100 / TOTAL_TEST_SUITES))%

## Test Suite Details

EOF

    # 각 테스트 스위트의 상세 결과 추가
    for log_file in "$TEST_RESULTS_DIR"/*_${TIMESTAMP}.log; do
        if [ -f "$log_file" ]; then
            local suite_name=$(basename "$log_file" "_${TIMESTAMP}.log")
            echo "### $suite_name" >> "$report_file"
            echo "" >> "$report_file"
            
            # 마지막 결과 요약 추가
            if grep -q "Test Results" "$log_file"; then
                echo "\`\`\`" >> "$report_file"
                grep -A 10 "Test Results" "$log_file" | head -n 15 >> "$report_file"
                echo "\`\`\`" >> "$report_file"
            fi
            echo "" >> "$report_file"
        fi
    done
    
    # 시스템 정보 추가
    cat >> "$report_file" << EOF

## System Information

- **dfx Version:** $(dfx --version)
- **Rust Version:** $(rustc --version 2>/dev/null || echo "Not available")
- **OS:** $(uname -s)
- **Timestamp:** $(date -Iseconds)

## Log Files

EOF

    # 로그 파일 목록 추가
    for log_file in "$TEST_RESULTS_DIR"/*_${TIMESTAMP}.log; do
        if [ -f "$log_file" ]; then
            local filename=$(basename "$log_file")
            echo "- [$filename](./$filename)" >> "$report_file"
        fi
    done
    
    log_success "Test report generated: $report_file"
}

# 메인 실행 함수
main() {
    local start_overall=$(date +%s)
    
    echo ""
    log_info "Starting complete integration test suite..."
    log_info "Working directory: $TEST_DIR"
    log_info "Project root: $PROJECT_ROOT"
    log_info "Results directory: $TEST_RESULTS_DIR"
    echo ""
    
    # 전처리 작업
    if ! start_ic_environment; then
        log_error "Failed to start IC environment"
        exit 1
    fi
    
    if ! build_and_deploy; then
        log_error "Failed to build and deploy"
        stop_ic_environment
        exit 1
    fi
    
    echo ""
    log_info "=========================================="
    log_info "Starting Test Execution Phase"
    log_info "=========================================="
    echo ""
    
    # 테스트 스위트 1: Upload & Validation - 주석 처리 (이미 성공)
    run_test_suite "Upload_Validation" "$TEST_DIR/upload_validation_test.sh"
    
    echo ""
    log_info "Upload & Validation test suite skipped (already successful)"
    log_info "------------------------------------------"
    echo ""
    
    # 테스트 스위트 2: Minting Integration
    run_test_suite "Minting_Integration" "$TEST_DIR/minting_integration_test.sh"
    
    # 향후 확장을 위한 주석
    # echo ""
    # log_info "------------------------------------------"
    # echo ""
    # 
    # # 테스트 스위트 3: Marketplace (향후 추가)
    # run_test_suite "Marketplace" "$TEST_DIR/3_marketplace_test.sh"
    
    echo ""
    log_info "=========================================="
    log_info "Test Execution Completed"
    log_info "=========================================="
    echo ""
    
    # 후처리 작업
    cleanup_test_environment
    generate_test_report
    
    # 최종 결과 출력
    local end_overall=$(date +%s)
    local total_duration=$((end_overall - start_overall))
    
    echo ""
    echo "================================================================"
    echo "FINAL TEST RESULTS"
    echo "================================================================"
    echo "Total Test Suites: $TOTAL_TEST_SUITES (Upload suite skipped)"
    echo "Passed Test Suites: $PASSED_TEST_SUITES"
    echo "Failed Test Suites: $((TOTAL_TEST_SUITES - PASSED_TEST_SUITES))"
    if [ $TOTAL_TEST_SUITES -gt 0 ]; then
        echo "Overall Success Rate: $((PASSED_TEST_SUITES * 100 / TOTAL_TEST_SUITES))%"
    else
        echo "Overall Success Rate: N/A (No tests run)"
    fi
    echo "Total Duration: ${total_duration}s"
    echo "================================================================"
    
    if [ $TOTAL_TEST_SUITES -eq 0 ]; then
        log_warning "No test suites were executed"
        exit 0
    elif [ $PASSED_TEST_SUITES -eq $TOTAL_TEST_SUITES ]; then
        log_success "🎉 ALL TEST SUITES PASSED!"
        echo ""
        log_info "The IC Data Marketplace is ready for production!"
        
        # IC 환경 정리 (선택사항)
        read -p "Stop IC environment? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            stop_ic_environment
        else
            log_info "IC environment left running for further testing"
            log_info "Backend Canister ID: $(dfx canister id backend 2>/dev/null || echo 'N/A')"
            log_info "You can run individual tests or interact with canisters manually"
        fi
        
        exit 0
    else
        log_error "❌ SOME TEST SUITES FAILED"
        echo ""
        log_info "Check the test logs in: $TEST_RESULTS_DIR"
        log_info "Test report available at: $TEST_RESULTS_DIR/test_report_${TIMESTAMP}.md"
        
        # 실패 시에는 IC 환경을 유지하여 디버깅 가능하도록 함
        log_warning "IC environment left running for debugging"
        log_info "Backend Canister ID: $(dfx canister id backend 2>/dev/null || echo 'N/A')"
        
        exit 1
    fi
}

# 사용법 출력
usage() {
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -h, --help     Show this help message"
    echo "  --no-cleanup   Keep IC environment running after tests"
    echo "  --quick        Run only essential tests (faster execution)"
    echo ""
    echo "Examples:"
    echo "  $0                    # Run all tests"
    echo "  $0 --quick            # Run quick test suite"
    echo "  $0 --no-cleanup       # Keep environment running"
    echo ""
}

# 명령행 인수 처리
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            exit 0
            ;;
        --no-cleanup)
            NO_CLEANUP=true
            shift
            ;;
        --quick)
            QUICK_MODE=true
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# 신호 핸들러 설정 (Ctrl+C 등)
trap 'log_warning "Test interrupted by user"; cleanup_test_environment; stop_ic_environment; exit 130' INT TERM

# 메인 함수 실행
main "$@"