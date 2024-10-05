import { writable } from "svelte/store";
import { confirmPopup, errorToast, infoToast } from "./toast";
import type {
    BoneLocation,
    GlobalConfig,
    Tracker,
    TrackerConfig,
    WebsocketClientMessage,
    Bone,
    WebsocketServerMessage,
    GlobalConfigUpdate,
} from "./server_bindings";

const WEBSOCKET_PORT = 8298;

export type TrackerDict = { [id in string]?: Tracker };
export type BoneDict = { [id in BoneLocation]: Bone };

export const trackers = writable<TrackerDict>({});
export const bones = writable<BoneDict>();
export const globalConfig = writable<GlobalConfig | undefined>();
export const defaultConfig = writable<GlobalConfig | undefined>();

export const serialPortName = writable<string | undefined>();
export const serialLog = writable<string[]>([]);

export let websocket: WebSocket | undefined;
export const websocketConnected = writable(false);

export function sendWebsocket(object: WebsocketClientMessage) {
    if (websocket && websocket.readyState == WebSocket.OPEN) {
        websocket.send(JSON.stringify(object));
    } else {
        errorToast("Websocket is not connected");
    }
}

export function connectWebsocket() {
    if (websocket) {
        return;
    }

    const protocol = location.protocol === "https:" ? "wss" : "ws";
    websocket = new WebSocket(`${protocol}://localhost:${WEBSOCKET_PORT}`);

    websocket.onopen = () => {
        console.log("Connected to websocket");
        websocketConnected.set(true);
    };

    websocket.onclose = () => {
        console.log("Websocket connection closed");
        websocket = undefined;
        websocketConnected.set(false);
        trackers.set({});
    };

    websocket.onerror = () => {
        console.log("Websocket error");
        websocket!.close();
    };

    websocket.onmessage = (event) => {
        const message = JSON.parse(event.data);
        if (message) {
            handleMessage(message);
        }
    };
}

function updateConfig(editFunc: (config: GlobalConfig) => void, configUpdate: GlobalConfigUpdate) {
    globalConfig.update((globalConfig) => {
        if (globalConfig) {
            sendWebsocket({
                type: "UpdateConfig",
                config: configUpdate,
            });

            // @ts-ignore
            editFunc(globalConfig);
        }

        return globalConfig;
    });
}

export function editConfig<K extends keyof GlobalConfig>(field: K, config: GlobalConfig[K]) {
    // @ts-ignore
    updateConfig((gc) => (gc[field] = config), { [field]: config });
}

export async function removeTracker(id: string) {
    const message =
        "This will also prevent the device from connecting to the server once all the associated trackers are removed as well.";
    await confirmPopup("Are you sure you want to remove the tracker?", message);
    sendWebsocket({
        type: "RemoveTracker",
        id,
    });

    globalConfig.update((config) => {
        delete config?.trackers[id];
        return config;
    });
    trackers.update((trackers) => {
        delete trackers[id];
        return trackers;
    });
}

export function editTrackerConfig(id: string, config: TrackerConfig) {
    updateConfig((gc) => (gc.trackers[id] = config), { trackers: { [id]: config } });
}

function handleMessage(message: WebsocketServerMessage) {
    switch (message.type) {
        case "Error":
            errorToast(message.error);
            console.error("Error from server: " + message.error);
            break;
        case "SerialPortChanged":
            serialPortName.set(message.port_name);
            if (message.port_name) {
                infoToast(`Serial port ${message.port_name} has been connected`);
            } else {
                infoToast(`Serial port has been disconnected`);
            }
            break;
        case "SerialLog":
            serialLog.update((log) => {
                if (log.length > 100) {
                    // Keep log size to less than
                    log.shift();
                }

                log.push(message.log);
                return log;
            });

            const status = getSerialStatusMessage(message.log);
            if (status) {
                infoToast(`Serial device: ${status}`);
            }
            break;
        case "SkeletonUpdate":
            bones.set(message.bones as BoneDict);
            break;
        case "InitialState":
            globalConfig.set(message.config);
            serialPortName.set(message.port_name);
            defaultConfig.set(message.default_config);
            trackers.set(message.trackers);
            break;
        case "TrackerUpdate":
            trackers.update((trackers) => {
                Object.entries(message.trackers).forEach(([id, tracker]) => {
                    if (!trackers[id]) {
                        infoToast(`New tracker connected from ${tracker!.info.address}`);
                    }

                    trackers[id] = tracker;
                });

                return trackers;
            });
            break;
    }
}

function getSerialStatusMessage(message: string): string {
    switch (message) {
        case "WifiConnecting":
            return "Connecting to the WiFi network";
        case "WifiConnectOk":
            return "Connected to the WiFi network";
        case "WifiConnectTimeout":
            return "Failed to connect to the WiFi network, reconnecting to previously saved";
        case "Connected":
            return "Connecting to the server";
        case "Restarting":
            return "Restarting";
        default:
            return "";
    }
}
