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
    removed: boolean;
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
export const trackers = writable<Tracker[]>([]);

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
            trackers.set([]);
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
                if (trackers[message.index]) {
                    trackers[message.index].info = message.info;
                } else {
                    trackers[message.index] = { info: message.info };
                }

                return trackers;
            });

            break;
        case "TrackerData":
            trackers.update((trackers) => {
                trackers[message.index].data = message.data;
                return trackers;
            });

            break;
    }
}
