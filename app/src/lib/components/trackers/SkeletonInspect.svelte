<script lang="ts">
    import PreviewCanvas from "./PreviewCanvas.svelte";
    import SkeletonScene from "./SkeletonScene.svelte";

    let showLines = true;
    let showModel = true;
</script>

<div class="flex gap-2 flex-col mb-4">
    <div class="flex justify-center">
        <span class="w-32 inline-block">Show Lines</span>
        <input type="checkbox" bind:checked={showLines} />
    </div>
    <div class="flex justify-center">
        <span class="w-32 inline-block">Show Model</span>
        <input type="checkbox" bind:checked={showModel} />
    </div>
</div>
<PreviewCanvas>
    <SkeletonScene {showLines} {showModel} />
</PreviewCanvas>
