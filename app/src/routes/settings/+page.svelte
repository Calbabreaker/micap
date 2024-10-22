<script lang="ts">
    import Card from "$lib/components/Card.svelte";
    import InterfaceSettings from "$lib/components/settings/InterfaceSettings.svelte";
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
        <Card title="VRChat">
            <VrChatSettings />
        </Card>
        <Card title="Interface">
            <InterfaceSettings />
        </Card>
        <Card title="Skeleton offsets">
            <SkeletonOffsetSettings />
        </Card>
    {/if}
</div>
