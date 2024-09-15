<script lang="ts">
    import Popup from "./Popup.svelte";

    export let title: string;
    export let message: string | undefined = undefined;
    export let showTextInput = false;

    export let onSubmit: (ok: boolean, text: string) => void;

    let textInput: HTMLInputElement;

    let text = "";

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
<Popup>
    <h1 class="font-bold text-2xl mb-4 text-center">{title}</h1>
    {#if message}
        <p>{message}</p>
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
        <button class="btn btn-primary w-full" on:click={() => confirm(true)}>
            Ok
        </button>
    </div>
</Popup>
