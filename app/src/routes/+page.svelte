<script lang="ts">
    import { onMount } from "svelte";
    import WifiCard from "../components/wifi_card.svelte";

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
            if (typeof msg.data === "string") {
                handle_message(msg.data);
            }
        });
    });

    function handle_message(message: string) {
        console.log(message);
        const args = message.split(":");
        if (args[0] == "DEVICE-CONNECT") {
            clients.push({ mac: args[1] });
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
</div>
