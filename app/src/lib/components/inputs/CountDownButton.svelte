<script lang="ts">
    export let onActivate: () => void;

    export let initialContent: string;
    export let countdownContent: string;

    let count = 0;

    function onClick() {
        count = 3;
        const interval = setInterval(() => {
            count -= 1;
            if (count == 0) {
                clearInterval(interval);
                onActivate();
            }
        }, 1000);
    }
</script>

<button class="btn" on:click={onClick} disabled={count != 0}>
    {#if count != 0}
        {countdownContent}
        {count}...
    {:else}
        {initialContent}
    {/if}
</button>
