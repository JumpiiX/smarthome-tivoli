# KNX HomeKit Bridge - Kubernetes Multi-Tenant Deployment

This project containerizes a Rust-based KNX smart home bridge application and deploys it in a multi-tenant Kubernetes environment with monitoring capabilities.

## Architecture Overview

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Tenant-1      │    │   Tenant-2      │    │   Monitoring    │
│                 │    │                 │    │                 │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │ KNX Bridge  │ │    │ │ KNX Bridge  │ │    │ │ Prometheus  │ │
│ │ Pod         │ │    │ │ Pod         │ │    │ │             │ │
│ └─────────────┘ │    │ └─────────────┘ │    │ └─────────────┘ │
│                 │    │                 │    │                 │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │ Service     │ │    │ │ Service     │ │    │ │ Grafana     │ │
│ │ (8080)      │ │    │ │ (8080)      │ │    │ │ (3000)      │ │
│ └─────────────┘ │    │ └─────────────┘ │    │ └─────────────┘ │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Project Structure

```
smarthome/
├── src/                        # Rust source code
│   ├── main.rs                 # Application entry point
│   ├── knx_client.rs           # KNX system client
│   ├── api_server.rs           # HTTP API server
│   └── ...
├── Dockerfile                  # Multi-stage Docker build
├── .dockerignore              # Docker build context optimization
├── device_mappings.toml       # KNX device configurations
│
├── k8s-deployment.yaml        # Basic single-tenant K8s deployment
├── k8s-multitenant.yaml       # Multi-tenant deployment with isolation
├── k8s-ingress.yaml           # Ingress configuration
├── k8s-namespace.yaml         # Namespace definitions
│
├── monitoring/
│   ├── prometheus-config.yaml  # Prometheus monitoring setup
│   └── grafana-config.yaml     # Grafana dashboards
│
├── deploy.sh                  # Single deployment script
├── setup-monitoring.sh        # Complete multi-tenant setup
└── stop-monitoring.sh         # Stop port forwards
```

## Features

### Multi-Tenancy
- **Namespace Isolation**: Each tenant runs in dedicated namespace
- **Network Policies**: Enforce tenant isolation at network level
- **Resource Quotas**: CPU and memory limits per tenant
- **Secret Management**: Separate credentials for each tenant

### Monitoring & Observability
- **Prometheus**: Metrics collection from all tenants
- **Grafana**: Multi-tenant dashboards and alerting
- **Custom Alerts**: KNX Bridge health and resource usage
- **Per-Tenant Metrics**: Isolated monitoring for each tenant

### Container Features
- **Multi-Stage Build**: Optimized image size with Rust builder stage
- **Security**: Non-root user execution
- **Chrome Integration**: Headless browser for KNX authentication
- **Persistent Sessions**: Chrome profile storage for session persistence

## Quick Start

### Prerequisites
- Kubernetes cluster (k3s, minikube, or full cluster)
- Docker installed locally
- kubectl configured for your cluster

### 1. Build and Deploy

```bash
# Build the Docker image
docker build -t knx-homekit-bridge:latest .

# Deploy multi-tenant setup with monitoring
./setup-monitoring.sh
```

### 2. Configure Credentials

Update credentials for each tenant:

```bash
# Tenant 1
kubectl create secret generic knx-credentials \
  --from-literal=username='your_username' \
  --from-literal=password='your_password' \
  -n tenant-1 --dry-run=client -o yaml | kubectl apply -f -

# Tenant 2  
kubectl create secret generic knx-credentials \
  --from-literal=username='your_username' \
  --from-literal=password='your_password' \
  -n tenant-2 --dry-run=client -o yaml | kubectl apply -f -
```

### 3. Access Services

After deployment, access points are available at:

- **Grafana Dashboard**: http://localhost:3000 (admin/admin123)
- **Prometheus**: http://localhost:9090
- **Tenant 1 API**: http://localhost:8081
- **Tenant 2 API**: http://localhost:8082

## Configuration

### Environment Variables

The application supports these environment variables:

- `SMARTHOME_USERNAME`: KNX system username
- `SMARTHOME_PASSWORD`: KNX system password  
- `RUST_LOG`: Logging level (default: "info,knx_homekit_bridge=debug")
- `TENANT_ID`: Identifier for multi-tenant setups

### Resource Limits

Each tenant is configured with:
- **CPU Limit**: 500m (0.5 cores)
- **Memory Limit**: 256Mi
- **CPU Request**: 100m  
- **Memory Request**: 128Mi

## API Endpoints

The KNX Bridge exposes these endpoints:

```bash
# Get all devices
GET /devices

# Get specific device state  
GET /device/{key}/state

# Toggle device on/off
POST /device/{key}/toggle
Body: {"on": true}

# Set blind position
POST /device/{key}/position  
Body: {"position": 50}

# Health check
GET /health
```

## Monitoring

### Prometheus Metrics

The system collects these key metrics:
- `up`: Service availability per tenant
- `container_memory_usage_bytes`: Memory consumption
- `container_cpu_usage_seconds_total`: CPU utilization
- `http_requests_total`: API request counts

### Grafana Dashboards

Pre-configured dashboards include:
- **Multi-Tenant Overview**: Status and resource usage across tenants
- **KNX Bridge Health**: Service availability and response times  
- **Resource Utilization**: CPU, memory, and network metrics
- **Alert Status**: Current firing alerts and notifications

### Alerts

Configured alerts:
- **KNXBridgeDown**: Service unavailable for >1 minute
- **HighMemoryUsage**: Memory usage >80% for >2 minutes
- **HighCPUUsage**: CPU usage >80% for >5 minutes

## Development

### Local Development

```bash
# Run locally (requires .env file)
cargo run

# Run in discovery mode
cargo run -- --discover

# Run in headless mode  
cargo run -- --headless
```

### Building Docker Image

```bash
# Standard build
docker build -t knx-homekit-bridge:latest .

# Build without cache
docker build --no-cache -t knx-homekit-bridge:latest .

# Multi-platform build
docker buildx build --platform linux/amd64,linux/arm64 -t knx-homekit-bridge:latest .
```

### Testing

```bash
# Test API endpoints
curl http://localhost:8080/devices
curl http://localhost:8080/health

# Check logs
kubectl logs -f deployment/knx-homekit-bridge -n tenant-1
```

## Troubleshooting

### Common Issues

1. **Container fails to start**
   ```bash
   kubectl describe pod <pod-name> -n <namespace>
   kubectl logs <pod-name> -n <namespace>
   ```

2. **Authentication issues**
   - Verify credentials in secrets
   - Check chrome_data volume mounting
   - Ensure network connectivity to KNX system

3. **Monitoring not working**
   ```bash
   kubectl get pods -n monitoring
   kubectl port-forward -n monitoring service/prometheus 9090:9090
   ```

### Debug Commands

```bash
# Check all deployments
kubectl get deployments --all-namespaces

# View service endpoints
kubectl get endpoints --all-namespaces

# Check network policies
kubectl get networkpolicies --all-namespaces

# Monitor resource usage
kubectl top pods --all-namespaces
```

## Security Considerations

- **Network Isolation**: NetworkPolicies prevent cross-tenant communication
- **RBAC**: Service accounts with minimal required permissions
- **Secret Management**: Credentials stored as Kubernetes secrets
- **Container Security**: Non-root user execution, read-only filesystem where possible
- **Image Scanning**: Regular vulnerability scans of base images

## Performance Tuning

- **Resource Requests/Limits**: Adjust based on actual usage patterns
- **HPA**: Horizontal Pod Autoscaler for demand-based scaling
- **Persistent Volumes**: Consider persistent storage for chrome_data
- **Network Optimization**: Tune Chrome browser arguments for performance

## Production Deployment

For production environments:

1. **Use external monitoring**: External Prometheus/Grafana setup
2. **Implement backup**: Regular backup of configurations and data  
3. **Set up alerting**: Integration with PagerDuty/Slack/email
4. **Enable audit logging**: Kubernetes audit logs for compliance
5. **Certificate management**: TLS certificates for all communications

## Contributing

1. Fork the repository
2. Create feature branch: `git checkout -b feature/new-feature`
3. Commit changes: `git commit -am 'Add new feature'`
4. Push branch: `git push origin feature/new-feature`
5. Submit pull request

## License

[Your License Here]

## Support

For issues and questions:
- Check logs: `kubectl logs -f deployment/knx-homekit-bridge -n <namespace>`
- Monitor status: Access Grafana dashboard
- Review metrics: Check Prometheus targets