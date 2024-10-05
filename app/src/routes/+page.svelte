<script lang="ts">
    import Card from "$lib/components/Card.svelte";
    import SkeletonInspect from "$lib/components/skeleton/SkeletonPreview.svelte";
    import TrackerGrid from "$lib/components/trackers/TrackerGrid.svelte";
</script>

<Card title="Trackers">
    <TrackerGrid />
</Card>
<Card title="Skeleton Preview">
    <SkeletonInspect />
</Card>
