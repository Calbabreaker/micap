<script lang="ts">
    import { onMount } from "svelte";
    import { fade } from "svelte/transition";

    onMount(() => {
        // Prevent user from scrolling in the background
        if (document.activeElement) {
            (document.activeElement as HTMLElement).blur();
        }

        const style = document.body.style;
        if (style.overflow != "hidden") {
            style.overflow = "hidden";
            return () => (style.overflow = "initial");
        }
    });
</script>

<div
    class="fixed top-0 left-0 bg-black/60 w-full h-full z-20 overflow-y-auto overflow-x-hidden p-4 grid place-items-center"
    transition:fade={{ duration: 100 }}
>
    <div class="bg-neutral-600 p-4 rounded max-w-lg h-fit z-30">
        <slot />
    </div>
</div>
