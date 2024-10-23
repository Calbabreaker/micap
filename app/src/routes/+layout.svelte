<script lang="ts">
    import NavBar from "$lib/components/NavBar.svelte";
    import {
        connectWebsocket,
        websocket,
        websocketConnected,
    } from "$lib/websocket";
    import { onMount } from "svelte";
    import "../app.css";
    import { SvelteToast } from "@zerodevx/svelte-toast";
    import PopupBox from "$lib/components/inputs/PopupBox.svelte";
    import { popupState } from "$lib/toast";
    import Popup from "$lib/components/inputs/Popup.svelte";

    onMount(() => {
        // Try to connect to websocket every so often
        connectWebsocket();
        const interval = setInterval(connectWebsocket, 1000);
        // Code to prevent problems when hot reloading
        return () => {
            websocket?.close();
            clearInterval(interval);
        };
    });

    function onSubmit(ok: boolean, text: string) {
        $popupState?.onSubmit!(ok, text);
        $popupState = undefined;
    }
</script>

<div class="p-4">
    <SvelteToast />
    {#if $popupState}
        <PopupBox
            title={$popupState.title}
            message={$popupState.message}
            showTextInput={$popupState.showTextInput}
            {onSubmit}
        />
    {/if}
    <NavBar />
    {#if !$websocketConnected}
        <Popup>Connecting to the server...</Popup>
    {/if}
    <main
        class="flex flex-col justify-center md:flex-row items-center md:items-start gap-4"
    >
        <slot />
    </main>
</div>
