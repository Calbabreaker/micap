<script lang="ts">
    import { sendWebsocket } from "$lib/websocket";
    import CountDownButton from "../inputs/CountDownButton.svelte";
</script>

<CountDownButton
    onActivate={() => {
        sendWebsocket({ type: "ResetSkeletonOrientation" });
    }}
    initialContent="Reset Orientation"
    countdownContent="Reseting in"
/>
