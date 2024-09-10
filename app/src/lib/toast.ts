import { toast } from "@zerodevx/svelte-toast";
import { writable } from "svelte/store";

interface PopupState {
    title: string;
    message: string;
    onClick: (ok: boolean) => void;
}

export let popupState = writable<PopupState | undefined>();

export function errorToast(message: string) {
    toast.push(message, {
        classes: ["toast", "error"],
        pausable: true,
        duration: 6000,
    });
}

export function infoToast(message: string) {
    toast.push(message, {
        classes: ["toast"],
        pausable: true,
        duration: 6000,
    });
}

export function confirmPopup(title: string, message: string): Promise<void> {
    return new Promise((resolve, reject) => {
        popupState.set({
            title,
            message,
            onClick: (ok) => {
                if (ok) {
                    resolve();
                } else {
                    reject("Pressed cancel");
                }
            },
        });
    });
}

export function promptPopup() {
    return new Promise((resolve, reject) => {
        popupState.set({
            title,
            message,
            onClick: (ok) => {
                if (ok) {
                    resolve();
                } else {
                    reject("Pressed cancel");
                }
            },
        });
    });
}
