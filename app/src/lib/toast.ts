import { toast } from "@zerodevx/svelte-toast";

export function error(message: string) {
    toast.push(message, {
        classes: ["toast", "error"],
        pausable: true,
        duration: 3000,
    });
}

export function info(message: string) {
    toast.push(message, {
        classes: ["toast"],
        pausable: true,
        duration: 3000,
    });
}
