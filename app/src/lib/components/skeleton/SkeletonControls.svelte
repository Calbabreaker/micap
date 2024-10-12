<script lang="ts">
    import { sendWebsocket } from "$lib/websocket";

    const initialButtonContent = "Reset Orientation";
    let buttonContent = initialButtonContent;

    function onClick() {
        buttonContent = "3";
        const interval = setInterval(() => {
            const count = parseInt(buttonContent) - 1;
            if (count == 0) {
                clearInterval(interval);
                sendWebsocket({ type: "ResetSkeletonOrientation" });
                buttonContent = initialButtonContent;
            } else {
                buttonContent = String(count);
            }
        }, 1000);
    }
</script>

<button
    class="btn btn-primary"
    on:click={onClick}
    disabled={buttonContent != initialButtonContent}>{buttonContent}</button
>
