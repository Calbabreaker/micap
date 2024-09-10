<script lang="ts">
    import { onMount } from "svelte";
    import { fade } from "svelte/transition";

    export let title: string;
    export let message: string;

    export let onClick: (ok: boolean) => void;

    onMount(() => {
        // Prevent user from scrolling in the background
        const style = document.body.style;
        if (style.overflow != "hidden") {
            style.overflow = "hidden";
            return () => (style.overflow = "initial");
        }
    });
</script>

<div
    class="fixed top-0 left-0 bg-black/60 w-full h-full z-20 overflow-y-auto overflow-x-hidden p-4 grid place-items-center"
    transition:fade={{ duration: 100 }}
>
    <div class="bg-neutral-600 p-4 rounded max-w-lg h-fit z-30">
        <div class="flex flex-row items-center mb-1">
            <h1 class="font-bold text-2xl">{title}</h1>
        </div>
        <p class="flex-1 mt-4">
            {message}
        </p>
        <div class="mt-4 flex gap-4">
            <button class="btn w-full" on:click={() => onClick(false)}>
                Cancel
            </button>
            <button
                class="btn btn-primary w-full"
                on:click={() => onClick(true)}
            >
                Ok
            </button>
        </div>
    </div>
</div>
