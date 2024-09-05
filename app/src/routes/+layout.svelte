<script lang="ts">
    import NavBar from "$lib/components/NavBar.svelte";
    import "../app.css";
    import { SvelteToast } from "@zerodevx/svelte-toast";
</script>

<main class="p-4">
    <SvelteToast />
    <NavBar />
    <slot />
</main>
