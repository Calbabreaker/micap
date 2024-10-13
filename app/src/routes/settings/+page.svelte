<script lang="ts">
    import Card from "$lib/components/Card.svelte";
    import SkeletonOffsetSettings from "$lib/components/settings/SkeletonOffsetSettings.svelte";
    import VmcSettings from "$lib/components/settings/VmcSettings.svelte";
    import VrChatSettings from "$lib/components/settings/VrChatSettings.svelte";
    import { globalConfig } from "$lib/websocket";
</script>

<div class="flex flex-col gap-4 items-center">
    {#if $globalConfig}
        <Card title="Virtual Motion Capture">
            <VmcSettings />
        </Card>
        <Card title="VRChat Settings">
            <VrChatSettings />
        </Card>
        <Card title="Skeleton offsets">
            <SkeletonOffsetSettings />
        </Card>
    {/if}
</div>
