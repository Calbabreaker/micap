<script lang="ts">
    export let title: string;
</script>

<div class="bg-neutral-700 p-4 shadow rounded">
    <h1 class="text-center text-2xl mb-4">{title}</h1>
    <slot />
</div>
