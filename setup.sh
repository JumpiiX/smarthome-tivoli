#!/bin/bash
set -e

echo "🚀 KNX-HomeKit Bridge Setup"
echo "=============================="
echo ""

if [ ! -f .env ]; then
    echo "❌ Error: .env file not found!"
    echo "Please create .env from .env.example:"
    echo "  cp .env.example .env"
    echo "  nano .env"
    exit 1
fi

source .env

if [ -z "$SMARTHOME_USERNAME" ] || [ -z "$SMARTHOME_PASSWORD" ] || [ -z "$SMARTHOME_BASE_URL" ]; then
    echo "❌ Error: Missing credentials in .env"
    echo "Please configure SMARTHOME_USERNAME, SMARTHOME_PASSWORD, and SMARTHOME_BASE_URL"
    exit 1
fi

echo "✅ .env file found and validated"
echo ""

if [ ! -f device_mappings.toml ]; then
    echo "📡 Running auto-discovery..."
    echo "This will discover all devices in your apartment."
    echo ""

    docker build -t knx-bridge-temp .

    docker run --rm \
        -v "$(pwd):/app/output" \
        -e SMARTHOME_USERNAME="$SMARTHOME_USERNAME" \
        -e SMARTHOME_PASSWORD="$SMARTHOME_PASSWORD" \
        -e SMARTHOME_BASE_URL="$SMARTHOME_BASE_URL" \
        knx-bridge-temp --discover

    if [ -f device_mappings_auto.toml ]; then
        mv device_mappings_auto.toml device_mappings.toml
    fi

    if [ ! -f device_mappings.toml ]; then
        echo "❌ Auto-discovery failed. Please check your credentials."
        exit 1
    fi

    echo "✅ Device mappings created: device_mappings.toml"
    echo ""
fi

if [ ! -d homebridge ]; then
    echo "🏠 Setting up Homebridge config..."
    mkdir -p homebridge

    cat > homebridge/config.json << 'EOF'
{
  "bridge": {
    "name": "Homebridge",
    "username": "CC:22:3D:E3:CE:30",
    "port": 51826,
    "pin": "031-45-154"
  },
  "platforms": [
    {
      "platform": "KNXBridge",
      "name": "KNX Bridge",
      "bridgeUrl": "http://localhost:8080"
    }
  ]
}
EOF

    echo "✅ Homebridge config created"
    echo ""
fi

cd homebridge-knx-bridge && npm install && cd ..

echo "📦 Installing Homebridge plugin..."
mkdir -p homebridge/node_modules
cp -r homebridge-knx-bridge homebridge/node_modules/

echo "🐳 Starting Docker containers..."
docker-compose up -d --build

echo ""
echo "⏳ Waiting for services to start..."
sleep 10

echo ""
echo "🎉 Setup complete!"
echo "===================="
echo ""
echo "Services running:"
echo "  - KNX Bridge:  http://localhost:8080"
echo "  - Homebridge:  http://localhost:8581 (UI)"
echo ""
echo "To add to HomeKit:"
echo "  1. Open Home app on iPhone"
echo "  2. Tap '+' → Add Accessory"
echo "  3. Tap 'I Don't Have a Code'"
echo "  4. Select 'Homebridge'"
echo "  5. Enter PIN: 031-45-154"
echo ""
echo "View logs:"
echo "  docker-compose logs -f knx-bridge"
echo "  docker-compose logs -f homebridge"
echo ""
echo "Stop:"
echo "  docker-compose down"
echo ""
