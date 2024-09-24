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
    import PopupBox from "$lib/components/PopupBox.svelte";
    import { popupState } from "$lib/toast";
    import Popup from "$lib/components/Popup.svelte";

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
</script>

<main class="p-4">
    <SvelteToast />
    {#if $popupState}
        <PopupBox
            title={$popupState.title}
            message={$popupState.message}
            showTextInput={$popupState.showTextInput}
            onSubmit={(ok, text) => {
                $popupState.onSubmit(ok, text);
                $popupState = undefined;
            }}
        />
    {/if}
    {#if !$websocketConnected}
        <Popup>Connecting to the server...</Popup>
    {/if}
    <NavBar />
    <div
        class="flex flex-col justify-center md:flex-row items-center md:items-start gap-4"
    >
        <slot />
    </div>
</main>
