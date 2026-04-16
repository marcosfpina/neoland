# SOPS Setup Guide

This repo supports local development secrets through an encrypted dotenv file:

- `secrets/neoland.sops.env`
- `secrets/*.sops.yaml`

The application itself still reads environment variables as before. The SOPS
integration only decrypts those variables immediately before launch.

## Why This Is Safe Enough For Local Development

- encrypted secrets are committed, not plaintext
- the private AGE key stays on your machine in `~/.config/sops/age/keys.txt`
- decrypted values are injected at runtime, not stored in the repo

## Files

```text
.sops.yaml
secrets/neoland.sops.env
secrets/cachix.sops.yaml
scripts/neoland-run.sh
```

## Edit Secrets

```bash
nix develop
sops secrets/neoland.sops.env
```

Or inside the dev shell:

```bash
neoland-secrets
```

## Run With Secrets

Inside `nix develop`, the standard shortcuts load SOPS secrets automatically:

```bash
neoland
nsrv
ncli
neoland-server
neoland-client
neoland-test
neoland-doctor
neoland-restart
```

They are also available as one-shot commands, for example:

```bash
nix develop --command neoland-server
nix develop --command neoland-doctor --json
```

They call `scripts/neoland-run.sh`, which:

1. decrypts `secrets/neoland.sops.env` if it exists
2. exports those values into the process environment
3. launches `cargo run --bin neoland -- ...`

## Override The Secret File

If you want a different encrypted file:

```bash
export NEOLAND_SOPS_ENV_FILE=/path/to/other.sops.env
nsrv
```

## Recommended Contents

Keep at least the Neoland API keys here:

```dotenv
NEOLAND_ADMIN_API_KEY=...
NEOLAND_USER_API_KEY=...
NEOLAND_READONLY_API_KEY=...
```

Optional provider keys can live here too:

```dotenv
DEEPSEEK_API_KEY=...
OPENAI_API_KEY=...
```

YAML files are also covered by the SOPS policy, which is useful for things like
Cachix credentials:

```yaml
cachix:
  auth_token: YOUR_TOKEN_HERE
```

Avoid putting empty `VAULT_ADDR` / `VAULT_TOKEN` values in the file, because the
app treats present environment variables as intentional configuration.
