# Runbook: NeolandDeploymentFailed

**Alert**: `NeolandDeploymentFailed`
**Severity**: CRITICAL 🔴
**Response Time**: 0-5 minutes

---

## Symptoms

- **Alert**: Kubernetes deployment failed or stuck in progress
- **User Impact**: New version not deployed, service using old/broken version
- **Metrics**: `kube_deployment_status_condition{deployment="neoland", condition="Progressing", status="false"} == 1` for >10 minutes
- **Common Causes**:
  - Image pull failure (invalid tag, registry auth)
  - Pod crash on startup (application error)
  - Resource constraints (CPU/memory unavailable)
  - Configuration error in deployment manifest
  - Rolling update stuck (old pods not terminating)

---

## Investigation

```bash
# Check deployment status
kubectl rollout status deployment/neoland -n default --timeout=30s

# Check deployment events
kubectl describe deployment neoland -n default | tail -20

# Check ReplicaSet status
kubectl get rs -l app=neoland -n default

# Check pod status
kubectl get pods -l app=neoland -n default

# Common failure states:
# - ImagePullBackOff: Cannot pull image
# - CrashLoopBackOff: Application crashes on startup
# - Pending: Cannot schedule (resources)
# - ContainerCreating: Stuck creating (storage issue)
```

## Resolution

### Scenario 1: ImagePullBackOff

```bash
# Rollback to previous working version
kubectl rollout undo deployment/neoland -n default

# Check image exists
docker pull registry.example.com/neoland:v1.2.3

# Fix image tag in deployment
kubectl set image deployment/neoland neoland=registry.example.com/neoland:v1.2.3 -n default
```

### Scenario 2: CrashLoopBackOff

```bash
# Check logs
kubectl logs -l app=neoland --previous -n default

# Rollback if broken
kubectl rollout undo deployment/neoland -n default
```

### Scenario 3: Resource Constraints

```bash
# Scale down other deployments temporarily
kubectl scale deployment/other-app --replicas=1 -n default

# Or increase node capacity
kubectl get nodes -o wide
```

## Verification

```bash
# Check deployment completed
kubectl rollout status deployment/neoland -n default

# Verify all pods running
kubectl get pods -l app=neoland -n default
# Expected: All READY 1/1
```

---

**Last Updated**: 2026-01-31
**Maintainer**: voidnx team
