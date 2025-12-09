#!/bin/bash

# Complete monitoring setup for multi-tenant KNX Bridge system

set -e

echo "🚀 Setting up Multi-Tenant KNX Bridge Monitoring System..."

# Check kubectl
if ! command -v kubectl &> /dev/null; then
    echo "❌ kubectl is not installed"
    exit 1
fi

# Create monitoring namespace
echo "📁 Creating monitoring namespace..."
kubectl create namespace monitoring --dry-run=client -o yaml | kubectl apply -f -

# Apply your main apartment setup
echo "🏠 Setting up your main apartment..."
kubectl apply -f k8s-multitenant.yaml

echo ""
echo "ℹ️  Your apartment is now running. To add neighbors later:"
echo "   ./add-neighbor.sh 002 smith"
echo "   ./add-neighbor.sh 003 johnson" 
echo "   # OR for full demo with 100 apartments:"
echo "   ./scale-demo.sh"

# Setup monitoring
echo "📊 Setting up Prometheus monitoring..."
kubectl apply -f monitoring/prometheus-config.yaml

echo "📈 Setting up Grafana dashboards..."
kubectl apply -f monitoring/grafana-config.yaml

# Wait for deployments
echo "⏳ Waiting for services to be ready..."
kubectl wait --for=condition=available --timeout=300s deployment/knx-homekit-bridge -n apartment-main || true
kubectl wait --for=condition=available --timeout=300s deployment/prometheus -n monitoring || true
kubectl wait --for=condition=available --timeout=300s deployment/grafana -n monitoring || true

# Port forward for access (background processes)
echo "🌐 Setting up port forwarding..."
echo "Starting port forwards in background..."
kubectl port-forward -n monitoring service/grafana 3000:3000 &
GRAFANA_PID=$!
kubectl port-forward -n monitoring service/prometheus 9090:9090 &
PROMETHEUS_PID=$!
kubectl port-forward -n apartment-main service/knx-homekit-bridge-service 8080:8080 &
MAIN_PID=$!

# Create script to stop port forwards
cat > stop-monitoring.sh << 'EOF'
#!/bin/bash
echo "🛑 Stopping port forwards..."
pkill -f "kubectl port-forward"
echo "✅ All port forwards stopped"
EOF

chmod +x stop-monitoring.sh

# Show status
echo ""
echo "✅ Multi-Tenant KNX Bridge Monitoring Setup Complete!"
echo ""
echo "📊 Access Points:"
echo "   • Grafana:           http://localhost:3000 (admin/admin123)"
echo "   • Prometheus:        http://localhost:9090"
echo "   • Your Apartment:    http://localhost:8080"
echo ""
echo "🔧 To update YOUR credentials:"
echo "   kubectl create secret generic knx-credentials \\"
echo "     --from-literal=username='your_real_username' \\"
echo "     --from-literal=password='your_real_password' \\"
echo "     -n apartment-main --dry-run=client -o yaml | kubectl apply -f -"
echo ""
echo "🏠 To add neighbors for monitoring demo:"
echo "   ./add-neighbor.sh 002 smith      # Add one neighbor"
echo "   ./scale-demo.sh                  # Add 99 fake neighbors (full demo)"
echo ""
echo "🛑 To stop all port forwards: ./stop-monitoring.sh"
echo ""
echo "📋 Monitor deployment status:"
kubectl get pods -n apartment-main
kubectl get pods -n monitoring

# Keep script running to maintain port forwards
echo ""
echo "🔄 Port forwards are running... Press Ctrl+C to stop"
trap "kill $GRAFANA_PID $PROMETHEUS_PID $MAIN_PID 2>/dev/null; echo 'Port forwards stopped.'; exit 0" INT
wait