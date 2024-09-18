<script lang="ts">
    import { globalConfig, updateConfig } from "$lib/websocket";
    import type { VmcConfig } from "$lib/server_bindings";

    let enabled: boolean;
    let sendPort: number;

    function setConfigState(config: VmcConfig) {
        sendPort = config.send_port;
        enabled = config.enabled;
    }

    $: if ($globalConfig) setConfigState($globalConfig.vmc);

    function setVmcConfig() {
        updateConfig((globalConfig) => {
            globalConfig.vmc = {
                enabled,
                send_port: Number(sendPort),
                receive_port: Number(sendPort),
            };
        });
    }
</script>

<form on:change={setVmcConfig}>
    <div class="mb-2">
        <span class="w-20 inline-block">Enabled</span>
        <input type="checkbox" bind:checked={enabled} class="w-4" />
    </div>
    <div class="mb-2">
        <span class="w-20 inline-block">Send port</span>
        <input
            placeholder="Send port"
            bind:value={sendPort}
            class="text-input"
            type="number"
            disabled={!enabled}
        />
    </div>
</form>
