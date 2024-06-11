<script lang="ts">
    import { onMount } from "svelte";
    import "../app.css";
    import { websocket } from "$lib/websocket";

    onMount(async () => {
        const urlParams = new URLSearchParams(window.location.search);
        const protocol = location.protocol === "https" ? "wss" : "ws";
        $websocket = new WebSocket(
            `${protocol}://localhost:${urlParams.get("websocket_port")}`,
        );

        $websocket.addEventListener("message", (msg) => {
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

<main class="p-4">
    <slot />
</main>
