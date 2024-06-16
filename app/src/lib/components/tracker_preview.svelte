<script lang="ts">
    import { Canvas } from "@threlte/core";
    import { T } from "@threlte/core";
    import type { TrackerData } from "$lib/websocket";

    export let data: TrackerData;
</script>

<Canvas>
    <T.PerspectiveCamera
        makeDefault
        position={[10, 10, 10]}
        on:create={({ ref }) => {
            ref.lookAt(0, 1, 0);
        }}
    />

    <T.DirectionalLight args={[0xffffff, 1]} />
    <T.AmbientLight args={[0xffffff, 0.5]} />
    <T.GridHelper args={[10, 10]} />
    <T.Mesh quaternion={data.orientation} scale={[4, 4, 4]}>
        <T.AxesHelper args={[5]} />
        <T.BoxGeometry />
        <T.MeshLambertMaterial color={0xffffff} />
    </T.Mesh>
</Canvas>
