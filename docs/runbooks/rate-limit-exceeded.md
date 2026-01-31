# Runbook: NeolandRateLimitExceeded

**Alert**: `NeolandRateLimitExceeded`
**Severity**: WARNING ⚠️
**Response Time**: 15-30 minutes

---

## Symptoms

- **Alert**: High rate of 429 (Too Many Requests) responses
- **User Impact**: Legitimate users getting rate limited, API requests rejected
- **Metrics**: `rate(neoland_http_requests_total{status="429"}[5m]) > 10` (>10 rate limit hits per second)
- **Common Causes**:
  - Legitimate traffic spike (viral event, marketing campaign)
  - Single user/IP abusing API
  - Bot/scraper activity
  - Misconfigured client (retry loop)
  - Rate limit thresholds too aggressive

---

## Investigation

### Step 1: Identify Rate-Limited Users/IPs

```bash
# Check recent 429 responses in logs
kubectl logs -l app=neoland --tail=1000 -n default \
  | jq 'select(.status == 429)' \
  | jq -r '. | "\(.timestamp) \(.source_ip) \(.user_id) \(.endpoint)"' \
  | sort | uniq -c | sort -rn | head -20

# Get top rate-limited IPs
kubectl logs -l app=neoland --tail=1000 -n default \
  | jq 'select(.status == 429)' \
  | jq -r '.source_ip' \
  | sort | uniq -c | sort -rn | head -10

# Expected output:
#  450 203.0.113.42
#   38 198.51.100.23
#   12 192.0.2.15

# If single IP dominates (>80% of 429s) → Single abuser
# If many IPs evenly distributed → Global traffic spike
```

### Step 2: Check Rate Limit Configuration

```bash
# Check current rate limits
kubectl get configmap neoland-config -n default -o yaml | grep -A 5 "rate_limit"

# Expected:
# RATE_LIMIT_GLOBAL: "1000/min"  # 1000 requests per minute globally
# RATE_LIMIT_PER_IP: "100/min"   # 100 requests per minute per IP
# RATE_LIMIT_PER_USER: "500/min" # 500 requests per minute per authenticated user

# Check metrics for rate limit hits
curl http://neoland:3001/metrics | grep neoland_rate_limit
```

### Step 3: Analyze Traffic Patterns

```bash
# Check request rate over time
curl -s 'http://prometheus:9090/api/v1/query?query=rate(neoland_http_requests_total[5m])' | jq

# Check which endpoints are being rate limited
kubectl logs -l app=neoland --tail=1000 | jq 'select(.status == 429) | .endpoint' | sort | uniq -c | sort -rn

# Check if requests are legitimate or bot-like
kubectl logs -l app=neoland --tail=500 | jq 'select(.status == 429) | .user_agent' | sort | uniq -c

# Bot indicators:
# - Generic user agents (curl, python-requests, bot)
# - Very high request rate (>10/sec from single IP)
# - Uniform intervals (exact timing, no human variance)
```

### Step 4: Check for Retry Loops

```bash
# Check if same requests being retried immediately
kubectl logs -l app=neoland --tail=500 -n default \
  | jq 'select(.status == 429)' \
  | jq -r '. | "\(.source_ip) \(.endpoint) \(.timestamp)"' \
  | sort

# If same IP + endpoint every second → Retry loop (misconfigured client)
```

---

## Resolution

### Scenario 1: Legitimate Traffic Spike

**Symptoms**: Many different IPs, legitimate user agents, all getting rate limited

```bash
# 1. Temporarily increase rate limits
kubectl set env deployment/neoland \
  RATE_LIMIT_GLOBAL="2000/min" \
  RATE_LIMIT_PER_IP="200/min" \
  RATE_LIMIT_PER_USER="1000/min" \
  -n default

# Doubles all rate limits

# 2. Monitor rollout
kubectl rollout status deployment/neoland -n default

# 3. Scale horizontally to handle load
kubectl scale deployment/neoland --replicas=5 -n default

# 4. Monitor 429 rate drops
watch 'curl -s http://prometheus:9090/api/v1/query?query=rate(neoland_http_requests_total{status="429"}[1m]) | jq'

# 5. After traffic spike ends (1-2 hours), restore original limits
kubectl set env deployment/neoland \
  RATE_LIMIT_GLOBAL="1000/min" \
  RATE_LIMIT_PER_IP="100/min" \
  RATE_LIMIT_PER_USER="500/min" \
  -n default
```

**Timeframe**: 3-5 minutes

### Scenario 2: Single Abusive IP/User

**Symptoms**: One IP has >80% of 429s, bot-like user agent, very high request rate

```bash
# 1. Identify abusive IP
ABUSIVE_IP=$(kubectl logs -l app=neoland --tail=1000 | jq -r 'select(.status == 429) | .source_ip' | sort | uniq -c | sort -rn | head -1 | awk '{print $2}')

echo "Abusive IP: $ABUSIVE_IP"

# 2. Block IP at load balancer (NGINX Ingress)
kubectl annotate ingress neoland \
  nginx.ingress.kubernetes.io/whitelist-source-range="0.0.0.0/0,!${ABUSIVE_IP}/32" \
  -n default

# 3. Or add to application blocklist (if implemented)
curl -X POST http://neoland:3001/admin/blocklist \
  -H "X-API-Key: $ADMIN_KEY" \
  -d "{\"ip\": \"${ABUSIVE_IP}\", \"reason\": \"Rate limit abuse\", \"duration\": \"24h\"}"

# 4. Verify 429s drop
watch 'kubectl logs -l app=neoland --tail=100 | jq "select(.status == 429)" | wc -l'

# Expected: Significant drop in 429s

# 5. Contact user (if authenticated) to explain rate limits
```

**Timeframe**: 2-3 minutes

### Scenario 3: Bot/Scraper Activity

**Symptoms**: Bot user agents, systematic endpoint crawling, no authentication

```bash
# 1. Identify bot user agent
kubectl logs -l app=neoland --tail=1000 | jq 'select(.status == 429) | .user_agent' | sort | uniq -c | sort -rn | head -5

# Common bots:
# - python-requests
# - curl/7.x
# - Go-http-client
# - scrapy

# 2. Add stricter rate limits for unauthenticated requests
kubectl set env deployment/neoland \
  RATE_LIMIT_UNAUTHENTICATED="20/min" \
  -n default

# 3. Implement CAPTCHA for repeated 429s (requires code change)
# (Add to roadmap: Phase 1 Security)

# 4. Add robots.txt to discourage scrapers
cat > robots.txt <<EOF
User-agent: *
Disallow: /v1/
Crawl-delay: 10
EOF

# 5. Monitor for compliance
```

**Timeframe**: 5-10 minutes

### Scenario 4: Misconfigured Client (Retry Loop)

**Symptoms**: Same IP, same endpoint, rapid retries (no backoff), 429 responses continuous

```bash
# 1. Identify client with retry loop
kubectl logs -l app=neoland --tail=500 | jq 'select(.status == 429)' \
  | jq -r '. | "\(.source_ip) \(.user_id)"' \
  | sort | uniq -c | sort -rn | head -5

# 2. Contact user directly (if known)
# Email: "Your client is in a retry loop. Please implement exponential backoff."

# 3. Temporarily block IP to force client to stop
kubectl annotate ingress neoland \
  nginx.ingress.kubernetes.io/whitelist-source-range="0.0.0.0/0,!${CLIENT_IP}/32" \
  -n default

# 4. After client fixes issue, unblock
kubectl annotate ingress neoland \
  nginx.ingress.kubernetes.io/whitelist-source-range="0.0.0.0/0" \
  -n default

# 5. Add Retry-After header to 429 responses (if not already)
# (Requires code change: add "Retry-After: 60" header)
```

**Timeframe**: 5-15 minutes (depends on user contact)

### Scenario 5: Rate Limits Too Aggressive

**Symptoms**: Many legitimate users complaining, 429s during normal usage, no abuse detected

```bash
# 1. Analyze typical user request patterns
kubectl logs -l app=neoland --tail=5000 | jq 'select(.user_id != null) | .user_id' \
  | sort | uniq -c | sort -rn | head -20

# Calculate average requests per user per minute

# 2. Review rate limit thresholds
# Current: 100 req/min per IP, 500 req/min per user
# Recommendation: Set to 95th percentile + 20% buffer

# 3. Increase limits based on analysis
kubectl set env deployment/neoland \
  RATE_LIMIT_PER_IP="150/min" \
  RATE_LIMIT_PER_USER="750/min" \
  -n default

# 4. Document decision in ADR
# ADR-021: "Rate Limiting Strategy and Thresholds"

# 5. Monitor for balance between abuse prevention and UX
```

**Timeframe**: 10-30 minutes (analysis + tuning)

---

## Verification

### Verify Rate Limits Effective

```bash
# 1. Check 429 rate dropped
curl -s 'http://prometheus:9090/api/v1/query?query=rate(neoland_http_requests_total{status="429"}[5m])' | jq

# Expected: <1 per second (was >10)

# 2. Verify legitimate users not rate limited
kubectl logs -l app=neoland --tail=200 | jq 'select(.status == 429 and .user_id != null)'

# Expected: Very few or none

# 3. Check alert cleared
# http://alertmanager:9093/#/alerts
# Expected: NeolandRateLimitExceeded resolved

# 4. Test API as regular user
for i in {1..100}; do
  curl -X POST http://neoland.example.com/v1/chat/completions \
    -H "X-API-Key: $API_KEY" \
    -d '{"messages":[{"role":"user","content":"test"}]}' &
done
wait

# Expected: All 200 OK (within rate limit)

for i in {1..200}; do
  curl -X POST http://neoland.example.com/v1/chat/completions \
    -H "X-API-Key: $API_KEY" \
    -d '{"messages":[{"role":"user","content":"test"}]}' &
done
wait

# Expected: Some 429s after exceeding limit
```

---

## Escalation

| Severity | Response | Escalation |
|----------|----------|------------|
| >10 rate limit hits/sec | 30 min | #neoland-warnings |
| >50 rate limit hits/sec | 15 min | #neoland-critical |
| Legitimate users blocked | Immediate | @oncall-voidnx |

**Contacts**:
- Primary: `@oncall-voidnx`
- Product: (if legitimate traffic spike from marketing campaign)
- Security: (if DDoS suspected)

---

## Post-Incident

### Immediate Actions (within 1 hour)

1. **Identify Root Cause**:
   - Traffic spike? → Improve autoscaling
   - Abusive user? → Document and block
   - Misconfigured client? → Contact user, provide docs
   - Limits too low? → Adjust based on data

2. **Document Decision**:
   ```
   Title: [INCIDENT] Rate limiting on YYYY-MM-DD
   - Rate limit hit rate: X per second
   - Affected users: Y
   - Cause: <description>
   - Mitigation: <what was done>
   - Outcome: <result>
   ```

### Follow-Up Actions (within 24 hours)

1. **Improve Rate Limiting Strategy**:
   - Implement tiered rate limits (free vs paid users)
   - Add burst allowance (100/min average, 150/min burst)
   - Implement rate limit by endpoint (stricter on expensive endpoints)
   - Add rate limit headers to all responses:
     ```
     X-RateLimit-Limit: 100
     X-RateLimit-Remaining: 87
     X-RateLimit-Reset: 1643723400
     ```

2. **Add Rate Limit Monitoring**:
   - Alert on >10% of requests getting 429
   - Track rate limit hits per user/IP
   - Monitor for retry storms

3. **Improve Client Documentation**:
   - Document rate limits clearly in API docs
   - Provide exponential backoff examples
   - Add rate limit headers documentation

4. **Create ADR-021**: "Rate Limiting Strategy and Thresholds"
   - Document rate limit values
   - Explain rationale (DDoS protection vs UX)
   - Define escalation process for limit increases

---

## Prevention

### Short-term (Immediate)

- ✅ Rate limiting enforced (100 req/min per IP, 500 per user)
- ✅ Alert on high 429 rate (>10/sec)
- ⚠️ **TODO**: Add rate limit headers to responses
- ⚠️ **TODO**: Implement IP blocklist

### Medium-term (1-2 weeks)

- [ ] Tiered rate limits (free: 100/min, paid: 1000/min)
- [ ] Burst allowance (short-term spike handling)
- [ ] CAPTCHA for repeated 429s (bot protection)
- [ ] Rate limit by endpoint (expensive endpoints stricter)
- [ ] Better client documentation with examples

### Long-term (1-3 months)

- [ ] Dynamic rate limiting based on load
- [ ] Machine learning for abuse detection
- [ ] API key usage analytics dashboard
- [ ] Self-service rate limit increase requests
- [ ] CDN integration for static responses

---

## Related Alerts

- `NeolandBruteForceAttack`: May fire alongside (auth endpoint rate limited)
- `NeolandHighLatency`: Rate limiting adds latency to rejected requests
- `NeolandHighCPUUsage`: Traffic spike may cause both

---

## Additional Resources

- **Rate Limiting Implementation**: `src/server/mod.rs` (tower-http RateLimitLayer)
- **API Documentation**: `docs/API.md` - Rate limit section
- **ADR-021**: "Rate Limiting Strategy" (TODO)
- **Client Best Practices**: `docs/CLIENT_BEST_PRACTICES.md` (TODO)

---

**Last Updated**: 2026-01-31
**Maintainer**: voidnx team
**Severity**: WARNING - User experience degradation
