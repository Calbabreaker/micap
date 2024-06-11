import { writable } from "svelte/store";

export enum TrackerStatus {
    Ok = 0,
    Error = 1,
    Off = 2,
}

export interface Tracker {
    id: string;
    status: TrackerStatus;
}

export const websocket = writable<WebSocket | undefined>();

export const trackers = writable<Tracker[]>([]);
