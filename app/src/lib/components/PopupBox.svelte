<script lang="ts">
    import { onMount } from "svelte";
    import { fade } from "svelte/transition";

    export let title: string;
    export let message: string | undefined = undefined;
    export let showTextInput = false;

    export let onSubmit: (ok: boolean, text: string) => void;

    let textInput: HTMLInputElement;

    let text = "";

    onMount(() => {
        // Prevent user from scrolling in the background
        document.activeElement.blur();
        const style = document.body.style;
        if (style.overflow != "hidden") {
            style.overflow = "hidden";
            return () => (style.overflow = "initial");
        }
    });

    function confirm(ok: boolean) {
        onSubmit(ok, text);
    }

    function keyDown(e: KeyboardEvent) {
        if (textInput) {
            textInput.focus();
        }

        if (e.code == "Enter") {
            confirm(true);
        } else if (e.code == "Escape") {
            confirm(false);
        }
    }
</script>

<svelte:window on:keydown={keyDown} />
<div
    class="fixed top-0 left-0 bg-black/60 w-full h-full z-20 overflow-y-auto overflow-x-hidden p-4 grid place-items-center"
    transition:fade={{ duration: 100 }}
>
    <div class="bg-neutral-600 p-4 rounded max-w-lg h-fit z-30">
        <div class="flex flex-row items-center mb-4">
            <h1 class="font-bold text-2xl">{title}</h1>
        </div>
        {#if message}
            <p>
                {message}
            </p>
        {/if}
        {#if showTextInput}
            <input
                bind:this={textInput}
                bind:value={text}
                class="text-input w-full"
                placeholder="Enter here"
            />
        {/if}
        <div class="mt-4 flex gap-4">
            <button class="btn w-full" on:click={() => confirm(false)}>
                Cancel
            </button>
            <button
                class="btn btn-primary w-full"
                on:click={() => confirm(true)}
            >
                Ok
            </button>
        </div>
    </div>
</div>
