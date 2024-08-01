<script lang="ts">
    import { sendWebsocket } from "$lib/websocket";

    let ssid = "";
    let password = "";

    function setWifi(e: SubmitEvent) {
        e.preventDefault();
        sendWebsocket({
            type: "Wifi",
            ssid,
            password,
        });
    }
</script>

<form on:submit={setWifi} class="bg-neutral-700 p-4 shadow rounded mb-4">
    <h1 class="text-2xl mb-4">Enter Wifi Credentials</h1>
    <input
        placeholder="WiFi SSID"
        bind:value={ssid}
        class="p-2 rounded text-black"
    />
    <input
        placeholder="Password"
        type="password"
        bind:value={password}
        class="p-2 rounded text-black"
    />
    <button type="submit" class="btn btn-primary mr-2 inline-block">
        Set
    </button>
</form>
