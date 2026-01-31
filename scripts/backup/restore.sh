#!/usr/bin/env bash
#
# NEOLAND Restore Script
# Restores from backup created by backup.sh
#
# Usage:
#   ./restore.sh [--backup-path PATH] [--components database,logs,config]
#
# Environment Variables:
#   DATABASE_URL  - PostgreSQL connection string
#

set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKUP_PATH=""
COMPONENTS="all"  # database,audit_logs,configuration

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

check_dependencies() {
    local missing_deps=()

    if ! command -v pg_restore &> /dev/null; then
        missing_deps+=("postgresql-client")
    fi

    if ! command -v tar &> /dev/null; then
        missing_deps+=("tar")
    fi

    if ! command -v gzip &> /dev/null; then
        missing_deps+=("gzip")
    fi

    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        log_error "Install with: nix-shell -p ${missing_deps[*]}"
        exit 1
    fi
}

validate_backup() {
    log_info "Validating backup at: ${BACKUP_PATH}"

    if [[ ! -d "${BACKUP_PATH}" ]]; then
        log_error "Backup directory not found: ${BACKUP_PATH}"
        exit 1
    fi

    local manifest="${BACKUP_PATH}/MANIFEST.json"
    if [[ ! -f "${manifest}" ]]; then
        log_error "Backup manifest not found: ${manifest}"
        log_error "This may not be a valid NEOLAND backup"
        exit 1
    fi

    log_info "Backup manifest found, contents:"
    cat "${manifest}" | jq '.' 2>/dev/null || cat "${manifest}"

    log_success "Backup validation passed"
}

confirm_restore() {
    log_warning "⚠️  WARNING: This will OVERWRITE existing data!"
    log_warning "Backup path: ${BACKUP_PATH}"
    log_warning "Components to restore: ${COMPONENTS}"

    echo -n "Are you sure you want to continue? (yes/no): "
    read -r confirmation

    if [[ "${confirmation}" != "yes" ]]; then
        log_info "Restore cancelled by user"
        exit 0
    fi
}

should_restore_component() {
    local component="$1"

    if [[ "${COMPONENTS}" == "all" ]]; then
        return 0
    fi

    if echo "${COMPONENTS}" | grep -q "${component}"; then
        return 0
    fi

    return 1
}

restore_database() {
    if ! should_restore_component "database"; then
        log_info "Skipping database restore (not in components list)"
        return 0
    fi

    local db_backup="${BACKUP_PATH}/postgresql_dump.sql.gz"

    if [[ ! -f "${db_backup}" ]]; then
        log_warning "Database backup not found: ${db_backup}"
        return 0
    fi

    log_info "Restoring PostgreSQL database..."

    if [[ -z "${DATABASE_URL:-}" ]]; then
        log_error "DATABASE_URL not set, cannot restore database"
        return 1
    fi

    # Drop existing database (DANGEROUS - already confirmed by user)
    log_warning "Dropping existing database..."

    # Extract database name from DATABASE_URL
    local db_name=$(echo "${DATABASE_URL}" | sed -n 's/.*\/\([^?]*\).*/\1/p')

    if [[ -z "${db_name}" ]]; then
        log_error "Could not extract database name from DATABASE_URL"
        return 1
    fi

    log_info "Database name: ${db_name}"

    # Drop and recreate database
    psql "${DATABASE_URL%/*}/postgres" << EOF || log_warning "Database drop failed (may not exist)"
DROP DATABASE IF EXISTS ${db_name};
CREATE DATABASE ${db_name};
EOF

    # Restore from backup
    if pg_restore \
        --dbname="${DATABASE_URL}" \
        --clean \
        --if-exists \
        --no-owner \
        --no-acl \
        --verbose \
        "${db_backup}" 2>&1 | tee "${BACKUP_PATH}/restore.log"; then

        log_success "Database restore completed"

        # Verify restore
        local table_count=$(psql "${DATABASE_URL}" -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';" | xargs)
        log_info "Restored ${table_count} tables"

    else
        log_error "Database restore failed"
        return 1
    fi
}

restore_audit_logs() {
    if ! should_restore_component "audit_logs"; then
        log_info "Skipping audit logs restore (not in components list)"
        return 0
    fi

    local audit_backup="${BACKUP_PATH}/audit_logs.tar.gz"

    if [[ ! -f "${audit_backup}" ]]; then
        log_warning "Audit logs backup not found: ${audit_backup}"
        return 0
    fi

    log_info "Restoring audit logs..."

    local audit_log_dir="/var/log/neoland"

    # Create directory if it doesn't exist
    sudo mkdir -p "${audit_log_dir}" 2>/dev/null || mkdir -p "${audit_log_dir}"

    # Backup existing logs before restore
    if [[ -f "${audit_log_dir}/audit.log" ]]; then
        local backup_name="audit.log.backup.$(date +%Y%m%d_%H%M%S)"
        log_info "Backing up existing audit log to: ${backup_name}"
        sudo cp "${audit_log_dir}/audit.log" "${audit_log_dir}/${backup_name}" 2>/dev/null || \
            cp "${audit_log_dir}/audit.log" "${audit_log_dir}/${backup_name}"
    fi

    # Restore audit logs
    if sudo tar -xzf "${audit_backup}" -C "${audit_log_dir}" 2>/dev/null || \
        tar -xzf "${audit_backup}" -C "${audit_log_dir}"; then

        local line_count=$(wc -l < "${audit_log_dir}/audit.log" 2>/dev/null || echo "0")
        log_success "Audit logs restore completed (${line_count} entries)"
    else
        log_error "Audit logs restore failed"
        return 1
    fi
}

restore_configuration() {
    if ! should_restore_component "configuration"; then
        log_info "Skipping configuration restore (not in components list)"
        return 0
    fi

    local config_backup="${BACKUP_PATH}/configuration.tar.gz"

    if [[ ! -f "${config_backup}" ]]; then
        log_warning "Configuration backup not found: ${config_backup}"
        return 0
    fi

    log_info "Restoring configuration files..."

    # Backup existing configuration
    local config_paths=(
        "/etc/neoland"
        "${HOME}/.config/neoland"
    )

    for path in "${config_paths[@]}"; do
        if [[ -e "${path}" ]]; then
            local backup_name="${path}.backup.$(date +%Y%m%d_%H%M%S)"
            log_info "Backing up existing config: ${path} -> ${backup_name}"
            sudo cp -r "${path}" "${backup_name}" 2>/dev/null || cp -r "${path}" "${backup_name}"
        fi
    done

    # Restore configuration
    if sudo tar -xzf "${config_backup}" -C / 2>/dev/null || \
        tar -xzf "${config_backup}" -C /; then

        log_success "Configuration restore completed"
    else
        log_error "Configuration restore failed"
        return 1
    fi
}

verify_restore() {
    log_info "Verifying restore..."

    local verification_failed=0

    # Verify database
    if should_restore_component "database" && [[ -n "${DATABASE_URL:-}" ]]; then
        if psql "${DATABASE_URL}" -c "SELECT 1" > /dev/null 2>&1; then
            log_success "✅ Database connectivity verified"
        else
            log_error "❌ Database connectivity check failed"
            verification_failed=1
        fi
    fi

    # Verify audit logs
    if should_restore_component "audit_logs"; then
        if [[ -f "/var/log/neoland/audit.log" ]]; then
            log_success "✅ Audit log file exists"
        else
            log_warning "⚠️  Audit log file not found"
        fi
    fi

    # Verify configuration
    if should_restore_component "configuration"; then
        if [[ -d "/etc/neoland" ]] || [[ -d "${HOME}/.config/neoland" ]]; then
            log_success "✅ Configuration directories exist"
        else
            log_warning "⚠️  Configuration directories not found"
        fi
    fi

    return ${verification_failed}
}

generate_restore_report() {
    local report="${BACKUP_PATH}/restore_report.txt"

    cat > "${report}" << EOF
================================================================================
NEOLAND Restore Report
================================================================================

Restore Date: $(date)
Hostname: $(hostname)
Backup Path: ${BACKUP_PATH}
Components Restored: ${COMPONENTS}

--------------------------------------------------------------------------------
Restore Status
--------------------------------------------------------------------------------

EOF

    if should_restore_component "database" && [[ -f "${BACKUP_PATH}/postgresql_dump.sql.gz" ]]; then
        echo "✅ PostgreSQL Database: RESTORED" >> "${report}"
    else
        echo "⏭️  PostgreSQL Database: SKIPPED" >> "${report}"
    fi

    if should_restore_component "audit_logs" && [[ -f "${BACKUP_PATH}/audit_logs.tar.gz" ]]; then
        echo "✅ Audit Logs: RESTORED" >> "${report}"
    else
        echo "⏭️  Audit Logs: SKIPPED" >> "${report}"
    fi

    if should_restore_component "configuration" && [[ -f "${BACKUP_PATH}/configuration.tar.gz" ]]; then
        echo "✅ Configuration: RESTORED" >> "${report}"
    else
        echo "⏭️  Configuration: SKIPPED" >> "${report}"
    fi

    cat >> "${report}" << EOF

--------------------------------------------------------------------------------
Next Steps
--------------------------------------------------------------------------------

1. Verify NEOLAND starts successfully:
   cargo run --bin neoland -- server

2. Check service health:
   curl http://localhost:3001/health

3. Verify data integrity:
   - Test chat completion endpoint
   - Check audit logs are accessible
   - Verify configuration is loaded

4. Monitor logs for any errors:
   tail -f /var/log/neoland/audit.log

================================================================================
EOF

    cat "${report}"
    log_success "Restore report generated: ${report}"
}

list_available_backups() {
    log_info "Scanning for available backups..."

    local storage_path="${BACKUP_STORAGE_PATH:-/var/backups/neoland}"

    if [[ ! -d "${storage_path}" ]]; then
        log_error "Backup storage path not found: ${storage_path}"
        exit 1
    fi

    echo ""
    echo "Available backups in ${storage_path}:"
    echo "========================================"

    local backup_count=0

    for backup_dir in "${storage_path}"/*/; do
        if [[ -f "${backup_dir}MANIFEST.json" ]]; then
            ((backup_count++))
            local timestamp=$(basename "${backup_dir}")
            local size=$(du -sh "${backup_dir}" | cut -f1)
            local date=$(jq -r '.backup_date' "${backup_dir}MANIFEST.json" 2>/dev/null || echo "Unknown")

            echo ""
            echo "Backup #${backup_count}:"
            echo "  Path: ${backup_dir}"
            echo "  Timestamp: ${timestamp}"
            echo "  Date: ${date}"
            echo "  Size: ${size}"

            if command -v jq &> /dev/null; then
                echo "  Components:"
                jq -r '.components | to_entries[] | "    \(.key): \(.value)"' "${backup_dir}MANIFEST.json"
            fi
        fi
    done

    if [[ ${backup_count} -eq 0 ]]; then
        log_warning "No backups found in ${storage_path}"
    else
        echo ""
        log_info "Found ${backup_count} backup(s)"
    fi
}

# ============================================================================
# Main
# ============================================================================

main() {
    if [[ -z "${BACKUP_PATH}" ]]; then
        log_error "Backup path not specified"
        log_error "Use --backup-path PATH or --list to see available backups"
        exit 1
    fi

    log_info "Starting NEOLAND restore process..."
    log_info "Backup path: ${BACKUP_PATH}"
    log_info "Components: ${COMPONENTS}"

    # Pre-flight checks
    check_dependencies
    validate_backup
    confirm_restore

    # Perform restore
    local restore_failed=0

    restore_database || restore_failed=1
    restore_audit_logs || restore_failed=1
    restore_configuration || restore_failed=1

    # Post-restore tasks
    verify_restore || restore_failed=1
    generate_restore_report

    if [[ ${restore_failed} -eq 0 ]]; then
        log_success "✅ Restore completed successfully!"
        log_info "Please restart NEOLAND services and verify functionality"
        exit 0
    else
        log_error "❌ Restore completed with errors"
        log_error "Check ${BACKUP_PATH}/restore.log for details"
        exit 1
    fi
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --backup-path)
            BACKUP_PATH="$2"
            shift 2
            ;;
        --components)
            COMPONENTS="$2"
            shift 2
            ;;
        --list)
            list_available_backups
            exit 0
            ;;
        --help)
            cat << EOF
Usage: $0 [OPTIONS]

Options:
  --backup-path PATH        Path to backup directory (required)
  --components COMPONENTS   Comma-separated list of components to restore
                           (database,audit_logs,configuration or "all")
                           Default: all
  --list                   List available backups
  --help                   Show this help message

Environment Variables:
  DATABASE_URL             PostgreSQL connection string (required for DB restore)

Examples:
  # List available backups
  $0 --list

  # Restore everything
  $0 --backup-path /var/backups/neoland/20260131_140000

  # Restore only database
  $0 --backup-path /var/backups/neoland/20260131_140000 --components database

  # Restore database and configuration
  $0 --backup-path /var/backups/neoland/20260131_140000 --components database,configuration
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
