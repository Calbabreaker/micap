<script lang="ts">
    import { T } from "@threlte/core";
    import { Bone, Euler, Skeleton } from "three";
    import PreviewCanvas from "./PreviewCanvas.svelte";
    import { GLTF, OrbitControls } from "@threlte/extras";
</script>

<PreviewCanvas>
    <T.PerspectiveCamera makeDefault position={[10, 10, -10]}>
        <OrbitControls />
    </T.PerspectiveCamera>
    <!-- <T.Group -->
    <!--     on:create={({ ref }) => { -->
    <!--         ref.add(bones[0]); -->
    <!--     }} -->
    <!-- ></T.Group> -->
    <!-- <T.SkeletonHelper args={[bones[0]]}></T.SkeletonHelper> -->
    <GLTF
        castShadow
        receiveShadow
        url={"default.glb"}
        scale={4}
        on:create={({ ref }) => {
            const root = ref.getObjectByName("Armature");
            window.root = root;
            root?.traverse((object) => {
                object.setRotationFromEuler(new Euler(0, 0, 0));
                // console.log(object.position);
            });

            // const foot = ref.getObjectByName("LeftToe");
            // foot.position.y = 10;
        }}
    />
</PreviewCanvas>
