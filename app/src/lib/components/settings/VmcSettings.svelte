<script lang="ts">
    import type { VmcConfig } from "$lib/server_bindings";
    import { globalConfig, editConfig, defaultConfig } from "$lib/websocket";
    import ResetButton from "./ResetButton.svelte";

    let enabled: boolean;
    let sendPort: number;

    function setConfigState(config: VmcConfig) {
        sendPort = config.send_port;
        enabled = config.enabled;
    }

    $: if ($globalConfig) setConfigState($globalConfig.vmc);

    function updateVmcConfig() {
        sendPort = Number(sendPort);
        if (sendPort == 0) {
            return;
        }

        editConfig("vmc", {
            enabled,
            send_port: sendPort,
            receive_port: sendPort,
        });
    }
</script>

<form
    class="grid grid-cols-[1fr_auto] gap-y-2 gap-x-4"
    on:change={updateVmcConfig}
>
    <span class="my-auto">Enabled</span>
    <div class="flex items-center gap-2">
        <input type="checkbox" bind:checked={enabled} />
        <ResetButton
            bind:value={enabled}
            defaultValue={$defaultConfig?.vmc?.enabled}
        />
    </div>

    <span class="my-auto">Send port</span>
    <div class="flex items-center gap-2">
        <input
            placeholder="Send port"
            bind:value={sendPort}
            class="text-input"
            type="number"
            disabled={!enabled}
        />
        <ResetButton
            bind:value={sendPort}
            defaultValue={$defaultConfig?.vmc?.send_port}
        />
    </div>
</form>
