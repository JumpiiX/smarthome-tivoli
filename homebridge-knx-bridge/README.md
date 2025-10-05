# homebridge-knx-bridge

Homebridge plugin for the KNX-HomeKit Bridge, connecting Enertex EibPC² KNX devices to Apple HomeKit.

## Prerequisites

1. **KNX-HomeKit Bridge** must be running:
   ```bash
   cd ../
   cargo run --release
   ```

2. **Homebridge** installed:
   ```bash
   sudo npm install -g homebridge
   ```

## Installation

### Option 1: Local Installation (Development)

```bash
cd homebridge-knx-bridge
npm install
npm link
```

Then add to your Homebridge `config.json`:

```json
{
  "platforms": [
    {
      "platform": "KNXBridge",
      "name": "KNX Bridge",
      "bridgeUrl": "http://localhost:8080"
    }
  ]
}
```

### Option 2: Manual Installation

1. Copy this folder to your Homebridge plugins directory:
   ```bash
   cp -r homebridge-knx-bridge ~/.homebridge/node_modules/
   ```

2. Install dependencies:
   ```bash
   cd ~/.homebridge/node_modules/homebridge-knx-bridge
   npm install
   ```

3. Update your `~/.homebridge/config.json`:
   ```json
   {
     "platforms": [
       {
         "platform": "KNXBridge",
         "name": "KNX Bridge",
         "bridgeUrl": "http://localhost:8080"
       }
     ]
   }
   ```

## Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `platform` | Must be `KNXBridge` | - |
| `name` | Name of the platform | `KNX Bridge` |
| `bridgeUrl` | URL of the KNX Bridge API | `http://localhost:8080` |

### Example config.json

```json
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
```

## Usage

1. Start the KNX Bridge:
   ```bash
   cargo run --release
   ```

2. Start Homebridge:
   ```bash
   homebridge
   ```

3. Add to Home app:
   - Open Home app on iPhone/iPad
   - Tap "+"  → "Add Accessory"
   - Tap "I Don't Have a Code"
   - Select your Homebridge
   - Enter PIN from config.json (default: 031-45-154)

## Supported Devices

- ✅ **Lights** - On/Off control
- ✅ **Dimmers** - On/Off control (brightness coming soon)
- ✅ **Window Coverings** - Open/Close (position control simplified)
- ✅ **Temperature Sensors** - Read-only temperature display
- ✅ **Fans** - On/Off control (speed levels coming soon)
- ✅ **Scenes** - Momentary activation switches

## Device Mapping

The plugin automatically discovers all devices from the KNX Bridge API:

```javascript
GET http://localhost:8080/devices
```

Example response:
```json
{
  "devices": [
    {
      "key": "Single_3_page01",
      "id": "Single_3",
      "name": "Eingang",
      "device_type": "Light",
      "page": "01",
      "state": {
        "type": "onoff",
        "on": false
      }
    }
  ],
  "total": 26
}
```

## Troubleshooting

### Plugin not loading

Check Homebridge logs:
```bash
homebridge -D
```

### Devices not appearing

1. Verify KNX Bridge is running:
   ```bash
   curl http://localhost:8080/devices
   ```

2. Check the bridge URL in config.json

3. Restart Homebridge:
   ```bash
   sudo systemctl restart homebridge
   ```

### Control not working

1. Test the API directly:
   ```bash
   curl -X POST http://localhost:8080/device/Single_3_page01/toggle \
        -H "Content-Type: application/json" \
        -d '{"on": true}'
   ```

2. Check KNX Bridge logs for errors

3. Verify device key is correct

## Development

```bash
# Install dependencies
npm install

# Link for local testing
npm link

# Watch for changes
npm run test
```

## API Endpoints Used

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/devices` | GET | Discover all devices |
| `/device/:key` | GET | Get device info |
| `/device/:key/state` | GET | Get device state |
| `/device/:key/toggle` | POST | Toggle device on/off |

## License

MIT
