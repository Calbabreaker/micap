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
            const quat = new THREE.Quaternion().fromArray(boneData.orientation);

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

<T.PerspectiveCamera makeDefault position={[4, 6, -4]}>
    <OrbitControls />
</T.PerspectiveCamera>
{#if showModel}
    <GLTF
        url={"default.glb"}
        scale={2}
        on:create={({ ref }) => (modelRef = ref)}
    />
{/if}
{#if showLines && skeletonLineSegments}
    <T is={skeletonLineSegments} scale={3} />
{/if}
