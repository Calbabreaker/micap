import { get, writable } from "svelte/store";
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
export const websocketConnected = writable(false);

export const serialPortName = writable<string | undefined>();
export const serialLog = writable<string[]>([]);

export let websocket: WebSocket | undefined;

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

export function updateConfig(configUpdate: GlobalConfigUpdate) {
    globalConfig.update((config) => {
        if (config) {
            sendWebsocket({
                type: "UpdateConfig",
                config: configUpdate,
            });

            Object.entries(configUpdate).forEach(([field, value]) => {
                // @ts-ignore
                config[field] = value;
            });
        }

        return config;
    });
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
}

export function editTrackerConfig(id: string, config: TrackerConfig) {
    const trackerConfig = get(globalConfig)?.trackers;
    if (trackerConfig) {
        trackerConfig[id] = config;
        updateConfig({ trackers: trackerConfig });
    }
}

function handleMessage(message: WebsocketServerMessage) {
    switch (message.type) {
        case "Error":
            errorToast(message.error);
            console.error("Error from server: " + message.error);
            break;
        case "SerialPortChanged":
            serialPortName.set(message.port_name);
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
                infoToast(status);
            }
            break;
        case "SkeletonUpdate":
            bones.set(message.bones as BoneDict);
            break;
        case "InitialState":
            globalConfig.set(message.config);
            serialPortName.set(message.port_name);
            break;
        case "TrackerUpdate":
            // Notify new trackers
            const currentTrackers = get(trackers);
            if (Object.keys(currentTrackers).length !== 0) {
                Object.entries(message.trackers).forEach(([id, tracker]) => {
                    if (!currentTrackers[id]) {
                        infoToast(`New tracker connected from ${tracker!.info.address}`);
                    }
                });
            }

            trackers.set(message.trackers);
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
