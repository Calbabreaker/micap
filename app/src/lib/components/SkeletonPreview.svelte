<script lang="ts">
    import { T } from "@threlte/core";
    import { Bone, Skeleton } from "three";
    import PreviewCanvas from "./PreviewCanvas.svelte";

    import { GLTF } from "@threlte/extras";

    const shoulder = new Bone();
    const elbow = new Bone();
    const hand = new Bone();

    shoulder.add(elbow);
    elbow.add(hand);

    const bones = [shoulder, elbow, hand];

    shoulder.position.z = 0;
    elbow.position.z = 1;
    elbow.position.y = 1;
    hand.position.y = 5;
    hand.position.z = 5;

    const armSkeleton = new Skeleton(bones);
</script>

<svelte:window on:mousemove={() => (shoulder.position.z = Math.random() * 2)} />
<PreviewCanvas>
    <T.Group
        on:create={({ ref }) => {
            ref.add(bones[0]);
        }}
    ></T.Group>
    <T.SkeletonHelper args={[bones[0]]}></T.SkeletonHelper>
    <GLTF
        castShadow
        receiveShadow
        url={"HEVA_Portal.glb"}
        scale={2}
        on:create={({ ref }) => {
            // const root = ref.getObjectByName("RootBone");
            // root?.traverse((object) => {
            //     console.log(object.rotation);
            // });

            const foot = ref.getObjectByName("leftFoot");
            foot.position.y = 10;
        }}
    />
</PreviewCanvas>
