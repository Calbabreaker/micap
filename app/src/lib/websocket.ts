import { writable } from "svelte/store";

interface TrackerConfig {
    name: string;
}

interface TrackerInfo {
    id: string;
    index: number;
    status: "Ok" | "Error" | "Off" | "TimedOut";
    config: TrackerConfig;
}

export interface Tracker {
    info: TrackerInfo;
}

export const websocket = writable<WebSocket>();
export const trackers = writable<Tracker[]>([]);
export const websocketError = writable("");

websocket.subscribe((websocket) => {
    if (websocket) {
        websocket.addEventListener("message", (msg) => {
            let message = JSON.parse(msg.data);
            if (message) {
                handleMessage(message);
            }
        });
    }
});

function handleMessage(message: Record<string, any>) {
    console.log(message);
    switch (message.type) {
        case "Error":
            websocketError.set(message.error);
            break;
        case "TrackerInfo":
            message.info.index;
            trackers.update((trackers) => {
                const tracker: Tracker = {
                    info: message.info,
                };

                trackers[message.info.index] = tracker;
                return trackers;
            });

            break;
    }
}
