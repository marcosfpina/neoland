# Runbook: NeolandVectorStoreUnavailable

**Alert**: `NeolandVectorStoreUnavailable`
**Severity**: CRITICAL 🔴
**Response Time**: 0-5 minutes

---

## Symptoms

- **Alert**: Vector store (PostgreSQL) health check failing
- **User Impact**: SEVERE - RAG queries fail, document search unavailable
- **Metrics**: `neoland_component_health{component="vector_store", status="healthy"} == 0` for >1 minute
- **Common Causes**:
  - PostgreSQL pod down/restarting
  - Database connection pool exhausted
  - Network connectivity issues
  - Database disk full
  - Long-running queries blocking connections

---

## Investigation

### Step 1: Check PostgreSQL Pod Status

```bash
# Check if PostgreSQL pod is running
kubectl get pods -l app=postgres -n default

# Expected (healthy):
# NAME                        READY   STATUS    RESTARTS
# postgres-7b9c8d5f6-abc12    1/1     Running   0

# If STATUS not "Running" → Pod failure scenario
# If RESTARTS >0 recent → Pod crash scenario

# Check pod events
kubectl describe pod -l app=postgres -n default | tail -20
```

### Step 2: Test Database Connectivity

```bash
# Test connection from NEOLAND pod
kubectl exec -it deployment/neoland -n default -- \
  psql -h postgres -U neoland_user -d neoland -c "SELECT 1;"

# Expected output:
#  ?column?
# ----------
#         1
# (1 row)

# If connection refused → Network/service issue
# If authentication failed → Credentials issue
# If timeout → Database overloaded

# Check database service
kubectl get svc postgres -n default
```

### Step 3: Check Database Health

```bash
# Check active connections
kubectl exec -it deployment/postgres -n default -- \
  psql -U postgres -c "SELECT count(*) FROM pg_stat_activity WHERE state = 'active';"

# If count >95% of max_connections → Connection pool exhausted

# Check for blocking queries
kubectl exec -it deployment/postgres -n default -- \
  psql -U postgres -c "SELECT pid, usename, state, wait_event_type, query FROM pg_stat_activity WHERE state != 'idle' ORDER BY query_start;"

# Check for long-running queries (>30s)
kubectl exec -it deployment/postgres -n default -- \
  psql -U postgres -c "SELECT pid, now() - query_start AS duration, query FROM pg_stat_activity WHERE state = 'active' AND now() - query_start > interval '30 seconds';"
```

### Step 4: Check Disk Space

```bash
# Check database disk usage
kubectl exec -it deployment/postgres -n default -- df -h /var/lib/postgresql/data

# Expected: <80% usage
# If >90% → Disk full scenario
```

---

## Resolution

### Scenario 1: PostgreSQL Pod Down/CrashLooping

**Symptoms**: Pod STATUS not "Running", recent restart, or CrashLoopBackOff

```bash
# 1. Check pod logs for crash reason
kubectl logs -l app=postgres --tail=200 -n default

# Common errors:
# - "FATAL: database files are incompatible" → Version mismatch
# - "PANIC: could not write lock file" → Permission issue
# - "FATAL: could not create shared memory segment" → Resource limits

# 2. If OOMKilled, increase memory limits
kubectl set resources deployment/postgres \
  --limits=memory=4Gi \
  --requests=memory=2Gi \
  -n default

# 3. If crash due to config, rollback
kubectl rollout undo deployment/postgres -n default

# 4. If persistent failure, restore from backup (see Scenario 6)

# 5. Monitor pod startup
kubectl rollout status deployment/postgres -n default

# 6. Verify NEOLAND reconnects
kubectl logs -l app=neoland --tail=50 | grep "vector_store"
```

**Timeframe**: 2-5 minutes

### Scenario 2: Connection Pool Exhausted

**Symptoms**: Active connections at max_connections limit, connection refused errors

```bash
# 1. Identify and kill idle/stuck connections
kubectl exec -it deployment/postgres -n default -- \
  psql -U postgres -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE state = 'idle' AND state_change < now() - interval '5 minutes';"

# 2. Kill long-running queries (>5 min, if safe)
kubectl exec -it deployment/postgres -n default -- \
  psql -U postgres -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE state = 'active' AND now() - query_start > interval '5 minutes';"

# ⚠️ WARNING: Only kill queries if you're sure they're stuck

# 3. Increase max_connections (if needed)
kubectl exec -it deployment/postgres -n default -- \
  psql -U postgres -c "ALTER SYSTEM SET max_connections = 200;"

# Current: 100, New: 200

# 4. Restart PostgreSQL to apply
kubectl rollout restart deployment/postgres -n default

# 5. Verify connections recovered
kubectl exec -it deployment/postgres -n default -- \
  psql -U postgres -c "SELECT count(*) FROM pg_stat_activity;"
```

**Timeframe**: 3-5 minutes

### Scenario 3: Network/Service Issues

**Symptoms**: Connection refused, DNS resolution failures, timeout

```bash
# 1. Check PostgreSQL service exists
kubectl get svc postgres -n default

# Expected: ClusterIP with port 5432

# 2. If service missing, recreate
kubectl apply -f k8s/postgres-service.yaml

# 3. Check service endpoints
kubectl get endpoints postgres -n default

# Expected: IP address of PostgreSQL pod
# If no endpoints → Pod selector mismatch

# 4. Test connectivity from NEOLAND pod
kubectl exec -it deployment/neoland -n default -- \
  nc -zv postgres 5432

# Expected: "Connection to postgres 5432 port [tcp/postgresql] succeeded!"

# 5. Check network policies (if any)
kubectl get networkpolicies -n default

# 6. Verify DNS resolution
kubectl exec -it deployment/neoland -n default -- \
  nslookup postgres.default.svc.cluster.local
```

**Timeframe**: 5-10 minutes

### Scenario 4: Disk Full

**Symptoms**: Disk usage >90%, database write failures, "No space left on device" errors

```bash
# 1. Check disk usage
kubectl exec -it deployment/postgres -n default -- df -h

# 2. Identify large files/tables
kubectl exec -it deployment/postgres -n default -- \
  psql -U postgres -c "SELECT schemaname, tablename, pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size FROM pg_tables ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC LIMIT 10;"

# 3. EMERGENCY: Clear old WAL files (if safe)
kubectl exec -it deployment/postgres -n default -- \
  find /var/lib/postgresql/data/pg_wal -type f -mtime +7 -delete

# ⚠️ DANGER: Only do this if disk is critically full

# 4. Increase PVC size (requires downtime)
kubectl patch pvc postgres-data -n default -p '{"spec":{"resources":{"requests":{"storage":"100Gi"}}}}'

# Current: 50Gi, New: 100Gi

# 5. Restart pod to use new space
kubectl delete pod -l app=postgres -n default

# 6. Verify space available
kubectl exec -it deployment/postgres -n default -- df -h
```

**Timeframe**: 10-15 minutes (including downtime)

### Scenario 5: Database Credentials Invalid

**Symptoms**: Authentication failed, "FATAL: password authentication failed" errors

```bash
# 1. Check if secret exists
kubectl get secret neoland-secrets -n default

# 2. Verify DATABASE_URL is correct
kubectl get secret neoland-secrets -n default -o json \
  | jq -r '.data.DATABASE_URL' | base64 -d

# Expected format: postgresql://user:password@postgres:5432/neoland

# 3. If credentials changed, restart NEOLAND pods
kubectl rollout restart deployment/neoland -n default

# 4. Test connection with correct credentials
kubectl exec -it deployment/neoland -n default -- \
  env | grep DATABASE_URL
```

**Timeframe**: 2-3 minutes

### Scenario 6: Database Corruption / Restore from Backup

**Symptoms**: Persistent crashes, data corruption errors, "PANIC" in logs

```bash
# ⚠️ CRITICAL: This is a disaster recovery scenario

# 1. Stop NEOLAND to prevent further writes
kubectl scale deployment/neoland --replicas=0 -n default

# 2. Backup current (corrupted) database
kubectl exec -it deployment/postgres -n default -- \
  pg_dumpall -U postgres > /tmp/corrupted-backup-$(date +%Y%m%d).sql

# 3. Restore from latest backup
# (Assumes backups in S3 bucket: s3://neoland-backups/)

# Download latest backup
aws s3 cp s3://neoland-backups/latest.sql /tmp/latest-backup.sql

# 4. Drop and recreate database
kubectl exec -it deployment/postgres -n default -- \
  psql -U postgres -c "DROP DATABASE IF EXISTS neoland; CREATE DATABASE neoland;"

# 5. Restore backup
kubectl exec -i deployment/postgres -n default -- \
  psql -U postgres neoland < /tmp/latest-backup.sql

# 6. Verify data integrity
kubectl exec -it deployment/postgres -n default -- \
  psql -U postgres -d neoland -c "SELECT COUNT(*) FROM embeddings;"

# 7. Restart NEOLAND
kubectl scale deployment/neoland --replicas=3 -n default

# 8. Monitor health recovery
watch 'curl -s http://neoland:3001/health | jq .components'
```

**Timeframe**: 20-60 minutes (depending on backup size)
**RPO**: Last backup timestamp (should be <24 hours)
**RTO**: Target 1 hour

---

## Verification

### Verify Database Accessible

```bash
# 1. Check PostgreSQL pod healthy
kubectl get pods -l app=postgres -n default

# Expected: STATUS = Running, READY = 1/1

# 2. Test database connection
kubectl exec -it deployment/neoland -n default -- \
  psql -h postgres -U neoland_user -d neoland -c "SELECT 1;"

# Expected: 1 row returned

# 3. Check vector store health in NEOLAND
curl http://neoland:3001/health | jq '.components[] | select(.name == "vector_store")'

# Expected: {"name": "vector_store", "status": "healthy"}

# 4. Test vector search functionality
curl -X POST http://neoland:3001/v1/chat/completions \
  -H "X-API-Key: $API_KEY" \
  -d '{"messages":[{"role":"user","content":"search test"}]}'

# Expected: 200 OK with response

# 5. Check alert cleared
# http://alertmanager:9093/#/alerts
# Expected: NeolandVectorStoreUnavailable resolved
```

### Monitor for Stability

```bash
# Monitor database connections for 5 minutes
watch 'kubectl exec -it deployment/postgres -n default -- psql -U postgres -c "SELECT count(*) FROM pg_stat_activity;"'

# Expected: Stable connection count (<50)

# Monitor NEOLAND logs for errors
kubectl logs -l app=neoland --tail=100 -f | grep -i "database\|vector_store"

# Expected: No connection errors
```

---

## Escalation

⚠️ **CRITICAL ALERT** - Vector store is core functionality

- **Slack**: #neoland-critical (IMMEDIATE)
- **PagerDuty**: Automatic critical page
- **Contacts**:
  - Primary: `@oncall-voidnx`
  - Database Team: `@oncall-database` (if available)
  - Infrastructure: `@oncall-infra` (for disk/network issues)
  - Manager: (if not resolved in 15 min)

**Escalation Timeline**:
- 0-5 min: On-call engineer (voidnx)
- 5-15 min: Team Lead + Database specialist
- 15-30 min: Manager + consider incident commander
- 30+ min: Restore from backup + full incident response

---

## Post-Incident

### Immediate Actions (within 1 hour)

1. **Root Cause Analysis**: Why did database fail?
   - Pod OOMKilled? → Increase memory limits
   - Disk full? → Implement log rotation, increase PVC size
   - Connection exhaustion? → Optimize queries, add pooling
   - Corruption? → Investigate what caused it

2. **Document in incident report**: `docs/incidents/YYYY-MM-DD-database-unavailable.md`
   - Timeline of events
   - Root cause
   - Resolution steps
   - Impact (downtime duration, requests failed)

### Follow-Up Actions (within 24 hours)

1. **Improve Backup Strategy**:
   - Verify backups are working (test restore)
   - Increase backup frequency (currently daily → every 6 hours)
   - Implement point-in-time recovery (WAL archiving)

2. **Implement High Availability**:
   - Deploy PostgreSQL with replication (primary + replica)
   - Use pgpool or patroni for automatic failover
   - ADR-020: "PostgreSQL High Availability Strategy"

3. **Improve Monitoring**:
   - Add connection pool metrics
   - Alert on disk usage >70%
   - Monitor slow queries (log queries >1s)
   - Track database replication lag (if HA implemented)

4. **Optimize Database**:
   - Review and optimize slow queries
   - Add missing indexes
   - Implement connection pooling (PgBouncer)
   - Tune PostgreSQL parameters (shared_buffers, work_mem)

---

## Prevention

### Short-term (Immediate)

- ✅ Database health check monitoring
- ✅ Daily backups to S3
- ⚠️ **TODO**: Implement connection pooling (PgBouncer)
- ⚠️ **TODO**: Add disk usage alerts (<70%)

### Medium-term (1-2 weeks)

- [ ] Deploy PostgreSQL replica for HA
- [ ] Implement automatic failover
- [ ] Add slow query logging and monitoring
- [ ] Increase backup frequency (6 hours)
- [ ] Implement WAL archiving (PITR)

### Long-term (1-3 months)

- [ ] Migrate to managed PostgreSQL (AWS RDS, Google Cloud SQL)
- [ ] Implement automatic backups with retention policies
- [ ] Add read replicas for scaling
- [ ] Implement database connection pooling service
- [ ] Set up multi-region replication (DR)

---

## Related Alerts

- `NeolandUnhealthy`: Vector store down triggers general health failure
- `NeolandHighErrorRate`: Database unavailable causes 500 errors
- `NeolandSlowLLMInference`: RAG failures may slow down LLM responses
- `NeolandDiskSpaceLow`: May precede database failure

---

## Additional Resources

- **PostgreSQL Documentation**: https://www.postgresql.org/docs/
- **Backup/Restore Guide**: `docs/BACKUP_RESTORE.md` (TODO: Phase 4.6)
- **Database Tuning**: `docs/DATABASE_TUNING.md` (TODO: Phase 4.7)
- **Disaster Recovery Plan**: `docs/DR_PLAN.md` (TODO: Phase 4.6)

---

**Last Updated**: 2026-01-31
**Maintainer**: voidnx team
**Severity**: CRITICAL - Core data storage failure
