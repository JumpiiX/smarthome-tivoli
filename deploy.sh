#!/bin/bash

# Deployment script for KNX HomeKit Bridge to Kubernetes

set -e

echo "🚀 Deploying KNX HomeKit Bridge to Kubernetes..."

# Check if kubectl is available
if ! command -v kubectl &> /dev/null; then
    echo "❌ kubectl is not installed or not in PATH"
    exit 1
fi

# Check if we can connect to cluster
if ! kubectl cluster-info &> /dev/null; then
    echo "❌ Cannot connect to Kubernetes cluster"
    exit 1
fi

echo "✅ Connected to Kubernetes cluster"

# Create namespace
echo "📁 Creating namespace..."
kubectl apply -f k8s-namespace.yaml

# Apply deployment
echo "🚢 Applying deployment..."
kubectl apply -f k8s-deployment.yaml -n smarthome

# Apply ingress
echo "🌐 Applying ingress..."
kubectl apply -f k8s-ingress.yaml -n smarthome

# Wait for deployment to be ready
echo "⏳ Waiting for deployment to be ready..."
kubectl wait --for=condition=available --timeout=300s deployment/knx-homekit-bridge -n smarthome

# Show status
echo "📊 Deployment status:"
kubectl get pods -n smarthome
kubectl get services -n smarthome
kubectl get ingress -n smarthome

echo ""
echo "✅ Deployment complete!"
echo "📱 Access your KNX bridge at: http://knx-bridge.local"
echo ""
echo "🔧 To update credentials:"
echo "   kubectl create secret generic knx-credentials \\"
echo "     --from-literal=username='your_username' \\"
echo "     --from-literal=password='your_password' \\"
echo "     -n smarthome --dry-run=client -o yaml | kubectl apply -f -"
echo ""
echo "📋 To view logs:"
echo "   kubectl logs -f deployment/knx-homekit-bridge -n smarthome"