# Runbook: NeolandUnauthorizedAccess

**Alert**: `NeolandUnauthorizedAccess`
**Severity**: CRITICAL 🔴 (Security)
**Response Time**: IMMEDIATE (0-5 minutes)

---

## Symptoms

- **Alert**: High rate of 401/403 (Unauthorized/Forbidden) responses
- **User Impact**: Potential security breach, unauthorized access attempts
- **Metrics**: `rate(neoland_http_requests_total{status=~"401|403"}[5m]) > 20` (>20 unauthorized attempts per second)
- **Common Causes**:
  - Credential stuffing attack
  - Stolen API keys
  - Misconfigured client (wrong API key)
  - Authorization bypass attempt
  - Expired tokens

---

## Investigation

```bash
# Check recent 401/403 responses
kubectl logs -l app=neoland --tail=1000 -n default \
  | jq 'select(.status == 401 or .status == 403)' \
  | jq -r '. | "\(.timestamp) \(.source_ip) \(.user_id) \(.endpoint)"' \
  | sort | uniq -c | sort -rn | head -20

# Get top attacking IPs
kubectl logs -l app=neoland --tail=1000 -n default \
  | jq 'select(.status == 401)' \
  | jq -r '.source_ip' \
  | sort | uniq -c | sort -rn | head -10

# Check for API key enumeration attempts
kubectl logs -l app=neoland --tail=1000 | grep "invalid_api_key" | wc -l
```

## Resolution

### Scenario 1: API Key Enumeration Attack

```bash
# Block attacking IP at load balancer
ATTACKING_IP=$(kubectl logs -l app=neoland --tail=1000 | jq -r 'select(.status == 401) | .source_ip' | sort | uniq -c | sort -rn | head -1 | awk '{print $2}')

kubectl annotate ingress neoland \
  nginx.ingress.kubernetes.io/whitelist-source-range="0.0.0.0/0,!${ATTACKING_IP}/32" \
  -n default

# Rotate compromised API keys (if any successful)
# Notify security team immediately
```

### Scenario 2: Compromised API Key

```bash
# Identify compromised key
kubectl logs -l app=neoland --tail=2000 | jq 'select(.status == 200 and .source_ip == "<suspicious-ip>") | .api_key_id'

# Revoke key immediately
curl -X DELETE http://neoland:3001/admin/api-keys/<key-id> \
  -H "X-Admin-Key: $ADMIN_KEY"

# Notify key owner
# Force password reset if linked to user account
```

## Verification

```bash
# Check 401 rate dropped
curl -s 'http://prometheus:9090/api/v1/query?query=rate(neoland_http_requests_total{status="401"}[5m])' | jq

# Expected: <1 per second

# Verify blocked IPs cannot access
curl -I http://neoland.example.com -H "X-Forwarded-For: $BLOCKED_IP"
# Expected: 403 Forbidden
```

---

⚠️ **SECURITY INCIDENT** - Follow incident response procedures

**Escalation**:
- Slack: `#security-alerts` (IMMEDIATE)
- Security Team: security@neoland.example.com
- Create incident ticket

**Last Updated**: 2026-01-31
**Maintainer**: voidnx team + Security team
