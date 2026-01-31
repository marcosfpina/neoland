# Runbook: NeolandCriticalMemoryUsage / NeolandHighMemoryUsage

**Alert**: `NeolandCriticalMemoryUsage` (critical) / `NeolandHighMemoryUsage` (warning)
**Severity**: CRITICAL 🔴 / WARNING ⚠️
**Response Time**: 0-5 min (critical) / 15-30 min (warning)

---

## Symptoms

- **Critical (>95% memory)**: OOMKill imminent, service may crash
- **Warning (>85% memory)**: Memory pressure, degraded performance
- **User Impact**: Slow responses, possible service crash

---

## Investigation

```bash
# Check memory usage
kubectl top pods -l app=neoland -n default

# Expected:
# NAME                       CPU   MEMORY
# neoland-7b9c8d5f6-abc12    500m  3500Mi/4096Mi  (85%)

# Check memory limits
kubectl describe pod -l app=neoland -n default | grep -A 3 "Limits:"

# Check for memory leaks in logs
kubectl logs -l app=neoland --tail=500 -n default | grep -i "memory\|oom\|allocation"

# Check heap usage (if available)
kubectl exec -it deployment/neoland -n default -- free -h
```

---

## Resolution

### IMMEDIATE ACTION (if >95%): Increase Memory Limits

```bash
# Increase memory limits immediately
kubectl set resources deployment/neoland \
  --limits=memory=8Gi \
  --requests=memory=4Gi \
  -n default

# Monitor rollout
kubectl rollout status deployment/neoland -n default

# Verify memory usage drops
watch 'kubectl top pods -l app=neoland -n default'
```

**Timeframe**: 2-3 minutes

### ROOT CAUSE INVESTIGATION (after immediate fix)

#### Scenario 1: Memory Leak

```bash
# Check if memory grows over time
# (Compare uptime vs memory usage)
kubectl logs -l app=neoland -n default | jq '{time: .timestamp, mem: .memory_mb, uptime: .uptime_seconds}'

# If memory grows linearly with uptime → memory leak
# Create issue for investigation
```

**Action**: Restart pod to free memory, investigate leak in code

#### Scenario 2: Large Dataset in Memory

```bash
# Check vector store size
kubectl logs -l app=neoland -n default | grep "vector_store" | grep "size"

# If vector store too large, migrate to PostgreSQL (Phase 4.7)
```

**Action**: Implement persistent vector store (ADR-019)

#### Scenario 3: Too Many Concurrent Requests

```bash
# Check active requests
curl http://neoland:3001/metrics | grep neoland_http_requests_active

# If very high, add more replicas to distribute load
kubectl scale deployment/neoland --replicas=5 -n default
```

**Action**: Scale horizontally

---

## Verification

```bash
# Check memory usage dropped
kubectl top pods -l app=neoland -n default
# Expected: <80% of limit

# Verify service still healthy
curl http://neoland:3001/health | jq '.status'

# Check alert cleared
```

---

## Escalation

- **Critical (>95%)**: IMMEDIATE PagerDuty + #neoland-critical
- **Warning (>85%)**: #neoland-warnings, investigate within 30 min
- **Contact**: `@oncall-voidnx`

---

## Post-Incident

1. **If memory leak suspected**:
   - Create GitHub issue for investigation
   - Enable memory profiling
   - Review recent code changes

2. **If legitimate memory usage**:
   - Permanently increase memory limits
   - Add horizontal scaling (HPA)
   - Migrate to persistent vector store

3. **Prevention**:
   - Add memory leak detection
   - Implement memory usage tracking
   - Set up memory profiling

---

## Related Alerts

- `NeolandHighCPUUsage`: Often correlates with memory pressure
- `NeolandHighLatency`: GC pauses due to memory pressure
- `NeolandDown`: OOMKill leads to service down

---

**Last Updated**: 2026-01-31
**Maintainer**: voidnx team
