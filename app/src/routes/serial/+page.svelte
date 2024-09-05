<script lang="ts">
    import { sendWebsocket } from "$lib/websocket";
    import WifiForm from "$lib/components/WifiForm.svelte";
    import Card from "$lib/components/Card.svelte";
</script>

<Card title="Enter WiFi credentials">
    <WifiForm />
</Card>
<Card title="Serial device control">
    <button
        class="btn btn-primary"
        on:click={() => {
            if (confirm("Are you sure?")) {
                sendWebsocket({
                    type: "FactoryReset",
                });
            }
        }}
    >
        Factory reset
    </button>
</Card>
