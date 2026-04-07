This directory stores encrypted local-development secrets.

Use SOPS to edit:

```bash
sops secrets/neoland.sops.env
```

Never commit plaintext `.env` files here.
