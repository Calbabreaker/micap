<script lang="ts">
    import { sendWebsocket, serialPortName, serialLog } from "$lib/websocket";
    import WifiForm from "$lib/components/WifiForm.svelte";
    import Card from "$lib/components/Card.svelte";
    import { afterUpdate } from "svelte";

    let logElm: HTMLDivElement;
    afterUpdate(() => {
        if (
            logElm.scrollTop + logElm.clientHeight >
            logElm.scrollHeight - 100
        ) {
            logElm.scroll({ top: logElm.scrollHeight });
        }
    });
</script>

<Card title="Send WiFi credentials">
    <WifiForm />
</Card>
<Card title="Serial device">
    {#if $serialPortName}
        <p class="text-center">Connected to port {$serialPortName}</p>
    {:else}
        <p class="text-center">Not connected to any port</p>
    {/if}
    <div
        bind:this={logElm}
        class="font-mono text-xs bg-neutral-800 rounded p-2 mt-2 w-96 h-64 overflow-scroll"
    >
        {#each $serialLog as line}
            <p>{line}</p>
        {/each}
    </div>
    <button
        class="btn btn-primary mt-4 w-full"
        on:click={() => {
            sendWebsocket({
                type: "SerialSend",
                data: "Restart\n",
            });
        }}
    >
        Restart
    </button>
    <button
        class="btn btn-primary w-full mt-2"
        on:click={() => {
            if (confirm("Are you sure?")) {
                sendWebsocket({
                    type: "SerialSend",
                    data: "FactoryReset\n",
                });
            }
        }}
    >
        Factory reset
    </button>
</Card>
