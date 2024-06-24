<script lang="ts">
    import { Canvas } from "@threlte/core";
    import { T } from "@threlte/core";
    import type { TrackerData } from "$lib/websocket";
    import { ArrowHelper, Vector3, Object3D } from "three";
    import { OrbitControls } from "@threlte/extras";

    Object3D.DEFAULT_UP = new Vector3(0, 0, 1);

    export let data: TrackerData;

    $: updateArrow(new Vector3().fromArray(data.acceleration));

    let arrowRef: ArrowHelper;

    // $: positionLooped = [
    //     data.position[0] % 6,
    //     data.position[1] % 6,
    //     data.position[2] % 6,
    // ];

    function updateArrow(vector: Vector3) {
        if (arrowRef) {
            arrowRef.setLength(vector.length() * 4);
            arrowRef.setDirection(vector.normalize());
            arrowRef.setColor(0xffffff);
        }
    }

    function formatArray(array: number[]) {
        return array.map((value) => value.toFixed(2)).join(", ");
    }
</script>

<p>Orientation: {formatArray(data.orientation)}</p>
<p>Acceleration: {formatArray(data.acceleration)}</p>
<div class="w-96 h-96 bg-black mt-2">
    <Canvas>
        <T.PerspectiveCamera
            makeDefault
            position={[10, 10, 10]}
            on:create={({ ref }) => {
                ref.lookAt(0, 1, 0);
            }}
        >
            <OrbitControls />
        </T.PerspectiveCamera>
        <T.DirectionalLight args={[0xffffff, 1]} position={[1, 1, 1]} />
        <T.AmbientLight args={[0xffffff, 0.5]} />
        <T.GridHelper
            args={[10, 10]}
            on:create={({ ref }) => {
                ref.rotateX(Math.PI / 2);
            }}
        />

        <T.Mesh quaternion={data.orientation} scale={[1.5, 1.5, 1.5]}>
            <T.AxesHelper args={[5]} />
            <T.BoxGeometry />
            <T.MeshLambertMaterial color={0xffffff} />
        </T.Mesh>
        <T.ArrowHelper bind:ref={arrowRef} />
    </Canvas>
</div>
