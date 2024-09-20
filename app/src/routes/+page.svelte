<script lang="ts">
    import Card from "$lib/components/Card.svelte";
    import PreviewCanvas from "$lib/components/trackers/PreviewCanvas.svelte";
    import SkeletonScene from "$lib/components/trackers/SkeletonScene.svelte";
    import TrackerGrid from "$lib/components/trackers/TrackerGrid.svelte";
</script>

<Card title="Trackers">
    <TrackerGrid />
</Card>
<Card title="Skeleton Preview">
    <PreviewCanvas>
        <SkeletonScene />
    </PreviewCanvas>
</Card>
