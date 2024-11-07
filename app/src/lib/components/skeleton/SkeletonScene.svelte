<script lang="ts">
    import { T, useThrelte } from "@threlte/core";
    import * as THREE from "three";
    import { GLTF, OrbitControls } from "@threlte/extras";
    import { bones, type BoneDict } from "$lib/websocket";
    import { SkeletonLineSegments } from "./skeleton_line_segments";

    const { invalidate } = useThrelte();

    export let showModel = true;
    export let showLines = true;

    let modelRef: THREE.Group;

    let skeletonLineSegments: SkeletonLineSegments;

    function updateModel(bonesData: BoneDict) {
        Object.entries(bonesData).forEach(([location, boneData]) => {
            // The model is flipped for some reason so rotate it 180 degrees and flip the quaternion
            const quat = new THREE.Quaternion().fromArray(
                boneData.local_orientation,
            );

            const part = modelRef.getObjectByName(location);
            if (part) {
                part.rotation.setFromQuaternion(quat);
            }
        });
    }

    function updateBones(bonesData?: BoneDict) {
        if (!bonesData) {
            return;
        }

        if (!skeletonLineSegments) {
            skeletonLineSegments = new SkeletonLineSegments(bonesData);
        } else {
            skeletonLineSegments.update(bonesData);
        }

        if (showModel && modelRef) {
            updateModel(bonesData);
        }

        invalidate();
    }

    $: updateBones($bones);
</script>

<T.PerspectiveCamera
    makeDefault
    position={[4, 4, -4]}
    on:create={({ ref }) => ref.lookAt(new THREE.Vector3(0, 1.5, 0))}
>
    <OrbitControls target={[0, 1.5, 0]} />
</T.PerspectiveCamera>
{#if showModel}
    <GLTF
        url={"default.glb"}
        scale={2}
        on:create={({ ref }) => (modelRef = ref)}
    />
{/if}
{#if showLines && skeletonLineSegments}
    <T is={skeletonLineSegments} scale={1.5} />
{/if}
