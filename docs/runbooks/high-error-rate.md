# Runbook: NeolandHighErrorRate / NeolandCriticalErrorRate

**Alert**: `NeolandHighErrorRate` (warning) / `NeolandCriticalErrorRate` (critical)
**Severity**: WARNING ⚠️ / CRITICAL 🔴
**Response Time**: 15-30 minutes (warning) / 0-5 minutes (critical)

---

## Symptoms

- **Alert**: NeolandHighErrorRate or NeolandCriticalErrorRate firing
- **User Impact**:
  - **Warning (>5% error rate)**: Some users experiencing failures
  - **Critical (>25% error rate)**: Majority of requests failing
- **Metrics**:
  - Warning: `rate(neoland_http_requests_total{status=~"5.."}[5m]) / rate(neoland_http_requests_total[5m]) > 0.05`
  - Critical: `> 0.25`
- **Dashboards**: High 5xx error rate on Grafana

---

## Investigation

### Step 1: Identify Error Pattern

```bash
# Check recent error logs
kubectl logs -l app=neoland --tail=500 -n default | grep -i "error\|500\|503"

# Count errors by type
kubectl logs -l app=neoland --tail=1000 -n default \
  | grep -i "error" \
  | awk '{print $NF}' \
  | sort | uniq -c | sort -rn | head -10

# Check error rate in Prometheus
curl -s 'http://prometheus:9090/api/v1/query?query=rate(neoland_http_requests_total{status=~"5.."}[5m])' | jq
```

### Step 2: Check Affected Endpoints

```bash
# Query Prometheus for errors by endpoint
curl -s 'http://prometheus:9090/api/v1/query?query=rate(neoland_http_requests_total{status=~"5.."}[5m])' \
  | jq '.data.result[] | {endpoint: .metric.endpoint, value: .value[1]}'
```

**Common patterns**:
- All endpoints: Infrastructure issue
- Single endpoint: Bug in that endpoint
- `/v1/chat/completions`: LLM provider issue

### Step 3: Check Dependencies

```bash
# Check LLM provider health
curl http://neoland:3001/health | jq '.components[] | select(.name | contains("llm"))'

# Check vector store health
curl http://neoland:3001/health | jq '.components[] | select(.name | contains("vector"))'

# Check database connectivity
kubectl exec -it deployment/neoland -n default -- \
  psql $DATABASE_URL -c "SELECT 1;"
```

### Step 4: Check Resource Constraints

```bash
# Check CPU/memory usage
kubectl top pods -l app=neoland -n default

# Check for OOMKills or throttling
kubectl describe pod -l app=neoland -n default | grep -A 10 "Events:"

# Check connection pool exhaustion
kubectl logs -l app=neoland --tail=200 -n default \
  | grep -i "connection\|pool"
```

### Step 5: Check Recent Changes

```bash
# Check recent deployments
kubectl rollout history deployment/neoland -n default

# Check recent config changes
kubectl get configmap neoland-config -n default -o yaml

# Check git log for recent commits
cd /path/to/neoland && git log --oneline -10
```

---

## Resolution

### Scenario 1: LLM Provider Failure

**Symptoms**: Errors contain "LLM", "provider", "timeout", "rate limit"

```bash
# 1. Check which provider is failing
kubectl logs -l app=neoland --tail=100 -n default \
  | grep -i "llm" | grep -i "error"

# 2. Check LLM provider health
curl http://neoland:3001/health | jq '.components[] | select(.name | contains("llm"))'

# 3. If ml-offload down, restart it
kubectl rollout restart deployment/ml-offload -n default

# 4. If SecureLLM API rate limited, wait for reset or switch provider
# (Check ml-offload logs for rate limit headers)

# 5. If all providers down, check external status pages
# - DeepSeek: https://status.deepseek.com
# - OpenAI: https://status.openai.com
```

**Timeframe**: 2-5 minutes (if internal), 10-30 minutes (if external)

### Scenario 2: Database Connection Issues

**Symptoms**: Errors contain "database", "connection", "timeout", "pool"

```bash
# 1. Check PostgreSQL status
kubectl get pods -l app=postgres -n default

# 2. Check connection count
kubectl exec -it deployment/postgres -n default -- \
  psql -U postgres -c "SELECT count(*) FROM pg_stat_activity;"

# 3. If connection pool exhausted, restart NEOLAND
kubectl rollout restart deployment/neoland -n default

# 4. If PostgreSQL down, check logs
kubectl logs -l app=postgres --tail=200 -n default

# 5. If PostgreSQL overloaded, scale up
kubectl scale deployment/postgres --replicas=2 -n default
```

**Timeframe**: 3-10 minutes

### Scenario 3: Memory/CPU Exhaustion

**Symptoms**: High CPU/memory usage, slow responses, timeouts

```bash
# 1. Check resource usage
kubectl top pods -l app=neoland -n default

# 2. If memory >90%, increase limits temporarily
kubectl set resources deployment/neoland \
  --limits=memory=4Gi,cpu=2000m \
  --requests=memory=2Gi,cpu=1000m \
  -n default

# 3. Monitor recovery
kubectl rollout status deployment/neoland -n default

# 4. If CPU >90%, check for infinite loops or heavy processing
kubectl logs -l app=neoland --tail=500 -n default | grep -i "processing\|query"
```

**Timeframe**: 5-10 minutes

### Scenario 4: Bad Deployment

**Symptoms**: Errors started after recent deployment

```bash
# 1. Check deployment time
kubectl rollout history deployment/neoland -n default

# 2. Check error spike correlation
# (Compare error start time with deployment time)

# 3. Rollback to previous version
kubectl rollout undo deployment/neoland -n default

# 4. Verify rollback
kubectl rollout status deployment/neoland -n default

# 5. Monitor error rate drop
watch 'curl -s http://prometheus:9090/api/v1/query?query=rate(neoland_http_requests_total{status=~"5.."}[1m]) | jq'
```

**Timeframe**: 2-4 minutes

### Scenario 5: External API Failure

**Symptoms**: Errors contain "external", "API", specific service name

```bash
# 1. Identify failing external service
kubectl logs -l app=neoland --tail=200 -n default \
  | grep -i "error" | grep -oP 'https?://[^\s]+'

# 2. Test external API directly
curl -i https://failing-api.example.com/health

# 3. Check status pages for external services
# - GitHub: https://www.githubstatus.com
# - AWS: https://health.aws.amazon.com

# 4. If external service down:
#    - Enable circuit breaker to fail fast
#    - Return cached responses if available
#    - Switch to alternative provider if available

# 5. Notify users via status page
```

**Timeframe**: Depends on external service recovery

### Scenario 6: Request Validation Errors

**Symptoms**: 500 errors due to unexpected input

```bash
# 1. Check validation errors
kubectl logs -l app=neoland --tail=200 -n default \
  | grep -i "validation\|invalid"

# 2. Identify problematic requests
kubectl logs -l app=neoland --tail=500 -n default \
  | grep -B 5 "validation error"

# 3. If due to malformed requests, add validation
# (Create GitHub issue, not immediate fix)

# 4. If due to bug in validation logic, rollback or hotfix
```

**Timeframe**: 10-30 minutes (requires code change)

---

## Verification

After fix, verify error rate returns to normal:

```bash
# 1. Check error rate in Prometheus
curl -s 'http://prometheus:9090/api/v1/query?query=rate(neoland_http_requests_total{status=~"5.."}[5m])' \
  | jq '.data.result[0].value[1]'
# Expected: < 0.01 (1%)

# 2. Check logs for new errors
kubectl logs -l app=neoland --tail=100 -n default | grep -i "error"

# 3. Test API manually
curl -X POST http://neoland.example.com/v1/chat/completions \
  -H "X-API-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"messages":[{"role":"user","content":"test"}]}'
# Expected: 200 OK

# 4. Verify alert clears in AlertManager
# http://alertmanager:9093/#/alerts
```

**Expected**: Error rate < 1% within 5 minutes of fix

---

## Escalation

### Escalation Thresholds

| Error Rate | Severity | Response | Escalation |
|------------|----------|----------|------------|
| 5-10% | Warning | 30 min | Team Slack |
| 10-25% | Warning | 15 min | Team Lead |
| >25% | Critical | Immediate | PagerDuty + Manager |

### Escalation Path

- **Slack**: `#neoland-critical` (for critical) or `#neoland-warnings` (for warning)
- **PagerDuty**: Automatic for critical
- **Contacts**:
  - Primary: `@oncall-voidnx`
  - Secondary: Team Lead (after 15 min)
  - Manager: (if >25% for >30 min)

---

## Post-Incident

### Immediate Actions

1. **Document in incident log** (if critical):
   - Error rate peak
   - Duration
   - Root cause
   - Affected endpoints
   - Number of failed requests

2. **Create GitHub issue**:
   ```
   Title: [Incident] High error rate on YYYY-MM-DD
   Labels: incident, bug
   Description:
   - Error rate peaked at X%
   - Root cause: [description]
   - Fix applied: [description]
   - Action items: [prevent recurrence]
   ```

### Prevention

1. **Add monitoring** for root cause (if new pattern)
2. **Improve error handling** (if validation issue)
3. **Add circuit breakers** (if external dependency)
4. **Increase timeouts** (if timeout issue)
5. **Add retries** (if transient failures)

---

## Related Alerts

- `NeolandHighLatency`: Often precedes high error rate
- `NeolandLLMProviderFailures`: LLM-specific errors
- `NeolandVectorStoreUnavailable`: Database errors
- `NeolandHighMemoryUsage`: May cause OOM errors

---

## Additional Resources

- **Grafana Dashboard**: [NEOLAND Error Rate](http://grafana/d/neoland-errors)
- **Error Rate Query**: `rate(neoland_http_requests_total{status=~"5.."}[5m])`
- **Logs**: `kubectl logs -l app=neoland -n default --tail=1000 | grep error`
- **Debugging Guide**: `docs/DEBUGGING.md`

---

**Last Updated**: 2026-01-31
**Maintainer**: voidnx team
**Tested**: ✅ Quarterly DR drill
