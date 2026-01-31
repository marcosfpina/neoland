# Runbook: NeolandHighLatency / NeolandVeryHighLatency

**Alert**: `NeolandHighLatency` (warning) / `NeolandVeryHighLatency` (critical)
**Severity**: WARNING ⚠️ / CRITICAL 🔴
**Response Time**: 15-30 min (warning) / 0-5 min (critical)

---

## Symptoms

- **Alert**: NeolandHighLatency or NeolandVeryHighLatency firing
- **User Impact**:
  - Warning: Slow responses (p99 >2s), users experiencing delays
  - Critical: Very slow responses (p99 >5s), major user impact
- **Metrics**:
  - Warning: `histogram_quantile(0.99, rate(neoland_http_request_duration_seconds_bucket[5m])) > 2.0`
  - Critical: `> 5.0`

---

## Investigation

### Step 1: Identify Slow Endpoints

```bash
# Check latency by endpoint in Prometheus
curl -s 'http://prometheus:9090/api/v1/query?query=histogram_quantile(0.99,rate(neoland_http_request_duration_seconds_bucket[5m]))' \
  | jq '.data.result[] | {endpoint: .metric.endpoint, latency: .value[1]}'

# Check recent slow requests in logs
kubectl logs -l app=neoland --tail=200 -n default \
  | jq 'select(.duration_ms > 2000)' \
  | jq -r '. | "\(.endpoint) \(.duration_ms)ms"'
```

### Step 2: Check LLM Provider Response Times

```bash
# Check LLM latency
kubectl logs -l app=neoland --tail=100 -n default \
  | jq 'select(.llm_provider)' \
  | jq -r '. | "\(.llm_provider) \(.llm_duration_ms)ms"'

# Check ml-offload health
curl http://ml-offload:8000/health | jq

# If ml-offload slow or down, check its logs
kubectl logs -l app=ml-offload --tail=200 -n default
```

### Step 3: Check Database Performance

```bash
# Check database slow queries
kubectl exec -it deployment/postgres -n default -- \
  psql -U postgres -c "SELECT query, mean_exec_time, calls FROM pg_stat_statements ORDER BY mean_exec_time DESC LIMIT 10;"

# Check active connections
kubectl exec -it deployment/postgres -n default -- \
  psql -U postgres -c "SELECT count(*) FROM pg_stat_activity WHERE state = 'active';"
```

### Step 4: Check Resource Constraints

```bash
# Check CPU/memory usage
kubectl top pods -l app=neoland -n default

# Check for CPU throttling
kubectl describe pod -l app=neoland -n default | grep -A 5 "Limits:"

# Check network latency
kubectl exec -it deployment/neoland -n default -- ping -c 5 postgres
```

---

## Resolution

### Scenario 1: LLM Provider Slow/Down

```bash
# 1. Check which provider is slow
kubectl logs -l app=neoland --tail=100 | jq 'select(.llm_duration_ms > 5000)'

# 2. If ml-offload slow, restart it
kubectl rollout restart deployment/ml-offload -n default

# 3. If SecureLLM API slow, switch to local engine
# (Update config to prioritize local engine)

# 4. Monitor latency improvement
watch 'curl -s http://prometheus:9090/api/v1/query?query=histogram_quantile(0.99,rate(neoland_http_request_duration_seconds_bucket[1m])) | jq'
```

**Timeframe**: 2-5 minutes

### Scenario 2: Database Slow Queries

```bash
# 1. Identify slow query
kubectl exec -it deployment/postgres -- \
  psql -U postgres -c "SELECT query FROM pg_stat_statements ORDER BY mean_exec_time DESC LIMIT 1;"

# 2. Check if missing index
# (Analyze query plan, add index if needed)

# 3. Temporarily increase connection pool
kubectl set env deployment/neoland DB_POOL_SIZE=50 -n default

# 4. Restart to apply
kubectl rollout restart deployment/neoland -n default
```

**Timeframe**: 5-10 minutes

### Scenario 3: CPU/Memory Exhaustion

```bash
# 1. Scale horizontally (add more pods)
kubectl scale deployment/neoland --replicas=5 -n default

# 2. Or increase CPU/memory limits
kubectl set resources deployment/neoland \
  --limits=cpu=2000m,memory=4Gi \
  --requests=cpu=1000m,memory=2Gi \
  -n default

# 3. Monitor latency improvement
```

**Timeframe**: 3-5 minutes

### Scenario 4: Network Latency

```bash
# 1. Check if database/ml-offload in different AZ
kubectl get pods -o wide -n default

# 2. Add pod affinity to colocate services
# (Requires deployment yaml change)

# 3. Or use connection pooling/caching to reduce DB calls
```

**Timeframe**: 10-30 minutes (requires config change)

---

## Verification

```bash
# 1. Check p99 latency dropped
curl -s 'http://prometheus:9090/api/v1/query?query=histogram_quantile(0.99,rate(neoland_http_request_duration_seconds_bucket[5m]))' | jq
# Expected: < 2.0s

# 2. Test API manually
time curl -X POST http://neoland.example.com/v1/chat/completions \
  -H "X-API-Key: $API_KEY" \
  -d '{"messages":[{"role":"user","content":"test"}]}'
# Expected: < 2s

# 3. Check alert cleared
# http://alertmanager:9093/#/alerts
```

---

## Escalation

| Latency | Severity | Response | Escalation |
|---------|----------|----------|------------|
| 2-5s | Warning | 30 min | #neoland-warnings |
| >5s | Critical | Immediate | PagerDuty + #neoland-critical |

**Contacts**: `@oncall-voidnx` (Slack), Team Lead (after 15 min)

---

## Post-Incident

1. **Create issue** if root cause requires code changes
2. **Add caching** if database queries are slow
3. **Optimize LLM prompts** if inference is slow
4. **Add connection pooling** if network latency high

---

## Prevention

- ✅ Latency monitoring (p99, p95, p50)
- ✅ Alerts at 2s and 5s thresholds
- ⚠️ **TODO**: Add request tracing (OpenTelemetry)
- ⚠️ **TODO**: Cache frequent queries
- ⚠️ **TODO**: Implement query result caching

---

## Related Alerts

- `NeolandSlowLLMInference`: LLM-specific latency
- `NeolandHighCPUUsage`: May cause latency
- `NeolandHighMemoryUsage`: May cause GC pauses
- `NeolandHighErrorRate`: Often follows high latency

---

## Additional Resources

- **Grafana Dashboard**: [NEOLAND Latency](http://grafana/d/neoland-latency)
- **Trace Analysis**: (TODO: Add OpenTelemetry link)
- **Performance Guide**: `docs/PERFORMANCE.md`

---

**Last Updated**: 2026-01-31
**Maintainer**: voidnx team
