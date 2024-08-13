import { get, writable } from "svelte/store";

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

export interface TrackerInfo {
    status: TrackerStatus;
    config: TrackerConfig;
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

export const websocket = writable<WebSocket | undefined>();
export const trackers = writable<Map<string, Tracker>>(new Map());

export function sendWebsocket(object: Record<string, any>) {
    let ws = get(websocket);
    if (ws) {
        ws.send(JSON.stringify(object));
    }
}

function connectWebsocket() {
    if (typeof window !== "undefined") {
        const protocol = location.protocol === "https:" ? "wss" : "ws";
        websocket.set(new WebSocket(`${protocol}://localhost:${WEBSOCKET_PORT}`));
    }
}

websocket.subscribe((ws) => {
    if (ws) {
        ws.onopen = () => {
            console.log("Connected to websocket");
        };

        ws.onclose = () => {
            console.log("Websocket connection closed");
            trackers.set(new Map());
            websocket.set(undefined);
        };

        ws.onerror = () => {
            console.log("Websocket error");
            ws.close();
        };

        ws.onmessage = (event) => {
            const message = JSON.parse(event.data);
            if (message) {
                handleMessage(message);
            }
        };
    } else {
        connectWebsocket();
    }
});

function handleMessage(message: Record<string, any>) {
    switch (message.type) {
        case "Error":
            alert(message.error);
            break;
        case "TrackerInfo":
            trackers.update((trackers) => {
                const tracker = trackers.get(message.id);

                if (!message.info) {
                    trackers.delete(message.id);
                } else if (tracker) {
                    tracker.info = message.info;
                } else {
                    trackers.set(message.id, { info: message.info });
                }

                return trackers;
            });

            break;
        case "TrackerData":
            trackers.update((trackers) => {
                const tracker = trackers.get(message.id);
                if (tracker) {
                    tracker.data = message.data;
                }
                return trackers;
            });

            break;
    }
}
