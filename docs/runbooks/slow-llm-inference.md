# Runbook: NeolandSlowLLMInference

**Alert**: `NeolandSlowLLMInference`
**Severity**: WARNING ⚠️
**Response Time**: 15-30 minutes

---

## Symptoms

- **Alert**: LLM inference taking longer than expected
- **User Impact**: Slow chat responses, degraded user experience
- **Metrics**: `histogram_quantile(0.95, rate(neoland_llm_duration_seconds_bucket[5m])) > 10` (p95 >10s)
- **Common Causes**:
  - ml-offload service slow or down
  - Model loading delays
  - GPU/CPU resource constraints
  - Network latency to external providers
  - Queue backlog in inference service
  - Slow fallback to SecureLLM API

---

## Investigation

### Step 1: Check LLM Provider Distribution

```bash
# Check which provider is being used
kubectl logs -l app=neoland --tail=500 -n default \
  | jq 'select(.llm_provider)' \
  | jq -r '. | "\(.llm_provider) \(.llm_duration_ms)ms"' \
  | sort | uniq -c | sort -rn

# Expected output:
#  450 ml_offload 250ms
#   30 local_engine 1200ms
#   20 securellm 3000ms

# If most requests on slow providers (local/SecureLLM) → ml-offload down/slow
```

### Step 2: Check ml-offload Service Health

```bash
# Check ml-offload pod status
kubectl get pods -l app=ml-offload -n default

# Expected: Running, READY 1/1

# Check ml-offload health
curl http://ml-offload:8000/health | jq

# Expected:
# {
#   "status": "healthy",
#   "model_loaded": true,
#   "gpu_available": true,
#   "queue_depth": 3
# }

# Check ml-offload logs for errors
kubectl logs -l app=ml-offload --tail=200 -n default | grep -i "error\|warn"
```

### Step 3: Analyze Inference Latency by Provider

```bash
# Check p95 latency per provider in Prometheus
curl -s 'http://prometheus:9090/api/v1/query?query=histogram_quantile(0.95,rate(neoland_llm_duration_seconds_bucket[5m]))' \
  | jq '.data.result[] | {provider: .metric.provider, latency: .value[1]}'

# Expected:
# ml_offload: <3s
# local_engine: <5s
# securellm: <8s

# If ml_offload >10s → Performance degradation
```

### Step 4: Check Resource Usage

```bash
# Check ml-offload CPU/memory usage
kubectl top pods -l app=ml-offload -n default

# Check ml-offload GPU usage (if available)
kubectl exec -it deployment/ml-offload -n default -- nvidia-smi

# Expected GPU utilization: 70-90% during inference

# Check if ml-offload is CPU/memory throttled
kubectl describe pod -l app=ml-offload -n default | grep -A 5 "Limits:"
```

---

## Resolution

### Scenario 1: ml-offload Service Down/Slow

**Symptoms**: ml-offload health check failing, requests falling back to local/SecureLLM

```bash
# 1. Restart ml-offload service
kubectl rollout restart deployment/ml-offload -n default

# 2. Wait for restart
kubectl rollout status deployment/ml-offload -n default

# 3. Verify model loaded
curl http://ml-offload:8000/health | jq '.model_loaded'

# Expected: true

# 4. Test inference
curl -X POST http://ml-offload:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"messages":[{"role":"user","content":"test"}]}'

# Expected: <3s response time

# 5. Monitor NEOLAND switches back to ml-offload
kubectl logs -l app=neoland --tail=100 | grep "llm_provider"

# Expected: Majority of requests using "ml_offload"
```

**Timeframe**: 2-5 minutes

### Scenario 2: Model Loading Delays (Cold Start)

**Symptoms**: First request after restart very slow (>30s), subsequent requests normal

```bash
# 1. Pre-load model on startup (add to ml-offload init script)
# (Requires code change in ml-offload)

# 2. Keep ml-offload warm with periodic health checks
# (Kubernetes readiness probe with actual inference test)

# 3. Add liveness probe with longer initialDelaySeconds
kubectl patch deployment ml-offload -n default --type='json' -p='[
  {
    "op": "replace",
    "path": "/spec/template/spec/containers/0/livenessProbe/initialDelaySeconds",
    "value": 120
  }
]'

# 4. Scale ml-offload to 2+ replicas for redundancy
kubectl scale deployment/ml-offload --replicas=2 -n default
```

**Timeframe**: Immediate fix (restart), long-term fix (code change)

### Scenario 3: GPU/CPU Resource Constraints

**Symptoms**: ml-offload pod CPU/memory at limit, GPU utilization 100%

```bash
# 1. Check resource usage
kubectl top pods -l app=ml-offload -n default

# 2. Increase resource limits
kubectl set resources deployment/ml-offload \
  --limits=cpu=4000m,memory=8Gi \
  --requests=cpu=2000m,memory=4Gi \
  -n default

# 3. Monitor rollout
kubectl rollout status deployment/ml-offload -n default

# 4. Verify inference latency improved
curl -s 'http://prometheus:9090/api/v1/query?query=histogram_quantile(0.95,rate(neoland_llm_duration_seconds_bucket{provider="ml_offload"}[5m]))' | jq
```

**Timeframe**: 3-5 minutes

### Scenario 4: High Queue Depth / Concurrent Requests

**Symptoms**: Queue depth >10, many concurrent inference requests, latency increases with load

```bash
# 1. Check current queue depth
curl http://ml-offload:8000/metrics | grep queue_depth

# 2. Scale ml-offload horizontally (if supported)
kubectl scale deployment/ml-offload --replicas=3 -n default

# 3. Implement request batching (requires code change)
# - Batch multiple requests into single inference call
# - Reduce overhead of model loading

# 4. Add load balancing across ml-offload replicas
# (Kubernetes Service automatically load balances)

# 5. Monitor latency improves with scaling
watch 'curl -s http://prometheus:9090/api/v1/query?query=histogram_quantile(0.95,rate(neoland_llm_duration_seconds_bucket{provider="ml_offload"}[1m])) | jq'
```

**Timeframe**: 3-5 minutes (scaling), longer for code changes

### Scenario 5: Network Latency to External Providers

**Symptoms**: SecureLLM API slow (>5s), no issues with ml-offload/local

```bash
# 1. Test SecureLLM API latency directly
time curl -X POST https://api.securellm.com/v1/chat/completions \
  -H "X-API-Key: $SECURELLM_KEY" \
  -H "Content-Type: application/json" \
  -d '{"messages":[{"role":"user","content":"test"}]}'

# 2. Check SecureLLM status page
curl https://status.securellm.com/api/v2/status.json | jq

# 3. If SecureLLM slow globally, consider:
#    - Use different region endpoint (if available)
#    - Switch to alternative provider (DeepSeek, OpenAI)
#    - Prioritize local engine over SecureLLM

# 4. Update routing strategy temporarily
# (Requires config change to deprioritize SecureLLM)

# 5. Create incident with SecureLLM support
```

**Timeframe**: Immediate workaround (change routing), longer for provider fix

### Scenario 6: Prompt Too Long / Context Window Exhausted

**Symptoms**: Only specific requests slow, error logs show "max tokens exceeded"

```bash
# 1. Check recent requests with long prompts
kubectl logs -l app=neoland --tail=500 | jq 'select(.prompt_tokens > 4000)'

# 2. Identify users sending very long prompts
kubectl logs -l app=neoland --tail=1000 | jq 'select(.prompt_tokens > 8000) | {user: .user_id, tokens: .prompt_tokens}'

# 3. Implement prompt truncation
# (Requires code change to limit context window)

# 4. Add rate limiting for expensive requests
# (Higher cost for longer prompts)

# 5. Return user-friendly error for too-long prompts
# HTTP 413 Payload Too Large
```

**Timeframe**: Immediate mitigation (manual user contact), long-term fix (code change)

---

## Verification

### Verify Inference Latency Improved

```bash
# 1. Check p95 latency dropped
curl -s 'http://prometheus:9090/api/v1/query?query=histogram_quantile(0.95,rate(neoland_llm_duration_seconds_bucket[5m]))' | jq

# Expected: <5s (ideally <3s for ml-offload)

# 2. Test API response time
time curl -X POST http://neoland.example.com/v1/chat/completions \
  -H "X-API-Key: $API_KEY" \
  -d '{"messages":[{"role":"user","content":"Hello, how are you?"}]}'

# Expected: <5s total response time

# 3. Check alert cleared
# http://alertmanager:9093/#/alerts
# Expected: NeolandSlowLLMInference resolved

# 4. Monitor latency over 10 minutes
watch 'curl -s http://prometheus:9090/api/v1/query?query=histogram_quantile(0.95,rate(neoland_llm_duration_seconds_bucket[5m])) | jq'

# Expected: Stable <5s
```

### Verify Provider Distribution

```bash
# Check most requests using fast provider (ml-offload)
kubectl logs -l app=neoland --tail=500 | jq 'select(.llm_provider) | .llm_provider' | sort | uniq -c | sort -rn

# Expected:
#  450 ml_offload  (90%+ of requests)
#   30 local_engine
#   20 securellm
```

---

## Escalation

| Latency | Severity | Response | Escalation |
|---------|----------|----------|------------|
| 5-10s (p95) | Warning | 30 min | #neoland-warnings |
| >10s (p95) | High | 15 min | #neoland-critical + @oncall-ml |

**Escalation Contacts**:
- Primary: `@oncall-voidnx`
- ML Team: `@oncall-ml` (for ml-offload issues)
- Infrastructure: `@oncall-infra` (for resource constraints)

**Escalation Timeline**:
- 0-15 min: On-call engineer investigates
- 15-30 min: Escalate to ML Team if ml-offload specific
- 30-60 min: Team Lead if not resolved
- 60+ min: Consider switching to backup provider

---

## Post-Incident

### Immediate Actions (within 1 hour)

1. **Identify Root Cause**:
   - ml-offload down? → Improve health checks, add redundancy
   - Resource constraints? → Increase limits, add autoscaling
   - Network latency? → Use closer provider region
   - Queue backlog? → Implement batching, add replicas

2. **Create GitHub Issue** if code changes needed:
   ```
   Title: [PERFORMANCE] Slow LLM inference from <provider>
   Labels: performance, llm, P2
   Description:
   - Provider: <ml-offload/local/SecureLLM>
   - p95 latency: <value>
   - Duration: <how long>
   - Root cause: <analysis>
   - Mitigation: <temporary fix>
   - Action items: <optimizations>
   ```

### Follow-Up Actions (within 24 hours)

1. **Improve ml-offload Reliability**:
   - Add multiple replicas (2-3 for HA)
   - Implement health checks with actual inference test
   - Pre-load model on startup
   - Add request queue metrics

2. **Optimize Inference Performance**:
   - Implement request batching (group requests)
   - Add model caching (keep model in GPU memory)
   - Optimize prompt templates (reduce tokens)
   - Use quantized models (INT8) for faster inference

3. **Add Fallback Monitoring**:
   - Alert if fallback rate >10% (ml-offload → local/SecureLLM)
   - Track latency per provider
   - Monitor provider availability

4. **Implement Smart Routing**:
   - Route based on prompt complexity (short → fast provider)
   - Load balance across multiple providers
   - Cache common responses

---

## Prevention

### Short-term (Immediate)

- ✅ LLM latency monitoring with alerts (p95 >10s)
- ✅ 3-tier fallback chain (ml-offload → local → SecureLLM)
- ⚠️ **TODO**: Deploy ml-offload with 2+ replicas
- ⚠️ **TODO**: Pre-load model on startup

### Medium-term (1-2 weeks)

- [ ] Implement request batching
- [ ] Add response caching for common queries
- [ ] Optimize prompt templates (reduce token count)
- [ ] Deploy model with INT8 quantization
- [ ] Add queue depth metrics and alerts

### Long-term (1-3 months)

- [ ] Dedicated GPU nodes for ml-offload (Kubernetes node pools)
- [ ] Implement auto-scaling based on queue depth
- [ ] Multi-model deployment (small/medium/large based on complexity)
- [ ] Smart routing based on prompt analysis
- [ ] Continuous performance benchmarking

---

## Related Alerts

- `NeolandHighLatency`: Overall API latency includes LLM inference
- `NeolandAllLLMProvidersFailing`: Complete LLM failure (worse than slow)
- `NeolandHighCPUUsage`: ml-offload CPU exhaustion
- `NeolandHighErrorRate`: Timeouts may cause 500 errors

---

## Additional Resources

- **ml-offload Documentation**: `docs/ML_OFFLOAD.md`
- **LLM Fallback Chain**: `docs/ADR/ADR-006-llm-fallback.md`
- **Performance Tuning**: `docs/PERFORMANCE.md`
- **Grafana Dashboard**: [LLM Latency](http://grafana/d/neoland-llm-latency)

---

**Last Updated**: 2026-01-31
**Maintainer**: voidnx team + ML team
**Severity**: WARNING - Performance degradation
