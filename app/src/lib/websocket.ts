import { writable } from "svelte/store";
import { error } from "./toast";
import { onMount } from "svelte";

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
    data?: TrackerData;
}

export const trackers = writable<{ [id: string]: Tracker }>({});
export const info = writable("");
export const globalConfig = writable<GlobalConfig | undefined>();

let websocket: WebSocket | undefined;

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
        case "Info":
            info.set(message.info);
            break;
        case "InitialState":
            globalConfig.set(message.config);
            trackers.set(message.trackers);
            break;
        case "TrackerInfo":
            trackers.update((trackers) => {
                const tracker = trackers[message.id];
                if (tracker) {
                    tracker.info = message.info;
                } else {
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
