<script lang="ts">
    import NavBar from "$lib/components/NavBar.svelte";
    import { connectWebsocket, websocket } from "$lib/websocket";
    import { onMount } from "svelte";
    import "../app.css";
    import { SvelteToast } from "@zerodevx/svelte-toast";

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
    <NavBar />
    <slot />
</main>
