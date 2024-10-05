<script lang="ts">
    import Card from "$lib/components/Card.svelte";
    import SkeletonOffsetSettings from "$lib/components/settings/SkeletonOffsetSettings.svelte";
    import VmcSettings from "$lib/components/settings/VmcSettings.svelte";
</script>

<Card title="Virtual Motion Capture">
    <VmcSettings />
</Card>
<Card title="Skeleton offsets">
    <SkeletonOffsetSettings />
</Card>
