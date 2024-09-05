<script lang="ts">
    import NavBar from "$lib/components/NavBar.svelte";
    import { connectWebsocket } from "$lib/websocket";
    import { onMount } from "svelte";
    import "../app.css";
    import { SvelteToast } from "@zerodevx/svelte-toast";

    onMount(connectWebsocket);
</script>

<main class="p-4">
    <SvelteToast />
    <NavBar />
    <slot />
</main>
