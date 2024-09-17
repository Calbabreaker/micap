import { writable } from "svelte/store";
import { confirmPopup, errorToast, infoToast } from "./toast";
import type {
    GlobalConfig,
    TrackerConfig,
    TrackerData,
    TrackerInfo,
    WebsocketClientMessage,
    WebsocketServerMessage,
} from "./server_bindings";

const WEBSOCKET_PORT = 8298;

// Copied from server
export const boneLocations = [
    "Hips",
    "LeftUpperLeg",
    "RightUpperLeg",
    "LeftLowerLeg",
    "RightLowerLeg",
    "LeftFoot",
    "RightFoot",
    "Spine",
    "Chest",
    "Neck",
    "Head",
    "LeftShoulder",
    "RightShoulder",
    "LeftUpperArm",
    "RightUpperArm",
    "LeftLowerArm",
    "RightLowerArm",
    "LeftHand",
    "RightHand",
];

interface Tracker {
    info: TrackerInfo;
    data?: TrackerData;
}

type TrackerDict = { [id: string]: Tracker };

export const trackers = writable<TrackerDict>({});
export const serialPortName = writable<string | undefined>();
export const serialLog = writable<string[]>([]);
export const globalConfig = writable<GlobalConfig | undefined>();

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
    };

    websocket.onclose = () => {
        console.log("Websocket connection closed");
        websocket = undefined;
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

export function updateConfig(updateFunc?: (config: GlobalConfig) => void) {
    globalConfig.update((config) => {
        if (config) {
            infoToast("Applied the config");
            if (updateFunc) {
                updateFunc(config);
            }

            sendWebsocket({
                type: "UpdateConfig",
                config,
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

    trackers.update((trackers) => {
        delete trackers[id];
        return trackers;
    });
    globalConfig.update((config) => {
        delete config?.trackers[id];
        return config;
    });
}

export function editTrackerConfig(id: string, editFunc: (config: TrackerConfig) => void) {
    updateConfig((config) => {
        if (config.trackers[id] == null) {
            config.trackers[id] = {};
        }

        editFunc(config.trackers[id]);
    });
}

function handleMessage(message: WebsocketServerMessage) {
    switch (message.type) {
        case "Error":
            errorToast(message.error);
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

            const status = getSerialStatus(message.log);
            if (status) {
                infoToast(status);
            }
            break;
        case "InitialState":
            globalConfig.set(message.config);
            serialPortName.set(message.port_name);
            trackers.update(() => {
                let trackers: TrackerDict = {};
                for (const [id, info] of Object.entries(message.tracker_infos)) {
                    trackers[id] = { info };
                }
                return trackers;
            });
            break;
        case "TrackerInfo":
            trackers.update((trackers) => {
                const tracker = trackers[message.id];
                if (tracker) {
                    tracker.info = message.info;
                } else {
                    infoToast(`New udp device connected from ${message.info.address}`);
                    trackers[message.id] = { info: message.info };
                }

                return trackers;
            });

            break;
        case "TrackerData":
            trackers.update((trackers) => {
                const tracker = trackers[message.id];
                if (tracker) {
                    tracker.data = message.data;
                }
                return trackers;
            });
            break;
    }
}

function getSerialStatus(message: string): string {
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
