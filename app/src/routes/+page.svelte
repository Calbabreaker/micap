<script lang="ts">
    import { onMount } from "svelte";

    let message = "";

    onMount(async () => {
        const urlParams = new URLSearchParams(window.location.search);
        const protocol = location.protocol === "https" ? "wss" : "ws";
        const ws = new WebSocket(
            `${protocol}://localhost:${urlParams.get("websocket_port")}`,
        );

        ws.addEventListener("message", (msg) => {
            console.log(msg.type);
        });

        setInterval(() => ws.send("test"), 1000);
    });
</script>

<h1>Welcome to SvelteKit</h1>
<p>
    {message}
</p>
