#!/bin/bash
# Test Configuration File
# 테스트 실행에 필요한 설정값들을 정의

# 테스트 환경 설정
export TEST_TIMEOUT=300                    # 5분 기본 타임아웃
export MINT_WAIT_TIMEOUT=120              # 민팅 대기 시간 (2분)
export MAX_UPLOAD_SIZE=10485760            # 10MB 업로드 제한
export MAX_CONCURRENT_REQUESTS=5          # 동시 요청 수 제한

# Canister 설정
export BACKEND_CYCLES=2000000000000        # 2T cycles for backend
export WORKER_CYCLES=1000000000000         # 1T cycles for worker

# 테스트 데이터 설정
export TEST_JSON_FILE="test_data.json"
export TEST_CSV_FILE="test_data.csv"

# 로그 설정
export LOG_LEVEL="INFO"                    # DEBUG, INFO, WARN, ERROR
export SAVE_LOGS=true                      # 로그 파일 저장 여부
export LOG_RETENTION_DAYS=7               # 로그 보관 기간

# 네트워크 설정
export DFX_NETWORK="local"                # local, ic, testnet
export IC_REPLICA_PORT=4943               # IC replica port

# 테스트 제외 설정 (선택적)
export SKIP_PERFORMANCE_TESTS=false       # 성능 테스트 건너뛰기
export SKIP_STRESS_TESTS=false           # 스트레스 테스트 건너뛰기
export SKIP_EDGE_CASE_TESTS=false        # 엣지 케이스 테스트 건너뛰기

# 검증 기준
export MIN_SUCCESS_RATE_UPLOAD=80         # 업로드 테스트 최소 성공률 (%)
export MIN_SUCCESS_RATE_MINTING=85        # 민팅 테스트 최소 성공률 (%)
export MIN_SUCCESS_RATE_OVERALL=80        # 전체 테스트 최소 성공률 (%)

# 테스트 데이터 생성 함수들
generate_test_json() {
    cat > "/tmp/test_data.json" << 'EOF'
[
  {"name": "Alice", "age": 30, "city": "Seoul", "type": "user"},
  {"name": "Bob", "age": 25, "city": "Busan", "type": "user"},
  {"name": "Charlie", "age": 35, "city": "Incheon", "type": "admin"}
]
EOF
}

generate_test_csv() {
    cat > "/tmp/test_data.csv" << 'EOF'
name,age,city,type
Alice,30,Seoul,user
Bob,25,Busan,user
Charlie,35,Incheon,admin
Diana,28,Daegu,user
EOF
}

# 바이트 배열 변환 유틸리티 함수들
json_to_bytes() {
    local json_file="$1"
    if [ -f "$json_file" ]; then
        od -t u1 -A n -v "$json_file" | tr '\n' ' ' | sed 's/^ *//' | sed 's/ *$//' | sed 's/ /; /g'
    fi
}

csv_to_bytes() {
    local csv_file="$1"
    if [ -f "$csv_file" ]; then
        od -t u1 -A n -v "$csv_file" | tr '\n' ' ' | sed 's/^ *//' | sed 's/ *$//' | sed 's/ /; /g'
    fi
}

string_to_bytes() {
    local input_string="$1"
    echo -n "$input_string" | od -t u1 -A n -v | tr '\n' ' ' | sed 's/^ *//' | sed 's/ *$//' | sed 's/ /; /g'
}

# Principal 생성 유틸리티
generate_test_principal() {
    # 테스트용 더미 principal들
    local principals=(
        "rdmx6-jaaaa-aaaah-qcaiq-cai"
        "rrkah-fqaaa-aaaah-qcurq-cai"
        "rno2w-sqaaa-aaaah-qcuya-cai"
        "rrkaj-aaaaa-aaaah-qcusq-cai"
        "rrkak-baaaa-aaaah-qcusq-cai"
    )
    local index=${1:-0}
    echo "${principals[$index]}"
}

# 테스트 환경 검증
validate_test_environment() {
    echo "Validating test environment..."
    
    # dfx 설치 확인
    if ! command -v dfx &> /dev/null; then
        echo "ERROR: dfx is not installed"
        return 1
    fi
    
    # Rust 설치 확인
    if ! command -v rustc &> /dev/null; then
        echo "WARNING: Rust is not installed, may affect building"
    fi
    
    # 디스크 공간 확인 (최소 1GB)
    local available_space=$(df . | tail -1 | awk '{print $4}')
    if [ "$available_space" -lt 1048576 ]; then
        echo "WARNING: Low disk space (less than 1GB available)"
    fi
    
    # 포트 사용 확인
    if lsof -i:$IC_REPLICA_PORT &> /dev/null; then
        echo "WARNING: Port $IC_REPLICA_PORT is already in use"
    fi
    
    echo "Environment validation completed"
    return 0
}

# 테스트 데이터 정리
cleanup_test_data() {
    echo "Cleaning up test data..."
    rm -f /tmp/test_data.json /tmp/test_data.csv
    rm -f /tmp/test_*.log /tmp/mint_* /tmp/upload_*
    echo "Test data cleanup completed"
}

# 설정 출력
print_test_config() {
    echo "==============================================="
    echo "Test Configuration"
    echo "==============================================="
    echo "Test Timeout: ${TEST_TIMEOUT}s"
    echo "Mint Wait Timeout: ${MINT_WAIT_TIMEOUT}s"
    echo "Max Upload Size: ${MAX_UPLOAD_SIZE} bytes"
    echo "Max Concurrent Requests: $MAX_CONCURRENT_REQUESTS"
    echo "Backend Cycles: $BACKEND_CYCLES"
    echo "Worker Cycles: $WORKER_CYCLES"
    echo "DFX Network: $DFX_NETWORK"
    echo "IC Replica Port: $IC_REPLICA_PORT"
    echo "Log Level: $LOG_LEVEL"
    echo "Save Logs: $SAVE_LOGS"
    echo "Min Success Rate (Upload): ${MIN_SUCCESS_RATE_UPLOAD}%"
    echo "Min Success Rate (Minting): ${MIN_SUCCESS_RATE_MINTING}%"
    echo "Min Success Rate (Overall): ${MIN_SUCCESS_RATE_OVERALL}%"
    echo "==============================================="
}

# 고급 테스트 데이터 생성
generate_advanced_test_data() {
    echo "Generating advanced test data..."
    
    # 대용량 JSON 데이터 (약 1MB)
    cat > "/tmp/large_test_data.json" << 'EOF'
{
  "users": [
EOF
    
    for i in {1..1000}; do
        if [ $i -eq 1000 ]; then
            echo "    {\"id\": $i, \"name\": \"User$i\", \"email\": \"user$i@test.com\", \"active\": true}" >> "/tmp/large_test_data.json"
        else
            echo "    {\"id\": $i, \"name\": \"User$i\", \"email\": \"user$i@test.com\", \"active\": true}," >> "/tmp/large_test_data.json"
        fi
    done
    
    echo "  ]," >> "/tmp/large_test_data.json"
    echo "  \"metadata\": {\"generated\": \"$(date -Iseconds)\", \"count\": 1000}" >> "/tmp/large_test_data.json"
    echo "}" >> "/tmp/large_test_data.json"
    
    # 복잡한 CSV 데이터
    cat > "/tmp/complex_test_data.csv" << 'EOF'
id,name,email,age,city,country,salary,department,join_date,active
1,Alice Johnson,alice@test.com,30,Seoul,Korea,75000,Engineering,2020-01-15,true
2,Bob Smith,bob@test.com,25,Busan,Korea,65000,Marketing,2021-03-20,true
3,Charlie Brown,charlie@test.com,35,Incheon,Korea,85000,Engineering,2019-06-10,false
4,Diana Prince,diana@test.com,28,Daegu,Korea,70000,Sales,2022-01-05,true
5,Eve Adams,eve@test.com,32,Gwangju,Korea,80000,Engineering,2020-11-30,true
EOF
    
    # 잘못된 형식의 데이터들
    echo "invalid json data without proper structure" > "/tmp/invalid_test_data.json"
    echo "invalid,csv,data" > "/tmp/invalid_test_data.csv"
    echo "missing,header" >> "/tmp/invalid_test_data.csv"
    
    echo "Advanced test data generated"
}

# 테스트 메트릭 수집
collect_test_metrics() {
    local test_name="$1"
    local start_time="$2"
    local end_time="$3"
    local success="$4"
    
    local duration=$((end_time - start_time))
    local metrics_file="/tmp/test_metrics.log"
    
    echo "$(date -Iseconds),$test_name,$duration,$success" >> "$metrics_file"
}

# 성능 벤치마크 데이터
get_performance_baseline() {
    echo "Performance Baselines:"
    echo "====================="
    echo "Small JSON Upload (< 1KB): < 2s"
    echo "Medium JSON Upload (< 100KB): < 5s"
    echo "Large JSON Upload (< 1MB): < 15s"
    echo "CSV Upload (< 100KB): < 3s"
    echo "Simple Mint Request: < 2s"
    echo "Mint Processing: < 60s"
    echo "Concurrent Requests (5x): < 10s"
}

# 시스템 리소스 모니터링
monitor_system_resources() {
    if command -v top &> /dev/null; then
        echo "System Resources:"
        echo "================"
        echo "Memory Usage:"
        free -h 2>/dev/null || echo "Memory info not available"
        echo ""
        echo "CPU Usage:"
        top -bn1 | grep "Cpu(s)" 2>/dev/null || echo "CPU info not available"
        echo ""
        echo "Disk Usage:"
        df -h . 2>/dev/null || echo "Disk info not available"
    fi
}

# 네트워크 연결 테스트
test_network_connectivity() {
    echo "Testing network connectivity..."
    
    if ping -c 1 127.0.0.1 >/dev/null 2>&1; then
        echo "✅ Localhost connectivity: OK"
    else
        echo "❌ Localhost connectivity: FAILED"
        return 1
    fi
    
    if command -v dfx &> /dev/null; then
        if timeout 10 dfx ping local >/dev/null 2>&1; then
            echo "✅ DFX local network: OK"
        else
            echo "⚠️  DFX local network: Not available (will be started during tests)"
        fi
    fi
    
    return 0
}

# 테스트 환경 초기화
initialize_test_environment() {
    echo "Initializing test environment..."
    
    # 테스트 디렉토리 생성
    mkdir -p /tmp/ic_test_logs
    mkdir -p /tmp/ic_test_data
    
    # 환경변수 파일 생성
    cat > "/tmp/ic_test_env.sh" << EOF
export TEST_SESSION_ID=$(date +%s)
export TEST_START_TIME=$(date -Iseconds)
export TEST_PID=$
export TEST_USER=$(whoami)
export TEST_HOSTNAME=$(hostname)
EOF
    
    # 테스트 데이터 생성
    generate_test_json
    generate_test_csv
    
    echo "Test environment initialized"
}

# 종료 시 정리 함수
cleanup_on_exit() {
    echo "Performing final cleanup..."
    cleanup_test_data
    
    # 메트릭 파일 정리
    if [ -f "/tmp/test_metrics.log" ]; then
        echo "Test metrics collected:"
        cat "/tmp/test_metrics.log"
        rm -f "/tmp/test_metrics.log"
    fi
    
    # 환경변수 파일 정리
    rm -f "/tmp/ic_test_env.sh"
    
    echo "Final cleanup completed"
}

# 테스트 재시도 로직
retry_test() {
    local test_command="$1"
    local max_retries="${2:-3}"
    local retry_delay="${3:-5}"
    local attempt=1
    
    while [ $attempt -le $max_retries ]; do
        echo "Attempt $attempt/$max_retries: $test_command"
        
        if eval "$test_command"; then
            echo "✅ Test succeeded on attempt $attempt"
            return 0
        else
            echo "❌ Test failed on attempt $attempt"
            if [ $attempt -lt $max_retries ]; then
                echo "Retrying in ${retry_delay}s..."
                sleep $retry_delay
            fi
        fi
        
        attempt=$((attempt + 1))
    done
    
    echo "❌ Test failed after $max_retries attempts"
    return 1
}

# 병렬 테스트 실행 도우미
run_parallel_tests() {
    local test_commands=("$@")
    local pids=()
    local results=()
    
    echo "Running ${#test_commands[@]} tests in parallel..."
    
    # 모든 테스트를 백그라운드에서 시작
    for i in "${!test_commands[@]}"; do
        {
            eval "${test_commands[$i]}"
            echo $? > "/tmp/parallel_test_$i.result"
        } &
        pids+=($!)
    done
    
    # 모든 테스트 완료 대기
    for i in "${!pids[@]}"; do
        wait "${pids[$i]}"
        local result=$(cat "/tmp/parallel_test_$i.result" 2>/dev/null || echo "1")
        results+=($result)
        rm -f "/tmp/parallel_test_$i.result"
    done
    
    # 결과 집계
    local failed_count=0
    for result in "${results[@]}"; do
        if [ "$result" != "0" ]; then
            failed_count=$((failed_count + 1))
        fi
    done
    
    echo "Parallel tests completed: $((${#results[@]} - failed_count))/${#results[@]} passed"
    
    return $failed_count
}

# 디버깅 정보 수집
collect_debug_info() {
    local debug_file="/tmp/ic_debug_info.log"
    
    echo "Collecting debug information..."
    
    {
        echo "=== Debug Information ==="
        echo "Timestamp: $(date -Iseconds)"
        echo "User: $(whoami)"
        echo "Working Directory: $(pwd)"
        echo "Shell: $SHELL"
        echo ""
        
        echo "=== Environment Variables ==="
        env | grep -E "(DFX|IC|TEST)" | sort
        echo ""
        
        echo "=== DFX Information ==="
        dfx --version 2>/dev/null || echo "dfx not available"
        dfx identity whoami 2>/dev/null || echo "No active dfx identity"
        dfx identity get-principal 2>/dev/null || echo "Cannot get principal"
        echo ""
        
        echo "=== System Information ==="
        uname -a
        echo ""
        
        echo "=== Process Information ==="
        ps aux | grep -E "(dfx|replica)" | grep -v grep || echo "No IC processes found"
        echo ""
        
        echo "=== Network Information ==="
        netstat -tlnp 2>/dev/null | grep ":4943" || echo "IC replica port not found"
        echo ""
        
        echo "=== File System ==="
        ls -la .dfx/ 2>/dev/null || echo ".dfx directory not found"
        echo ""
        
    } > "$debug_file"
    
    echo "Debug information saved to: $debug_file"
    return 0
}

# 테스트 조건 확인
check_test_preconditions() {
    echo "Checking test preconditions..."
    
    local errors=0
    
    # 필수 도구 확인
    if ! command -v dfx &> /dev/null; then
        echo "❌ dfx is not installed"
        errors=$((errors + 1))
    fi
    
    # 프로젝트 구조 확인
    if [ ! -f "dfx.json" ]; then
        echo "❌ dfx.json not found - not in a DFX project directory"
        errors=$((errors + 1))
    fi
    
    if [ ! -d "src/backend" ]; then
        echo "❌ Backend source directory not found"
        errors=$((errors + 1))
    fi
    
    if [ ! -f "src/backend/Cargo.toml" ]; then
        echo "❌ Backend Cargo.toml not found"
        errors=$((errors + 1))
    fi
    
    # 권한 확인
    if [ ! -w "." ]; then
        echo "❌ No write permission in current directory"
        errors=$((errors + 1))
    fi
    
    # 필수 디렉토리 생성 가능 여부 확인
    if ! mkdir -p "/tmp/ic_test_check" 2>/dev/null; then
        echo "❌ Cannot create temporary directories"
        errors=$((errors + 1))
    else
        rmdir "/tmp/ic_test_check" 2>/dev/null
    fi
    
    if [ $errors -eq 0 ]; then
        echo "✅ All preconditions satisfied"
        return 0
    else
        echo "❌ $errors precondition(s) failed"
        return 1
    fi
}

# 설정 로드 함수 (다른 스크립트에서 source로 사용)
load_test_config() {
    echo "Loading test configuration..."
    
    # 사용자 정의 설정 파일이 있으면 로드
    if [ -f "test_config_local.sh" ]; then
        echo "Loading local configuration..."
        source "test_config_local.sh"
    fi
    
    # 환경변수로 설정 오버라이드
    TEST_TIMEOUT=${TEST_TIMEOUT_OVERRIDE:-$TEST_TIMEOUT}
    MINT_WAIT_TIMEOUT=${MINT_WAIT_TIMEOUT_OVERRIDE:-$MINT_WAIT_TIMEOUT}
    MAX_UPLOAD_SIZE=${MAX_UPLOAD_SIZE_OVERRIDE:-$MAX_UPLOAD_SIZE}
    
    echo "Configuration loaded successfully"
}

# 이 스크립트가 직접 실행될 때
if [ "${BASH_SOURCE[0]}" = "${0}" ]; then
    echo "IC Data Marketplace - Test Configuration"
    echo "========================================"
    
    case "${1:-help}" in
        "help"|"-h"|"--help")
            echo "Usage: $0 [command]"
            echo ""
            echo "Commands:"
            echo "  config     Show current configuration"
            echo "  check      Check test preconditions"
            echo "  init       Initialize test environment"
            echo "  cleanup    Clean up test data"
            echo "  debug      Collect debug information"
            echo "  test-net   Test network connectivity"
            echo ""
            ;;
        "config")
            print_test_config
            ;;
        "check")
            check_test_preconditions
            ;;
        "init")
            initialize_test_environment
            ;;
        "cleanup")
            cleanup_test_data
            ;;
        "debug")
            collect_debug_info
            ;;
        "test-net")
            test_network_connectivity
            ;;
        *)
            echo "Unknown command: $1"
            echo "Use '$0 help' for usage information"
            exit 1
            ;;
    esac
fi