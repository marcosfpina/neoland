# NEOLAND Backup and Disaster Recovery Scripts

This directory contains scripts for automated backups, restoration, and disaster recovery testing.

## Contents

| Script | Purpose | Usage |
|--------|---------|-------|
| `backup.sh` | Daily backup script | `./backup.sh [--retention DAYS] [--storage-path PATH]` |
| `restore.sh` | Restore from backup | `./restore.sh --backup-path PATH [--components COMPONENTS]` |
| `test_dr.sh` | DR testing automation | `./test_dr.sh` |

## Quick Start

### 1. Run Manual Backup

```bash
# Set database connection
export DATABASE_URL="postgresql://user:pass@localhost/neoland"

# Optional: S3 offsite backup
export S3_BUCKET="neoland-backups-prod"

# Run backup
./backup.sh

# Output:
# [INFO] Starting NEOLAND backup process...
# [SUCCESS] Database backup completed: /var/backups/neoland/20260131_120000/postgresql_dump.sql.gz (45MB)
# [SUCCESS] Audit logs backup completed: /var/backups/neoland/20260131_120000/audit_logs.tar.gz (12MB)
# [SUCCESS] Configuration backup completed: /var/backups/neoland/20260131_120000/configuration.tar.gz (2MB)
# [SUCCESS] ✅ Backup completed successfully!
```

### 2. Restore from Backup

```bash
# List available backups
./restore.sh --list

# Restore everything
./restore.sh --backup-path /var/backups/neoland/20260131_120000

# Restore only database
./restore.sh --backup-path /var/backups/neoland/20260131_120000 --components database
```

### 3. Test Disaster Recovery

```bash
# Run full DR test suite
DATABASE_URL=$DATABASE_URL ./test_dr.sh

# Expected output:
# Total Tests: 11
# Passed: 11
# Failed: 0
# Pass Rate: 100%
# ✅ ALL TESTS PASSED
```

## Backup Script (`backup.sh`)

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | No | - | PostgreSQL connection string |
| `BACKUP_RETENTION_DAYS` | No | 30 | Days to retain local backups |
| `BACKUP_STORAGE_PATH` | No | `/var/backups/neoland` | Local backup directory |
| `S3_BUCKET` | No | - | S3 bucket for offsite backups |

### Command-Line Options

```bash
./backup.sh [OPTIONS]

Options:
  --retention DAYS       Number of days to retain backups (default: 30)
  --storage-path PATH    Path to store backups (default: /var/backups/neoland)
  --help                 Show help message
```

### What Gets Backed Up

1. **PostgreSQL Database**
   - All tables and data
   - Vector embeddings (pgvector)
   - Indexes and constraints
   - Stored procedures

2. **Audit Logs**
   - `/var/log/neoland/audit.log`
   - Security events
   - Authentication logs
   - API access logs

3. **Configuration Files**
   - `/etc/neoland/*`
   - `~/.config/neoland/*`
   - TLS certificates (encrypted)
   - Application settings

4. **Backup Manifest**
   - JSON file with metadata
   - Backup timestamp
   - Component inventory
   - Size information
   - Retention policy

### Backup Directory Structure

```
/var/backups/neoland/
├── 20260131_120000/
│   ├── MANIFEST.json
│   ├── postgresql_dump.sql.gz
│   ├── audit_logs.tar.gz
│   ├── configuration.tar.gz
│   ├── backup.log
│   └── backup_report.txt
├── 20260130_120000/
└── 20260129_120000/
```

### S3 Offsite Backup

When `S3_BUCKET` is set, backups are automatically synced to S3:

```bash
export S3_BUCKET="neoland-backups-prod"
./backup.sh
```

**S3 Structure**:
```
s3://neoland-backups-prod/
└── neoland-backups/
    ├── 20260131_120000/
    │   ├── MANIFEST.json
    │   ├── postgresql_dump.sql.gz
    │   ├── audit_logs.tar.gz
    │   └── configuration.tar.gz
    └── ...
```

**S3 Features**:
- Server-side encryption (SSE-AES256)
- Standard-IA storage class (cost-optimized)
- Multi-region replication (optional)
- 90-day retention with lifecycle policy

## Restore Script (`restore.sh`)

### Command-Line Options

```bash
./restore.sh [OPTIONS]

Options:
  --backup-path PATH        Path to backup directory (required)
  --components COMPONENTS   Comma-separated list: database,audit_logs,configuration
                           or "all" (default)
  --list                   List available backups
  --help                   Show help message
```

### Restore Examples

#### Full Restore

Restores all components from a backup:

```bash
./restore.sh --backup-path /var/backups/neoland/20260131_120000
```

**Confirmation Required**:
```
⚠️  WARNING: This will OVERWRITE existing data!
Backup path: /var/backups/neoland/20260131_120000
Components to restore: all
Are you sure you want to continue? (yes/no):
```

#### Selective Restore

Restore only specific components:

```bash
# Database only
./restore.sh --backup-path /var/backups/neoland/20260131_120000 --components database

# Configuration only
./restore.sh --backup-path /var/backups/neoland/20260131_120000 --components configuration

# Multiple components
./restore.sh --backup-path /var/backups/neoland/20260131_120000 --components database,audit_logs
```

#### List Available Backups

```bash
./restore.sh --list

# Output:
# Available backups in /var/backups/neoland:
# ========================================
#
# Backup #1:
#   Path: /var/backups/neoland/20260131_120000/
#   Timestamp: 20260131_120000
#   Date: 2026-01-31T12:00:00+00:00
#   Size: 59M
#   Components:
#     database: true
#     audit_logs: true
#     configuration: true
```

#### Restore from S3

```bash
# 1. Download from S3
aws s3 sync \
  s3://neoland-backups-prod/neoland-backups/20260131_120000 \
  /tmp/restore-backup

# 2. Restore
./restore.sh --backup-path /tmp/restore-backup
```

### Restore Process

1. **Validation**: Verifies backup integrity and manifest
2. **Confirmation**: Prompts for user confirmation (type "yes")
3. **Database Restore**: Drops and recreates database, restores from dump
4. **Audit Logs Restore**: Backs up existing logs, restores from archive
5. **Configuration Restore**: Backs up existing config, restores from archive
6. **Verification**: Tests database connectivity and data integrity
7. **Report**: Generates restore report with next steps

### Post-Restore Verification

After restoration, verify the system:

```bash
# 1. Check database
psql $DATABASE_URL -c "SELECT COUNT(*) FROM information_schema.tables;"

# 2. Start NEOLAND
cargo run --bin neoland -- server

# 3. Health check
curl http://localhost:3001/health

# 4. Test endpoint
curl -X POST http://localhost:3001/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "X-API-Key: ${API_KEY}" \
  -d '{"messages": [{"role": "user", "content": "test"}]}'
```

## DR Testing Script (`test_dr.sh`)

### Purpose

Automated disaster recovery testing to validate:
- Backup script functionality
- Restore script functionality
- Data integrity
- RTO/RPO targets

### Usage

```bash
DATABASE_URL=$DATABASE_URL ./test_dr.sh [OPTIONS]

Options:
  --skip-backup   Skip backup tests
  --skip-restore  Skip restore tests
  --help         Show help message
```

### Test Coverage

| Test | Purpose | Pass Criteria |
|------|---------|---------------|
| **Backup script exists** | Verify script is present | File exists and executable |
| **Backup dependencies** | Check required tools | `pg_dump`, `tar`, `gzip` installed |
| **Backup execution** | Run full backup | Exit code 0, no errors |
| **Backup manifest** | Validate metadata | Valid JSON, required fields present |
| **Database backup file** | Check DB dump | File exists, valid gzip format |
| **Backup integrity** | Verify DB dump | `pg_restore --list` succeeds |
| **Backup retention** | Test cleanup policy | Multiple backups retained |
| **Restore script exists** | Verify script is present | File exists and executable |
| **Restore validation** | Check backup validation | Manifest validated correctly |
| **Restore execution** | Run full restore | Exit code 0, no errors |
| **Data integrity** | Verify restored data | Record count matches, data intact |
| **RTO measurement** | Measure restore time | Restore time < 60s (test DB) |

### Test Output

```
[TEST] Test 1: Backup script exists and is executable
[SUCCESS] ✅ PASS: Backup script exists and is executable

[TEST] Test 2: Backup dependencies are installed
[SUCCESS] ✅ PASS: All backup dependencies installed

[TEST] Test 3: Backup script executes successfully
[INFO] Creating test database: neoland_dr_test_20260131_143022
[INFO] Starting NEOLAND backup process...
[SUCCESS] ✅ PASS: Backup script executed successfully

...

========================================
DISASTER RECOVERY TEST SUMMARY
========================================

Total Tests: 12
Passed: 12
Failed: 0

Pass Rate: 100%

✅ ALL TESTS PASSED

[SUCCESS] Disaster recovery procedures validated successfully
[INFO] RTO target: 4 hours
[INFO] RPO target: 24 hours
[INFO] Retention: 30 days
```

### Running in CI/CD

```yaml
# .github/workflows/dr-test.yml
name: DR Test

on:
  schedule:
    - cron: '0 2 * * 0'  # Weekly on Sunday at 2 AM

jobs:
  test-dr:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v4

      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y postgresql-client

      - name: Run DR tests
        env:
          DATABASE_URL: postgresql://postgres:postgres@localhost:5432/postgres
        run: ./scripts/backup/test_dr.sh
```

## Automated Backup Setup

### Option 1: Cron Job

```bash
# Create cron job as neoland user
crontab -e

# Add daily backup at midnight
0 0 * * * /opt/neoland/scripts/backup/backup.sh >> /var/log/neoland/backup-cron.log 2>&1
```

### Option 2: systemd Timer (Recommended)

#### Create Timer Unit

```bash
# /etc/systemd/system/neoland-backup.timer
sudo tee /etc/systemd/system/neoland-backup.timer << 'EOF'
[Unit]
Description=NEOLAND Daily Backup Timer
Requires=neoland-backup.service

[Timer]
OnCalendar=daily
OnCalendar=*-*-* 00:00:00
Persistent=true
RandomizedDelaySec=600

[Install]
WantedBy=timers.target
EOF
```

#### Create Service Unit

```bash
# /etc/systemd/system/neoland-backup.service
sudo tee /etc/systemd/system/neoland-backup.service << 'EOF'
[Unit]
Description=NEOLAND Backup Service
After=postgresql.service

[Service]
Type=oneshot
User=neoland
Group=neoland
Environment="DATABASE_URL=postgresql://neoland:password@localhost/neoland"
Environment="S3_BUCKET=neoland-backups-prod"
ExecStart=/opt/neoland/scripts/backup/backup.sh
StandardOutput=journal
StandardError=journal
SyslogIdentifier=neoland-backup

[Install]
WantedBy=multi-user.target
EOF
```

#### Enable and Start Timer

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable timer (start on boot)
sudo systemctl enable neoland-backup.timer

# Start timer immediately
sudo systemctl start neoland-backup.timer

# Check status
sudo systemctl status neoland-backup.timer

# View logs
sudo journalctl -u neoland-backup.service -f
```

### Option 3: NixOS Configuration

```nix
# /etc/nixos/configuration.nix
{
  systemd.timers.neoland-backup = {
    wantedBy = [ "timers.target" ];
    timerConfig = {
      OnCalendar = "daily";
      Persistent = true;
      RandomizedDelaySec = "10m";
    };
  };

  systemd.services.neoland-backup = {
    description = "NEOLAND Daily Backup";
    after = [ "postgresql.service" ];

    serviceConfig = {
      Type = "oneshot";
      User = "neoland";
      Group = "neoland";
      ExecStart = "${pkgs.bash}/bin/bash /opt/neoland/scripts/backup/backup.sh";
    };

    environment = {
      DATABASE_URL = "postgresql://neoland:password@localhost/neoland";
      S3_BUCKET = "neoland-backups-prod";
    };
  };
}
```

## Monitoring and Alerts

### Backup Monitoring

Create alerts for backup failures:

```yaml
# prometheus/alerts/backup.yml
groups:
  - name: backup
    rules:
      - alert: BackupFailed
        expr: time() - neoland_backup_last_success_timestamp > 86400
        for: 1h
        annotations:
          summary: "Backup has not succeeded in 24+ hours"
          description: "Last successful backup: {{ $value }}s ago"

      - alert: BackupTooLarge
        expr: neoland_backup_size_bytes > 10737418240  # 10 GB
        annotations:
          summary: "Backup size exceeds 10 GB"
          description: "Current size: {{ humanize $value }}"

      - alert: BackupStorageLow
        expr: node_filesystem_avail_bytes{mountpoint="/var/backups"} / node_filesystem_size_bytes < 0.2
        for: 30m
        annotations:
          summary: "Backup storage below 20% free space"
```

### Log Monitoring

```bash
# Monitor backup logs in real-time
tail -f /var/log/neoland/backup.log

# Check for errors
grep -i error /var/log/neoland/backup.log

# View systemd journal
journalctl -u neoland-backup.service -f
```

## Troubleshooting

### Backup Issues

#### Disk Full

```bash
# Check disk usage
df -h /var/backups

# Clean old backups
./backup.sh --retention 7

# Change backup location
./backup.sh --storage-path /mnt/external-disk
```

#### Permission Denied

```bash
# Check backup directory permissions
ls -ld /var/backups/neoland

# Fix ownership
sudo chown -R neoland:neoland /var/backups/neoland
sudo chmod 700 /var/backups/neoland
```

#### Database Connection Failed

```bash
# Test connection
psql $DATABASE_URL -c "SELECT version();"

# Check credentials
echo $DATABASE_URL

# Verify PostgreSQL is running
systemctl status postgresql
```

### Restore Issues

#### Database Already Exists

```bash
# Drop database manually
psql postgresql://user:pass@localhost/postgres -c "DROP DATABASE neoland;"

# Or use --clean flag (already default)
./restore.sh --backup-path /path/to/backup
```

#### Restore Takes Too Long

```bash
# Monitor restore progress
tail -f /var/backups/neoland/*/restore.log

# Check database size
psql $DATABASE_URL -c "SELECT pg_size_pretty(pg_database_size('neoland'));"

# Use parallel restore (PostgreSQL 11+)
pg_restore --jobs=4 ...
```

## Best Practices

1. **Test Restores Regularly**
   - Weekly automated DR tests
   - Quarterly full DR drills
   - Document lessons learned

2. **Monitor Backup Health**
   - Set up alerts for failures
   - Review backup sizes
   - Check retention policies

3. **Secure Backups**
   - Encrypt at rest (AES-256)
   - Encrypt in transit (TLS)
   - Restrict access (chmod 700)
   - Use separate credentials

4. **Document Everything**
   - Keep DR plan updated
   - Document restore procedures
   - Maintain runbooks
   - Record test results

5. **Multiple Backup Locations**
   - Local backups (fast restore)
   - S3 offsite (disaster protection)
   - Multi-region replication
   - Consider tape archives for compliance

6. **Automate Everything**
   - Automated daily backups
   - Automated testing
   - Automated monitoring
   - Automated alerts

## Recovery Objectives

| Metric | Target | Description |
|--------|--------|-------------|
| **RTO** | 4 hours | Maximum recovery time |
| **RPO** | 24 hours | Maximum data loss |
| **Backup Frequency** | Daily | Automated backups |
| **Retention** | 30 days | Local storage |
| **Offsite Retention** | 90 days | S3 storage |

## Related Documentation

- [Disaster Recovery Runbook](../../docs/runbooks/neoland-disaster-recovery.md) - Complete DR documentation
- [Runbooks](../../docs/runbooks/) - Operational procedures
- [Architecture](../../docs/neoland-architecture.md) - System architecture
- [Progress](../../docs/neoland-progress.md) - current delivery status

## Support

For issues or questions:
- Open an issue on GitHub
- Contact: infrastructure@company.com
- On-call: PagerDuty escalation
