<script lang="ts">
    import PreviewCanvas from "../PreviewCanvas.svelte";
    import SkeletonScene from "$lib/components/skeleton/SkeletonScene.svelte";

    let showLines = false;
    let showModel = true;
</script>

<div class="grid grid-cols-4 mb-2">
    <span>Show Lines</span>
    <div>
        <input type="checkbox" bind:checked={showLines} />
    </div>
    <span>Show Model</span>
    <div class="ml-2">
        <input type="checkbox" bind:checked={showModel} />
    </div>
</div>
<PreviewCanvas>
    <SkeletonScene {showLines} {showModel} />
</PreviewCanvas>
