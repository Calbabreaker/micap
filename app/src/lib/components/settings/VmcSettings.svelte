<script lang="ts">
    import { globalConfig, setConfig, type GlobalConfig } from "$lib/websocket";

    let enabled = false;
    let marionettePort = 39540;

    function setVmcState(globalConfig: GlobalConfig) {
        marionettePort = globalConfig.vmc.marionette_port;
        enabled = globalConfig.vmc.enabled;
    }

    $: if ($globalConfig) setVmcState($globalConfig);

    function setVmcConfig() {
        console.log(enabled);
        setConfig((globalConfig) => {
            globalConfig.vmc = {
                enabled,
                marionette_port: Number(marionettePort),
            };
        });
    }
</script>

<form on:submit|preventDefault={setVmcConfig}>
    <div class="mb-2">
        <span class="w-20 inline-block">Enabled</span>
        <input type="checkbox" bind:checked={enabled} />
    </div>
    <div class="mb-2">
        <span class="w-20 inline-block">Port</span>
        <input
            placeholder="Marionette port"
            bind:value={marionettePort}
            class="p-2 rounded text-black text-sm"
            type="number"
            disabled={!enabled}
        />
    </div>
    <button type="submit" class="btn btn-primary mr-2 w-full"> Apply </button>
</form>
