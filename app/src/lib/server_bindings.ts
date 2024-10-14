// Bindings generated by build.rs (using ts-rs). Do not edit manually.
export type Bone = { 
/**
 * Orientation of joint
 */
orientation: [number, number, number, number], tail_world_position: [number, number, number], parent: BoneLocation | null, };
export type BoneLocation = "Hip" | "LeftUpperLeg" | "RightUpperLeg" | "LeftLowerLeg" | "RightLowerLeg" | "LeftFoot" | "RightFoot" | "Waist" | "Chest" | "UpperChest" | "Neck" | "Head" | "LeftShoulder" | "RightShoulder" | "LeftUpperArm" | "RightUpperArm" | "LeftLowerArm" | "RightLowerArm" | "LeftHand" | "RightHand" | "LeftHip" | "RightHip";
/**
 * Offset type for a specific body part used to offset the bone (joints) in meters
 * See BoneLocation::get_offset
 */
export type BoneOffsetKind = "NeckLength" | "WaistLength" | "ChestLength" | "UpperChestLength" | "HipLength" | "HipsWidth" | "UpperLegLength" | "LowerLegLength" | "ShouldersWidth" | "ShoulderOffset" | "UpperArmLength" | "LowerArmLength" | "FootLength" | "HandLength";
export type GlobalConfig = { trackers: { [key in string]?: TrackerConfig }, vmc: VmcConfig, vrchat: VrChatConfig, skeleton: SkeletonConfig, };
export type GlobalConfigUpdate = { trackers: { [key in string]?: TrackerConfig } | null, vmc: VmcConfig | null, vrchat: VrChatConfig | null, skeleton: SkeletonConfig | null, };
export type SkeletonConfig = { 
/**
 * Contains the length offset in meters from a bone to its connecting one
 */
offsets: { [key in BoneOffsetKind]?: number }, };
export type Tracker = { info: TrackerInfo, data: TrackerData, };
export type TrackerConfig = { name?: string, location?: BoneLocation, };
export type TrackerData = { orientation: [number, number, number, number], acceleration: [number, number, number], position: [number, number, number], };
export type TrackerInfo = { to_be_removed: boolean, status: TrackerStatus, latency_ms?: number, battery_level: number, address?: string, };
export type TrackerStatus = "Ok" | "Error" | "Off" | "TimedOut";
export type VmcConfig = { enabled: boolean, send_port: number, receive_port: number, };
export type VrChatConfig = { enabled: boolean, send_port: number, bones_to_send: Array<BoneLocation>, };
export type WebsocketClientMessage = { "type": "SerialSend", data: string, } | { "type": "RemoveTracker", id: string, } | { "type": "UpdateConfig", config: GlobalConfigUpdate, } | { "type": "ResetTrackerOrientations" } | { "type": "ResetSkeleton" } | { "type": "StartRecord" } | { "type": "StopRecord", save_path: string, };
export type WebsocketServerMessage = { "type": "TrackerUpdate", trackers: { [key in string]?: Tracker }, } | { "type": "InitialState", config: GlobalConfig, port_name?: string, default_config: GlobalConfig, trackers: { [key in string]?: Tracker }, } | { "type": "SkeletonUpdate", bones: { [key in BoneLocation]?: Bone }, } | { "type": "ConfigUpdate", config: GlobalConfig, } | { "type": "SerialLog", log: string, } | { "type": "SerialPortChanged", port_name?: string, } | { "type": "Error", error: string, };
