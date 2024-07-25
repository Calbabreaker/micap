<script lang="ts">
    import NavBar from "$lib/components/nav_bar.svelte";
    import "../app.css";
</script>

<main class="p-4">
    <NavBar />
    <slot />
</main>
