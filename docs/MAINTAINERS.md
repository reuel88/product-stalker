# Maintainer Guide: Release Signing

This document covers how repository maintainers should handle the Tauri signing keys used for secure auto-updates.

## Overview

Product Stalker uses [tauri-plugin-updater](https://v2.tauri.app/plugin/updater/) to provide secure auto-updates. This requires cryptographic signing of release artifacts:

- **Private key**: Used during CI builds to sign release artifacts
- **Public key**: Embedded in the app to verify downloaded updates

When users update the app, it verifies the signature before installing, ensuring the update hasn't been tampered with.

## Key Storage Locations

### Public Key

Stored in version control at `apps/desktop/src-tauri/tauri.conf.json`:

```json
{
  "plugins": {
    "updater": {
      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6..."
    }
  }
}
```

This is safe to commit publicly as it's only used for verification.

### Private Key

**Never commit the private key to version control.**

| Location | Purpose |
|----------|---------|
| `~/.tauri/product-stalker.key` (local) | Backup on maintainer's machine |
| GitHub Secrets | Used by CI during releases |

## Required GitHub Secrets

Configure these in your repository: **Settings > Secrets and variables > Actions > New repository secret**

| Secret Name | Value | Required |
|-------------|-------|----------|
| `TAURI_SIGNING_PRIVATE_KEY` | Full contents of the `.key` file | Yes |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Password used during key generation (empty string if none) | Yes |

Direct link: `https://github.com/reuel88/product-stalker/settings/secrets/actions`

## How to Generate New Keys

If you need to generate keys for the first time or regenerate after compromise:

### Step 1: Generate the Key Pair

```bash
npx tauri signer generate -w ~/.tauri/product-stalker.key
```

You'll be prompted for a password (optional but recommended). The command outputs:
- Private key saved to the specified path
- Public key displayed in the terminal

### Step 2: Update the Public Key

Copy the displayed public key and update `apps/desktop/src-tauri/tauri.conf.json`:

```json
{
  "plugins": {
    "updater": {
      "pubkey": "YOUR_NEW_PUBLIC_KEY_HERE"
    }
  }
}
```

Commit this change to the repository.

### Step 3: Update GitHub Secrets

1. Go to **Settings > Secrets and variables > Actions**
2. Update `TAURI_SIGNING_PRIVATE_KEY`:
   ```bash
   # On macOS/Linux, copy the file contents:
   cat ~/.tauri/product-stalker.key | pbcopy  # macOS
   cat ~/.tauri/product-stalker.key | xclip   # Linux

   # On Windows PowerShell:
   Get-Content ~/.tauri/product-stalker.key | Set-Clipboard
   ```
3. Update `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` with the password you used (or empty string)

### Step 4: Secure Backup

Store the private key securely:
- Password manager (1Password, Bitwarden, etc.)
- Encrypted backup drive
- Share with other maintainers via secure channel if needed

## Key Rotation Procedure

Rotate keys periodically or immediately if compromised:

1. **Generate new key pair** (see above)
2. **Update GitHub Secrets** with new private key
3. **Update public key** in `tauri.conf.json` and commit
4. **Release a new version** with the new public key
5. **Notify users** if rotation was due to compromise
6. **Delete old key** from local storage

**Important:** After key rotation, users on older versions will need to manually download the new version since the old public key won't verify new signatures.

## Troubleshooting

### Build fails with signing error

**Symptoms:**
```
Error: Failed to sign
```

**Solutions:**
1. Verify `TAURI_SIGNING_PRIVATE_KEY` secret contains the full key file contents
2. Verify `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` matches what was used during generation
3. Ensure no trailing whitespace or newlines in the secrets

### Updates fail signature verification

**Symptoms:**
- Users see "Update verification failed" or similar
- App refuses to install update

**Solutions:**
1. Ensure the public key in `tauri.conf.json` matches the private key used to sign
2. Verify the release was built with the correct secrets
3. Check that `latest.json` contains valid signatures

### Lost private key

If the private key is lost and not backed up:

1. Generate a new key pair
2. Update secrets and public key
3. Release a new version
4. Inform users to manually download the new release

Users on previous versions will not be able to auto-update; they must manually download.

### "Secret not found" in CI

**Symptoms:**
```
Error: Input required and not supplied: TAURI_SIGNING_PRIVATE_KEY
```

**Solutions:**
1. Verify the secret name is exactly `TAURI_SIGNING_PRIVATE_KEY` (case-sensitive)
2. Check that secrets are accessible to the workflow (not restricted by environment protection rules)
3. For forks: secrets are not available in PRs from forks for security reasons

## Security Best Practices

1. **Limit access**: Only grant repository admin access to trusted maintainers
2. **Use strong password**: Protect the private key with a strong password
3. **Rotate periodically**: Consider rotating keys annually
4. **Audit access**: Review who has access to secrets periodically
5. **Monitor releases**: Watch for unauthorized releases

## Related Documentation

- [CI/CD Plan](./CI_CD_PLAN.md) - Full CI/CD workflow documentation
- [Tauri Updater Plugin](https://v2.tauri.app/plugin/updater/) - Official documentation
- [Tauri Code Signing](https://v2.tauri.app/distribute/sign/) - Signing guide
