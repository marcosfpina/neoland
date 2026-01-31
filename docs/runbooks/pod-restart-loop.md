# Runbook: NeolandPodRestartLoop

**Alert**: `NeolandPodRestartLoop`
**Severity**: CRITICAL 🔴
**Response Time**: 0-5 minutes

---

## Symptoms

- **Alert**: NEOLAND pod restarting repeatedly (>5 restarts in 10 minutes)
- **User Impact**: Service intermittent or unavailable, degraded performance
- **Metrics**: `rate(kube_pod_container_status_restarts_total{pod=~"neoland-.*"}[10m]) > 0.5`
- **Common Causes**:
  - Application crash (panic, unhandled error)
  - OOMKilled (out of memory)
  - Liveness probe failure
  - Dependency unavailable (database, ml-offload)
  - Configuration error
  - Resource limits too low

---

## Investigation

### Step 1: Check Pod Status and Restart Count

```bash
# Check pod status
kubectl get pods -l app=neoland -n default

# Expected (unhealthy):
# NAME                       READY   STATUS             RESTARTS
# neoland-7b9c8d5f6-abc12    0/1     CrashLoopBackOff   15

# If STATUS = CrashLoopBackOff → Application crashing
# If STATUS = OOMKilled → Memory exhaustion
# If STATUS = Error → Failed to start

# Check restart count and timing
kubectl describe pod -l app=neoland -n default | grep -A 10 "State:\|Last State:"

# Shows:
# State: Waiting (CrashLoopBackOff)
# Last State: Terminated (Exit Code: 1/137)
# Restart Count: 15
```

### Step 2: Check Pod Logs for Crash Reason

```bash
# Check current pod logs
kubectl logs -l app=neoland --tail=100 -n default

# Check previous container logs (from crash)
kubectl logs -l app=neoland --previous --tail=200 -n default

# Look for:
# - "panic:" → Rust panic
# - "FATAL" → Fatal error
# - "Error" → Unhandled error
# - "signal: killed" → OOMKilled
# - Connection errors (database, ml-offload)
```

### Step 3: Check Exit Code

```bash
# Get exit code from last termination
kubectl get pod -l app=neoland -n default -o jsonpath='{.items[0].status.containerStatuses[0].lastState.terminated.exitCode}'

# Exit codes:
# 0 = Success (shouldn't restart)
# 1 = General error
# 2 = Misuse of shell command
# 137 = OOMKilled (128 + 9 SIGKILL)
# 139 = Segmentation fault
# 143 = Terminated (128 + 15 SIGTERM)

# If 137 → OOMKilled scenario
# If 1 → Application error scenario
```

### Step 4: Check Resource Usage

```bash
# Check memory usage before crash
kubectl top pods -l app=neoland -n default

# Check resource limits
kubectl describe pod -l app=neoland -n default | grep -A 5 "Limits:\|Requests:"

# Expected:
# Limits:
#   cpu:     2000m
#   memory:  2Gi
# Requests:
#   cpu:     1000m
#   memory:  1Gi

# If memory usage near limit → OOMKill likely
```

---

## Resolution

### Scenario 1: Application Panic/Crash (Exit Code 1)

**Symptoms**: Logs show "panic:", "FATAL", or error before crash

```bash
# 1. Get full panic/error details
kubectl logs -l app=neoland --previous --tail=500 -n default | grep -A 20 "panic:\|FATAL"

# Example panic:
# thread 'main' panicked at 'Failed to connect to database', src/main.rs:42

# 2. Identify root cause from panic message
# Common causes:
# - Database connection failed
# - Required environment variable missing
# - File not found
# - Network timeout

# 3. If database connection issue, check database
kubectl get pods -l app=postgres -n default
curl http://postgres:5432

# 4. If environment variable missing, check secrets/configmap
kubectl get secret neoland-secrets -n default
kubectl get configmap neoland-config -n default

# 5. Rollback to previous working version
kubectl rollout undo deployment/neoland -n default

# 6. Monitor rollback
kubectl rollout status deployment/neoland -n default

# 7. Verify pod stable
kubectl get pods -l app=neoland -n default --watch

# Expected: READY 1/1, RESTARTS stable

# 8. Create GitHub issue for bug fix
```

**Timeframe**: 3-5 minutes

### Scenario 2: OOMKilled (Exit Code 137)

**Symptoms**: Exit code 137, logs show "signal: killed", memory at limit

```bash
# 1. Confirm OOMKill
kubectl describe pod -l app=neoland -n default | grep -i "oom"

# Expected: "OOMKilled" in Last State

# 2. Check current memory limits
kubectl get deployment neoland -n default -o jsonpath='{.spec.template.spec.containers[0].resources.limits.memory}'

# 3. Increase memory limits immediately
kubectl set resources deployment/neoland \
  --limits=memory=4Gi \
  --requests=memory=2Gi \
  -n default

# Current: 2Gi
# New: 4Gi (doubled)

# 4. Monitor rollout
kubectl rollout status deployment/neoland -n default

# 5. Verify pod stable
kubectl top pods -l app=neoland -n default

# Expected: Memory usage <80% of limit

# 6. Monitor for 10 minutes
watch 'kubectl get pods -l app=neoland -n default'

# Expected: No restarts

# 7. Investigate memory leak (if repeated OOMKills)
# - Check memory usage trends
# - Enable memory profiling
# - Review recent code changes
```

**Timeframe**: 3-5 minutes

### Scenario 3: Liveness Probe Failure

**Symptoms**: Logs healthy, but pod restarted by liveness probe

```bash
# 1. Check liveness probe configuration
kubectl get deployment neoland -n default -o jsonpath='{.spec.template.spec.containers[0].livenessProbe}'

# Expected:
# {
#   "httpGet": {"path": "/health", "port": 3001},
#   "initialDelaySeconds": 30,
#   "periodSeconds": 10,
#   "timeoutSeconds": 5,
#   "failureThreshold": 3
# }

# 2. Test liveness endpoint manually
kubectl exec -it deployment/neoland -n default -- \
  curl -v http://localhost:3001/health

# If fails → Liveness probe legitimately failing
# If succeeds → Probe too aggressive (timeout/threshold too low)

# 3. If probe too aggressive, increase thresholds
kubectl patch deployment neoland -n default --type='json' -p='[
  {
    "op": "replace",
    "path": "/spec/template/spec/containers/0/livenessProbe/timeoutSeconds",
    "value": 10
  },
  {
    "op": "replace",
    "path": "/spec/template/spec/containers/0/livenessProbe/failureThreshold",
    "value": 5
  }
]'

# Timeout: 5s → 10s
# Threshold: 3 fails → 5 fails

# 4. Monitor rollout
kubectl rollout status deployment/neoland -n default

# 5. Verify pod stable
kubectl get pods -l app=neoland -n default --watch
```

**Timeframe**: 5-10 minutes

### Scenario 4: Dependency Unavailable (Database, ml-offload)

**Symptoms**: Logs show connection errors to database or ml-offload

```bash
# 1. Check database connectivity
kubectl get pods -l app=postgres -n default
curl http://postgres:5432

# 2. Check ml-offload connectivity
kubectl get pods -l app=ml-offload -n default
curl http://ml-offload:8000/health

# 3. If database down, restart
kubectl rollout restart deployment/postgres -n default

# 4. If ml-offload down, restart
kubectl rollout restart deployment/ml-offload -n default

# 5. Wait for dependencies to be healthy
kubectl wait --for=condition=ready pod -l app=postgres -n default --timeout=60s
kubectl wait --for=condition=ready pod -l app=ml-offload -n default --timeout=60s

# 6. Restart NEOLAND to reconnect
kubectl rollout restart deployment/neoland -n default

# 7. Monitor recovery
kubectl get pods -l app=neoland -n default --watch

# Expected: READY 1/1, no restarts
```

**Timeframe**: 5-10 minutes

### Scenario 5: Configuration Error

**Symptoms**: Pod starts but immediately crashes, logs show config errors

```bash
# 1. Check recent ConfigMap/Secret changes
kubectl get configmap neoland-config -n default -o yaml
kubectl get secret neoland-secrets -n default -o yaml

# 2. Check pod environment variables
kubectl exec -it deployment/neoland -n default -- env | sort

# 3. If recent config change caused issue, rollback
kubectl rollout undo deployment/neoland -n default

# 4. Or fix config directly
kubectl edit configmap neoland-config -n default

# 5. Trigger rollout to apply fix
kubectl rollout restart deployment/neoland -n default

# 6. Verify pod stable
kubectl get pods -l app=neoland -n default --watch
```

**Timeframe**: 5-10 minutes

### Scenario 6: Resource Limits Too Low (CPU Throttling)

**Symptoms**: Pod slow to start, liveness probe times out due to CPU throttling

```bash
# 1. Check CPU throttling
kubectl describe pod -l app=neoland -n default | grep -i "throttl"

# 2. Check CPU limits
kubectl get deployment neoland -n default -o jsonpath='{.spec.template.spec.containers[0].resources.limits.cpu}'

# 3. Increase CPU limits
kubectl set resources deployment/neoland \
  --limits=cpu=4000m \
  --requests=cpu=2000m \
  -n default

# Current: 2000m
# New: 4000m (doubled)

# 4. Increase liveness probe timeout (temporarily)
kubectl patch deployment neoland -n default --type='json' -p='[
  {
    "op": "replace",
    "path": "/spec/template/spec/containers/0/livenessProbe/initialDelaySeconds",
    "value": 60
  }
]'

# Give more time for slow startup

# 5. Monitor rollout
kubectl rollout status deployment/neoland -n default

# 6. Verify pod stable
kubectl get pods -l app=neoland -n default --watch
```

**Timeframe**: 5-10 minutes

---

## Verification

### Verify Pod Stable

```bash
# 1. Check pod not restarting
kubectl get pods -l app=neoland -n default

# Expected: READY 1/1, RESTARTS count not increasing

# 2. Monitor for 10 minutes
watch 'kubectl get pods -l app=neoland -n default'

# Expected: No new restarts

# 3. Check restart rate metric
curl -s 'http://prometheus:9090/api/v1/query?query=rate(kube_pod_container_status_restarts_total{pod=~"neoland-.*"}[10m])' | jq

# Expected: 0 (no restarts in last 10 minutes)

# 4. Verify service healthy
curl http://neoland:3001/health | jq '.status'

# Expected: "healthy"

# 5. Check alert cleared
# http://alertmanager:9093/#/alerts
# Expected: NeolandPodRestartLoop resolved
```

---

## Escalation

⚠️ **CRITICAL ALERT** - Service unstable

- **Slack**: #neoland-critical (IMMEDIATE)
- **PagerDuty**: Automatic critical page
- **Contacts**:
  - Primary: `@oncall-voidnx`
  - Infrastructure: `@oncall-infra` (for resource issues)
  - Database: (if database connection errors)

**Escalation Timeline**:
- 0-5 min: On-call engineer
- 5-15 min: Team Lead (if not resolved)
- 15-30 min: Manager + consider rollback/hotfix

---

## Post-Incident

### Immediate Actions (within 1 hour)

1. **Root Cause Analysis**:
   - Application bug? → Fix and deploy
   - OOMKill? → Increase memory, investigate leak
   - Dependency down? → Improve health checks
   - Config error? → Validate config in CI/CD

2. **Document in incident report**: `docs/incidents/YYYY-MM-DD-pod-restart-loop.md`
   - Timeline
   - Root cause
   - Number of restarts
   - User impact
   - Resolution

### Follow-Up Actions (within 24 hours)

1. **Fix Application Bug** (if crash):
   - Debug panic/error
   - Add error handling
   - Add unit tests
   - Deploy fix

2. **Improve Resource Limits**:
   - Review memory usage trends
   - Set appropriate limits (usage + 30% buffer)
   - Add resource requests for guaranteed allocation

3. **Improve Health Checks**:
   - Tune liveness probe timeouts
   - Add dependency checks in /health endpoint
   - Implement graceful degradation

4. **Add Monitoring**:
   - Alert on repeated restarts (>3 in 5 min)
   - Track restart reasons (OOMKill vs crash)
   - Monitor resource usage trends

---

## Prevention

### Short-term (Immediate)

- ✅ Pod restart monitoring with alerts
- ✅ Resource limits configured
- ⚠️ **TODO**: Improve liveness probe tuning
- ⚠️ **TODO**: Add dependency health checks

### Medium-term (1-2 weeks)

- [ ] Implement circuit breakers for dependencies
- [ ] Add graceful degradation (run without ml-offload if needed)
- [ ] Improve error handling (no panics)
- [ ] Add memory leak detection
- [ ] Validate config in CI/CD (schema validation)

### Long-term (1-3 months)

- [ ] Chaos engineering (kill pods randomly, test resilience)
- [ ] Automated resource sizing (VPA - Vertical Pod Autoscaler)
- [ ] Application performance monitoring (APM)
- [ ] Predictive alerting (ML-based)

---

## Related Alerts

- `NeolandDown`: Restart loop leads to service down
- `NeolandCriticalMemoryUsage`: Often precedes OOMKill
- `NeolandHighCPUUsage`: CPU exhaustion can cause crashes
- `NeolandVectorStoreUnavailable`: Database down causes restarts

---

## Additional Resources

- **Kubernetes Pod Lifecycle**: https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/
- **OOMKilled Debugging**: `docs/DEBUGGING.md` (TODO)
- **Liveness Probe Best Practices**: `docs/HEALTH_CHECKS.md` (TODO)
- **Resource Management**: `docs/RESOURCE_SIZING.md` (TODO: Phase 5)

---

**Last Updated**: 2026-01-31
**Maintainer**: voidnx team
**Severity**: CRITICAL - Service instability
