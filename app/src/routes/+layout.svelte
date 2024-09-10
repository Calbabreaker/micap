<script lang="ts">
    import NavBar from "$lib/components/NavBar.svelte";
    import { connectWebsocket, websocket } from "$lib/websocket";
    import { onMount } from "svelte";
    import "../app.css";
    import { SvelteToast } from "@zerodevx/svelte-toast";
    import PopupBox from "$lib/components/PopupBox.svelte";
    import { popupState } from "$lib/toast";

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
            onClick={(ok) => {
                $popupState.onClick(ok);
                $popupState = undefined;
            }}
        />
    {/if}
    <NavBar />
    <slot />
</main>
