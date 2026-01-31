# Runbook: NeolandDiskSpaceLow / NeolandDiskSpaceCritical

**Alert**: `NeolandDiskSpaceLow` (warning) / `NeolandDiskSpaceCritical` (critical)
**Severity**: WARNING ⚠️ / CRITICAL 🔴
**Response Time**: 1-4 hours (warning) / 0-30 min (critical)

---

## Symptoms

- **Alert**: Disk space low on NEOLAND or PostgreSQL pods
- **User Impact**:
  - Warning (70-85%): No immediate impact, action needed
  - High (85-95%): Performance degradation, writes may fail soon
  - Critical (>95%): Service failure imminent, database writes failing
- **Metrics**:
  - Warning: `(node_filesystem_avail_bytes / node_filesystem_size_bytes) < 0.30` (<30% free)
  - Critical: `(node_filesystem_avail_bytes / node_filesystem_size_bytes) < 0.10` (<10% free)
- **Common Causes**:
  - Log files accumulating (no rotation)
  - Database growing (PostgreSQL WAL, data)
  - Container image cache buildup
  - Temporary files not cleaned
  - PVC too small for workload

---

## Investigation

### Step 1: Identify Which Pod/Volume is Low on Space

```bash
# Check disk usage on all NEOLAND pods
kubectl exec -it deployment/neoland -n default -- df -h

# Expected output:
# Filesystem      Size  Used Avail Use% Mounted on
# /dev/sda1        50G   38G   10G  80% /
# /dev/sdb1       100G   72G   23G  76% /var/lib/postgresql/data

# If Use% >85% → Critical threshold approaching

# Check disk usage on PostgreSQL
kubectl exec -it deployment/postgres -n default -- df -h /var/lib/postgresql/data

# Check PVC status
kubectl get pvc -n default

# Expected:
# NAME            STATUS   CAPACITY   ACCESS MODES
# postgres-data   Bound    100Gi      RWO
```

### Step 2: Identify What's Using Space

```bash
# Check top space consumers in NEOLAND pod
kubectl exec -it deployment/neoland -n default -- du -sh /* 2>/dev/null | sort -hr | head -10

# Common culprits:
# /var/log → Log files
# /tmp → Temporary files
# /var/lib/docker → Container layers (shouldn't be in pod)

# Check log directory specifically
kubectl exec -it deployment/neoland -n default -- du -sh /var/log/*

# Check PostgreSQL database size
kubectl exec -it deployment/postgres -n default -- \
  psql -U postgres -c "SELECT pg_database.datname, pg_size_pretty(pg_database_size(pg_database.datname)) FROM pg_database ORDER BY pg_database_size(pg_database.datname) DESC;"

# Check PostgreSQL WAL files
kubectl exec -it deployment/postgres -n default -- \
  du -sh /var/lib/postgresql/data/pg_wal
```

### Step 3: Check for Log Rotation Issues

```bash
# Check if log rotation configured
kubectl exec -it deployment/neoland -n default -- cat /etc/logrotate.conf

# Check recent log files
kubectl exec -it deployment/neoland -n default -- ls -lh /var/log/ | grep -E "\.log$"

# Check for very large log files (>1GB)
kubectl exec -it deployment/neoland -n default -- \
  find /var/log -type f -size +1G -exec ls -lh {} \;
```

### Step 4: Check Node Disk Space (if pod issue)

```bash
# Check node disk space
kubectl get nodes -o wide
kubectl describe node <node-name> | grep -A 10 "Allocated resources"

# Check node filesystem
ssh <node-name> df -h

# If node disk full → Node-level issue (affects all pods on node)
```

---

## Resolution

### Scenario 1: Log Files Accumulating (Most Common)

**Symptoms**: /var/log using most space, large .log files, no rotation

```bash
# 1. Identify large log files
kubectl exec -it deployment/neoland -n default -- \
  find /var/log -type f -size +100M -exec ls -lh {} \;

# 2. Compress old logs (quick space recovery)
kubectl exec -it deployment/neoland -n default -- \
  find /var/log -type f -name "*.log" -mtime +7 -exec gzip {} \;

# 3. Delete very old compressed logs (>30 days)
kubectl exec -it deployment/neoland -n default -- \
  find /var/log -type f -name "*.gz" -mtime +30 -delete

# 4. Truncate current large log (if safe)
kubectl exec -it deployment/neoland -n default -- \
  truncate -s 0 /var/log/neoland.log

# ⚠️ WARNING: This deletes log content. Only if not needed for debugging.

# 5. Implement log rotation (permanent fix)
# Add to Dockerfile or init container:
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: ConfigMap
metadata:
  name: logrotate-config
  namespace: default
data:
  logrotate.conf: |
    /var/log/*.log {
        daily
        rotate 7
        compress
        delaycompress
        missingok
        notifempty
        create 0640 root root
    }
EOF

# 6. Verify space freed
kubectl exec -it deployment/neoland -n default -- df -h
```

**Timeframe**: 5-10 minutes

### Scenario 2: PostgreSQL WAL Files Accumulating

**Symptoms**: /var/lib/postgresql/data/pg_wal using >10GB, old WAL files not cleaned

```bash
# 1. Check WAL file size
kubectl exec -it deployment/postgres -n default -- \
  du -sh /var/lib/postgresql/data/pg_wal

# 2. Check PostgreSQL archiving status
kubectl exec -it deployment/postgres -n default -- \
  psql -U postgres -c "SELECT * FROM pg_stat_archiver;"

# If archive_command failing → WAL files pile up

# 3. Force checkpoint (flushes WAL to disk)
kubectl exec -it deployment/postgres -n default -- \
  psql -U postgres -c "CHECKPOINT;"

# 4. Tune WAL settings (reduce retention)
kubectl exec -it deployment/postgres -n default -- \
  psql -U postgres -c "ALTER SYSTEM SET wal_keep_size = '1GB';"

# Current: default (often 16GB+)
# New: 1GB (keeps last ~60 WAL files)

# 5. Restart PostgreSQL to apply
kubectl rollout restart deployment/postgres -n default

# 6. Manually remove old WAL files (DANGER - only if desperate)
# ⚠️ DO NOT DO THIS unless PostgreSQL is stopped and backed up
# kubectl exec -it deployment/postgres -- \
#   find /var/lib/postgresql/data/pg_wal -type f -mtime +7 -delete
```

**Timeframe**: 5-15 minutes

### Scenario 3: Database Too Large for PVC

**Symptoms**: PostgreSQL data using >90% of PVC, legitimate data growth

```bash
# 1. Check current PVC size
kubectl get pvc postgres-data -n default -o jsonpath='{.spec.resources.requests.storage}'

# 2. Check how much space used
kubectl exec -it deployment/postgres -n default -- df -h /var/lib/postgresql/data

# 3. Increase PVC size (if supported by storage class)
kubectl patch pvc postgres-data -n default -p '{"spec":{"resources":{"requests":{"storage":"200Gi"}}}}'

# Current: 100Gi
# New: 200Gi

# Note: Some storage classes don't support online resize

# 4. If resize not supported, migrate to larger PVC:

# a. Create backup
kubectl exec -it deployment/postgres -n default -- \
  pg_dumpall -U postgres > /tmp/postgres-backup.sql

# b. Create new larger PVC
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: postgres-data-large
  namespace: default
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 200Gi
  storageClassName: fast-ssd
EOF

# c. Scale down PostgreSQL
kubectl scale deployment/postgres --replicas=0 -n default

# d. Copy data to new PVC (using init container or manual copy)
# e. Update deployment to use new PVC
# f. Restore from backup
# g. Scale up PostgreSQL

# 5. Verify space available
kubectl exec -it deployment/postgres -n default -- df -h
```

**Timeframe**: 10-30 minutes (online resize), 1-2 hours (migration)

### Scenario 4: Temporary Files Not Cleaned

**Symptoms**: /tmp directory using significant space, old temp files

```bash
# 1. Check /tmp size
kubectl exec -it deployment/neoland -n default -- du -sh /tmp

# 2. List old temp files (>7 days)
kubectl exec -it deployment/neoland -n default -- \
  find /tmp -type f -mtime +7 -ls

# 3. Delete old temp files
kubectl exec -it deployment/neoland -n default -- \
  find /tmp -type f -mtime +7 -delete

# 4. Add tmpfs mount (auto-cleanup on restart)
# Update deployment spec:
kubectl patch deployment neoland -n default --type='json' -p='[
  {
    "op": "add",
    "path": "/spec/template/spec/volumes/-",
    "value": {
      "name": "tmp",
      "emptyDir": {
        "medium": "Memory",
        "sizeLimit": "1Gi"
      }
    }
  },
  {
    "op": "add",
    "path": "/spec/template/spec/containers/0/volumeMounts/-",
    "value": {
      "name": "tmp",
      "mountPath": "/tmp"
    }
  }
]'

# This uses RAM for /tmp, auto-cleared on pod restart

# 5. Verify space freed
kubectl exec -it deployment/neoland -n default -- df -h
```

**Timeframe**: 5-10 minutes

### Scenario 5: Node Disk Full (Affects All Pods)

**Symptoms**: All pods on node affected, node disk >90%, image cache full

```bash
# 1. Identify affected node
kubectl get pods -o wide | grep neoland
NODE=<node-name>

# 2. SSH to node and check space
ssh $NODE df -h

# 3. Clean Docker/containerd image cache
ssh $NODE docker system prune -a -f

# Or for containerd:
ssh $NODE crictl rmi --prune

# 4. Clean old logs on node
ssh $NODE find /var/log -type f -name "*.log" -mtime +7 -delete

# 5. If still critical, evict pods from node
kubectl cordon $NODE
kubectl drain $NODE --ignore-daemonsets --delete-emptydir-data

# Pods will reschedule on other nodes

# 6. Expand node disk (cloud provider)
# AWS: Modify EBS volume size
# GCP: Resize persistent disk
# Azure: Resize managed disk

# 7. After expansion, resize filesystem
ssh $NODE sudo resize2fs /dev/sda1

# 8. Uncordon node
kubectl uncordon $NODE

# 9. Verify space available
ssh $NODE df -h
```

**Timeframe**: 15-45 minutes

### Scenario 6: Database Vacuum Needed (PostgreSQL Bloat)

**Symptoms**: Database size large but data small, dead tuples accumulating

```bash
# 1. Check database bloat
kubectl exec -it deployment/postgres -n default -- \
  psql -U postgres neoland -c "SELECT schemaname, tablename, pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size FROM pg_tables ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC LIMIT 10;"

# 2. Check dead tuples
kubectl exec -it deployment/postgres -n default -- \
  psql -U postgres neoland -c "SELECT schemaname, tablename, n_dead_tup FROM pg_stat_user_tables WHERE n_dead_tup > 1000 ORDER BY n_dead_tup DESC;"

# 3. Run VACUUM (reclaims space)
kubectl exec -it deployment/postgres -n default -- \
  psql -U postgres neoland -c "VACUUM VERBOSE;"

# 4. Run VACUUM FULL (more aggressive, requires locks)
# ⚠️ WARNING: Locks tables, causes downtime
kubectl exec -it deployment/postgres -n default -- \
  psql -U postgres neoland -c "VACUUM FULL VERBOSE;"

# 5. Enable autovacuum (automatic cleanup)
kubectl exec -it deployment/postgres -n default -- \
  psql -U postgres -c "ALTER SYSTEM SET autovacuum = on;"

# 6. Restart PostgreSQL
kubectl rollout restart deployment/postgres -n default

# 7. Verify space freed
kubectl exec -it deployment/postgres -n default -- df -h
```

**Timeframe**: 10-30 minutes (depending on database size)

---

## Verification

### Verify Disk Space Recovered

```bash
# 1. Check disk usage dropped
kubectl exec -it deployment/neoland -n default -- df -h
kubectl exec -it deployment/postgres -n default -- df -h

# Expected: Use% <70%

# 2. Check PVC usage
kubectl get pvc -n default

# 3. Verify alert cleared
# http://alertmanager:9093/#/alerts
# Expected: NeolandDiskSpaceLow resolved

# 4. Monitor disk usage over 24 hours
watch 'kubectl exec -it deployment/neoland -n default -- df -h'

# Ensure disk usage stable or decreasing
```

---

## Escalation

| Disk Usage | Severity | Response | Escalation |
|------------|----------|----------|------------|
| 70-85% | Warning | 4 hours | #neoland-warnings |
| 85-95% | High | 1 hour | #neoland-warnings + @oncall-voidnx |
| >95% | Critical | Immediate | PagerDuty + #neoland-critical |
| 100% (full) | EMERGENCY | Immediate | PagerDuty + Manager |

**Contacts**:
- Primary: `@oncall-voidnx`
- Infrastructure: `@oncall-infra` (for node/PVC expansion)
- Database: `@oncall-database` (for PostgreSQL issues)

---

## Post-Incident

### Immediate Actions (within 1 hour)

1. **Identify Root Cause**:
   - Log rotation not working? → Fix logrotate
   - Database growing? → Implement archival strategy
   - WAL files accumulating? → Fix archiving
   - PVC too small? → Plan capacity expansion

2. **Document in incident report**: `docs/incidents/YYYY-MM-DD-disk-full.md`

### Follow-Up Actions (within 24 hours)

1. **Implement Log Management**:
   - Configure log rotation (daily, 7 days retention)
   - Ship logs to centralized logging (Loki, Elasticsearch)
   - Set log retention policies
   - Monitor log growth rate

2. **Improve Database Management**:
   - Enable autovacuum
   - Configure WAL archival to S3
   - Set up automatic backups with cleanup
   - Monitor database growth trends

3. **Add Capacity Planning**:
   - Set alerts at 60%, 70%, 85%, 95%
   - Monitor growth rate (GB per day)
   - Forecast when disk will fill (trend analysis)
   - Plan PVC expansions proactively

4. **Implement Cleanup Automation**:
   - Cron job to clean old logs
   - Automatic temp file cleanup
   - Docker image pruning
   - Database vacuum scheduling

---

## Prevention

### Short-term (Immediate)

- ✅ Disk usage monitoring with alerts (70%, 85%, 95%)
- ⚠️ **TODO**: Implement log rotation
- ⚠️ **TODO**: Ship logs to external storage
- ⚠️ **TODO**: Enable PostgreSQL autovacuum

### Medium-term (1-2 weeks)

- [ ] Centralized logging (Loki/Elasticsearch)
- [ ] Log retention policies (7 days local, 30 days remote)
- [ ] Automatic cleanup cron jobs
- [ ] Database archival strategy (old data to S3)
- [ ] Capacity planning with growth forecasts

### Long-term (1-3 months)

- [ ] Auto-expanding PVCs (CSI driver support)
- [ ] Automated capacity management
- [ ] Predictive alerting (ML-based)
- [ ] Multi-tier storage (hot/warm/cold)
- [ ] Database partitioning for large tables

---

## Related Alerts

- `NeolandDown`: Disk full causes service crash
- `NeolandVectorStoreUnavailable`: PostgreSQL fails when disk full
- `NodeDiskPressure`: Kubernetes evicts pods on disk-full nodes

---

## Additional Resources

- **PostgreSQL VACUUM**: https://www.postgresql.org/docs/current/sql-vacuum.html
- **Kubernetes PVC Expansion**: https://kubernetes.io/docs/concepts/storage/persistent-volumes/#expanding-persistent-volumes-claims
- **Log Rotation**: `man logrotate`
- **Capacity Planning**: `docs/CAPACITY_PLANNING.md` (TODO: Phase 5)

---

**Last Updated**: 2026-01-31
**Maintainer**: voidnx team + Infrastructure team
**Severity**: WARNING → CRITICAL (usage-based escalation)
