# Runbook: NeolandDown

**Alert**: `NeolandDown`
**Severity**: CRITICAL 🔴
**Response Time**: 0-5 minutes

---

## Symptoms

- **Alert**: NeolandDown firing in AlertManager
- **User Impact**: **COMPLETE SERVICE OUTAGE** - Users cannot access NEOLAND at all
- **Metrics**: `up{job="neoland"} == 0` for >2 minutes
- **Dashboards**: Prometheus target down, health check failing

---

## Investigation

### Step 1: Check Kubernetes Pod Status

```bash
# Check if pods are running
kubectl get pods -l app=neoland -n default

# Expected output:
# NAME                       READY   STATUS    RESTARTS   AGE
# neoland-7b9c8d5f6-abc12    1/1     Running   0          5m

# If STATUS != Running, check pod events
kubectl describe pod -l app=neoland -n default
```

**Common issues**:
- `CrashLoopBackOff`: Application crashes on startup
- `ImagePullBackOff`: Cannot pull container image
- `Pending`: No resources available for scheduling
- `OOMKilled`: Out of memory

### Step 2: Check Pod Logs

```bash
# Recent logs (last 100 lines)
kubectl logs -l app=neoland --tail=100 -n default

# Logs from previous crash (if pod restarted)
kubectl logs -l app=neoland --previous -n default

# Follow logs in real-time
kubectl logs -l app=neoland -f -n default
```

**Look for**:
- Panic messages
- Fatal errors
- Port binding failures
- Database connection errors
- Out of memory errors

### Step 3: Check Service and Endpoints

```bash
# Check service exists
kubectl get svc neoland -n default

# Check endpoints are populated
kubectl get endpoints neoland -n default

# Expected: Should have pod IPs listed
```

### Step 4: Check Recent Deployments

```bash
# Check recent deployments
kubectl rollout history deployment/neoland -n default

# Check current deployment status
kubectl rollout status deployment/neoland -n default

# View recent events
kubectl get events -n default --sort-by='.lastTimestamp' | head -20
```

### Step 5: Check Infrastructure

```bash
# Check node health
kubectl get nodes

# Check node resources
kubectl top nodes

# Check cluster events
kubectl get events --all-namespaces --sort-by='.lastTimestamp' | head -30
```

---

## Resolution

### Scenario 1: Pod CrashLoopBackOff (Application Crash)

```bash
# 1. Check logs for panic/error
kubectl logs -l app=neoland --tail=200 -n default | grep -i "panic\|fatal\|error"

# 2. If recent deployment broke it, rollback
kubectl rollout undo deployment/neoland -n default

# 3. Verify rollback
kubectl rollout status deployment/neoland -n default

# 4. Monitor recovery
watch kubectl get pods -l app=neoland -n default
```

**Timeframe**: 2-3 minutes

### Scenario 2: OOMKilled (Out of Memory)

```bash
# 1. Check memory limits
kubectl describe pod -l app=neoland -n default | grep -A 3 Limits

# 2. Increase memory limits temporarily (emergency)
kubectl set resources deployment/neoland \
  --limits=memory=4Gi \
  --requests=memory=2Gi \
  -n default

# 3. Monitor restart
kubectl rollout status deployment/neoland -n default
```

**Note**: File issue to investigate memory leak, this is temporary fix

**Timeframe**: 3-5 minutes

### Scenario 3: ImagePullBackOff (Cannot Pull Image)

```bash
# 1. Check image name and tag
kubectl describe pod -l app=neoland -n default | grep Image:

# 2. Verify image exists in registry
# docker pull <registry>/<image>:<tag>

# 3. If wrong tag, fix deployment
kubectl set image deployment/neoland \
  neoland=<registry>/neoland:<correct-tag> \
  -n default
```

**Timeframe**: 2-4 minutes

### Scenario 4: No Resources Available (Pending Pod)

```bash
# 1. Check why pod is pending
kubectl describe pod -l app=neoland -n default | grep -A 10 Events

# 2. If insufficient CPU/memory, scale down other workloads temporarily
kubectl scale deployment/<non-critical-service> --replicas=0 -n default

# 3. Or add node to cluster (takes longer)
# (Depends on cloud provider)
```

**Timeframe**: 5-10 minutes (or longer if adding nodes)

### Scenario 5: Service/Endpoints Issue

```bash
# 1. Recreate service
kubectl delete svc neoland -n default
kubectl apply -f k8s/service.yaml

# 2. Verify endpoints populated
kubectl get endpoints neoland -n default

# 3. Check label selectors match
kubectl get pods -l app=neoland --show-labels -n default
```

**Timeframe**: 1-2 minutes

### Scenario 6: Complete Cluster Failure

```bash
# 1. Check cluster API server
kubectl cluster-info

# 2. If cluster is down, contact infrastructure team immediately
# 3. Consider switching to backup cluster if available
```

**Escalate immediately** to infrastructure team

---

## Verification

After fix, verify service is healthy:

```bash
# 1. Check pods running
kubectl get pods -l app=neoland -n default

# 2. Check health endpoint
curl http://neoland.example.com/health
# Expected: {"status":"healthy","version":"..."}

# 3. Check metrics endpoint
curl http://neoland.example.com/metrics | grep neoland_health_status

# 4. Test REST API
curl -X POST http://neoland.example.com/v1/chat/completions \
  -H "X-API-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"messages":[{"role":"user","content":"test"}]}'

# 5. Verify alert clears in AlertManager
# http://alertmanager:9093/#/alerts
```

**Expected**: All checks pass within 2 minutes of fix

---

## Escalation

### Immediate Escalation (if not resolved in 5 minutes)

- **Slack**: Post in `#neoland-critical` with:
  - Current status
  - Steps already tried
  - Error messages/logs
  - Request help from team

- **PagerDuty**: Alert automatically sent to on-call engineer

- **Team Contacts**:
  - Primary: `@oncall-voidnx` (Slack)
  - Secondary: Team Lead (after 15 min)
  - Manager: (after 30 min)

### Escalation Path

| Time Elapsed | Action |
|--------------|--------|
| 0-5 min | On-call engineer investigates |
| 5-15 min | Notify team in #neoland-critical |
| 15-30 min | Escalate to Team Lead |
| 30+ min | Escalate to Manager, consider incident commander |

---

## Post-Incident

### Immediate Actions (within 1 hour)

1. **Document in incident log**:
   ```bash
   # Add to docs/incidents/YYYY-MM-DD-service-down.md
   - Timestamp: When alert fired
   - Duration: How long service was down
   - Root cause: What caused the outage
   - Resolution: How it was fixed
   - Impact: Number of affected users/requests
   ```

2. **Notify stakeholders**:
   - Post incident summary in #neoland-incidents
   - Update status page (if applicable)
   - Notify affected customers (if external)

### Follow-up Actions (within 24 hours)

1. **Create GitHub issue** for root cause:
   ```
   Title: [Incident] Service down on YYYY-MM-DD
   Labels: incident, severity-critical
   Assignee: On-call engineer
   ```

2. **Schedule post-mortem** (within 48 hours):
   - Review timeline
   - Identify root cause
   - Document contributing factors
   - Create action items to prevent recurrence

---

## Prevention

### Monitoring

- ✅ Alert fires after 2 minutes of downtime
- ✅ PagerDuty notification immediate
- ✅ Slack notification to #neoland-critical

### Best Practices

1. **Blue-Green Deployments**: Always test in staging first
2. **Resource Limits**: Set appropriate CPU/memory limits
3. **Health Checks**: Ensure health endpoint works correctly
4. **Graceful Shutdown**: Handle SIGTERM properly
5. **Dependency Checks**: Verify all dependencies in startup

### Related Alerts

- `NeolandHighRestartRate`: May fire before complete outage
- `NeolandUnhealthy`: Health check failures
- `NeolandNotReady`: Readiness probe failures

---

## Additional Resources

- **Grafana Dashboard**: [NEOLAND Service Health](http://grafana/d/neoland-health)
- **Logs**: `kubectl logs -l app=neoland -n default --tail=1000`
- **Metrics**: `http://prometheus:9090/graph?g0.expr=up{job="neoland"}`
- **Deployment Guide**: `docs/DEPLOYMENT.md`

---

**Last Updated**: 2026-01-31
**Maintainer**: voidnx team
**Tested**: ✅ Quarterly DR drill
