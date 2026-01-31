# Runbook: NeolandConfigError

**Alert**: `NeolandConfigError`
**Severity**: CRITICAL 🔴
**Response Time**: 0-5 minutes

---

## Symptoms

- **Alert**: Configuration validation failed or invalid config detected
- **User Impact**: Service may crash or behave incorrectly
- **Metrics**: `neoland_config_validation_errors_total > 0`
- **Common Causes**:
  - Invalid environment variable
  - Malformed JSON/YAML in ConfigMap
  - Missing required secret
  - Invalid database URL
  - Incompatible configuration version

---

## Investigation

```bash
# Check ConfigMap
kubectl get configmap neoland-config -n default -o yaml

# Check Secret
kubectl get secret neoland-secrets -n default -o yaml

# Check pod logs for config errors
kubectl logs -l app=neoland --tail=200 -n default | grep -i "config\|error\|invalid"

# Check environment variables
kubectl exec -it deployment/neoland -n default -- env | sort
```

## Resolution

### Scenario 1: Invalid ConfigMap

```bash
# Backup current config
kubectl get configmap neoland-config -n default -o yaml > /tmp/config-backup.yaml

# Edit and fix
kubectl edit configmap neoland-config -n default

# Restart pods to apply
kubectl rollout restart deployment/neoland -n default
```

### Scenario 2: Missing Secret

```bash
# Check which secret is missing
kubectl logs -l app=neoland -n default | grep "secret"

# Create missing secret
kubectl create secret generic neoland-secrets \
  --from-literal=DATABASE_URL="postgresql://..." \
  -n default
```

## Verification

```bash
# Check pods healthy
kubectl get pods -l app=neoland -n default
# Expected: READY 1/1

# Test config endpoint (if available)
curl http://neoland:3001/admin/config-status
```

---

**Last Updated**: 2026-01-31
**Maintainer**: voidnx team
