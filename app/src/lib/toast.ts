import { toast } from "@zerodevx/svelte-toast";

export function error(message: string) {
    toast.push(message, {
        classes: ["toast-error"],
    });
}
