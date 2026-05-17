# NEOLAND Disaster Recovery Plan

**Document Version**: 1.0
**Last Updated**: 2026-01-31
**Owner**: Infrastructure Team
**Review Cycle**: Quarterly

---

## Executive Summary

This document outlines the disaster recovery (DR) procedures for the NEOLAND AI Agent Platform. It defines recovery objectives, backup strategies, restoration procedures, and testing protocols to ensure business continuity in the event of a catastrophic failure.

### Recovery Objectives

| Metric | Target | Description |
|--------|--------|-------------|
| **RTO** (Recovery Time Objective) | 4 hours | Maximum acceptable downtime |
| **RPO** (Recovery Point Objective) | 24 hours | Maximum acceptable data loss |
| **Backup Frequency** | Daily | Automated backups every 24 hours |
| **Backup Retention** | 30 days | Local and offsite retention period |
| **Testing Frequency** | Quarterly | DR drill execution |

### Critical Components

1. **PostgreSQL Database** (Vector Store + Application Data)
2. **Audit Logs** (Compliance and security events)
3. **Configuration** (System and application settings)
4. **Secrets** (API keys, certificates - managed by Vault)

---

## Table of Contents

1. [Disaster Scenarios](#disaster-scenarios)
2. [Backup Strategy](#backup-strategy)
3. [Restoration Procedures](#restoration-procedures)
4. [DR Testing](#dr-testing)
5. [Roles and Responsibilities](#roles-and-responsibilities)
6. [Communication Plan](#communication-plan)
7. [Recovery Workflow](#recovery-workflow)
8. [Appendices](#appendices)

---

## 1. Disaster Scenarios

### Scenario 1: Database Corruption

**Impact**: Data inconsistency, service degradation
**Probability**: Medium
**Detection**: Automated health checks, error logs
**Recovery**: Database restore from latest backup

### Scenario 2: Complete Server Failure

**Impact**: Total service outage
**Probability**: Low
**Detection**: Monitoring alerts, service unavailability
**Recovery**: Full system restore on new infrastructure

### Scenario 3: Data Center Outage

**Impact**: Regional service disruption
**Probability**: Low
**Detection**: Multi-region monitoring
**Recovery**: Failover to secondary region, restore from S3

### Scenario 4: Ransomware/Malware Attack

**Impact**: Data encryption, system compromise
**Probability**: Low
**Detection**: Security monitoring, anomaly detection
**Recovery**: Restore from verified clean backup, security audit

### Scenario 5: Accidental Data Deletion

**Impact**: Partial data loss
**Probability**: Medium
**Detection**: User reports, audit logs
**Recovery**: Selective restore from backup

### Scenario 6: Storage Failure

**Impact**: Data unavailability
**Probability**: Low
**Detection**: Storage monitoring alerts
**Recovery**: Restore from offsite backup

---

## 2. Backup Strategy

### 2.1 Backup Schedule

```
Daily:  00:00 UTC - Full backup
Weekly: Sunday 02:00 UTC - Verification and testing
Monthly: 1st of month - Archive to cold storage
```

### 2.2 Backup Components

#### Database Backup

**Tool**: `pg_dump` with custom format
**Location**: `/var/backups/neoland/{timestamp}/postgresql_dump.sql.gz`
**Compression**: gzip level 9
**Encryption**: AES-256 (at rest)

**What's Included**:
- Vector embeddings (pgvector)
- Application tables
- User data
- Indexes and constraints

**What's Excluded**:
- Temporary tables
- Cache data
- Session data

#### Audit Logs Backup

**Tool**: `tar` + gzip
**Location**: `/var/backups/neoland/{timestamp}/audit_logs.tar.gz`
**Retention**: 90 days (compliance requirement)

**What's Included**:
- Security audit events
- Authentication logs
- API access logs
- Configuration changes

#### Configuration Backup

**Tool**: `tar` + gzip
**Location**: `/var/backups/neoland/{timestamp}/configuration.tar.gz`

**What's Included**:
- `/etc/neoland/*`
- `~/.config/neoland/*`
- TLS certificates (encrypted)
- Application configuration files

**What's Excluded**:
- Secrets (managed by Vault)
- Temporary files
- Lock files

### 2.3 Backup Storage

#### Local Storage

**Path**: `/var/backups/neoland`
**Retention**: 30 days
**Permissions**: `700` (owner only)
**Monitoring**: Disk space alerts at 80% capacity

#### Offsite Storage (S3)

**Bucket**: `s3://neoland-backups-prod`
**Region**: Multi-region replication (us-east-1, eu-west-1)
**Storage Class**: Standard-IA (Infrequent Access)
**Encryption**: Server-side (SSE-S3) + Client-side
**Retention**: 90 days with lifecycle policy
**Versioning**: Enabled

### 2.4 Backup Verification

**Automated Checks** (post-backup):
1. File integrity (checksums)
2. Backup size validation (min/max thresholds)
3. `pg_restore --list` verification
4. Archive extraction test

**Manual Verification** (weekly):
1. Random backup restore to staging
2. Data integrity spot checks
3. Application startup test

### 2.5 Backup Execution

#### Manual Backup

```bash
# Full backup
./scripts/backup/backup.sh

# Custom retention
./scripts/backup/backup.sh --retention 90

# Custom storage path
./scripts/backup/backup.sh --storage-path /mnt/backups
```

#### Automated Backup (Cron)

```bash
# /etc/cron.d/neoland-backup
0 0 * * * neoland /opt/neoland/scripts/backup/backup.sh >> /var/log/neoland/backup.log 2>&1
```

#### Automated Backup (systemd timer)

```ini
# /etc/systemd/system/neoland-backup.timer
[Unit]
Description=NEOLAND Daily Backup

[Timer]
OnCalendar=daily
Persistent=true

[Install]
WantedBy=timers.target
```

```ini
# /etc/systemd/system/neoland-backup.service
[Unit]
Description=NEOLAND Backup Service

[Service]
Type=oneshot
ExecStart=/opt/neoland/scripts/backup/backup.sh
User=neoland
StandardOutput=journal
StandardError=journal
```

---

## 3. Restoration Procedures

### 3.1 Pre-Restoration Checklist

- [ ] Verify backup integrity
- [ ] Identify root cause of failure
- [ ] Notify stakeholders (use communication plan)
- [ ] Ensure infrastructure is stable
- [ ] Review backup manifest
- [ ] Confirm DATABASE_URL and credentials
- [ ] Stop running NEOLAND services

### 3.2 Full System Restore

```bash
# 1. List available backups
./scripts/backup/restore.sh --list

# 2. Verify backup
export DATABASE_URL="postgresql://user:pass@localhost/neoland"
./scripts/backup/restore.sh --backup-path /var/backups/neoland/20260131_120000

# 3. Confirm restoration (type "yes" when prompted)
# Output: Backup validation, component restore, verification

# 4. Start services
systemctl start neoland
# OR
cargo run --bin neoland -- server

# 5. Verify health
curl http://localhost:3001/health
```

**Expected Downtime**: 30 minutes - 2 hours (depending on data size)

### 3.3 Selective Component Restore

#### Database Only

```bash
./scripts/backup/restore.sh \
  --backup-path /var/backups/neoland/20260131_120000 \
  --components database
```

#### Audit Logs Only

```bash
./scripts/backup/restore.sh \
  --backup-path /var/backups/neoland/20260131_120000 \
  --components audit_logs
```

#### Configuration Only

```bash
./scripts/backup/restore.sh \
  --backup-path /var/backups/neoland/20260131_120000 \
  --components configuration
```

### 3.4 Point-in-Time Recovery (PITR)

For PostgreSQL with Write-Ahead Logging (WAL):

```bash
# Enable WAL archiving (postgresql.conf)
wal_level = replica
archive_mode = on
archive_command = 'test ! -f /var/lib/postgresql/wal_archive/%f && cp %p /var/lib/postgresql/wal_archive/%f'

# Restore to specific timestamp
pg_restore \
  --dbname=neoland \
  --clean \
  /var/backups/neoland/20260131_120000/postgresql_dump.sql.gz

# Apply WAL logs up to target time
recovery_target_time = '2026-01-31 14:30:00'
```

### 3.5 Cross-Region Restore

**Scenario**: Primary region is unavailable, restore in secondary region

```bash
# 1. Download backup from S3
aws s3 sync \
  s3://neoland-backups-prod/neoland-backups/20260131_120000 \
  /tmp/restore-20260131

# 2. Provision new infrastructure (Terraform/NixOS)
cd terraform/
terraform apply -var="region=eu-west-1"

# 3. Restore database
export DATABASE_URL="postgresql://user:pass@new-db.eu-west-1.rds.amazonaws.com/neoland"
./scripts/backup/restore.sh --backup-path /tmp/restore-20260131

# 4. Update DNS to point to new region
# 5. Verify service health
# 6. Monitor for issues
```

**Expected Downtime**: 2-4 hours

### 3.6 Post-Restoration Verification

```bash
# 1. Check database connectivity
psql $DATABASE_URL -c "SELECT version();"

# 2. Verify table counts
psql $DATABASE_URL -c "
  SELECT schemaname, tablename,
         pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size
  FROM pg_tables
  WHERE schemaname = 'public'
  ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
"

# 3. Test chat completion endpoint
curl -X POST http://localhost:3001/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "X-API-Key: ${API_KEY}" \
  -d '{"messages": [{"role": "user", "content": "test"}]}'

# 4. Check audit logs
tail -n 100 /var/log/neoland/audit.log

# 5. Verify metrics endpoint
curl http://localhost:3001/metrics

# 6. Monitor error rates
# Check Grafana dashboard for anomalies
```

---

## 4. DR Testing

### 4.1 Quarterly DR Drill

**Objective**: Validate backup and restore procedures
**Frequency**: Every 3 months
**Duration**: 4 hours
**Participants**: Infrastructure team, Engineering lead, On-call engineer

#### Drill Procedure

```bash
# 1. Automated DR test
DATABASE_URL=$DATABASE_URL ./scripts/backup/test_dr.sh

# Expected output:
# ✅ PASS: Backup script exists and is executable
# ✅ PASS: Backup dependencies are installed
# ✅ PASS: Backup script executed successfully
# ✅ PASS: Backup manifest is valid JSON
# ✅ PASS: Database backup file is valid gzip
# ✅ PASS: Database backup integrity verified
# ✅ PASS: Restore script exists and is executable
# ✅ PASS: Data integrity after restore
# ✅ PASS: RTO: 45s < 60s target
#
# Pass Rate: 100%
# ✅ ALL TESTS PASSED
```

#### Manual Drill Steps

1. **Preparation** (30 min)
   - Schedule maintenance window
   - Notify stakeholders
   - Prepare fresh infrastructure (staging)

2. **Backup Validation** (30 min)
   - Verify latest backup exists
   - Check backup integrity
   - Review backup manifest

3. **Restore Execution** (1.5 hours)
   - Restore database to staging
   - Restore configuration
   - Start services
   - Verify functionality

4. **Testing** (1 hour)
   - Execute smoke tests
   - Verify data integrity
   - Check all endpoints
   - Monitor metrics

5. **Documentation** (30 min)
   - Record RTO/RPO metrics
   - Document issues encountered
   - Update procedures if needed
   - Generate DR test report

### 4.2 DR Test Report Template

```markdown
# DR Test Report - YYYY-MM-DD

## Test Summary
- **Date**: YYYY-MM-DD
- **Duration**: X hours
- **Participants**: Names
- **Environment**: Staging
- **Backup Used**: /var/backups/neoland/YYYYMMDD_HHMMSS

## Metrics
- **RTO Achieved**: X hours (Target: 4 hours)
- **RPO Achieved**: X hours (Target: 24 hours)
- **Data Loss**: None/Minimal/Significant
- **Success Rate**: X%

## Issues Encountered
1. Issue description
   - Root cause
   - Resolution
   - Action items

## Procedure Updates
- [ ] Update backup script
- [ ] Update restore script
- [ ] Update documentation
- [ ] Train team on changes

## Recommendations
1. Recommendation 1
2. Recommendation 2

## Sign-off
- Infrastructure Lead: ___________
- Engineering Manager: ___________
```

### 4.3 Continuous Testing

**Backup Verification** (daily):
- Automated integrity checks
- File size validation
- `pg_restore --list` verification

**Restore Testing** (weekly):
- Automated restore to test database
- Data integrity spot checks
- Performance benchmarks

---

## 5. Roles and Responsibilities

### Incident Commander (IC)

**Primary**: Infrastructure Lead
**Backup**: Engineering Manager

**Responsibilities**:
- Declare disaster
- Activate DR plan
- Coordinate recovery efforts
- Communicate with stakeholders
- Make critical decisions

### Database Administrator (DBA)

**Primary**: Senior Backend Engineer
**Backup**: DevOps Engineer

**Responsibilities**:
- Execute database restore
- Verify data integrity
- Optimize post-restore performance
- Monitor database health

### Infrastructure Engineer

**Primary**: DevOps Engineer
**Backup**: SRE

**Responsibilities**:
- Provision new infrastructure
- Configure networking
- Manage secrets and credentials
- Deploy application

### Application Engineer

**Primary**: Backend Lead
**Backup**: Full-stack Engineer

**Responsibilities**:
- Verify application functionality
- Execute smoke tests
- Monitor application logs
- Fix application-level issues

### Communications Lead

**Primary**: Engineering Manager
**Backup**: Product Manager

**Responsibilities**:
- Notify stakeholders
- Provide status updates
- Manage customer communications
- Document incident timeline

---

## 6. Communication Plan

### Stakeholder Matrix

| Stakeholder | Notification Method | Update Frequency | Contact |
|-------------|---------------------|------------------|---------|
| **Engineering Team** | Slack #incidents | Real-time | @engineering |
| **Executive Team** | Email + Phone | Hourly | exec-team@company.com |
| **Customers** | Status page | Every 2 hours | status.neoland.com |
| **Support Team** | Slack #support | Every 30 min | @support |
| **Investors** | Email | Daily | investors@company.com |

### Communication Templates

#### Initial Incident Notification

```
Subject: [P1] NEOLAND Service Disruption - DR Activation

Priority: P1 - Critical
Status: Investigating

We are experiencing a service disruption affecting [component].
The disaster recovery plan has been activated.

Impact: [description]
Started: [timestamp]
RTO: 4 hours
Next Update: [timestamp + 1 hour]

Incident Commander: [name]
Incident Channel: #incident-YYYY-MM-DD
```

#### Progress Update

```
Subject: [P1] NEOLAND DR Update - [HH:MM]

Status: [In Progress/Resolved]
Elapsed: [X hours Y minutes]
Progress: [X%]

Actions Completed:
- [Action 1]
- [Action 2]

Current Status:
- [Current activity]

Next Steps:
- [Next action]

Next Update: [timestamp]
```

#### Resolution Notification

```
Subject: [RESOLVED] NEOLAND Service Restored

Status: Resolved
Downtime: [X hours Y minutes]
RTO Met: [Yes/No]
RPO Met: [Yes/No]

The service has been fully restored and verified.

Recovery Summary:
- Database restored from backup: [timestamp]
- Data loss: [None/Minimal description]
- Services restarted: [timestamp]
- Verification completed: [timestamp]

Post-Incident Review:
- Scheduled: [date/time]
- Participants: [names]

Thank you for your patience during this incident.
```

---

## 7. Recovery Workflow

### Flowchart

```
[Incident Detected]
        |
        v
[Assess Severity] --> [Not Critical] --> [Standard Incident Response]
        |
        v
   [Critical]
        |
        v
[Activate DR Plan]
        |
        v
[Notify Stakeholders]
        |
        v
[Identify Recovery Strategy]
        |
        +-- [Database Restore] --> [Restore from Backup]
        |
        +-- [Full System] --> [Provision Infrastructure] --> [Restore All Components]
        |
        +-- [Cross-Region] --> [S3 Restore] --> [Regional Failover]
        |
        v
[Execute Restoration]
        |
        v
[Verify Integrity]
        |
        v
[Start Services]
        |
        v
[Smoke Tests] --> [Fail] --> [Rollback/Investigate]
        |
        v
   [Success]
        |
        v
[Monitor Metrics]
        |
        v
[Declare Resolved]
        |
        v
[Post-Incident Review]
        |
        v
[Update DR Plan]
```

### Decision Tree

**Q1: Is the database accessible?**
- NO → Database restore required
- YES → Check application layer

**Q2: Is the application running?**
- NO → Restart application services
- YES → Check configuration

**Q3: Is data corrupted?**
- YES → Point-in-time recovery or full restore
- NO → Investigate other issues

**Q4: Is the infrastructure available?**
- NO → Cross-region failover
- YES → Check network/connectivity

---

## 8. Appendices

### Appendix A: Backup Manifest Example

```json
{
  "backup_timestamp": "20260131_120000",
  "backup_date": "2026-01-31T12:00:00+00:00",
  "hostname": "neoland-prod-01",
  "neoland_version": "0.1.0",
  "components": {
    "database": true,
    "audit_logs": true,
    "configuration": true
  },
  "backup_size_bytes": 1073741824,
  "backup_size_human": "1.0G",
  "retention_until": "2026-03-02T12:00:00+00:00"
}
```

### Appendix B: Common Issues and Solutions

#### Issue: Backup script fails with "disk full"

**Solution**:
```bash
# Check disk usage
df -h /var/backups

# Clean old backups manually
./scripts/backup/backup.sh --retention 7

# Increase storage or change backup path
./scripts/backup/backup.sh --storage-path /mnt/large-disk
```

#### Issue: Restore fails with "database already exists"

**Solution**:
```bash
# Drop existing database
psql postgresql://user:pass@localhost/postgres -c "DROP DATABASE neoland;"

# Re-run restore
./scripts/backup/restore.sh --backup-path /path/to/backup
```

#### Issue: S3 sync fails with "access denied"

**Solution**:
```bash
# Verify AWS credentials
aws sts get-caller-identity

# Check IAM permissions
aws iam get-user-policy --user-name backup-user --policy-name s3-access

# Test S3 access
aws s3 ls s3://neoland-backups-prod/
```

### Appendix C: Contact Information

| Role | Name | Phone | Email | Pager |
|------|------|-------|-------|-------|
| Incident Commander | TBD | +1-XXX-XXX-XXXX | ic@company.com | PagerDuty |
| DBA Lead | TBD | +1-XXX-XXX-XXXX | dba@company.com | PagerDuty |
| Infrastructure Lead | TBD | +1-XXX-XXX-XXXX | infra@company.com | PagerDuty |
| On-Call Engineer | Rotation | +1-XXX-XXX-XXXX | oncall@company.com | PagerDuty |

### Appendix D: Related Documents

- [ARCHITECTURE.md](./ARCHITECTURE.md) - System architecture
- [RUNBOOKS.md](./runbooks/README.md) - Operational runbooks
- [SECURITY.md](./SECURITY.md) - Security documentation
- [PROGRESS.md](./PROGRESS.md) - Production readiness status

### Appendix E: Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-31 | Infrastructure Team | Initial DR plan creation |

---

## Document Approval

| Role | Name | Signature | Date |
|------|------|-----------|------|
| Infrastructure Lead | __________ | __________ | ______ |
| Engineering Manager | __________ | __________ | ______ |
| CTO | __________ | __________ | ______ |

---

**Next Review Date**: 2026-05-01 (Quarterly)
