# Runbook: NeolandAllLLMProvidersFailing

**Alert**: `NeolandAllLLMProvidersFailing`
**Severity**: CRITICAL 🔴
**Response Time**: 0-5 minutes

---

## Symptoms

- **Alert**: All LLM providers showing unhealthy status
- **User Impact**: COMPLETE FAILURE - No LLM responses possible
- **Metrics**: `sum(neoland_component_health{component=~"llm_.*", status="healthy"}) == 0`
- **Fallback**: All fallback chain exhausted (ml-offload → local → SecureLLM)

---

## Investigation

```bash
# Check LLM provider health
curl http://neoland:3001/health | jq '.components[] | select(.name | contains("llm"))'

# Expected output showing all unhealthy:
# {"name": "llm_ml_offload", "status": "unhealthy", ...}
# {"name": "llm_local", "status": "unhealthy", ...}
# {"name": "llm_securellm", "status": "unhealthy", ...}

# Check each provider individually
```

### Check ml-offload

```bash
# Check if ml-offload service is running
kubectl get pods -l app=ml-offload -n default

# Check ml-offload health
curl http://ml-offload:8000/health

# Check ml-offload logs
kubectl logs -l app=ml-offload --tail=200 -n default
```

### Check Local Engine

```bash
# Check if local model loaded
kubectl logs -l app=neoland --tail=200 -n default | grep "local_engine"

# Check if model files accessible
kubectl exec -it deployment/neoland -n default -- ls -lh /models/
```

### Check SecureLLM API

```bash
# Test SecureLLM API directly
curl -X POST https://api.securellm.com/v1/chat/completions \
  -H "X-API-Key: $SECURELLM_KEY" \
  -H "Content-Type: application/json" \
  -d '{"messages":[{"role":"user","content":"test"}]}'

# Check SecureLLM status page
curl https://status.securellm.com/api/v2/status.json
```

---

## Resolution

### Step 1: Restart ml-offload (Primary Provider)

```bash
# Restart ml-offload service
kubectl rollout restart deployment/ml-offload -n default

# Wait for restart
kubectl rollout status deployment/ml-offload -n default

# Test health
curl http://ml-offload:8000/health | jq '.status'

# If healthy, verify NEOLAND detects it
curl http://neoland:3001/health | jq '.components[] | select(.name == "llm_ml_offload")'
```

**Timeframe**: 2-3 minutes

### Step 2: Fix Local Engine (Fallback 1)

```bash
# Check if model files missing
kubectl exec -it deployment/neoland -n default -- ls /models/

# If missing, check volume mount
kubectl describe pod -l app=neoland -n default | grep -A 5 "Volumes:"

# Reload local engine
kubectl exec -it deployment/neoland -n default -- \
  curl -X POST http://localhost:3001/admin/reload-local-engine

# Verify health
curl http://neoland:3001/health | jq '.components[] | select(.name == "llm_local")'
```

**Timeframe**: 3-5 minutes

### Step 3: Verify SecureLLM API Access (Fallback 2)

```bash
# Check API key is valid
kubectl get secret neoland-secrets -n default -o json \
  | jq -r '.data.SECURELLM_API_KEY' | base64 -d

# Test API key directly
curl -i https://api.securellm.com/v1/chat/completions \
  -H "X-API-Key: $(kubectl get secret neoland-secrets -n default -o json | jq -r '.data.SECURELLM_API_KEY' | base64 -d)"

# If 401 Unauthorized, API key expired/invalid
# Rotate API key in Vault
```

**Timeframe**: 2-5 minutes

### Step 4: Emergency Fallback (if all else fails)

```bash
# Deploy emergency fallback service (pre-built container)
kubectl apply -f k8s/emergency-llm-fallback.yaml

# Update NEOLAND to use emergency endpoint
kubectl set env deployment/neoland \
  EMERGENCY_LLM_URL=http://emergency-llm:8080 \
  -n default
```

**Timeframe**: 5-10 minutes

---

## Verification

```bash
# Check at least one provider healthy
curl http://neoland:3001/health | jq '.components[] | select(.name | contains("llm")) | select(.status == "healthy")'
# Expected: At least 1 healthy provider

# Test LLM request end-to-end
curl -X POST http://neoland:3001/v1/chat/completions \
  -H "X-API-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"messages":[{"role":"user","content":"Hello"}]}'
# Expected: 200 OK with response

# Check alert cleared
# http://alertmanager:9093/#/alerts
```

---

## Escalation

⚠️ **CRITICAL ALERT** - Service cannot function without LLM providers

- **Slack**: #neoland-critical (IMMEDIATE)
- **PagerDuty**: Automatic critical page
- **Contacts**:
  - Primary: `@oncall-voidnx` + `@oncall-ml`
  - ML Team Lead: (if ml-offload issue)
  - Infrastructure: (if SecureLLM API issue)
  - Manager: (if not resolved in 15 min)

**Escalation Timeline**:
- 0-5 min: On-call engineers (voidnx + ML)
- 5-15 min: Team Leads
- 15-30 min: Manager + consider incident commander

---

## Post-Incident

1. **Root Cause Analysis**: Why did ALL providers fail simultaneously?
   - Network issue affecting all external APIs?
   - Resource constraints affecting both ml-offload and local engine?
   - Configuration error pushed to both services?

2. **Improve Resilience**:
   - Add fourth fallback provider (e.g., OpenAI API)
   - Implement cache for recent responses (serve from cache during outages)
   - Pre-load local engine on startup (don't lazy-load)
   - Add provider health checks with faster detection

3. **Create GitHub Issue**:
   ```
   Title: [CRITICAL] All LLM providers down on YYYY-MM-DD
   Labels: incident, critical, llm
   Description:
   - All 3 LLM providers failed simultaneously
   - Root cause: [description]
   - Duration: [X minutes]
   - Impact: [number of failed requests]
   - Action items: [improvements]
   ```

---

## Prevention

- ✅ 3-tier fallback chain (ml-offload → local → SecureLLM)
- ✅ Health checks for all providers
- ✅ Alert fires after 1 minute of all providers down
- ⚠️ **TODO**: Add 4th fallback provider
- ⚠️ **TODO**: Cache recent responses
- ⚠️ **TODO**: Pre-load local engine on startup

---

## Related Alerts

- `NeolandLLMProviderFailures`: Single provider failing (may fire first)
- `NeolandSlowLLMInference`: LLM latency issues
- `NeolandHighErrorRate`: Will fire if all LLM requests fail

---

## Additional Resources

- **ml-offload Docs**: `docs/ML_OFFLOAD.md`
- **LLM Fallback Chain**: `docs/ADR/ADR-006-llm-fallback.md`
- **SecureLLM Status**: https://status.securellm.com
- **Local Engine Guide**: `docs/LOCAL_ENGINE.md`

---

**Last Updated**: 2026-01-31
**Maintainer**: voidnx team + ML team
**Severity**: CRITICAL - Core functionality failure
