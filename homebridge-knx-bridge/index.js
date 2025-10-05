const fetch = require('node-fetch');

let Service, Characteristic;

module.exports = function(homebridge) {
    Service = homebridge.hap.Service;
    Characteristic = homebridge.hap.Characteristic;
    homebridge.registerPlatform('homebridge-knx-bridge', 'KNXBridge', KNXBridgePlatform);
};

class KNXBridgePlatform {
    constructor(log, config, api) {
        this.log = log;
        this.config = config;
        this.api = api;

        this.bridgeUrl = config.bridgeUrl || 'http://localhost:8080';
        this.accessories = [];

        this.log('KNX Bridge Platform initialized');

        this.api.on('didFinishLaunching', () => {
            this.log('Discovering devices from KNX Bridge...');
            this.discoverDevices();
        });
    }

    async discoverDevices() {
        try {
            const response = await fetch(`${this.bridgeUrl}/devices`);
            const data = await response.json();

            this.log(`Discovered ${data.total} devices`);

            data.devices.forEach(device => {
                this.log(`  - ${device.name} (${device.device_type})`);
                this.addAccessory(device);
            });
        } catch (error) {
            this.log.error('Failed to discover devices:', error.message);
            this.log.error('Make sure the KNX Bridge is running at:', this.bridgeUrl);
        }
    }

    addAccessory(device) {
        const uuid = this.api.hap.uuid.generate(device.key);
        const existingAccessory = this.accessories.find(acc => acc.UUID === uuid);

        if (existingAccessory) {
            this.log(`Accessory already exists: ${device.name}`);
            return;
        }

        const accessory = new this.api.platformAccessory(device.name, uuid);
        accessory.context.device = device;

        switch (device.device_type) {
            case 'Light':
            case 'Switch':
            case 'Dimmer':
                this.addLightService(accessory, device);
                break;
            case 'TemperatureSensor':
                this.addTemperatureService(accessory, device);
                break;
            case 'WindowCovering':
                this.addWindowCoveringService(accessory, device);
                break;
            case 'Fan':
                this.addFanService(accessory, device);
                break;
            case 'Scene':
                this.addSceneService(accessory, device);
                break;
            default:
                this.log.warn(`Unsupported device type: ${device.device_type} for ${device.name}`);
                return;
        }

        this.api.registerPlatformAccessories('homebridge-knx-bridge', 'KNXBridge', [accessory]);
        this.accessories.push(accessory);
        this.log(`Added accessory: ${device.name}`);
    }

    addLightService(accessory, device) {
        const service = accessory.addService(Service.Lightbulb, device.name);

        const onCharacteristic = service.getCharacteristic(Characteristic.On);

        if (device.state.type === 'onoff') {
            onCharacteristic.updateValue(device.state.on);
        } else if (device.state.type === 'brightness') {
            onCharacteristic.updateValue(device.state.on);
        }

        onCharacteristic.on('set', async (value, callback) => {
            try {
                await this.toggleDevice(device.key, value);
                this.log(`${device.name} set to ${value ? 'ON' : 'OFF'}`);
                callback(null);
            } catch (error) {
                this.log.error(`Failed to set ${device.name}:`, error.message);
                callback(error);
            }
        });

        setInterval(async () => {
            try {
                const state = await this.getDeviceState(device.key);
                if (state.type === 'onoff') {
                    onCharacteristic.updateValue(state.on);
                } else if (state.type === 'brightness') {
                    onCharacteristic.updateValue(state.on);
                }
            } catch (error) {
            }
        }, 5000);
    }

    addTemperatureService(accessory, device) {
        const service = accessory.addService(Service.TemperatureSensor, device.name);

        const tempCharacteristic = service.getCharacteristic(Characteristic.CurrentTemperature);

        if (device.state.type === 'temperature') {
            tempCharacteristic.updateValue(device.state.celsius);
        }

        setInterval(async () => {
            try {
                const state = await this.getDeviceState(device.key);
                if (state.type === 'temperature') {
                    tempCharacteristic.updateValue(state.celsius);
                }
            } catch (error) {
            }
        }, 30000);
    }

    addWindowCoveringService(accessory, device) {
        const service = accessory.addService(Service.WindowCovering, device.name);

        const positionCharacteristic = service.getCharacteristic(Characteristic.CurrentPosition);
        const targetCharacteristic = service.getCharacteristic(Characteristic.TargetPosition);

        if (device.state.type === 'windowcovering') {
            positionCharacteristic.updateValue(device.state.position);
            targetCharacteristic.updateValue(device.state.position);
        }

        targetCharacteristic.on('set', async (value, callback) => {
            try {
                await this.setBlindPosition(device.key, value);
                positionCharacteristic.updateValue(value);
                this.log(`${device.name} set to ${value}%`);
                callback(null);
            } catch (error) {
                this.log.error(`Failed to set ${device.name}:`, error.message);
                callback(error);
            }
        });
    }

    addFanService(accessory, device) {
        const service = accessory.addService(Service.Fanv2, device.name);

        const activeCharacteristic = service.getCharacteristic(Characteristic.Active);

        if (device.state.type === 'onoff') {
            activeCharacteristic.updateValue(device.state.on ? 1 : 0);
        }

        activeCharacteristic.on('set', async (value, callback) => {
            try {
                const on = value === 1;
                await this.toggleDevice(device.key, on);
                this.log(`${device.name} set to ${on ? 'ON' : 'OFF'}`);
                callback(null);
            } catch (error) {
                this.log.error(`Failed to set ${device.name}:`, error.message);
                callback(error);
            }
        });
    }

    addSceneService(accessory, device) {
        const service = accessory.addService(Service.Switch, device.name);

        const onCharacteristic = service.getCharacteristic(Characteristic.On);
        onCharacteristic.updateValue(false);

        onCharacteristic.on('set', async (value, callback) => {
            if (value) {
                try {
                    await this.toggleDevice(device.key, true);
                    this.log(`${device.name} activated`);

                    setTimeout(() => {
                        onCharacteristic.updateValue(false);
                    }, 100);

                    callback(null);
                } catch (error) {
                    this.log.error(`Failed to activate ${device.name}:`, error.message);
                    callback(error);
                }
            } else {
                callback(null);
            }
        });
    }

    async toggleDevice(deviceKey, on) {
        const response = await fetch(`${this.bridgeUrl}/device/${deviceKey}/toggle`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ on })
        });

        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        return await response.json();
    }

    async getDeviceState(deviceKey) {
        const response = await fetch(`${this.bridgeUrl}/device/${deviceKey}/state`);

        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        return await response.json();
    }

    async setBlindPosition(deviceKey, position) {
        const response = await fetch(`${this.bridgeUrl}/device/${deviceKey}/position`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ position })
        });

        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        return await response.json();
    }

    configureAccessory(accessory) {
        this.log('Loading accessory from cache:', accessory.displayName);
        this.accessories.push(accessory);
    }
}
