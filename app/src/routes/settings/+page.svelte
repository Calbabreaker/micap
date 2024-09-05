<script lang="ts">
    import Card from "$lib/components/Card.svelte";
    import { globalConfig, setConfig } from "$lib/websocket";

    $: marionettePort = $globalConfig?.vmc?.marionette_port ?? "";
    $: enabled = $globalConfig?.vmc?.enabled ?? false;

    function setVmcConfig() {
        setConfig((globalConfig) => {
            globalConfig.vmc = {
                enabled,
                marionette_port: Number(marionettePort),
            };
        });
    }
</script>

<Card title="Virtual Motion Capture">
    <form on:submit|preventDefault={setVmcConfig}>
        <div>
            Enabled:
            <input type="checkbox" bind:checked={enabled} />
        </div>
        <div>
            Port:
            <input
                placeholder="Marionette port"
                bind:value={marionettePort}
                class="p-2 rounded text-black"
                disabled={!enabled}
            />
        </div>
        <div>
            <button type="submit" class="btn btn-primary mr-2 inline-block">
                Apply
            </button>
        </div>
    </form>
</Card>
