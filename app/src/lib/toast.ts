import { toast } from "@zerodevx/svelte-toast";
import { writable } from "svelte/store";

interface PopupState {
    title: string;
    message?: string;
    showTextInput: boolean;
    onSubmit?: (ok: boolean, text: string) => void;
}

export let popupState = writable<PopupState | undefined>();

export function errorToast(message: string) {
    toast.push(message, {
        classes: ["toast", "error"],
        pausable: true,
        duration: 5000,
    });
}

export function infoToast(message: string) {
    toast.push(message, {
        classes: ["toast"],
        pausable: true,
        duration: 2000,
    });
}

// These promises will reject if the user cancels
export function confirmPopup(title: string, message: string | undefined): Promise<string> {
    return showPopup(title, message, false);
}

export function promptPopup(title: string, message: string | undefined): Promise<string> {
    return showPopup(title, message, true);
}

function showPopup(
    title: string,
    message: string | undefined,
    showTextInput: boolean,
): Promise<string> {
    return new Promise((resolve, reject) => {
        popupState.set({
            title,
            message,
            showTextInput,
            onSubmit: (ok, text) => {
                if (ok || (showTextInput && text)) {
                    resolve(text);
                } else {
                    reject("Pressed cancel");
                }
            },
        });
    });
}
