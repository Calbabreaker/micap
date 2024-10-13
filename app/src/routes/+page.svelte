<script lang="ts">
    import Card from "$lib/components/Card.svelte";
    import Controls from "$lib/components/Controls.svelte";
    import SkeletonPreview from "$lib/components/skeleton/SkeletonPreview.svelte";
    import TrackerGrid from "$lib/components/trackers/TrackerGrid.svelte";
</script>

<Card title="Trackers">
    <div class="flex items-center flex-col gap-4">
        <TrackerGrid />
    </div>
</Card>
<div class="flex flex-col gap-4">
    <Card title="Controls">
        <Controls />
    </Card>
    <Card title="Skeleton Preview">
        <SkeletonPreview />
    </Card>
</div>
