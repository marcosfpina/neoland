# Runbook: NeolandHighCPUUsage / NeolandCriticalCPUUsage

**Alert**: `NeolandHighCPUUsage` (warning) / `NeolandCriticalCPUUsage` (critical)
**Severity**: WARNING ⚠️ / CRITICAL 🔴
**Response Time**: 15-30 min (warning) / 0-5 min (critical)

---

## Symptoms

- **Alert**: High CPU usage on NEOLAND pods
- **User Impact**:
  - Warning: Slower responses, increased latency
  - Critical: Service degradation, potential pod eviction
- **Metrics**:
  - Warning: `avg(rate(container_cpu_usage_seconds_total{pod=~"neoland-.*"}[5m])) > 0.80` (>80% CPU)
  - Critical: `> 0.95` (>95% CPU)
- **Common Causes**:
  - Heavy LLM inference load
  - Vector similarity search overload
  - Resource-intensive NLP operations
  - Too many concurrent requests
  - Inefficient code (hot path)

---

## Investigation

### Step 1: Identify CPU-Intensive Pods

```bash
# Check CPU usage by pod
kubectl top pods -l app=neoland -n default --sort-by=cpu

# Expected output:
# NAME                       CPU      MEMORY
# neoland-7b9c8d5f6-abc12    1800m    2048Mi
# neoland-7b9c8d5f6-def34    1500m    1536Mi

# If CPU >1800m (90% of 2000m limit) on multiple pods → Critical
```

### Step 2: Check Request Rate

```bash
# Check current request rate
curl -s 'http://prometheus:9090/api/v1/query?query=rate(neoland_http_requests_total[1m])' | jq '.data.result[0].value[1]'

# Check concurrent requests
curl http://neoland:3001/metrics | grep neoland_http_requests_active

# If RPS >500 or active requests >100 → High load scenario
```

### Step 3: Identify Hot Endpoints

```bash
# Check which endpoints are CPU-intensive
kubectl logs -l app=neoland --tail=500 -n default \
  | jq 'select(.duration_ms > 1000)' \
  | jq -r '.endpoint' \
  | sort | uniq -c | sort -rn | head -10

# Check LLM inference times
kubectl logs -l app=neoland --tail=500 -n default \
  | jq 'select(.llm_duration_ms)' \
  | jq -r '. | "\(.llm_provider) \(.llm_duration_ms)ms"' \
  | sort | uniq -c | sort -rn | head -10
```

### Step 4: Check for Resource Throttling

```bash
# Check if pods are being CPU throttled
kubectl describe pod -l app=neoland -n default | grep -A 5 "Limits:\|Requests:"

# Check node CPU usage
kubectl top nodes

# If node CPU >85% → Node capacity issue
```

---

## Resolution

### Scenario 1: High Request Load (Most Common)

**Symptoms**: High RPS, many active requests, all pods at high CPU

```bash
# 1. Scale horizontally (add more pods)
kubectl scale deployment/neoland --replicas=5 -n default

# Current: 3 replicas
# Target: 5 replicas to distribute load

# 2. Wait for new pods to be ready
kubectl rollout status deployment/neoland -n default

# 3. Monitor CPU usage drops
watch 'kubectl top pods -l app=neoland -n default'

# 4. Verify load distributed
curl -s 'http://prometheus:9090/api/v1/query?query=rate(neoland_http_requests_total[1m])' | jq
```

**Timeframe**: 2-3 minutes (pod startup + load distribution)

### Scenario 2: LLM Inference Overload

**Symptoms**: High `llm_duration_ms`, ml-offload slow or down

```bash
# 1. Check ml-offload status
curl http://ml-offload:8000/health | jq

# 2. If ml-offload slow/down, restart it
kubectl rollout restart deployment/ml-offload -n default

# 3. Check if local engine is overwhelmed
kubectl logs -l app=neoland --tail=100 | grep "local_engine" | grep "duration_ms"

# 4. If local engine slow, consider temporarily switching to SecureLLM only
# (Requires config change, discuss with team first)

# 5. Monitor LLM provider distribution
curl http://neoland:3001/metrics | grep neoland_llm_requests_total
```

**Timeframe**: 3-5 minutes

### Scenario 3: Vector Search Overload

**Symptoms**: High CPU in vector similarity search, large vector store

```bash
# 1. Check vector store size
kubectl logs -l app=neoland --tail=100 | grep "vector_store_size"

# 2. If vector store very large (>100K documents), consider:
#    - Implement result caching
#    - Reduce top-k results
#    - Optimize embedding model

# 3. Temporarily reduce concurrent searches (rate limiting)
kubectl set env deployment/neoland \
  RATE_LIMIT_SEARCH="10/min" \
  -n default

# 4. Monitor CPU drops
```

**Timeframe**: 5-10 minutes

### Scenario 4: Inefficient Code / Hot Path

**Symptoms**: CPU high even with low request rate, specific endpoint slow

```bash
# 1. Enable CPU profiling (if available)
kubectl exec -it deployment/neoland -n default -- \
  curl -X POST http://localhost:3001/admin/enable-profiling

# 2. Collect profile for 60 seconds
# (Profile data saved to /tmp/cpu-profile.pb.gz)

# 3. Analyze hot functions
# (Download and analyze with pprof or flamegraph)

# 4. Create GitHub issue for code optimization
# Title: [PERFORMANCE] CPU hotspot in <endpoint>
# Labels: performance, P2
```

**Timeframe**: Immediate mitigation via scaling, long-term fix requires code changes

### Scenario 5: Node Resource Exhaustion

**Symptoms**: Node CPU >85%, multiple services affected, pod eviction warnings

```bash
# 1. Check node capacity
kubectl top nodes
kubectl describe node <node-name> | grep -A 10 "Allocated resources:"

# 2. If node overloaded, move NEOLAND to different node
kubectl cordon <overloaded-node>

# 3. Evict NEOLAND pods from overloaded node
kubectl delete pod -l app=neoland --grace-period=30

# 4. Pods will reschedule on healthy nodes

# 5. Verify CPU usage normal
kubectl top pods -l app=neoland -n default

# 6. Uncordon node after issue resolved
kubectl uncordon <node-name>
```

**Timeframe**: 5-10 minutes

### Scenario 6: CPU Limits Too Low

**Symptoms**: CPU throttling even with capacity available, pod CPU at limit

```bash
# 1. Check current limits
kubectl get deployment neoland -n default -o jsonpath='{.spec.template.spec.containers[0].resources}'

# 2. Increase CPU limits (if justified)
kubectl set resources deployment/neoland \
  --limits=cpu=4000m \
  --requests=cpu=2000m \
  -n default

# Current limits: 2000m (2 cores)
# New limits: 4000m (4 cores)

# 3. Monitor rollout
kubectl rollout status deployment/neoland -n default

# 4. Verify CPU no longer throttled
kubectl describe pod -l app=neoland -n default | grep -A 3 "cpu"
```

**Timeframe**: 3-5 minutes

---

## Verification

### Verify CPU Usage Dropped

```bash
# 1. Check CPU usage by pod
kubectl top pods -l app=neoland -n default

# Expected: <70% of CPU limit (e.g., <1400m for 2000m limit)

# 2. Check Prometheus metric
curl -s 'http://prometheus:9090/api/v1/query?query=avg(rate(container_cpu_usage_seconds_total{pod=~"neoland-.*"}[5m]))' | jq

# Expected: <0.70 (70%)

# 3. Verify no CPU throttling
kubectl describe pod -l app=neoland -n default | grep "cpu" | grep -i "throttl"

# Expected: No throttling warnings

# 4. Check alert cleared
# http://alertmanager:9093/#/alerts
# Expected: NeolandHighCPUUsage / NeolandCriticalCPUUsage resolved
```

### Verify Service Performance

```bash
# 1. Check latency returned to normal
curl -s 'http://prometheus:9090/api/v1/query?query=histogram_quantile(0.99,rate(neoland_http_request_duration_seconds_bucket[5m]))' | jq

# Expected: p99 <2s

# 2. Test API response time
time curl -X POST http://neoland.example.com/v1/chat/completions \
  -H "X-API-Key: $API_KEY" \
  -d '{"messages":[{"role":"user","content":"test"}]}'

# Expected: <2s response time

# 3. Verify health check
curl http://neoland:3001/health | jq '.status'

# Expected: "healthy"
```

---

## Escalation

| CPU Usage | Severity | Response | Escalation |
|-----------|----------|----------|------------|
| 80-94% | Warning | 30 min | #neoland-warnings |
| 95%+ | Critical | Immediate | PagerDuty + #neoland-critical |

**Escalation Timeline**:
- **0-5 min**: On-call engineer investigates
- **5-15 min**: If not resolved, escalate to Team Lead
- **15-30 min**: If still high, escalate to Manager + consider infrastructure team

**Contacts**:
- Primary: `@oncall-voidnx` (Slack, PagerDuty)
- Infrastructure: `@oncall-infra` (if node capacity issue)
- ML Team: `@oncall-ml` (if ml-offload issue)

---

## Post-Incident

### Immediate Actions (within 1 hour)

1. **Document what caused high CPU**:
   - High request load? → Consider autoscaling (HPA)
   - Inefficient code? → Create performance issue
   - LLM overload? → Optimize inference strategy
   - Vector search? → Implement caching

2. **Create GitHub Issue** if code optimization needed:
   ```
   Title: [PERFORMANCE] High CPU usage in <component>
   Labels: performance, P2
   Description:
   - CPU usage: <percentage>
   - Hot path: <endpoint or function>
   - Duration: <how long it lasted>
   - Mitigation: <what was done>
   - Root cause: <analysis>
   - Action items: <optimizations needed>
   ```

### Follow-Up Actions (within 24 hours)

1. **Implement Horizontal Pod Autoscaling (HPA)**:
   ```yaml
   # k8s/hpa.yaml
   apiVersion: autoscaling/v2
   kind: HorizontalPodAutoscaler
   metadata:
     name: neoland
   spec:
     scaleTargetRef:
       apiVersion: apps/v1
       kind: Deployment
       name: neoland
     minReplicas: 3
     maxReplicas: 10
     metrics:
     - type: Resource
       resource:
         name: cpu
         target:
           type: Utilization
           averageUtilization: 70
   ```

2. **Enable CPU Profiling**:
   - Add pprof endpoint for continuous profiling
   - Collect flamegraphs for hot paths
   - Analyze with profiling tools

3. **Optimize Hot Paths** (if identified):
   - Reduce allocations in hot loops
   - Implement connection pooling
   - Cache expensive computations
   - Parallelize independent operations

4. **Review Resource Limits**:
   - Are current limits appropriate?
   - Should we increase CPU requests/limits?
   - Document resource sizing decisions

---

## Prevention

### Short-term (Immediate)

- ✅ CPU monitoring with alerts (>80% warning, >95% critical)
- ✅ Horizontal scaling capability (manual)
- ⚠️ **TODO**: Implement Horizontal Pod Autoscaler (HPA)
- ⚠️ **TODO**: Add CPU profiling endpoint

### Medium-term (1-2 weeks)

- [ ] Implement request result caching
- [ ] Optimize vector similarity search (HNSW indexing)
- [ ] Add rate limiting per user/IP
- [ ] Implement request queueing with backpressure

### Long-term (1-3 months)

- [ ] Continuous profiling (Pyroscope or similar)
- [ ] Auto-scaling based on custom metrics (request queue depth)
- [ ] Dedicated inference service (offload LLM from main service)
- [ ] Implement request batching for LLM inference

---

## Related Alerts

- `NeolandHighLatency`: Often correlates with high CPU
- `NeolandHighMemoryUsage`: May also fire during heavy load
- `NeolandSlowLLMInference`: CPU-intensive LLM operations
- `NeolandPodRestartLoop`: OOMKill due to resource exhaustion

---

## Additional Resources

- **Grafana Dashboard**: [NEOLAND CPU Usage](http://grafana/d/neoland-cpu)
- **Performance Guide**: `docs/PERFORMANCE.md`
- **Resource Sizing**: `docs/RESOURCE_SIZING.md` (TODO: Phase 5)
- **HPA Configuration**: `k8s/hpa.yaml` (TODO: Phase 5)

---

**Last Updated**: 2026-01-31
**Maintainer**: voidnx team
**Severity**: WARNING / CRITICAL (based on threshold)
