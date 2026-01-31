#!/usr/bin/env bash
#
# NEOLAND Backup Script
# Performs daily backups of critical data (PostgreSQL, audit logs, configuration)
#
# Usage:
#   ./backup.sh [--retention DAYS] [--storage-path PATH]
#
# Environment Variables:
#   BACKUP_RETENTION_DAYS  - Number of days to retain backups (default: 30)
#   BACKUP_STORAGE_PATH    - Path to store backups (default: /var/backups/neoland)
#   DATABASE_URL           - PostgreSQL connection string
#   S3_BUCKET             - S3 bucket for offsite backups (optional)
#

set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TIMESTAMP="$(date +%Y%m%d_%H%M%S)"
RETENTION_DAYS="${BACKUP_RETENTION_DAYS:-30}"
STORAGE_PATH="${BACKUP_STORAGE_PATH:-/var/backups/neoland}"
BACKUP_DIR="${STORAGE_PATH}/${TIMESTAMP}"

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

    if ! command -v pg_dump &> /dev/null; then
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

create_backup_dir() {
    log_info "Creating backup directory: ${BACKUP_DIR}"
    mkdir -p "${BACKUP_DIR}"
    chmod 700 "${BACKUP_DIR}"
}

backup_database() {
    log_info "Backing up PostgreSQL database..."

    if [[ -z "${DATABASE_URL:-}" ]]; then
        log_warning "DATABASE_URL not set, skipping database backup"
        return 0
    fi

    local db_backup="${BACKUP_DIR}/postgresql_dump.sql"
    local db_backup_gz="${db_backup}.gz"

    # Use pg_dump with custom format for better compression and restore options
    if pg_dump "${DATABASE_URL}" \
        --format=custom \
        --compress=9 \
        --file="${db_backup_gz}" \
        --verbose 2>&1 | tee -a "${BACKUP_DIR}/backup.log"; then

        local size=$(du -h "${db_backup_gz}" | cut -f1)
        log_success "Database backup completed: ${db_backup_gz} (${size})"

        # Verify backup integrity
        if pg_restore --list "${db_backup_gz}" > /dev/null 2>&1; then
            log_success "Database backup integrity verified"
        else
            log_error "Database backup verification failed!"
            return 1
        fi
    else
        log_error "Database backup failed"
        return 1
    fi
}

backup_audit_logs() {
    log_info "Backing up audit logs..."

    local audit_log_path="/var/log/neoland/audit.log"

    if [[ ! -f "${audit_log_path}" ]]; then
        log_warning "Audit log not found at ${audit_log_path}, skipping"
        return 0
    fi

    local audit_backup="${BACKUP_DIR}/audit_logs.tar.gz"

    if tar -czf "${audit_backup}" \
        -C /var/log/neoland \
        audit.log \
        --exclude='*.tmp' \
        2>&1 | tee -a "${BACKUP_DIR}/backup.log"; then

        local size=$(du -h "${audit_backup}" | cut -f1)
        local line_count=$(zcat "${audit_backup}" | wc -l)
        log_success "Audit logs backup completed: ${audit_backup} (${size}, ${line_count} entries)"
    else
        log_error "Audit logs backup failed"
        return 1
    fi
}

backup_configuration() {
    log_info "Backing up configuration files..."

    local config_paths=(
        "/etc/neoland"
        "${HOME}/.config/neoland"
    )

    local config_backup="${BACKUP_DIR}/configuration.tar.gz"
    local files_to_backup=()

    # Collect existing config paths
    for path in "${config_paths[@]}"; do
        if [[ -e "${path}" ]]; then
            files_to_backup+=("${path}")
        fi
    done

    if [[ ${#files_to_backup[@]} -eq 0 ]]; then
        log_warning "No configuration files found, skipping"
        return 0
    fi

    if tar -czf "${config_backup}" \
        "${files_to_backup[@]}" \
        --exclude='*.tmp' \
        --exclude='*.lock' \
        2>&1 | tee -a "${BACKUP_DIR}/backup.log"; then

        local size=$(du -h "${config_backup}" | cut -f1)
        log_success "Configuration backup completed: ${config_backup} (${size})"
    else
        log_error "Configuration backup failed"
        return 1
    fi
}

backup_vector_store() {
    log_info "Backing up vector store data..."

    # Note: This assumes PostgreSQL with pgvector extension
    # If using in-memory store, this will be empty

    if [[ -z "${DATABASE_URL:-}" ]]; then
        log_warning "Vector store is in-memory only (no persistence), skipping"
        return 0
    fi

    # Vector embeddings are part of the main database backup
    log_info "Vector store data included in PostgreSQL backup"
}

create_backup_manifest() {
    log_info "Creating backup manifest..."

    local manifest="${BACKUP_DIR}/MANIFEST.json"

    cat > "${manifest}" << EOF
{
  "backup_timestamp": "${TIMESTAMP}",
  "backup_date": "$(date -Iseconds)",
  "hostname": "$(hostname)",
  "neoland_version": "$(cargo pkgid 2>/dev/null | cut -d'#' -f2 || echo 'unknown')",
  "components": {
    "database": $([ -f "${BACKUP_DIR}/postgresql_dump.sql.gz" ] && echo "true" || echo "false"),
    "audit_logs": $([ -f "${BACKUP_DIR}/audit_logs.tar.gz" ] && echo "true" || echo "false"),
    "configuration": $([ -f "${BACKUP_DIR}/configuration.tar.gz" ] && echo "true" || echo "false")
  },
  "backup_size_bytes": $(du -sb "${BACKUP_DIR}" | cut -f1),
  "backup_size_human": "$(du -sh "${BACKUP_DIR}" | cut -f1)",
  "retention_until": "$(date -d "+${RETENTION_DAYS} days" -Iseconds)"
}
EOF

    log_success "Backup manifest created: ${manifest}"
}

sync_to_s3() {
    if [[ -z "${S3_BUCKET:-}" ]]; then
        log_info "S3_BUCKET not set, skipping offsite backup"
        return 0
    fi

    log_info "Syncing backup to S3: ${S3_BUCKET}"

    if command -v aws &> /dev/null; then
        if aws s3 sync "${BACKUP_DIR}" "s3://${S3_BUCKET}/neoland-backups/${TIMESTAMP}" \
            --storage-class STANDARD_IA \
            --sse AES256 \
            2>&1 | tee -a "${BACKUP_DIR}/backup.log"; then
            log_success "S3 sync completed: s3://${S3_BUCKET}/neoland-backups/${TIMESTAMP}"
        else
            log_error "S3 sync failed"
            return 1
        fi
    else
        log_warning "AWS CLI not found, skipping S3 sync"
        log_warning "Install with: nix-shell -p awscli2"
    fi
}

cleanup_old_backups() {
    log_info "Cleaning up backups older than ${RETENTION_DAYS} days..."

    local deleted_count=0

    # Find and delete old local backups
    while IFS= read -r -d '' backup_path; do
        log_info "Deleting old backup: ${backup_path}"
        rm -rf "${backup_path}"
        ((deleted_count++))
    done < <(find "${STORAGE_PATH}" -maxdepth 1 -type d -mtime "+${RETENTION_DAYS}" -print0)

    if [[ ${deleted_count} -gt 0 ]]; then
        log_success "Deleted ${deleted_count} old backup(s)"
    else
        log_info "No old backups to delete"
    fi

    # Cleanup old S3 backups if configured
    if [[ -n "${S3_BUCKET:-}" ]] && command -v aws &> /dev/null; then
        log_info "Cleaning up old S3 backups..."

        local cutoff_date=$(date -d "-${RETENTION_DAYS} days" +%s)

        aws s3 ls "s3://${S3_BUCKET}/neoland-backups/" | while read -r line; do
            local backup_name=$(echo "${line}" | awk '{print $2}')
            if [[ -n "${backup_name}" ]]; then
                # Extract timestamp from backup name (format: YYYYMMDD_HHMMSS)
                local backup_date=$(echo "${backup_name}" | grep -oP '\d{8}' || echo "")
                if [[ -n "${backup_date}" ]]; then
                    local backup_timestamp=$(date -d "${backup_date}" +%s 2>/dev/null || echo "0")
                    if [[ ${backup_timestamp} -lt ${cutoff_date} ]]; then
                        log_info "Deleting old S3 backup: ${backup_name}"
                        aws s3 rm "s3://${S3_BUCKET}/neoland-backups/${backup_name}" --recursive
                    fi
                fi
            fi
        done
    fi
}

generate_backup_report() {
    local report="${BACKUP_DIR}/backup_report.txt"

    cat > "${report}" << EOF
================================================================================
NEOLAND Backup Report
================================================================================

Backup Timestamp: ${TIMESTAMP}
Backup Date: $(date)
Hostname: $(hostname)
Retention Until: $(date -d "+${RETENTION_DAYS} days")

--------------------------------------------------------------------------------
Backup Contents
--------------------------------------------------------------------------------

EOF

    if [[ -f "${BACKUP_DIR}/postgresql_dump.sql.gz" ]]; then
        echo "✅ PostgreSQL Database: $(du -h "${BACKUP_DIR}/postgresql_dump.sql.gz" | cut -f1)" >> "${report}"
    else
        echo "❌ PostgreSQL Database: NOT BACKED UP" >> "${report}"
    fi

    if [[ -f "${BACKUP_DIR}/audit_logs.tar.gz" ]]; then
        echo "✅ Audit Logs: $(du -h "${BACKUP_DIR}/audit_logs.tar.gz" | cut -f1)" >> "${report}"
    else
        echo "⚠️  Audit Logs: NOT FOUND" >> "${report}"
    fi

    if [[ -f "${BACKUP_DIR}/configuration.tar.gz" ]]; then
        echo "✅ Configuration: $(du -h "${BACKUP_DIR}/configuration.tar.gz" | cut -f1)" >> "${report}"
    else
        echo "⚠️  Configuration: NOT FOUND" >> "${report}"
    fi

    cat >> "${report}" << EOF

--------------------------------------------------------------------------------
Backup Statistics
--------------------------------------------------------------------------------

Total Size: $(du -sh "${BACKUP_DIR}" | cut -f1)
Total Files: $(find "${BACKUP_DIR}" -type f | wc -l)
Storage Path: ${BACKUP_DIR}

EOF

    if [[ -n "${S3_BUCKET:-}" ]]; then
        echo "S3 Bucket: s3://${S3_BUCKET}/neoland-backups/${TIMESTAMP}" >> "${report}"
    fi

    cat >> "${report}" << EOF

================================================================================
EOF

    cat "${report}"
    log_success "Backup report generated: ${report}"
}

# ============================================================================
# Main
# ============================================================================

main() {
    log_info "Starting NEOLAND backup process..."
    log_info "Timestamp: ${TIMESTAMP}"
    log_info "Storage Path: ${STORAGE_PATH}"
    log_info "Retention: ${RETENTION_DAYS} days"

    # Pre-flight checks
    check_dependencies
    create_backup_dir

    # Perform backups
    local backup_failed=0

    backup_database || backup_failed=1
    backup_audit_logs || backup_failed=1
    backup_configuration || backup_failed=1
    backup_vector_store || backup_failed=1

    # Post-backup tasks
    create_backup_manifest
    sync_to_s3 || log_warning "S3 sync failed but continuing..."
    cleanup_old_backups
    generate_backup_report

    if [[ ${backup_failed} -eq 0 ]]; then
        log_success "✅ Backup completed successfully!"
        log_success "Backup location: ${BACKUP_DIR}"
        exit 0
    else
        log_error "❌ Backup completed with errors"
        exit 1
    fi
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --retention)
            RETENTION_DAYS="$2"
            shift 2
            ;;
        --storage-path)
            STORAGE_PATH="$2"
            shift 2
            ;;
        --help)
            cat << EOF
Usage: $0 [OPTIONS]

Options:
  --retention DAYS       Number of days to retain backups (default: 30)
  --storage-path PATH    Path to store backups (default: /var/backups/neoland)
  --help                 Show this help message

Environment Variables:
  DATABASE_URL           PostgreSQL connection string
  S3_BUCKET             S3 bucket for offsite backups (optional)

Example:
  $0 --retention 90 --storage-path /mnt/backups/neoland
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
