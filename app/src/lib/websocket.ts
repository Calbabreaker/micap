import { writable } from "svelte/store";
import { error, info } from "./toast";

const WEBSOCKET_PORT = 8298;

// Copied from server
export const trackerLocations = [
    "Hip",
    "LeftUpperLeg",
    "RightUpperLeg",
    "LeftLowerLeg",
    "RightLowerLeg",
    "LeftFoot",
    "RightFoot",
    "Waist",
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

export type TrackerLocation = (typeof trackerLocations)[number];

export type TrackerStatus = "Ok" | "Error" | "Off" | "TimedOut";

export interface TrackerConfig {
    name: string;
    location?: TrackerLocation;
}

export interface GlobalConfig {
    trackers: { [id: string]: TrackerConfig };
    vmc: {
        enabled: boolean;
        marionette_port: number;
    };
}

export interface TrackerInfo {
    status: TrackerStatus;
    latency_ms?: number;
    battery_level: number;
}

export interface TrackerData {
    orientation: [number, number, number, number];
    acceleration: [number, number, number];
    position: [number, number, number];
}

export interface Tracker {
    info: TrackerInfo;
    data: TrackerData;
}

export const trackers = writable<{ [id: string]: Tracker }>({});
export const serialPortName = writable<string | undefined>();
export const serialLog = writable<string[]>([]);
export const globalConfig = writable<GlobalConfig | undefined>();

export let websocket: WebSocket | undefined;

export function sendWebsocket(object: Record<string, any>) {
    if (websocket && websocket.readyState == WebSocket.OPEN) {
        websocket.send(JSON.stringify(object));
    } else {
        error("Websocket is not connected");
    }
}

export function connectWebsocket() {
    const protocol = location.protocol === "https:" ? "wss" : "ws";
    websocket = new WebSocket(`${protocol}://localhost:${WEBSOCKET_PORT}`);

    websocket.onopen = () => {
        console.log("Connected to websocket");
    };

    websocket.onclose = () => {
        console.log("Websocket connection closed");
        trackers.set({});
        connectWebsocket();
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

export function setConfig(setFunc: (config: GlobalConfig) => void) {
    globalConfig.update((config) => {
        if (config) {
            setFunc(config);
            sendWebsocket({
                type: "UpdateConfig",
                config,
            });
        }

        return config;
    });
}

function handleMessage(message: Record<string, any>) {
    switch (message.type) {
        case "Error":
            error(message.error);
            break;
        case "SerialPort":
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
                info(status);
            }
            break;
        case "InitialState":
            globalConfig.set(message.config);
            trackers.set(message.trackers);
            serialPortName.set(message.port_name);
            break;
        case "TrackerInfo":
            trackers.update((trackers) => {
                const tracker = trackers[message.id];
                if (tracker) {
                    tracker.info = message.info;
                } else {
                    trackers[message.id] = {
                        info: message.info,
                        data: {
                            position: [0, 0, 0],
                            orientation: [0, 0, 0, 0],
                            acceleration: [0, 0, 0],
                        },
                    };
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
            return "Connecting to WiFi network";
        case "WifiConnectOk":
            return "Connected to WiFi network";
        case "WifiConnectTimeout":
            return "Failed to connect to WiFi network";
        case "Connecting":
            return "Connecting to server";
        case "Connected":
            return "Connected to server";
        default:
            return "";
    }
}
