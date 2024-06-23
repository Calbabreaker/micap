import { writable } from "svelte/store";

const WEBSOCKET_PORT = 8298;

export interface TrackerConfig {
    name: string;
}

export interface TrackerInfo {
    id: string;
    index: number;
    status: "Ok" | "Error" | "Off" | "TimedOut";
    config: TrackerConfig;
}

export interface TrackerData {
    orientation: [number, number, number, number];
    acceleration: [number, number, number];
    velocity: [number, number, number];
    position: [number, number, number];
}

export interface Tracker {
    info: TrackerInfo;
    data?: TrackerData;
}

export const websocket = writable<WebSocket | undefined>();
export const trackers = writable<Tracker[]>([]);
export const websocketError = writable("");

function connectWebsocket() {
    if (typeof window !== "undefined") {
        const protocol = location.protocol === "https" ? "wss" : "ws";
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
            websocketError.set(message.error);
            break;
        case "TrackerInfo":
            trackers.update((trackers) => {
                const tracker: Tracker = {
                    info: message.info,
                };

                trackers[message.info.index] = tracker;
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
