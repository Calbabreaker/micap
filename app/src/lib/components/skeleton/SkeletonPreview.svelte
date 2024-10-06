<script lang="ts">
    import PreviewCanvas from "../PreviewCanvas.svelte";
    import SkeletonScene from "$lib/components/skeleton/SkeletonScene.svelte";

    let showLines = false;
    let showModel = true;
</script>

<div class="grid grid-cols-[1fr_auto] mb-4 w-32">
    <span>Show Lines</span>
    <div>
        <input type="checkbox" bind:checked={showLines} />
    </div>

    <span>Show Model</span>
    <div>
        <input type="checkbox" bind:checked={showModel} />
    </div>
</div>
<PreviewCanvas>
    <SkeletonScene {showLines} {showModel} />
</PreviewCanvas>
