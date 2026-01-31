# NEOLAND Operational Runbooks

**Purpose**: Step-by-step incident response procedures for production alerts

**Audience**: On-call engineers, SREs, operations team

**Last Updated**: 2026-01-31

---

## Quick Reference: Alert → Runbook Mapping

| Alert Name | Severity | Runbook | Response Time |
|------------|----------|---------|---------------|
| **NeolandDown** | 🔴 CRITICAL | [service-down.md](./service-down.md) | 0-5 min |
| **NeolandAllLLMProvidersFailing** | 🔴 CRITICAL | [all-llm-providers-down.md](./all-llm-providers-down.md) | 0-5 min |
| **NeolandBruteForceAttack** | 🔴 CRITICAL (Security) | [brute-force-attack.md](./brute-force-attack.md) | 0-2 min |
| **NeolandCriticalErrorRate** | 🔴 CRITICAL | [high-error-rate.md](./high-error-rate.md) | 0-5 min |
| **NeolandCriticalMemoryUsage** | 🔴 CRITICAL | [critical-memory.md](./critical-memory.md) | 0-5 min |
| **NeolandVeryHighLatency** | 🔴 CRITICAL | [high-latency.md](./high-latency.md) | 0-5 min |
| **NeolandUnhealthy** | 🔴 CRITICAL | [health-check-failure.md](./health-check-failure.md) | 0-5 min |
| **NeolandHighErrorRate** | ⚠️ WARNING | [high-error-rate.md](./high-error-rate.md) | 15-30 min |
| **NeolandHighMemoryUsage** | ⚠️ WARNING | [critical-memory.md](./critical-memory.md) | 15-30 min |
| **NeolandHighLatency** | ⚠️ WARNING | [high-latency.md](./high-latency.md) | 15-30 min |
| **NeolandNotReady** | ⚠️ WARNING | [health-check-failure.md](./health-check-failure.md) | 15-30 min |

**Note**: More runbooks coming soon for remaining 50+ alerts (see [Phase 4.5 Roadmap](#phase-45-roadmap))

---

## Runbook Index

### Critical Incidents (P1 - Immediate Response)

1. **[Service Down](./service-down.md)** - `NeolandDown`
   - Complete service outage, all pods unavailable
   - **User Impact**: Total failure, no service available
   - **Scenarios**: CrashLoopBackOff, OOMKilled, ImagePullBackOff, cluster failure

2. **[All LLM Providers Down](./all-llm-providers-down.md)** - `NeolandAllLLMProvidersFailing`
   - All 3 LLM backends failed (ml-offload, local, SecureLLM)
   - **User Impact**: No LLM inference possible
   - **Scenarios**: Provider restarts, local engine reload, emergency fallback

3. **[Brute Force Attack](./brute-force-attack.md)** - `NeolandBruteForceAttack` 🚨
   - **SECURITY INCIDENT**: >20 failed auth attempts/second
   - **User Impact**: Potential security breach in progress
   - **Scenarios**: IP blocking, rate limiting, account lockout
   - **⚠️ MANDATORY escalation to security team**

4. **[High Error Rate](./high-error-rate.md)** - `NeolandCriticalErrorRate` / `NeolandHighErrorRate`
   - Error rate >10% (critical) or >5% (warning)
   - **User Impact**: Frequent failures, degraded service
   - **Scenarios**: LLM failure, database issues, bad deployment, resource exhaustion

5. **[Critical Memory Usage](./critical-memory.md)** - `NeolandCriticalMemoryUsage` / `NeolandHighMemoryUsage`
   - Memory >95% (critical) or >85% (warning)
   - **User Impact**: OOMKill imminent, service may crash
   - **Scenarios**: Memory leak, large dataset, concurrent requests

6. **[High Latency](./high-latency.md)** - `NeolandVeryHighLatency` / `NeolandHighLatency`
   - p99 latency >5s (critical) or >2s (warning)
   - **User Impact**: Very slow responses, user frustration
   - **Scenarios**: LLM slow, database slow, resource exhaustion, network latency

7. **[Health Check Failure](./health-check-failure.md)** - `NeolandUnhealthy` / `NeolandNotReady`
   - Health endpoint returning unhealthy, readiness probe failing
   - **User Impact**: Pod removed from load balancer
   - **Scenarios**: Vector store down, LLM down, startup failure, resource constraints

---

## Runbook Structure

Every runbook follows this consistent template:

```markdown
# Runbook: <Alert Name>

**Alert**: <AlertManager alert name>
**Severity**: CRITICAL 🔴 / WARNING ⚠️
**Response Time**: 0-5 min / 15-30 min

## Symptoms
- Alert firing conditions
- User impact
- Metrics thresholds

## Investigation
1. Check logs: kubectl logs ...
2. Check metrics: curl prometheus ...
3. Check dependencies: curl ...

## Resolution
### Scenario 1: <Most Common>
- Step-by-step fix
- Commands to run
- Expected output

### Scenario 2: <Second Most Common>
...

## Verification
- Check metrics returned to normal
- Verify service healthy
- Confirm alert cleared

## Escalation
- Response timeline
- Contact information
- Escalation criteria

## Post-Incident
- Root cause analysis
- Prevention measures
- Create GitHub issue

## Related Alerts
- Other alerts that may fire together
```

---

## Using These Runbooks

### When Alert Fires

1. **Acknowledge alert** in PagerDuty/Slack immediately
2. **Open runbook** for the specific alert
3. **Follow Investigation** section to diagnose
4. **Execute Resolution** for the identified scenario
5. **Verify** the fix worked (metrics, health checks)
6. **Document** what you did in incident log
7. **Complete Post-Incident** procedures

### Key Principles

- ✅ **Follow steps sequentially** - Don't skip investigation
- ✅ **Verify before escalating** - Check if issue is real
- ✅ **Document everything** - Create audit trail
- ✅ **Escalate if stuck** - Don't waste time if unsure
- ❌ **Don't guess** - Follow the runbook
- ❌ **Don't panic** - These are tested procedures

### Common Commands Reference

```bash
# Check pod status
kubectl get pods -l app=neoland -n default
kubectl describe pod -l app=neoland -n default

# Check logs (last 200 lines)
kubectl logs -l app=neoland --tail=200 -n default

# Check recent errors
kubectl logs -l app=neoland --tail=500 -n default | grep -i "error\|fatal\|panic"

# Check health endpoint
curl http://neoland:3001/health | jq

# Check metrics
curl http://neoland:3001/metrics | grep neoland_

# Check Prometheus
curl -s 'http://prometheus:9090/api/v1/query?query=<QUERY>' | jq

# Restart deployment
kubectl rollout restart deployment/neoland -n default
kubectl rollout status deployment/neoland -n default

# Rollback deployment
kubectl rollout undo deployment/neoland -n default

# Scale deployment
kubectl scale deployment/neoland --replicas=5 -n default

# Check resource usage
kubectl top pods -l app=neoland -n default
```

---

## Escalation Contacts

### On-Call Rotation
- **Primary**: `@oncall-voidnx` (Slack, PagerDuty)
- **Secondary**: `@oncall-ml` (ML Team, for ml-offload issues)
- **Security**: `@oncall-security` (Security incidents only)

### Escalation Timeline

| Severity | Initial Response | Escalation 1 | Escalation 2 | Escalation 3 |
|----------|------------------|--------------|--------------|--------------|
| **CRITICAL** | 0-5 min: On-call engineer | 5-15 min: Team Lead | 15-30 min: Manager | 30+ min: Incident Commander |
| **WARNING** | 15-30 min: On-call engineer | 1-2 hr: Team Lead | 4+ hr: Manager | N/A |

### Communication Channels
- **Critical Alerts**: `#neoland-critical` (Slack) + PagerDuty
- **Warning Alerts**: `#neoland-warnings` (Slack)
- **Security Incidents**: `#security-alerts` (Slack) + security@neoland.example.com
- **Post-Mortems**: `#neoland-postmortems` (Slack)

---

## Post-Incident Procedures

After resolving any incident:

1. **Create Incident Report** (within 1 hour for critical)
   ```bash
   # Template: docs/incidents/YYYY-MM-DD-<alert-name>.md
   # Include:
   # - Timeline of events
   # - Root cause
   # - Resolution steps
   # - Impact metrics (duration, users affected)
   # - Action items for prevention
   ```

2. **Create GitHub Issue** for action items
   ```
   Title: [INCIDENT] <Alert Name> on YYYY-MM-DD
   Labels: incident, <severity>
   Assignee: Team Lead
   Description:
   - Root cause: [description]
   - Impact: [duration, users]
   - Action items: [preventive measures]
   ```

3. **Update Runbook** if new scenario discovered
   - Add new resolution scenario
   - Update investigation steps
   - Add to prevention section

4. **Post-Mortem** (for critical incidents, within 48 hours)
   - Team review meeting
   - Blameless analysis
   - Identify gaps in monitoring/alerting
   - Create action items for hardening

---

## Phase 4.5 Roadmap

### ✅ Completed Runbooks (17/60) - **PAUSED FOR PHASE 2 TESTING**

**Critical P1 (Immediate Response)**:
- [x] Service Down (NeolandDown)
- [x] All LLM Providers Down (NeolandAllLLMProvidersFailing)
- [x] Brute Force Attack (NeolandBruteForceAttack - Security)
- [x] High Error Rate (NeolandHighErrorRate/Critical)
- [x] Critical Memory (NeolandCriticalMemoryUsage/High)
- [x] Health Check Failure (NeolandUnhealthy/NotReady)
- [x] Database Unavailable (NeolandVectorStoreUnavailable)
- [x] Pod Restart Loop (NeolandPodRestartLoop)
- [x] Deployment Failed (NeolandDeploymentFailed)
- [x] Configuration Error (NeolandConfigError)
- [x] Unauthorized Access (NeolandUnauthorizedAccess - Security)

**High Priority P2**:
- [x] High Latency (NeolandHighLatency/VeryHigh)
- [x] High CPU Usage (NeolandHighCPUUsage/Critical)
- [x] Slow LLM Inference (NeolandSlowLLMInference)
- [x] Rate Limit Exceeded (NeolandRateLimitExceeded)
- [x] TLS Certificate Expiring (NeolandTLSCertExpiringSoon/Expired)
- [x] Disk Space Low (NeolandDiskSpaceLow/Critical)

### 🔄 Remaining Runbooks (~43) - **To be created incrementally**

**Next Priority (when returning to Phase 4.5)**:
- [ ] Network Connectivity Issues
- [ ] Backup Failures
- [ ] Log Pipeline Failures
- [ ] Metrics Collection Failures
- [ ] Database Connection Pool Exhausted
- [ ] Slow Database Queries
- [ ] LLM Provider Timeouts
- [ ] Authentication Service Down
- [ ] Secrets Vault Unavailable
- [ ] Load Balancer Failures

**Lower Priority** (~33 remaining):
- Container image vulnerabilities
- Alerting pipeline failures
- Certificate transparency issues
- DNS failures
- Storage quota exceeded
- ... (and more from Phase 4.4 alerts)

**Total Progress**: 17 of ~60 runbooks (28% complete)

**Time Investment**:
- Completed: ~13-14 hours (17 runbooks averaging ~45min each)
- Remaining: ~40-45 hours for all 43 runbooks (can be done incrementally)
- **Phase 4.5 Allocation**: 16 hours (87% used for critical runbooks)

**Status**: **Phase 4.5 PAUSED** - All critical/high-priority runbooks complete. Returning to Phase 2.3-2.5 (Testing) as planned. Remaining runbooks will be created incrementally as needed.

---

## Runbook Template

Use this template when creating new runbooks:

```markdown
# Runbook: <Alert Display Name>

**Alert**: `<AlertManagerAlertName>`
**Severity**: CRITICAL 🔴 / WARNING ⚠️ / INFO ℹ️
**Response Time**: 0-5 min / 15-30 min / 1-4 hours

---

## Symptoms

- **Alert**: <What alert is firing>
- **User Impact**: <How users are affected>
- **Metrics**: <Prometheus query that triggered alert>
- **Common Causes**: <Quick list of likely reasons>

---

## Investigation

```bash
# Step 1: Check <first thing to check>
<command>

# Expected output:
# <what healthy looks like>

# Step 2: Check <second thing>
<command>
```

---

## Resolution

### Scenario 1: <Most Common Cause>

```bash
# Step-by-step resolution
<commands>
```

**Timeframe**: <How long this takes>

### Scenario 2: <Second Common Cause>

...

---

## Verification

```bash
# 1. Check metric returned to normal
<prometheus query>
# Expected: <threshold>

# 2. Verify service healthy
curl http://neoland:3001/health | jq '.status'
# Expected: "healthy"

# 3. Check alert cleared
# AlertManager: http://alertmanager:9093/#/alerts
```

---

## Escalation

- **Severity Level**: P1 (Critical) / P2 (High) / P3 (Medium)
- **Initial Response**: <Who to contact first>
- **Escalation Path**: <Timeline and contacts>
- **Communication**: <Which Slack channels>

**Contacts**:
- Primary: `@oncall-<team>`
- Secondary: Team Lead (after <time>)
- Manager: (if not resolved in <time>)

---

## Post-Incident

1. **Root Cause Analysis**: <What to investigate>
2. **Prevention**: <What to implement>
3. **Create GitHub Issue**:
   ```
   Title: [INCIDENT] <Alert> on YYYY-MM-DD
   Labels: incident, <severity>
   Description: <Template>
   ```

---

## Related Alerts

- `<RelatedAlert1>`: <How they relate>
- `<RelatedAlert2>`: <May fire together>

---

## Additional Resources

- **Grafana Dashboard**: [<Dashboard Name>](http://grafana/d/<dashboard-id>)
- **Documentation**: `docs/<relevant-doc>.md`
- **ADR**: `docs/ADR/ADR-XXX-<topic>.md`

---

**Last Updated**: YYYY-MM-DD
**Maintainer**: <team-name>
**Severity**: <CRITICAL/WARNING/INFO>
```

---

## Contributing

### Adding New Runbooks

1. Copy the template above
2. Fill in all sections with tested procedures
3. Get review from team lead
4. Test commands in staging environment
5. Add to this README index
6. Submit PR with `docs/runbook:` prefix

### Updating Existing Runbooks

1. Document what changed and why
2. Update "Last Updated" date
3. Add to changelog section
4. Get review before merging

### Runbook Quality Checklist

- [ ] All commands tested in production-like environment
- [ ] Expected outputs documented
- [ ] Timeframes realistic
- [ ] Escalation paths correct
- [ ] Links to related resources valid
- [ ] Follows consistent template structure
- [ ] No sensitive information (credentials, IPs)

---

## Additional Resources

- **Alerting Rules**: `k8s/prometheus/alerts.yml`
- **Metrics Documentation**: `docs/METRICS.md`
- **Architecture**: `docs/ARCHITECTURE.md`
- **Production Readiness**: `docs/PROGRESS.md`
- **Incident Response Plan**: `docs/SECURITY_INCIDENT_RESPONSE.md` (TODO: Phase 6)

---

**Last Updated**: 2026-01-31
**Maintainer**: voidnx team
**Status**: Phase 4.5 in progress (7 of ~60 runbooks complete)
