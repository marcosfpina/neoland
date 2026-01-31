# Runbook: NeolandTLSCertExpiringSoon / NeolandTLSCertExpired

**Alert**: `NeolandTLSCertExpiringSoon` (warning) / `NeolandTLSCertExpired` (critical)
**Severity**: WARNING ⚠️ / CRITICAL 🔴
**Response Time**: 1-4 hours (warning) / IMMEDIATE (critical)

---

## Symptoms

- **Alert**: TLS certificate expiring soon or already expired
- **User Impact**:
  - Warning (7-30 days): No impact yet, action needed
  - Critical (<7 days): Browser warnings imminent
  - Expired: Service unavailable, browser errors, clients cannot connect
- **Metrics**:
  - Warning: `(cert_expirydate_timestamp_seconds - time()) < 7 * 24 * 3600` (<7 days)
  - Critical: `(cert_expirydate_timestamp_seconds - time()) < 24 * 3600` (<1 day)
  - Expired: `(cert_expirydate_timestamp_seconds - time()) < 0`
- **Common Causes**:
  - cert-manager renewal failure
  - Let's Encrypt rate limit exceeded
  - DNS validation failure
  - Manual certificate not renewed

---

## Investigation

### Step 1: Check Certificate Expiry

```bash
# Check certificate expiry for all NEOLAND services
kubectl get certificates -n default

# Expected output:
# NAME              READY   SECRET            AGE
# neoland-tls       True    neoland-tls       89d
# neoland-grpc-tls  True    neoland-grpc-tls  89d

# If READY = False → Certificate issue

# Check certificate details
kubectl describe certificate neoland-tls -n default

# Check actual certificate expiry
kubectl get secret neoland-tls -n default -o json \
  | jq -r '.data."tls.crt"' \
  | base64 -d \
  | openssl x509 -noout -dates

# Output shows:
# notBefore=Jan  1 00:00:00 2026 GMT
# notAfter=Mar 31 23:59:59 2026 GMT

# Calculate days until expiry
kubectl get secret neoland-tls -n default -o json \
  | jq -r '.data."tls.crt"' \
  | base64 -d \
  | openssl x509 -noout -enddate \
  | sed 's/notAfter=//' \
  | xargs -I {} date -d {} +%s \
  | awk '{print int(($1 - systime()) / 86400) " days until expiry"}'
```

### Step 2: Check cert-manager Status

```bash
# Check if cert-manager is running
kubectl get pods -n cert-manager

# Expected:
# cert-manager-xxxxx         1/1  Running
# cert-manager-cainjector-xxx 1/1  Running
# cert-manager-webhook-xxx    1/1  Running

# Check cert-manager logs for errors
kubectl logs -n cert-manager -l app=cert-manager --tail=200 | grep -i "error\|fail"

# Check certificate requests
kubectl get certificaterequest -n default

# If CertificateRequest stuck → Renewal issue
```

### Step 3: Check Let's Encrypt Rate Limits

```bash
# Check cert-manager ClusterIssuer status
kubectl describe clusterissuer letsencrypt-prod

# Look for rate limit errors:
# - "too many certificates already issued"
# - "rate limit exceeded"

# Let's Encrypt rate limits:
# - 50 certificates per domain per week
# - 5 duplicate certificates per week
# - 300 pending authorizations per account

# Check recent certificate issuance
kubectl get certificaterequest -n default --sort-by=.metadata.creationTimestamp
```

### Step 4: Check DNS Validation

```bash
# Check if DNS is configured for ACME challenge
# (For Let's Encrypt HTTP-01 or DNS-01 challenge)

# Get challenge domain
kubectl describe certificaterequest <name> -n default | grep "DNS Names"

# Test DNS resolution
nslookup neoland.example.com

# Test ACME challenge endpoint (HTTP-01)
curl http://neoland.example.com/.well-known/acme-challenge/test

# Expected: 404 (endpoint exists) or challenge response

# Check ingress for ACME challenge route
kubectl get ingress -n default -o yaml | grep -A 5 "acme-challenge"
```

---

## Resolution

### Scenario 1: Certificate Expiring Soon (7-30 days) - Proactive Renewal

**Symptoms**: Warning alert, certificate valid but expiring soon, READY = True

```bash
# 1. Force cert-manager to renew certificate early
kubectl annotate certificate neoland-tls \
  cert-manager.io/issue-temporary-certificate="true" \
  -n default

# This triggers immediate renewal

# 2. Monitor certificate renewal
kubectl get certificaterequest -n default --watch

# Wait for new CertificateRequest to be "Ready"

# 3. Verify new certificate issued
kubectl get secret neoland-tls -n default -o json \
  | jq -r '.data."tls.crt"' \
  | base64 -d \
  | openssl x509 -noout -dates

# Expected: New notAfter date (+90 days for Let's Encrypt)

# 4. Restart pods to use new certificate
kubectl rollout restart deployment/neoland -n default

# 5. Verify TLS working
curl -vvv https://neoland.example.com 2>&1 | grep "expire date"
```

**Timeframe**: 5-15 minutes (depends on Let's Encrypt validation)

### Scenario 2: Certificate Expired (<1 day) - Emergency Renewal

**Symptoms**: Critical alert, certificate expires very soon, service still working

```bash
# ⚠️ URGENT: Certificate about to expire

# 1. Check if automatic renewal failed
kubectl describe certificate neoland-tls -n default | grep -A 10 "Events:"

# Common errors:
# - "Failed to determine Issuer" → ClusterIssuer issue
# - "Failed to create Order" → Let's Encrypt API issue
# - "Waiting for DNS propagation" → DNS issue

# 2. Delete and recreate certificate (forces renewal)
kubectl delete certificaterequest --all -n default
kubectl delete certificate neoland-tls -n default
kubectl apply -f k8s/certificates.yaml

# 3. Monitor recreation
kubectl get certificate neoland-tls -n default --watch

# 4. If still failing, use staging issuer temporarily
kubectl patch certificate neoland-tls -n default --type='json' -p='[
  {
    "op": "replace",
    "path": "/spec/issuerRef/name",
    "value": "letsencrypt-staging"
  }
]'

# Staging has higher rate limits (not trusted by browsers, but temporary)

# 5. Verify renewal
kubectl get secret neoland-tls -n default -o json \
  | jq -r '.data."tls.crt"' \
  | base64 -d \
  | openssl x509 -noout -enddate
```

**Timeframe**: 10-30 minutes

### Scenario 3: Certificate Already Expired - EMERGENCY

**Symptoms**: CRITICAL, service unavailable, browser shows "NET::ERR_CERT_DATE_INVALID"

```bash
# 🚨 EMERGENCY: Service DOWN due to expired certificate

# OPTION A: Use self-signed certificate temporarily (quick fix)

# 1. Generate self-signed certificate (valid 365 days)
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout /tmp/tls.key -out /tmp/tls.crt \
  -subj "/CN=neoland.example.com/O=NEOLAND"

# 2. Create Kubernetes secret
kubectl create secret tls neoland-tls-emergency \
  --cert=/tmp/tls.crt \
  --key=/tmp/tls.key \
  -n default

# 3. Update ingress to use emergency certificate
kubectl patch ingress neoland -n default --type='json' -p='[
  {
    "op": "replace",
    "path": "/spec/tls/0/secretName",
    "value": "neoland-tls-emergency"
  }
]'

# ⚠️ WARNING: Self-signed cert causes browser warnings, but service is accessible

# 4. Notify users via status page
# "We are experiencing TLS certificate issues. Service may show security warnings but is safe to use."

# 5. Fix cert-manager in parallel (see Scenario 2)

# OPTION B: Disable TLS temporarily (NOT RECOMMENDED - security risk)
# Only if self-signed also failing

kubectl patch ingress neoland -n default --type='json' -p='[
  {
    "op": "remove",
    "path": "/spec/tls"
  }
]'

# Service now accessible via HTTP only (http://neoland.example.com)
```

**Timeframe**: 5-10 minutes (self-signed), 10-30 minutes (proper renewal)

### Scenario 4: cert-manager Broken - Manual Certificate

**Symptoms**: cert-manager not working, cannot auto-renew, need manual certificate

```bash
# If cert-manager completely broken, use manual certificate

# 1. Obtain certificate manually from Let's Encrypt
# (Using certbot on local machine or separate server)

certbot certonly --manual \
  --preferred-challenges dns \
  -d neoland.example.com \
  -d *.neoland.example.com

# Follow DNS challenge instructions

# 2. Copy certificates to Kubernetes
kubectl create secret tls neoland-tls-manual \
  --cert=/etc/letsencrypt/live/neoland.example.com/fullchain.pem \
  --key=/etc/letsencrypt/live/neoland.example.com/privkey.pem \
  -n default

# 3. Update ingress
kubectl patch ingress neoland -n default --type='json' -p='[
  {
    "op": "replace",
    "path": "/spec/tls/0/secretName",
    "value": "neoland-tls-manual"
  }
]'

# 4. Set calendar reminder to renew in 60 days
# ⚠️ Manual certificates are NOT auto-renewed

# 5. Fix cert-manager for future renewals
kubectl logs -n cert-manager -l app=cert-manager --tail=500
```

**Timeframe**: 30-60 minutes (manual process)

### Scenario 5: Let's Encrypt Rate Limited

**Symptoms**: cert-manager errors mention "rate limit exceeded", cannot issue new certificate

```bash
# Let's Encrypt rate limits:
# - 50 certificates per domain per week
# - 5 duplicate certificates per week

# 1. Check rate limit status
# https://crt.sh/?q=neoland.example.com
# Shows all recent certificate issuances

# 2. If rate limited, use alternative ACME provider temporarily
# (ZeroSSL, BuyPass Go SSL)

# Create new ClusterIssuer for ZeroSSL
cat <<EOF | kubectl apply -f -
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: zerossl-prod
spec:
  acme:
    server: https://acme.zerossl.com/v2/DV90
    email: admin@neoland.example.com
    privateKeySecretRef:
      name: zerossl-account-key
    solvers:
    - http01:
        ingress:
          class: nginx
EOF

# 3. Update certificate to use ZeroSSL
kubectl patch certificate neoland-tls -n default --type='json' -p='[
  {
    "op": "replace",
    "path": "/spec/issuerRef/name",
    "value": "zerossl-prod"
  }
]'

# 4. Monitor renewal
kubectl get certificate neoland-tls -n default --watch

# 5. After rate limit expires (1 week), switch back to Let's Encrypt
```

**Timeframe**: 15-30 minutes

---

## Verification

### Verify Certificate Valid

```bash
# 1. Check certificate expiry date
kubectl get secret neoland-tls -n default -o json \
  | jq -r '.data."tls.crt"' \
  | base64 -d \
  | openssl x509 -noout -dates

# Expected: notAfter = 60+ days from now

# 2. Test HTTPS endpoint
curl -vvv https://neoland.example.com 2>&1 | grep -A 5 "SSL certificate"

# Expected: "SSL certificate verify ok"

# 3. Check in browser
# Open: https://neoland.example.com
# Click padlock icon → Certificate info
# Expected: Valid, trusted, 60+ days remaining

# 4. Verify cert-manager tracking
kubectl get certificate neoland-tls -n default

# Expected: READY = True

# 5. Check alert cleared
# http://alertmanager:9093/#/alerts
# Expected: NeolandTLSCertExpiringSoon resolved
```

### Monitor Certificate Renewal

```bash
# Set up continuous monitoring
watch 'kubectl get certificates -n default'

# Expected: All certificates READY = True

# Monitor cert-manager logs
kubectl logs -n cert-manager -l app=cert-manager --follow | grep neoland
```

---

## Escalation

| Status | Severity | Response | Escalation |
|--------|----------|----------|------------|
| 7-30 days | Warning | 4 hours | #neoland-warnings |
| 1-7 days | High | 1 hour | #neoland-warnings + @oncall-voidnx |
| <1 day | Critical | Immediate | PagerDuty + #neoland-critical |
| Expired | EMERGENCY | Immediate | PagerDuty + Manager + Status page update |

**Contacts**:
- Primary: `@oncall-voidnx`
- Infrastructure: `@oncall-infra` (for cert-manager issues)
- Security: (for certificate compromise)

---

## Post-Incident

### Immediate Actions (within 1 hour)

1. **Root Cause Analysis**:
   - Why didn't cert-manager auto-renew?
   - Was there a cert-manager failure?
   - DNS validation issue?
   - Rate limit hit?

2. **Document in incident report**: `docs/incidents/YYYY-MM-DD-tls-cert-expired.md`
   - Timeline
   - User impact (how long service down)
   - Root cause
   - Mitigation

### Follow-Up Actions (within 24 hours)

1. **Improve Monitoring**:
   - Add alert at 30 days (early warning)
   - Alert at 14 days (escalation)
   - Alert at 7 days (critical)
   - Monitor cert-manager health

2. **Improve Automation**:
   - Verify cert-manager CRDs up to date
   - Add cert-manager health checks
   - Implement certificate renewal testing (staging)
   - Add backup ACME provider (ZeroSSL)

3. **Add Redundancy**:
   - Use multiple ACME providers
   - Implement certificate backup (store in Vault)
   - Add manual certificate fallback procedure
   - Document emergency self-signed cert process

4. **Calendar Reminders**:
   - If using manual certificates, set renewal reminders
   - Review all certificates quarterly

---

## Prevention

### Short-term (Immediate)

- ✅ cert-manager automatic renewal configured
- ✅ Alerts at 7 days before expiry
- ⚠️ **TODO**: Add alert at 30 days (early warning)
- ⚠️ **TODO**: Monitor cert-manager health

### Medium-term (1-2 weeks)

- [ ] Configure backup ACME provider (ZeroSSL)
- [ ] Implement certificate backup in Vault
- [ ] Add cert-manager health checks
- [ ] Test certificate renewal in staging environment
- [ ] Document manual renewal procedure

### Long-term (1-3 months)

- [ ] Implement certificate rotation testing (chaos engineering)
- [ ] Add certificate transparency monitoring
- [ ] Automated certificate inventory (all services)
- [ ] Implement mTLS with automatic cert rotation
- [ ] Multi-region certificate management

---

## Related Alerts

- `NeolandDown`: Expired certificate causes service unavailable
- `NeolandUnhealthy`: TLS issues may fail health checks
- `CertManagerDown`: cert-manager pod failure (separate alert, TODO)

---

## Additional Resources

- **cert-manager Documentation**: https://cert-manager.io/docs/
- **Let's Encrypt Rate Limits**: https://letsencrypt.org/docs/rate-limits/
- **Certificate Transparency Log**: https://crt.sh/
- **TLS Best Practices**: `docs/SECURITY.md` (TODO: Phase 6)
- **cert-manager Configuration**: `k8s/cert-manager/`

---

**Last Updated**: 2026-01-31
**Maintainer**: voidnx team + Infrastructure team
**Severity**: WARNING → CRITICAL (time-based escalation)
