# ADR-018: Prometheus Alerting & AlertManager

**Status**: Accepted
**Date**: 2026-01-31
**Decision Makers**: Architecture Team, SRE Team
**Phase**: 4.4 - Operational Readiness

---

## Context

NEOLAND requires production-grade alerting for:
- **Incident Response**: Immediate notification of service degradation
- **On-Call Support**: PagerDuty integration for 24/7 coverage
- **Proactive Monitoring**: Early detection of performance degradation
- **Security Incidents**: Real-time alerts for suspicious activity
- **Cost Control**: Notification of unexpected usage spikes
- **Compliance**: Audit trail of system health events (SOC 2, ISO 27001)

Current state: Metrics collected (Phase 4.1) but no alerting configured. Incidents discovered reactively by users.

---

## Decision

Implement **comprehensive Prometheus alerting** with AlertManager routing:

### 1. Alert Rule Categories (60+ alerts)

**Service Availability**:
- `NeolandDown`: Service completely unavailable (critical)
- `NeolandHighRestartRate`: Pods crash-looping (warning)

**Health Checks**:
- `NeolandUnhealthy`: Health check failing (critical)
- `NeolandNotReady`: Readiness probe failing (warning)
- `NeolandComponentDegraded`: Individual component degraded (warning)

**Performance & Latency**:
- `NeolandHighLatency`: p99 >2s (warning)
- `NeolandVeryHighLatency`: p99 >5s (critical)
- `NeolandSlowLLMInference`: LLM inference >10s (warning)

**Error Rates**:
- `NeolandHighErrorRate`: 5xx >5% (warning)
- `NeolandCriticalErrorRate`: 5xx >25% (critical)
- `NeolandLLMProviderFailures`: LLM errors (warning)

**Security**:
- `NeolandHighAuthFailureRate`: >5 auth failures/sec (warning)
- `NeolandBruteForceAttack`: >20 auth failures/sec (critical)
- `NeolandRateLimitExceeded`: Frequent rate limiting (info)
- `NeolandSuspiciousActivity`: Suspicious events (warning)

**Resource Utilization**:
- `NeolandHighMemoryUsage`: >85% memory (warning)
- `NeolandCriticalMemoryUsage`: >95% memory (critical, OOMKill risk)
- `NeolandHighCPUUsage`: >90% CPU (warning)

**Dependencies**:
- `NeolandVectorStoreUnavailable`: Database down (critical)
- `NeolandAllLLMProvidersFailing`: All LLM providers down (critical)

**Cost & Usage**:
- `NeolandHighLLMCost`: >$100/hour (warning)
- `NeolandExcessiveTokenUsage`: >10k tokens/sec (info)

**Data Quality**:
- `NeolandHighPromptRejectionRate`: >10% validation failures (warning)

### 2. Severity Levels

| Level | Response Time | Notification | Example |
|-------|---------------|--------------|---------|
| **critical** | 0-5 minutes | PagerDuty + Slack | Service down |
| **warning** | 15-30 minutes | Slack | High latency |
| **info** | Best effort | Slack (low priority) | Rate limit exceeded |

### 3. AlertManager Routing

```
Alert → AlertManager
    ├─ Critical → PagerDuty (wakes on-call)
    ├─ Critical → Slack #neoland-critical
    ├─ Security → Slack #security-alerts + Email
    ├─ Warning → Slack #neoland-warnings
    ├─ Info → Slack #neoland-info
    ├─ Cost → Email finance@
    └─ ML → Slack #ml-alerts
```

**Grouping**: Alerts grouped by `alertname`, `component`, `severity`
**Inhibition**: Suppress redundant alerts (e.g., if service is down, don't alert on latency)

### 4. Runbook Links

Every alert includes:
- **summary**: One-line description
- **description**: Detailed context
- **runbook**: Link to investigation/resolution steps
- **action**: Immediate action to take
- **impact**: User-facing impact (for critical alerts)

Example:
```yaml
annotations:
  summary: "NEOLAND service is down"
  description: "Instance {{ $labels.instance }} has been down for >2 minutes."
  runbook: "https://docs.neoland/runbooks/service-down"
  impact: "Users cannot access the service"
  action: "Check pod logs: kubectl logs -l app=neoland"
```

---

## Implementation

### File Structure

```
deploy/prometheus/
├── alerts.yml              # 60+ Prometheus alert rules
├── alertmanager.yml        # AlertManager routing config
├── prometheus.yml          # Prometheus scrape + alerting config
└── README.md               # Deployment and usage guide
```

### Alert Rule Example

```yaml
- alert: NeolandHighLatency
  expr: |
    histogram_quantile(0.99,
      rate(neoland_http_request_duration_seconds_bucket[5m])
    ) > 2.0
  for: 5m
  labels:
    severity: warning
    component: performance
    team: platform
  annotations:
    summary: "NEOLAND p99 latency is high"
    description: "99th percentile latency is {{ $value }}s."
    runbook: "https://docs.neoland/runbooks/high-latency"
    action: "Check LLM provider response times"
```

### AlertManager Routing

```yaml
route:
  receiver: 'default'
  group_by: ['alertname', 'component', 'severity']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h

  routes:
    # Critical → PagerDuty
    - match:
        severity: critical
      receiver: 'pagerduty'
      group_wait: 10s
      repeat_interval: 5m

    # Security → Security team
    - match:
        team: security
      receiver: 'security-team'
      group_wait: 10s
```

### PagerDuty Integration

```yaml
receivers:
  - name: 'pagerduty'
    pagerduty_configs:
      - service_key: "${PAGERDUTY_SERVICE_KEY}"
        description: '{{ .GroupLabels.alertname }}: {{ .CommonAnnotations.summary }}'
        details:
          runbook: '{{ .CommonAnnotations.runbook }}'
          action: '{{ .CommonAnnotations.action }}'
```

### Slack Integration

```yaml
receivers:
  - name: 'slack-critical'
    slack_configs:
      - channel: '#neoland-critical'
        icon_emoji: ':fire:'
        title: ':fire: CRITICAL: {{ .GroupLabels.alertname }}'
        text: |
          *Action Required:* {{ .Annotations.action }}
          *Runbook:* {{ .Annotations.runbook }}
        color: 'danger'
```

---

## Alternatives Considered

### 1. Built-in Kubernetes Monitoring Only
**Rejected**: Insufficient for application-level alerts. No custom business logic alerts (e.g., LLM cost).

### 2. Datadog or New Relic SaaS
**Rejected**: Higher cost ($150-300/month), vendor lock-in. Prometheus is open-source and already integrated.

### 3. ElastAlert (Elasticsearch-based)
**Rejected**: Requires separate Elasticsearch deployment. Prometheus already collects metrics.

### 4. Custom Alerting Service
**Rejected**: Reinventing the wheel. Prometheus/AlertManager is industry standard with proven reliability.

### 5. Alert on Everything
**Rejected**: Causes alert fatigue. Only alert on actionable conditions requiring human intervention.

---

## Consequences

### Positive
✅ **Rapid Incident Response**: Critical alerts page on-call within seconds
✅ **Reduced Downtime**: Proactive detection before user impact
✅ **Clear Ownership**: Team-based routing (platform, security, ML, finance)
✅ **Actionable Alerts**: Every alert includes runbook and immediate action
✅ **Cost Visibility**: Real-time notification of cost spikes
✅ **Security Monitoring**: Immediate alerts for brute force, suspicious activity
✅ **Reduced Alert Fatigue**: Severity-based routing, inhibition rules
✅ **Compliance**: Audit trail for SOC 2, ISO 27001

### Negative
⚠️ **Configuration Complexity**: 60+ alert rules, routing logic requires maintenance
⚠️ **False Positives**: Initial tuning period to adjust thresholds
⚠️ **On-Call Burden**: Critical alerts require 24/7 on-call rotation
⚠️ **Integration Costs**: PagerDuty ~$40/user/month for on-call management

### Neutral
🔧 **Threshold Tuning Required**: Quarterly review of alert thresholds based on SLOs
🔧 **Runbook Maintenance**: Must keep runbooks up-to-date with architecture changes
🔧 **External Dependencies**: Relies on Slack, PagerDuty availability

---

## Deployment

### Kubernetes (Helm)

```bash
# Install kube-prometheus-stack
helm install neoland-monitoring prometheus-community/kube-prometheus-stack \
  --namespace monitoring \
  --create-namespace \
  --set prometheus.additionalScrapeConfigs=deploy/prometheus/prometheus.yml \
  --set alertmanager.config=deploy/prometheus/alertmanager.yml
```

### Docker Compose (Development)

```yaml
version: '3.8'
services:
  prometheus:
    image: prom/prometheus:latest
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - ./alerts.yml:/etc/prometheus/alerts.yml
    ports:
      - "9090:9090"

  alertmanager:
    image: prom/alertmanager:latest
    volumes:
      - ./alertmanager.yml:/etc/alertmanager/alertmanager.yml
    ports:
      - "9093:9093"
```

---

## Testing

### Test Alert Delivery

```bash
# Trigger test alert
curl -X POST http://alertmanager:9093/api/v1/alerts \
  -H 'Content-Type: application/json' \
  -d '[{"labels":{"alertname":"TestAlert","severity":"warning"}}]'
```

### Simulate High Error Rate

```bash
# Generate 500 errors
for i in {1..100}; do
  curl -X POST http://neoland:3001/v1/chat/completions \
    -d '{"invalid": "request"}' &
done

# Wait 5 minutes
# Expected: NeolandHighErrorRate alert fires
```

### Verify PagerDuty Integration

```bash
# Check PagerDuty incidents API
curl -H "Authorization: Token token=${PAGERDUTY_API_TOKEN}" \
  https://api.pagerduty.com/incidents?service_ids[]=${SERVICE_ID}
```

---

## Operational Procedures

### Monthly On-Call Rotation

1. **Week Before**: Assign next on-call rotation in PagerDuty
2. **Monday**: Handoff meeting (30 min)
   - Review open incidents
   - Discuss recent alerts
   - Update runbooks
3. **During Rotation**: Respond to alerts per severity SLA
4. **End of Week**: Incident retrospective
   - What alerts fired?
   - Were runbooks accurate?
   - What can be improved?

### Quarterly Alert Review

1. **Identify Noisy Alerts**: Alerts firing >10 times/day
2. **Tune Thresholds**: Adjust based on operational experience
3. **Add Missing Coverage**: Gaps in monitoring
4. **Update Runbooks**: Architecture changes
5. **Remove Obsolete Alerts**: Deprecated components

### Planned Maintenance

```bash
# Silence alerts for 2-hour maintenance window
amtool silence add \
  --alertmanager.url=http://alertmanager:9093 \
  --author="ops-team" \
  --comment="Database migration maintenance" \
  --duration=2h \
  alertname=~"Neoland.*"
```

---

## Runbook Requirements

Every alert must have a runbook documenting:

1. **Symptoms**: How to identify the issue
2. **Investigation**: Steps to diagnose root cause
3. **Resolution**: How to fix the problem
4. **Escalation**: When to escalate and to whom

**Runbook Template**:

```markdown
# Runbook: NeolandHighLatency

## Symptoms
- Alert: NeolandHighLatency firing
- User reports: Slow response times
- Metrics: p99 latency >2s

## Investigation
1. Check Grafana dashboard: Request latency by endpoint
2. Check LLM provider status: ml-offload, SecureLLM API
3. Check database query performance
4. Review recent deployments

## Resolution
- If ml-offload slow: Restart ml-offload service
- If SecureLLM rate limited: Switch to local engine
- If database slow: Check slow query log, add indexes
- If recent deployment: Rollback

## Escalation
- After 30 min: Escalate to Team Lead
- After 1 hour: Escalate to Engineering Manager
- Contact: @oncall-platform in Slack
```

---

## Compliance Mapping

| Requirement | Implementation | Status |
|-------------|----------------|--------|
| **SOC 2 (CC7.2)** | Continuous monitoring with alerting | ✅ |
| **ISO 27001 (A.12.1.4)** | Availability monitoring and incident response | ✅ |
| **ISO 27001 (A.16.1.2)** | Security event alerting and escalation | ✅ |
| **PCI-DSS 10.6** | Security monitoring and alerting | ✅ |
| **GDPR (Art. 32)** | Incident detection and response capability | ✅ |

---

## Metrics

Track alerting effectiveness:

- **MTTD** (Mean Time To Detect): Time from incident to alert firing
- **MTTR** (Mean Time To Resolve): Time from alert to resolution
- **Alert Accuracy**: % of actionable vs. false positive alerts
- **On-Call Burden**: Alerts per on-call shift
- **Runbook Effectiveness**: % of incidents resolved using runbook

**Targets** (after tuning):
- MTTD: <2 minutes
- MTTR: <30 minutes
- Alert Accuracy: >90%
- On-Call Burden: <3 pages per shift
- Runbook Effectiveness: >80%

---

## Future Enhancements

1. **Anomaly Detection** (Phase 5)
   - Machine learning-based anomaly detection
   - Adaptive thresholds based on historical patterns
   - Tools: Prometheus Adaptive Metrics, LinkedIn's Luminol

2. **Multi-Region Alerting** (Phase 5)
   - Region-specific alerts
   - Cross-region correlation
   - Federated Prometheus setup

3. **SLO-Based Alerting** (Phase 4.5)
   - Error budget burn rate alerts
   - Multi-window, multi-burn-rate approach
   - Based on Google SRE SLO alerting principles

4. **Auto-Remediation** (Phase 6)
   - Automated mitigation for common incidents
   - Self-healing: restart pods, scale resources
   - Tools: Kubernetes Operators, ArgoCD

5. **Alert Correlation** (Phase 6)
   - Root cause analysis automation
   - Incident timeline reconstruction
   - Tools: LinkedIn's Iris, Netflix's Dispatch

---

## References

- [Prometheus Alerting Best Practices](https://prometheus.io/docs/practices/alerting/)
- [Google SRE Book: Monitoring Distributed Systems](https://sre.google/sre-book/monitoring-distributed-systems/)
- [AlertManager Configuration](https://prometheus.io/docs/alerting/latest/configuration/)
- [PagerDuty Incident Response Guide](https://response.pagerduty.com/)
- [My Philosophy on Alerting (Rob Ewaschuk)](https://docs.google.com/document/d/199PqyG3UsyXlwieHaqbGiWVa8eMWi8zzAn0YfcApr8Q/)

---

**Approved By**: Architecture Team, SRE Team
**Implementation**: Phase 4.4
**Status**: ✅ Implemented
