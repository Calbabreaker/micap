<script lang="ts">
    import { sendWebsocket } from "$lib/websocket";

    let ssid = "";
    let password = "";

    function setWifi() {
        if (ssid.length == 0) {
            return;
        }
        sendWebsocket({
            type: "SerialSend",
            data: `Wifi\0${ssid}\0${password}\n`,
        });
    }
</script>

<form on:submit|preventDefault={setWifi}>
    <div class="mb-2">
        <span class="w-24 inline-block">SSID</span>
        <input
            placeholder="WiFi SSID"
            bind:value={ssid}
            maxlength="32"
            class="p-2 rounded text-black text-sm"
        />
    </div>
    <div class="mb-2">
        <span class="w-24 inline-block">Password</span>
        <input
            placeholder="Password"
            bind:value={password}
            maxlength="64"
            class="p-2 rounded text-black text-sm"
        />
    </div>
    <div>
        <button type="submit" class="btn btn-primary mr-2 w-full">
            Send
        </button>
    </div>
</form>
