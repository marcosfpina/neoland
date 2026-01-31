#!/usr/bin/env bash
#
# NEOLAND Disaster Recovery Testing Script
# Tests backup and restore procedures end-to-end
#
# Usage:
#   ./test_dr.sh [--skip-backup] [--skip-restore]
#

set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_TIMESTAMP="$(date +%Y%m%d_%H%M%S)"
TEST_DIR="/tmp/neoland_dr_test_${TEST_TIMESTAMP}"
TEST_BACKUP_PATH="${TEST_DIR}/backups"
TEST_DB_NAME="neoland_dr_test_${TEST_TIMESTAMP}"

SKIP_BACKUP=false
SKIP_RESTORE=false

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# Test results
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_TOTAL=0

# ============================================================================
# Functions
# ============================================================================

log_info() {
    echo -e "${BLUE}[INFO]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $*"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $*"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $*" >&2
}

log_test() {
    echo -e "${MAGENTA}[TEST]${NC} $*"
}

test_start() {
    ((TESTS_TOTAL++))
    log_test "Test ${TESTS_TOTAL}: $*"
}

test_pass() {
    ((TESTS_PASSED++))
    log_success "✅ PASS: $*"
}

test_fail() {
    ((TESTS_FAILED++))
    log_error "❌ FAIL: $*"
}

setup_test_environment() {
    log_info "Setting up test environment..."

    # Create test directory
    mkdir -p "${TEST_DIR}"
    mkdir -p "${TEST_BACKUP_PATH}"

    # Create test database
    log_info "Creating test database: ${TEST_DB_NAME}"

    local base_db_url="${DATABASE_URL%/*}"

    psql "${base_db_url}/postgres" << EOF || true
DROP DATABASE IF EXISTS ${TEST_DB_NAME};
CREATE DATABASE ${TEST_DB_NAME};
EOF

    # Set test database URL
    export TEST_DATABASE_URL="${base_db_url}/${TEST_DB_NAME}"

    log_info "Test database created: ${TEST_DATABASE_URL}"

    # Populate test database with sample data
    psql "${TEST_DATABASE_URL}" << 'EOF'
-- Create test tables
CREATE TABLE IF NOT EXISTS test_data (
    id SERIAL PRIMARY KEY,
    data TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Insert test data
INSERT INTO test_data (data) VALUES
    ('Test record 1'),
    ('Test record 2'),
    ('Test record 3'),
    ('Test record 4'),
    ('Test record 5');

-- Verify insertion
SELECT COUNT(*) as record_count FROM test_data;
EOF

    log_success "Test environment setup complete"
}

cleanup_test_environment() {
    log_info "Cleaning up test environment..."

    # Drop test database
    local base_db_url="${DATABASE_URL%/*}"

    psql "${base_db_url}/postgres" << EOF || true
DROP DATABASE IF EXISTS ${TEST_DB_NAME};
EOF

    # Remove test directory
    rm -rf "${TEST_DIR}"

    log_success "Test environment cleaned up"
}

# ============================================================================
# Backup Tests
# ============================================================================

test_backup_script_exists() {
    test_start "Backup script exists and is executable"

    if [[ -x "${SCRIPT_DIR}/backup.sh" ]]; then
        test_pass "Backup script exists and is executable"
        return 0
    else
        test_fail "Backup script not found or not executable"
        return 1
    fi
}

test_backup_dependencies() {
    test_start "Backup dependencies are installed"

    local missing_deps=()

    for cmd in pg_dump tar gzip; do
        if ! command -v ${cmd} &> /dev/null; then
            missing_deps+=("${cmd}")
        fi
    done

    if [[ ${#missing_deps[@]} -eq 0 ]]; then
        test_pass "All backup dependencies installed"
        return 0
    else
        test_fail "Missing dependencies: ${missing_deps[*]}"
        return 1
    fi
}

test_backup_execution() {
    test_start "Backup script executes successfully"

    # Run backup with test database
    DATABASE_URL="${TEST_DATABASE_URL}" \
    BACKUP_STORAGE_PATH="${TEST_BACKUP_PATH}" \
    BACKUP_RETENTION_DAYS=1 \
        "${SCRIPT_DIR}/backup.sh" 2>&1 | tee "${TEST_DIR}/backup.log"

    local exit_code=$?

    if [[ ${exit_code} -eq 0 ]]; then
        test_pass "Backup script executed successfully"
        return 0
    else
        test_fail "Backup script failed with exit code ${exit_code}"
        return 1
    fi
}

test_backup_manifest() {
    test_start "Backup manifest is created"

    # Find the latest backup directory
    local latest_backup=$(ls -1dt "${TEST_BACKUP_PATH}"/*/ 2>/dev/null | head -n1)

    if [[ -z "${latest_backup}" ]]; then
        test_fail "No backup directory found"
        return 1
    fi

    local manifest="${latest_backup}MANIFEST.json"

    if [[ -f "${manifest}" ]]; then
        # Validate JSON
        if jq '.' "${manifest}" > /dev/null 2>&1; then
            test_pass "Backup manifest is valid JSON"

            # Check required fields
            local required_fields=("backup_timestamp" "backup_date" "components")
            for field in "${required_fields[@]}"; do
                if ! jq -e ".${field}" "${manifest}" > /dev/null 2>&1; then
                    test_fail "Manifest missing required field: ${field}"
                    return 1
                fi
            done

            test_pass "Backup manifest contains all required fields"
            return 0
        else
            test_fail "Backup manifest is not valid JSON"
            return 1
        fi
    else
        test_fail "Backup manifest not found"
        return 1
    fi
}

test_backup_database_file() {
    test_start "Database backup file is created"

    local latest_backup=$(ls -1dt "${TEST_BACKUP_PATH}"/*/ 2>/dev/null | head -n1)

    if [[ -z "${latest_backup}" ]]; then
        test_fail "No backup directory found"
        return 1
    fi

    local db_backup="${latest_backup}postgresql_dump.sql.gz"

    if [[ -f "${db_backup}" ]]; then
        # Verify it's a valid gzip file
        if gzip -t "${db_backup}" 2>/dev/null; then
            local size=$(stat -c%s "${db_backup}")
            test_pass "Database backup file is valid gzip (${size} bytes)"
            return 0
        else
            test_fail "Database backup file is not a valid gzip"
            return 1
        fi
    else
        test_fail "Database backup file not found"
        return 1
    fi
}

test_backup_integrity() {
    test_start "Database backup integrity"

    local latest_backup=$(ls -1dt "${TEST_BACKUP_PATH}"/*/ 2>/dev/null | head -n1)
    local db_backup="${latest_backup}postgresql_dump.sql.gz"

    if [[ ! -f "${db_backup}" ]]; then
        test_fail "Database backup file not found"
        return 1
    fi

    # Try to list contents with pg_restore
    if pg_restore --list "${db_backup}" > /dev/null 2>&1; then
        test_pass "Database backup integrity verified"
        return 0
    else
        test_fail "Database backup integrity check failed"
        return 1
    fi
}

# ============================================================================
# Restore Tests
# ============================================================================

test_restore_script_exists() {
    test_start "Restore script exists and is executable"

    if [[ -x "${SCRIPT_DIR}/restore.sh" ]]; then
        test_pass "Restore script exists and is executable"
        return 0
    else
        test_fail "Restore script not found or not executable"
        return 1
    fi
}

test_restore_validation() {
    test_start "Restore script validates backup"

    local latest_backup=$(ls -1dt "${TEST_BACKUP_PATH}"/*/ 2>/dev/null | head -n1 | sed 's:/$::')

    # Run restore with validation only (will fail at confirmation)
    if echo "no" | DATABASE_URL="${TEST_DATABASE_URL}" \
        "${SCRIPT_DIR}/restore.sh" --backup-path "${latest_backup}" --components database \
        2>&1 | grep -q "Backup validation passed"; then
        test_pass "Restore script validates backup"
        return 0
    else
        test_fail "Restore script validation failed"
        return 1
    fi
}

test_restore_execution() {
    test_start "Restore script executes successfully"

    local latest_backup=$(ls -1dt "${TEST_BACKUP_PATH}"/*/ 2>/dev/null | head -n1 | sed 's:/$::')

    # First, corrupt the database to verify restore
    psql "${TEST_DATABASE_URL}" << 'EOF'
DELETE FROM test_data WHERE id <= 3;
SELECT COUNT(*) as remaining_records FROM test_data;
EOF

    # Run restore (auto-confirm with "yes")
    if echo "yes" | DATABASE_URL="${TEST_DATABASE_URL}" \
        "${SCRIPT_DIR}/restore.sh" --backup-path "${latest_backup}" --components database \
        2>&1 | tee "${TEST_DIR}/restore.log"; then
        test_pass "Restore script executed successfully"
        return 0
    else
        test_fail "Restore script failed"
        return 1
    fi
}

test_data_integrity_after_restore() {
    test_start "Data integrity after restore"

    # Verify data was restored
    local record_count=$(psql "${TEST_DATABASE_URL}" -t -c "SELECT COUNT(*) FROM test_data;" | xargs)

    if [[ "${record_count}" == "5" ]]; then
        test_pass "All ${record_count} records restored successfully"

        # Verify specific data
        local test_record=$(psql "${TEST_DATABASE_URL}" -t -c "SELECT data FROM test_data WHERE id = 1;" | xargs)
        if [[ "${test_record}" == "Test record 1" ]]; then
            test_pass "Data integrity verified"
            return 0
        else
            test_fail "Data content mismatch after restore"
            return 1
        fi
    else
        test_fail "Expected 5 records, found ${record_count}"
        return 1
    fi
}

# ============================================================================
# RTO/RPO Tests
# ============================================================================

test_rto_measurement() {
    test_start "Recovery Time Objective (RTO) measurement"

    local latest_backup=$(ls -1dt "${TEST_BACKUP_PATH}"/*/ 2>/dev/null | head -n1 | sed 's:/$::')

    # Measure restore time
    local start_time=$(date +%s)

    echo "yes" | DATABASE_URL="${TEST_DATABASE_URL}" \
        "${SCRIPT_DIR}/restore.sh" --backup-path "${latest_backup}" --components database \
        > /dev/null 2>&1 || true

    local end_time=$(date +%s)
    local restore_time=$((end_time - start_time))

    # RTO target: 4 hours (14400 seconds)
    # For this test DB, should be < 60 seconds
    local rto_target=60

    if [[ ${restore_time} -lt ${rto_target} ]]; then
        test_pass "RTO: ${restore_time}s < ${rto_target}s target"
        return 0
    else
        test_fail "RTO: ${restore_time}s >= ${rto_target}s target"
        return 1
    fi
}

test_backup_retention() {
    test_start "Backup retention policy"

    # Create multiple backups
    for i in {1..3}; do
        DATABASE_URL="${TEST_DATABASE_URL}" \
        BACKUP_STORAGE_PATH="${TEST_BACKUP_PATH}" \
        BACKUP_RETENTION_DAYS=1 \
            "${SCRIPT_DIR}/backup.sh" > /dev/null 2>&1
        sleep 2
    done

    # Count backups
    local backup_count=$(ls -1d "${TEST_BACKUP_PATH}"/*/ 2>/dev/null | wc -l)

    if [[ ${backup_count} -ge 3 ]]; then
        test_pass "Multiple backups retained (${backup_count} backups)"
        return 0
    else
        test_fail "Expected >= 3 backups, found ${backup_count}"
        return 1
    fi
}

# ============================================================================
# Main
# ============================================================================

print_test_summary() {
    echo ""
    echo "========================================"
    echo "DISASTER RECOVERY TEST SUMMARY"
    echo "========================================"
    echo ""
    echo "Total Tests: ${TESTS_TOTAL}"
    echo -e "${GREEN}Passed: ${TESTS_PASSED}${NC}"
    echo -e "${RED}Failed: ${TESTS_FAILED}${NC}"
    echo ""

    local pass_rate=0
    if [[ ${TESTS_TOTAL} -gt 0 ]]; then
        pass_rate=$((TESTS_PASSED * 100 / TESTS_TOTAL))
    fi

    echo "Pass Rate: ${pass_rate}%"
    echo ""

    if [[ ${TESTS_FAILED} -eq 0 ]]; then
        echo -e "${GREEN}✅ ALL TESTS PASSED${NC}"
        echo ""
        log_success "Disaster recovery procedures validated successfully"
        log_info "RTO target: 4 hours"
        log_info "RPO target: 24 hours"
        log_info "Retention: 30 days"
        return 0
    else
        echo -e "${RED}❌ SOME TESTS FAILED${NC}"
        echo ""
        log_error "Disaster recovery testing failed"
        log_error "Review test output and fix issues before production deployment"
        return 1
    fi
}

main() {
    log_info "Starting Disaster Recovery Testing"
    log_info "Test directory: ${TEST_DIR}"
    log_info "Test database: ${TEST_DB_NAME}"

    # Verify DATABASE_URL is set
    if [[ -z "${DATABASE_URL:-}" ]]; then
        log_error "DATABASE_URL not set"
        log_error "Set DATABASE_URL before running DR tests"
        exit 1
    fi

    # Setup
    setup_test_environment

    # Backup Tests
    if [[ "${SKIP_BACKUP}" == "false" ]]; then
        test_backup_script_exists
        test_backup_dependencies
        test_backup_execution
        test_backup_manifest
        test_backup_database_file
        test_backup_integrity
        test_backup_retention
    else
        log_warning "Skipping backup tests (--skip-backup)"
    fi

    # Restore Tests
    if [[ "${SKIP_RESTORE}" == "false" ]]; then
        test_restore_script_exists
        test_restore_validation
        test_restore_execution
        test_data_integrity_after_restore
        test_rto_measurement
    else
        log_warning "Skipping restore tests (--skip-restore)"
    fi

    # Cleanup
    cleanup_test_environment

    # Summary
    print_test_summary
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --skip-backup)
            SKIP_BACKUP=true
            shift
            ;;
        --skip-restore)
            SKIP_RESTORE=true
            shift
            ;;
        --help)
            cat << EOF
Usage: $0 [OPTIONS]

Options:
  --skip-backup   Skip backup tests
  --skip-restore  Skip restore tests
  --help         Show this help message

Environment Variables:
  DATABASE_URL   PostgreSQL connection string (required)

Example:
  DATABASE_URL=postgresql://user:pass@localhost/neoland ./test_dr.sh
EOF
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

main
