#!/bin/bash

# Script to create sealed secrets for KNX credentials
# This script should be run ON THE SERVER with real credentials

if [ -z "$SMARTHOME_USERNAME" ] || [ -z "$SMARTHOME_PASSWORD" ]; then
    echo "❌ Missing environment variables!"
    echo "Please set:"
    echo "  export SMARTHOME_USERNAME='your_real_username'"
    echo "  export SMARTHOME_PASSWORD='your_real_password'"
    exit 1
fi

echo "🔐 Creating sealed secret for apartment-main..."

# Install kubeseal CLI if not present
if ! command -v kubeseal &> /dev/null; then
    echo "📦 Installing kubeseal CLI..."
    wget https://github.com/bitnami-labs/sealed-secrets/releases/download/v0.24.0/kubeseal-0.24.0-linux-amd64.tar.gz
    tar xfz kubeseal-0.24.0-linux-amd64.tar.gz
    sudo mv kubeseal /usr/local/bin/
    rm kubeseal-0.24.0-linux-amd64.tar.gz
fi

# Create temporary secret file
kubectl create secret generic knx-credentials \
  --from-literal=username="$SMARTHOME_USERNAME" \
  --from-literal=password="$SMARTHOME_PASSWORD" \
  --namespace=apartment-main \
  --dry-run=client -o yaml > temp-secret.yaml

# Seal the secret
kubeseal -o yaml < temp-secret.yaml > apartment-main-sealed-secret.yaml

# Clean up
rm temp-secret.yaml

echo "✅ Sealed secret created: apartment-main-sealed-secret.yaml"
echo "📋 Apply with: kubectl apply -f apartment-main-sealed-secret.yaml"

# Apply immediately
kubectl apply -f apartment-main-sealed-secret.yaml

echo "🎉 Sealed secret deployed successfully!"