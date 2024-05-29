<script lang="ts">
    import { onMount } from "svelte";
    import WifiCard from "../components/wifi_card.svelte";
    import { json } from "@sveltejs/kit";

    interface Client {
        mac: string;
    }

    let ws: WebSocket;

    let clients: Client[] = [];

    onMount(async () => {
        const urlParams = new URLSearchParams(window.location.search);
        const protocol = location.protocol === "https" ? "wss" : "ws";
        ws = new WebSocket(
            `${protocol}://localhost:${urlParams.get("websocket_port")}`,
        );

        ws.addEventListener("message", (msg) => {
            let message = JSON.parse(msg.data);
            if (message) {
                handle_message(message);
            }
        });
    });

    function handle_message(message: any) {
        if (message.type == "Error") {
            console.error(message.error);
        }
    }
</script>

<h1 class="text-white text-lg">Welcome to SvelteKit</h1>
<div>
    <div>
        <p>Clients:</p>
        {#each clients as client}
            {client.mac}
        {/each}
    </div>
    <WifiCard {ws} />
    <button
        class="btn btn-primary"
        on:click={() => {
            if (confirm("Are you sure?")) {
                ws.send(JSON.stringify({ type: "FactoryReset" }));
            }
        }}
    >
        Factory reset
    </button>
</div>
