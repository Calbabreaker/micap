<script lang="ts" generics="T">
    import RotateLeftIcon from "../icons/RotateLeftIcon.svelte";

    export let value: T;
    export let defaultValue: T;

    function reset(button: HTMLButtonElement) {
        if (defaultValue instanceof Array) {
            value = defaultValue.slice();
        } else {
            value = defaultValue;
        }

        const form = button.closest("form");
        if (form) {
            form.dispatchEvent(new Event("change"));
        }
    }

    function checkEquals(value: T, defaultValue: T): boolean {
        if (value instanceof Array && defaultValue instanceof Array) {
            return (
                value.length == defaultValue.length &&
                value.every((x) => defaultValue.includes(x))
            );
        } else {
            return value === defaultValue;
        }
    }
</script>

<button
    class="btn-icon"
    on:mousedown={(e) => reset(e.currentTarget)}
    on:touchstart={(e) => reset(e.currentTarget)}
    disabled={checkEquals(value, defaultValue)}
>
    <RotateLeftIcon />
</button>
