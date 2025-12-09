# Secrets Management with Sealed Secrets

This directory contains the setup for secure credential management using Sealed Secrets.

## How Sealed Secrets Work

1. **Encrypt secrets locally** with cluster's public key
2. **Store encrypted secrets in git** safely 
3. **Controller decrypts** in cluster automatically
4. **No manual credential updates** needed

## Setup Process

### 1. On Server: Install Sealed Secrets Controller

```bash
# Install the controller
kubectl apply -f sealed-secrets-controller.yaml

# Wait for it to be ready
kubectl wait --for=condition=available --timeout=300s deployment/sealed-secrets-controller -n kube-system
```

### 2. On Server: Set Your Real Credentials

```bash
# Set environment variables with your actual credentials
export SMARTHOME_USERNAME="your_actual_username"
export SMARTHOME_PASSWORD="your_actual_password"
```

### 3. Create and Deploy Sealed Secret

```bash
# Make script executable and run
chmod +x create-sealed-secret.sh
./create-sealed-secret.sh
```

This will:
- Install `kubeseal` CLI tool
- Create temporary secret with your real credentials
- Encrypt it with cluster's public key
- Generate `apartment-main-sealed-secret.yaml`
- Apply the sealed secret to cluster
- Clean up temporary files

## Benefits

✅ **Git Safe**: Encrypted secrets can be committed to git
✅ **No Manual Updates**: Credentials automatically decrypted in cluster  
✅ **Secure**: Only your cluster can decrypt the secrets
✅ **Scalable**: Easy to manage multiple apartments/tenants
✅ **Professional**: Industry standard approach

## Generated Files

- `apartment-main-sealed-secret.yaml` - Encrypted secret for your apartment
- Future: `apartment-XXX-sealed-secret.yaml` for neighbor apartments

## Security Notes

- Sealed secrets are tied to the specific cluster
- Only the cluster that generated the encryption key can decrypt
- Safe to store in git repository
- Credentials never appear in plain text in git