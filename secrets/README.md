This directory stores encrypted local-development secrets for the preferred
SOPS workflow.

Use SOPS to edit:

```bash
sops secrets/neoland.sops.env
```

Never commit plaintext `.env` files here.

If you are not using SOPS, keep plaintext local env files outside version
control.
