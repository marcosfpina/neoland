# Prometheus Alerting Configuration

**Phase 4.4: Production-Ready Alerting for NEOLAND**

## Overview

This directory contains Prometheus and AlertManager configurations for production monitoring and alerting.

### Files

- **`alerts.yml`**: Prometheus alert rules (60+ production alerts)
- **`alertmanager.yml`**: AlertManager routing and notification configuration
- **`prometheus.yml`**: Prometheus scrape configuration
- **`README.md`**: This file

---

## Alert Categories

### 1. Service Availability (Critical)
- **NeolandDown**: Service completely unavailable
- **NeolandHighRestartRate**: Pods restarting frequently (crash loops)

### 2. Health Checks
- **NeolandUnhealthy**: Health check failing
- **NeolandNotReady**: Readiness probe failing (removed from load balancer)
- **NeolandComponentDegraded**: Individual component degraded

### 3. Performance & Latency
- **NeolandHighLatency**: p99 latency >2s (warning)
- **NeolandVeryHighLatency**: p99 latency >5s (critical)
- **NeolandSlowLLMInference**: LLM inference >10s

### 4. Error Rates
- **NeolandHighErrorRate**: 5xx error rate >5%
- **NeolandCriticalErrorRate**: 5xx error rate >25%
- **NeolandLLMProviderFailures**: LLM provider errors

### 5. Security
- **NeolandHighAuthFailureRate**: >5 auth failures/sec
- **NeolandBruteForceAttack**: >20 auth failures/sec (brute force)
- **NeolandRateLimitExceeded**: Frequent rate limit hits
- **NeolandSuspiciousActivity**: Suspicious events detected

### 6. Resource Utilization
- **NeolandHighMemoryUsage**: >85% memory
- **NeolandCriticalMemoryUsage**: >95% memory (OOMKill risk)
- **NeolandHighCPUUsage**: >90% CPU

### 7. Dependencies
- **NeolandVectorStoreUnavailable**: Database/vector store down
- **NeolandAllLLMProvidersFailing**: All LLM providers unavailable

### 8. Cost & Usage
- **NeolandHighLLMCost**: LLM costs >$100/hour
- **NeolandExcessiveTokenUsage**: >10k tokens/sec

### 9. Data Quality
- **NeolandHighPromptRejectionRate**: >10% validation failures

---

## Alert Severity Levels

| Severity | Description | Response Time | Notification |
|----------|-------------|---------------|--------------|
| **critical** | Service outage or imminent failure | Immediate (0-5 min) | PagerDuty + Slack |
| **warning** | Degraded performance, non-critical | 15-30 minutes | Slack |
| **info** | Informational, no action required | Best effort | Slack (low priority) |

---

## Deployment

### Local Development

```bash
# Start Prometheus and AlertManager with docker-compose
cd deploy/prometheus
docker-compose up -d

# Access UIs
# Prometheus: http://localhost:9090
# AlertManager: http://localhost:9093
```

### Kubernetes Deployment

```bash
# Create namespace
kubectl create namespace monitoring

# Create secrets for sensitive values
kubectl create secret generic alertmanager-secrets \
  --from-literal=slack-webhook-url="${SLACK_WEBHOOK_URL}" \
  --from-literal=pagerduty-service-key="${PAGERDUTY_SERVICE_KEY}" \
  --from-literal=smtp-username="${SMTP_USERNAME}" \
  --from-literal=smtp-password="${SMTP_PASSWORD}" \
  -n monitoring

# Deploy Prometheus
kubectl apply -f k8s/prometheus-config.yaml
kubectl apply -f k8s/prometheus-deployment.yaml

# Deploy AlertManager
kubectl apply -f k8s/alertmanager-config.yaml
kubectl apply -f k8s/alertmanager-deployment.yaml

# Verify deployment
kubectl get pods -n monitoring
kubectl logs -f deployment/prometheus -n monitoring
```

### Helm Deployment (Recommended)

```bash
# Add Prometheus community Helm repo
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo update

# Install kube-prometheus-stack (includes Prometheus, AlertManager, Grafana)
helm install neoland-monitoring prometheus-community/kube-prometheus-stack \
  --namespace monitoring \
  --create-namespace \
  --values helm-values.yaml

# Custom values (helm-values.yaml)
cat <<EOF > helm-values.yaml
prometheus:
  prometheusSpec:
    additionalScrapeConfigs:
      - job_name: 'neoland'
        static_configs:
          - targets: ['neoland.default.svc.cluster.local:3001']
    ruleFiles:
      - /etc/prometheus/rules/alerts.yml

alertmanager:
  config:
    global:
      slack_api_url: "${SLACK_WEBHOOK_URL}"
    receivers:
      - name: 'pagerduty'
        pagerduty_configs:
          - service_key: "${PAGERDUTY_SERVICE_KEY}"

grafana:
  enabled: true
  adminPassword: "${GRAFANA_ADMIN_PASSWORD}"
EOF
```

---

## Configuration

### Environment Variables

Set these environment variables before deployment:

```bash
# Slack Integration
export SLACK_WEBHOOK_URL="https://hooks.slack.com/services/YOUR/WEBHOOK/URL"

# PagerDuty Integration
export PAGERDUTY_SERVICE_KEY="your-pagerduty-integration-key"

# Email SMTP
export SMTP_USERNAME="alerts@neoland.example.com"
export SMTP_PASSWORD="your-smtp-password"

# Prometheus Remote Write (optional - for long-term storage)
export PROMETHEUS_REMOTE_WRITE_URL="https://prometheus-remote.example.com/api/v1/write"
export PROMETHEUS_REMOTE_WRITE_USER="your-username"
export PROMETHEUS_REMOTE_WRITE_PASSWORD="your-password"
```

### Slack Setup

1. Create Slack app: https://api.slack.com/apps
2. Enable Incoming Webhooks
3. Create webhook for channels:
   - `#neoland-critical`
   - `#neoland-warnings`
   - `#neoland-info`
   - `#security-alerts`
   - `#ml-alerts`
4. Copy webhook URL to `SLACK_WEBHOOK_URL`

### PagerDuty Setup

1. Create PagerDuty service: https://neoland.pagerduty.com/services
2. Add "Prometheus" integration
3. Copy Integration Key to `PAGERDUTY_SERVICE_KEY`
4. Configure escalation policy:
   - Level 1: On-call engineer (immediate)
   - Level 2: Team lead (after 15 min)
   - Level 3: Manager (after 30 min)

---

## Testing Alerts

### Trigger Test Alert

```bash
# Manually trigger alert via AlertManager
curl -X POST http://localhost:9093/api/v1/alerts \
  -H 'Content-Type: application/json' \
  -d '[{
    "labels": {
      "alertname": "TestAlert",
      "severity": "warning",
      "component": "test",
      "team": "voidnx"
    },
    "annotations": {
      "summary": "This is a test alert",
      "description": "Testing alert routing and notifications",
      "runbook": "https://docs.neoland/runbooks/test"
    }
  }]'
```

### Simulate High Error Rate

```bash
# Generate 500 errors to trigger NeolandHighErrorRate
for i in {1..100}; do
  curl -X POST http://neoland:3001/v1/chat/completions \
    -H "Content-Type: application/json" \
    -d '{"invalid": "request"}' &
done
```

### Simulate Memory Pressure

```bash
# Scale down memory limits to trigger memory alerts
kubectl set resources deployment/neoland \
  --limits=memory=256Mi \
  -n default
```

---

## Alert Routing

### Flow Diagram

```
Alert Fired
    ↓
Prometheus Evaluation
    ↓
AlertManager Routing
    ├─ Critical → PagerDuty (immediate)
    ├─ Critical → Slack #neoland-critical
    ├─ Security → Slack #security-alerts + Email
    ├─ Warning → Slack #neoland-warnings
    ├─ Info → Slack #neoland-info
    └─ Cost → Email finance@neoland.example.com
```

### Inhibition Rules

AlertManager automatically suppresses redundant alerts:

- If `NeolandDown` fires, suppress all other alerts for that instance
- If `NeolandUnhealthy` fires, suppress `NeolandHighLatency`
- If critical alert fires, suppress corresponding warning

---

## Runbooks

Each alert includes a runbook link. Create runbooks in `docs/runbooks/`:

```bash
docs/runbooks/
├── service-down.md              # NeolandDown
├── high-error-rate.md           # NeolandHighErrorRate
├── high-latency.md              # NeolandHighLatency
├── health-check-failure.md      # NeolandUnhealthy
├── readiness-failure.md         # NeolandNotReady
├── auth-failures.md             # NeolandHighAuthFailureRate
├── brute-force-attack.md        # NeolandBruteForceAttack
├── high-memory.md               # NeolandHighMemoryUsage
├── critical-memory.md           # NeolandCriticalMemoryUsage
├── high-cpu.md                  # NeolandHighCPUUsage
├── vector-store-down.md         # NeolandVectorStoreUnavailable
├── all-llm-providers-down.md    # NeolandAllLLMProvidersFailing
└── high-llm-cost.md             # NeolandHighLLMCost
```

**Runbook Template**:

```markdown
# Runbook: [Alert Name]

## Symptoms
- Alert: [AlertName] firing
- User impact: [Description]

## Investigation
1. Check [system/logs/metrics]
2. Verify [dependencies]
3. Review [recent changes]

## Resolution
- If [condition]: [action]
- Otherwise: [alternative action]

## Escalation
- Contact: @oncall-[team]
- Slack: #[channel]
```

---

## Monitoring the Monitors

### Check AlertManager Health

```bash
# AlertManager health endpoint
curl http://alertmanager:9093/-/healthy

# Check alert routing
curl http://alertmanager:9093/api/v2/alerts | jq
```

### Check Prometheus Health

```bash
# Prometheus health endpoint
curl http://prometheus:9090/-/healthy

# Check alert rules loaded
curl http://prometheus:9090/api/v1/rules | jq

# Check targets
curl http://prometheus:9090/api/v1/targets | jq
```

### Grafana Dashboards

Import pre-built dashboards:

1. **AlertManager Dashboard**: ID 9578
2. **Prometheus Stats**: ID 3662
3. **Node Exporter**: ID 1860

---

## Maintenance

### Silence Alerts (Planned Maintenance)

```bash
# Silence all alerts for 2 hours
amtool silence add \
  --alertmanager.url=http://alertmanager:9093 \
  --author="ops-team" \
  --comment="Planned maintenance window" \
  --duration=2h \
  alertname=~"Neoland.*"

# Silence specific instance
amtool silence add \
  --alertmanager.url=http://alertmanager:9093 \
  instance="neoland-7b9c8d5f6-abc12" \
  --duration=1h
```

### Update Alert Rules

```bash
# Edit alerts.yml
vim deploy/prometheus/alerts.yml

# Reload Prometheus configuration (without restart)
curl -X POST http://prometheus:9090/-/reload

# Or send SIGHUP to Prometheus pod
kubectl exec -it deployment/prometheus -n monitoring -- kill -HUP 1
```

---

## Best Practices

1. **Alert Fatigue**: Don't alert on everything
   - Only alert on actionable conditions
   - Use `for: Xm` to avoid flapping alerts
   - Tune thresholds based on actual operational experience

2. **Runbooks**: Every alert MUST have a runbook
   - Document investigation steps
   - Include resolution procedures
   - Link to relevant dashboards

3. **Test Regularly**: Schedule monthly alert drills
   - Verify PagerDuty escalation works
   - Check Slack notifications arrive
   - Test runbook procedures

4. **Review & Refine**: Quarterly alert review
   - Identify noisy alerts (tune or disable)
   - Add missing alerts (coverage gaps)
   - Update thresholds based on SLOs

5. **Ownership**: Assign alert ownership
   - Each alert has a responsible team
   - Team responds during their on-call rotation
   - Clear escalation path documented

---

## Troubleshooting

### Alerts Not Firing

```bash
# Check Prometheus can scrape NEOLAND metrics
curl http://prometheus:9090/api/v1/query?query=up{job="neoland"}

# Check alert rule syntax
promtool check rules deploy/prometheus/alerts.yml

# Check AlertManager configuration
amtool check-config deploy/prometheus/alertmanager.yml
```

### Notifications Not Sent

```bash
# Check AlertManager logs
kubectl logs -f deployment/alertmanager -n monitoring

# Test Slack webhook
curl -X POST "${SLACK_WEBHOOK_URL}" \
  -H 'Content-Type: application/json' \
  -d '{"text":"Test from AlertManager"}'

# Check receiver configuration
amtool config routes --alertmanager.url=http://alertmanager:9093
```

### Alert Storm

```bash
# If too many alerts firing, silence temporarily
amtool silence add \
  --alertmanager.url=http://alertmanager:9093 \
  --author="ops" \
  --comment="Investigating alert storm" \
  --duration=30m \
  severity="warning"

# Investigate root cause
# Fix underlying issue
# Remove silence when stable
```

---

## References

- [Prometheus Alerting Documentation](https://prometheus.io/docs/alerting/latest/overview/)
- [AlertManager Configuration](https://prometheus.io/docs/alerting/latest/configuration/)
- [Alert Rule Best Practices](https://prometheus.io/docs/practices/alerting/)
- [Runbook Standards](https://github.com/kubernetes-monitoring/kubernetes-mixin/tree/master/runbook.md)
- [PagerDuty Integration Guide](https://www.pagerduty.com/docs/guides/prometheus-integration-guide/)

---

**Maintained By**: voidnx team
**Last Updated**: 2026-01-31
**Questions**: #neoland-ops on Slack
