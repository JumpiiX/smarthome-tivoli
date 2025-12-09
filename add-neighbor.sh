#!/bin/bash

# Script to add a neighbor's apartment (fake instance for monitoring demo)

if [ $# -eq 0 ]; then
    echo "Usage: $0 <apartment_number> [neighbor_name]"
    echo "Example: $0 002 smith"
    echo "Example: $0 003 johnson"
    exit 1
fi

APARTMENT_ID=$1
NEIGHBOR_NAME=${2:-neighbor$1}

echo "🏠 Adding apartment ${APARTMENT_ID} for ${NEIGHBOR_NAME}..."

# Create namespace
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: Namespace
metadata:
  name: apartment-${APARTMENT_ID}
  labels:
    tenant: apartment-${APARTMENT_ID}
    monitoring: enabled
    apartment-id: "${APARTMENT_ID}"
    apartment-type: "fake"
    neighbor-name: "${NEIGHBOR_NAME}"
EOF

# Create fake KNX bridge deployment (lightweight, no real KNX connection)
cat <<EOF | kubectl apply -f -
apiVersion: apps/v1
kind: Deployment
metadata:
  name: knx-homekit-bridge
  namespace: apartment-${APARTMENT_ID}
  labels:
    app: knx-homekit-bridge
    tenant: apartment-${APARTMENT_ID}
    apartment-type: "fake"
spec:
  replicas: 1
  selector:
    matchLabels:
      app: knx-homekit-bridge
      tenant: apartment-${APARTMENT_ID}
  template:
    metadata:
      labels:
        app: knx-homekit-bridge
        tenant: apartment-${APARTMENT_ID}
        apartment-type: "fake"
        neighbor-name: "${NEIGHBOR_NAME}"
    spec:
      containers:
      - name: knx-homekit-bridge-fake
        # Use nginx as a lightweight fake service that responds to health checks
        image: nginx:alpine
        ports:
        - containerPort: 80
        env:
        - name: TENANT_ID
          value: "apartment-${APARTMENT_ID}"
        - name: APARTMENT_ID
          value: "${APARTMENT_ID}"
        - name: NEIGHBOR_NAME
          value: "${NEIGHBOR_NAME}"
        resources:
          limits:
            cpu: "50m"
            memory: "32Mi"
          requests:
            cpu: "10m"
            memory: "16Mi"
        # Simple nginx config to simulate API responses
        volumeMounts:
        - name: nginx-config
          mountPath: /etc/nginx/conf.d/default.conf
          subPath: nginx.conf
      volumes:
      - name: nginx-config
        configMap:
          name: fake-api-config
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: fake-api-config
  namespace: apartment-${APARTMENT_ID}
data:
  nginx.conf: |
    server {
        listen 80;
        location /health {
            add_header Content-Type application/json;
            return 200 '{"status": "ok", "apartment": "${APARTMENT_ID}", "neighbor": "${NEIGHBOR_NAME}"}';
        }
        location /devices {
            add_header Content-Type application/json;
            return 200 '{"total": 12, "devices": [{"name": "Living Room Light", "type": "Light"}, {"name": "Kitchen Dimmer", "type": "Dimmer"}]}';
        }
        location / {
            add_header Content-Type application/json;
            return 200 '{"message": "Fake KNX Bridge for ${NEIGHBOR_NAME} - Apartment ${APARTMENT_ID}"}';
        }
    }
EOF

# Create service
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: Service
metadata:
  name: knx-homekit-bridge-service
  namespace: apartment-${APARTMENT_ID}
  labels:
    tenant: apartment-${APARTMENT_ID}
spec:
  selector:
    app: knx-homekit-bridge
    tenant: apartment-${APARTMENT_ID}
  ports:
  - protocol: TCP
    port: 8080
    targetPort: 80
  type: ClusterIP
EOF

echo "✅ Apartment ${APARTMENT_ID} (${NEIGHBOR_NAME}) added successfully!"
echo "📊 Check status: kubectl get pods -n apartment-${APARTMENT_ID}"
echo "🔍 View in monitoring dashboard"