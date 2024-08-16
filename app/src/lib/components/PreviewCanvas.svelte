<script lang="ts">
    import { Canvas } from "@threlte/core";
    import { T } from "@threlte/core";
</script>

<div class="w-96 h-96 bg-black">
    <Canvas>
        <T.DirectionalLight args={[0xffffff, 1]} position={[1, 1, 1]} />
        <T.AmbientLight args={[0xffffff, 0.5]} />
        <T.GridHelper args={[10, 10]} />
        <slot />
    </Canvas>
</div>
