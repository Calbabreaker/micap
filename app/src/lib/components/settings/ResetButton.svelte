<script lang="ts">
    import RotateLeftIcon from "../icons/RotateLeftIcon.svelte";

    export let value: any;
    export let defaultValue: any;

    function reset(button: HTMLButtonElement) {
        value = defaultValue;
        const form = button.closest("form");
        if (form) {
            form.dispatchEvent(new Event("change"));
        }
    }
</script>

<button
    class="btn-icon"
    on:mousedown={(e) => reset(e.currentTarget)}
    on:touchstart={(e) => reset(e.currentTarget)}
    disabled={value == defaultValue}
>
    <RotateLeftIcon />
</button>
