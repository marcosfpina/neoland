# Runbook: NeolandBruteForceAttack

**Alert**: `NeolandBruteForceAttack`
**Severity**: CRITICAL 🔴 (Security Incident)
**Response Time**: IMMEDIATE (0-2 minutes)

---

## Symptoms

- **Alert**: NeolandBruteForceAttack firing in AlertManager
- **User Impact**: Potential security breach in progress
- **Metrics**: `rate(neoland_auth_failures_total[1m]) > 20` (>20 failed auth attempts per second)
- **Security**: Active attack attempting to guess credentials

⚠️ **THIS IS A SECURITY INCIDENT** - Follow incident response procedures

---

## Investigation

### Step 1: Identify Attack Source (IMMEDIATE)

```bash
# Check recent failed auth attempts in audit logs
kubectl logs -l app=neoland --tail=500 -n default \
  | grep "auth_failure" \
  | jq -r '. | "\(.timestamp) \(.source_ip) \(.user_id)"' \
  | sort | uniq -c | sort -rn | head -20

# Get top attacking IPs
kubectl logs -l app=neoland --tail=1000 -n default \
  | grep "auth_failure" \
  | jq -r '.source_ip' \
  | sort | uniq -c | sort -rn | head -10

# Check if attack is distributed (many IPs) or single source
```

**Expected output**:
```
    245 203.0.113.42
     38 198.51.100.23
     12 192.0.2.15
```

If top IP has >100 attempts in last minute → **Single-source attack**
If top 10 IPs all have <50 attempts → **Distributed attack (botnet)**

### Step 2: Check Attack Pattern

```bash
# Analyze attack pattern
kubectl logs -l app=neoland --tail=1000 -n default \
  | grep "auth_failure" \
  | jq -r '. | "\(.user_id) \(.source_ip)"' \
  | sort | uniq -c | sort -rn | head -20

# Check if targeting specific users or random guessing
kubectl logs -l app=neoland --tail=1000 -n default \
  | grep "auth_failure" \
  | jq -r '.user_id' \
  | sort | uniq -c | sort -rn | head -10
```

**Attack patterns**:
- **Credential Stuffing**: Same IP, many different usernames
- **Password Spraying**: Many IPs, common passwords, few usernames
- **Targeted Attack**: Single username, many password attempts

### Step 3: Check Current Impact

```bash
# Check if any attempts succeeded
kubectl logs -l app=neoland --tail=1000 -n default \
  | jq 'select(.action == "AuthSuccess")' \
  | jq -r '. | "\(.timestamp) \(.user_id) \(.source_ip)"'

# If any successes from attacking IP, CRITICAL ESCALATION
```

⚠️ **If attacker succeeded in authentication, ESCALATE IMMEDIATELY**

### Step 4: Verify Alert is Not False Positive

```bash
# Check rate limit metrics
curl -s 'http://prometheus:9090/api/v1/query?query=rate(neoland_auth_failures_total[1m])' \
  | jq '.data.result[0].value[1]'

# If < 20, alert may be flapping, continue monitoring
# If > 20, proceed with blocking
```

---

## Resolution

### IMMEDIATE ACTION (0-2 minutes): Block Attack Source

#### Option 1: Block at Application Level (Fastest)

```bash
# 1. Get attacking IP(s)
ATTACKING_IPS=$(kubectl logs -l app=neoland --tail=1000 -n default \
  | grep "auth_failure" \
  | jq -r '.source_ip' \
  | sort | uniq -c | sort -rn | head -5 | awk '{print $2}')

# 2. Add to blocklist (requires application support)
# (If implemented, send command to block IPs)
# Example:
# curl -X POST http://neoland:3001/admin/blocklist \
#   -H "X-API-Key: $ADMIN_KEY" \
#   -d '{"ips": ["203.0.113.42", "198.51.100.23"]}'
```

**Timeframe**: 30 seconds

#### Option 2: Block at Load Balancer / Ingress (Recommended)

```bash
# For Kubernetes Ingress (NGINX)
kubectl annotate ingress neoland \
  nginx.ingress.kubernetes.io/whitelist-source-range="0.0.0.0/0,!203.0.113.42/32" \
  -n default

# For AWS ALB
aws wafv2 update-ip-set \
  --name neoland-blocklist \
  --id <ip-set-id> \
  --addresses 203.0.113.42/32 198.51.100.23/32

# For Cloudflare (if using)
curl -X POST "https://api.cloudflare.com/client/v4/zones/{zone_id}/firewall/access_rules/rules" \
  -H "X-Auth-Email: your-email@example.com" \
  -H "X-Auth-Key: your-api-key" \
  -d '{"mode":"block","configuration":{"target":"ip","value":"203.0.113.42"}}'
```

**Timeframe**: 1-2 minutes

#### Option 3: Block at Firewall (Network Level)

```bash
# For iptables (on node)
sudo iptables -I INPUT -s 203.0.113.42 -j DROP

# For cloud provider firewall
# AWS Security Group: Add deny rule for IP
# GCP Firewall: Add deny rule for IP
# Azure NSG: Add deny rule for IP
```

**Timeframe**: 2-5 minutes

### FOLLOW-UP ACTIONS (2-10 minutes)

#### Action 1: Enable Enhanced Rate Limiting

```bash
# 1. Update rate limit to be more aggressive temporarily
kubectl set env deployment/neoland \
  RATE_LIMIT_AUTH="10/min" \
  -n default

# Default is 100/min, reduce to 10/min during attack

# 2. Monitor recovery
kubectl rollout status deployment/neoland -n default
```

#### Action 2: Collect Evidence

```bash
# 1. Extract full audit log for investigation
kubectl logs -l app=neoland --since=30m -n default \
  | grep "auth_failure" \
  > /tmp/attack-evidence-$(date +%Y%m%d-%H%M%S).json

# 2. Analyze attack timeline
jq -r '"\(.timestamp) \(.source_ip) \(.user_id)"' /tmp/attack-evidence-*.json \
  | sort | head -100

# 3. Send to security team for analysis
# (Upload to incident response S3 bucket or similar)
```

#### Action 3: Check for Compromised Accounts

```bash
# 1. Check if any accounts were successfully accessed
kubectl logs -l app=neoland --since=30m -n default \
  | jq 'select(.action == "AuthSuccess")' \
  | jq -r '. | "\(.timestamp) \(.user_id) \(.source_ip)"' \
  > /tmp/successful-auths.txt

# 2. Cross-reference with attacking IPs
grep -Ff <(echo "$ATTACKING_IPS") /tmp/successful-auths.txt

# 3. If any matches, FORCE PASSWORD RESET for those accounts
# (See Account Compromise runbook)
```

#### Action 4: Notify Security Team

```bash
# 1. Post to security Slack channel
# Slack: #security-alerts
# Message:
# 🚨 BRUTE FORCE ATTACK DETECTED
# - Time: [timestamp]
# - Source IPs: [top 5 IPs]
# - Attempts: [number]
# - Targeted users: [usernames]
# - Successful logins: [YES/NO]
# - Action taken: [blocked IPs, enhanced rate limiting]
# - Evidence: [link to logs]

# 2. Create security incident ticket
# Title: [SECURITY] Brute force attack on YYYY-MM-DD HH:MM
# Priority: P1 (Critical)
# Assignee: Security team
```

---

## Verification

### Verify Attack Stopped

```bash
# 1. Check auth failure rate dropped
curl -s 'http://prometheus:9090/api/v1/query?query=rate(neoland_auth_failures_total[1m])' \
  | jq '.data.result[0].value[1]'
# Expected: < 5 (back to normal)

# 2. Verify no new failures from blocked IPs
kubectl logs -l app=neoland --tail=100 -n default \
  | grep "auth_failure" \
  | jq -r '.source_ip' \
  | grep -E '203.0.113.42|198.51.100.23'
# Expected: No output (IPs blocked)

# 3. Check alert cleared
# http://alertmanager:9093/#/alerts
# Expected: NeolandBruteForceAttack resolved

# 4. Monitor for 10 minutes to ensure attack stopped
watch 'curl -s http://prometheus:9090/api/v1/query?query=rate(neoland_auth_failures_total[1m]) | jq'
```

---

## Escalation

### IMMEDIATE Escalation (Security Incident)

⚠️ **ALL brute force attacks are security incidents and MUST be escalated**

- **Slack**: `#security-alerts` (IMMEDIATE)
- **PagerDuty**: Automatic critical alert
- **Security Team**: Notify security@neoland.example.com (IMMEDIATE)
- **Contacts**:
  - Primary: Security On-Call (PagerDuty)
  - Secondary: Security Team Lead
  - CISO: (if compromised accounts found)

### Escalation Criteria

| Severity | Criteria | Escalation |
|----------|----------|------------|
| **P1 (Critical)** | Successful auth from attacking IP | CISO + Legal |
| **P2 (High)** | >1000 attempts, no success | Security Team Lead |
| **P3 (Medium)** | <1000 attempts, blocked | Security On-Call |

---

## Post-Incident

### MANDATORY Actions (within 1 hour)

1. **Security Incident Report**:
   ```
   Create: docs/incidents/YYYY-MM-DD-brute-force-attack.md
   Include:
   - Attack timeline
   - Source IPs and geolocation
   - Number of attempts
   - Targeted accounts
   - Successful compromises (if any)
   - Mitigation actions taken
   - Evidence files
   ```

2. **Threat Intelligence**:
   - Submit attacking IPs to AbuseIPDB
   - Check if IPs are known threat actors
   - Update threat intelligence database

3. **Account Review**:
   - Review all accounts that were targeted
   - Force MFA enrollment for targeted accounts
   - Consider password reset for high-value targets

### Follow-up Actions (within 24 hours)

1. **Improve Detection**:
   - Review if alert threshold needs tuning
   - Add geolocation-based anomaly detection
   - Implement behavioral analysis

2. **Improve Prevention**:
   - Implement CAPTCHA after N failed attempts
   - Add MFA requirement for all users
   - Implement account lockout policy (5 failed attempts = 15 min lockout)

3. **Improve Response**:
   - Automate IP blocking (auto-block after threshold)
   - Integrate with cloud WAF for faster blocking
   - Set up honeypot accounts to detect attacks earlier

4. **Post-Mortem** (within 48 hours):
   - Security team review
   - Identify gaps in defenses
   - Create action items for hardening

---

## Prevention

### Short-term (Immediate)

- ✅ Rate limiting enforced (100 req/min per IP)
- ✅ Brute force detection alert (>20 failures/sec)
- ✅ Audit logging of all auth events
- ⚠️ **TODO**: Auto-blocking after threshold
- ⚠️ **TODO**: CAPTCHA on failed auth

### Medium-term (1-2 weeks)

- [ ] Implement MFA for all accounts
- [ ] Add geolocation anomaly detection
- [ ] Integrate with threat intelligence feeds
- [ ] Deploy Web Application Firewall (WAF)

### Long-term (1-3 months)

- [ ] Behavioral analysis (ML-based)
- [ ] Passwordless authentication (WebAuthn)
- [ ] Zero-trust architecture
- [ ] Honeypot accounts for early detection

---

## Related Alerts

- `NeolandHighAuthFailureRate`: Lower threshold (>5 failures/sec) - may fire first
- `NeolandSuspiciousActivity`: General suspicious behavior detection
- `NeolandRateLimitExceeded`: May fire during attack

---

## Legal & Compliance

⚠️ **If attack results in data breach, follow breach notification procedures**:

- **GDPR**: Notify DPA within 72 hours
- **CCPA**: Notify affected users
- **SOC 2**: Document in compliance log
- **ISO 27001**: Incident response procedure

---

## Additional Resources

- **Audit Logs**: `kubectl logs -l app=neoland -n default | grep auth_failure`
- **Security Dashboard**: [Grafana Security](http://grafana/d/neoland-security)
- **IP Geolocation**: https://ipinfo.io/{ip}
- **Threat Intelligence**: https://www.abuseipdb.com/check/{ip}
- **Incident Response Plan**: `docs/SECURITY_INCIDENT_RESPONSE.md`

---

**Last Updated**: 2026-01-31
**Maintainer**: voidnx team + Security team
**Severity**: CRITICAL - Security Incident
**Compliance**: SOC 2, GDPR, ISO 27001
