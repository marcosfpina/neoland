# Runbook: NeolandUnhealthy / NeolandNotReady

**Alert**: `NeolandUnhealthy` (critical) / `NeolandNotReady` (warning)
**Severity**: CRITICAL 🔴 / WARNING ⚠️
**Response Time**: 0-5 min (critical) / 15-30 min (warning)

---

## Symptoms

- **NeolandUnhealthy**: Health check `/health` returning unhealthy, pod may be removed from service
- **NeolandNotReady**: Readiness probe `/ready` failing, pod removed from load balancer
- **User Impact**: Service degraded or unavailable

---

## Investigation

```bash
# Check health endpoint
curl http://neoland:3001/health | jq

# Expected healthy response:
# {
#   "status": "healthy",
#   "version": "...",
#   "uptime_seconds": 123,
#   "components": [
#     {"name": "vector_store", "status": "healthy"},
#     {"name": "llm", "status": "healthy"},
#     ...
#   ]
# }

# Check which component is unhealthy
curl http://neoland:3001/health | jq '.components[] | select(.status != "healthy")'

# Check pod status
kubectl get pods -l app=neoland -n default
kubectl describe pod -l app=neoland -n default | grep -A 10 "Liveness:\|Readiness:"
```

---

## Resolution

### Scenario 1: Vector Store Unhealthy

```bash
# Check PostgreSQL status
kubectl get pods -l app=postgres -n default

# If PostgreSQL down, restart
kubectl rollout restart deployment/postgres -n default

# Verify health recovers
watch 'curl -s http://neoland:3001/health | jq .status'
```

**Timeframe**: 2-5 minutes

### Scenario 2: LLM Provider Unhealthy

```bash
# Check ml-offload health
curl http://ml-offload:8000/health

# If down, restart
kubectl rollout restart deployment/ml-offload -n default

# Check SecureLLM API status
curl -H "X-API-Key: $SECURELLM_KEY" https://api.securellm.com/health
```

**Timeframe**: 2-5 minutes

### Scenario 3: Pod Not Ready (Startup Issue)

```bash
# Check pod logs for startup errors
kubectl logs -l app=neoland --tail=200 -n default | grep -i "error\|fatal"

# Common issues:
# - Database connection failed: Check DB credentials
# - Port already in use: Check if old pod still running
# - Missing config: Check ConfigMap

# Restart pod
kubectl delete pod -l app=neoland -n default
```

**Timeframe**: 3-5 minutes

### Scenario 4: Resource Constraints

```bash
# Check if pod is OOMKilled or CPU throttled
kubectl describe pod -l app=neoland -n default | grep -A 5 "State:\|Last State:"

# Increase limits if needed
kubectl set resources deployment/neoland \
  --limits=memory=4Gi,cpu=2000m \
  -n default
```

**Timeframe**: 5-10 minutes

---

## Verification

```bash
# Check health endpoint returns healthy
curl http://neoland:3001/health | jq '.status'
# Expected: "healthy"

# Check readiness
curl http://neoland:3001/ready
# Expected: HTTP 200

# Verify pod is Ready
kubectl get pods -l app=neoland -n default
# Expected: READY 1/1

# Check alert cleared
```

---

## Escalation

- **NeolandUnhealthy**: PagerDuty + #neoland-critical (immediate)
- **NeolandNotReady**: #neoland-warnings (15-30 min)
- **Contact**: `@oncall-voidnx`

---

## Related Alerts

- `NeolandDown`: Complete service outage (worse than unhealthy)
- `NeolandComponentDegraded`: Individual component degraded
- `NeolandVectorStoreUnavailable`: Database-specific health failure

---

**Last Updated**: 2026-01-31
**Maintainer**: voidnx team
