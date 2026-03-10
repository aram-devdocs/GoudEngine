#!/usr/bin/env python3
"""Shared section builders for the TypeScript Node wrapper generator."""

from ts_node_shared import mapping, schema, to_camel, ts_iface_type


def append_animation_wrappers(lines):
    anim_wrappers = [
        ("animationLayerStackCreate", "entity: IEntity", "return (this.native as any).animationLayerStackCreate(entity as unknown as NativeEntity);", "number"),
        ("animationLayerAdd", "entity: IEntity, name: string, blendMode: number", "return (this.native as any).animationLayerAdd(entity as unknown as NativeEntity, name, blendMode);", "number"),
        ("animationLayerSetWeight", "entity: IEntity, layerIndex: number, weight: number", "return (this.native as any).animationLayerSetWeight(entity as unknown as NativeEntity, layerIndex, weight);", "number"),
        ("animationLayerPlay", "entity: IEntity, layerIndex: number", "return (this.native as any).animationLayerPlay(entity as unknown as NativeEntity, layerIndex);", "number"),
        ("animationLayerSetClip", "entity: IEntity, layerIndex: number, frameCount: number, frameDuration: number, mode: number", "return (this.native as any).animationLayerSetClip(entity as unknown as NativeEntity, layerIndex, frameCount, frameDuration, mode);", "number"),
        ("animationLayerAddFrame", "entity: IEntity, layerIndex: number, x: number, y: number, w: number, h: number", "return (this.native as any).animationLayerAddFrame(entity as unknown as NativeEntity, layerIndex, x, y, w, h);", "number"),
        ("animationLayerReset", "entity: IEntity, layerIndex: number", "return (this.native as any).animationLayerReset(entity as unknown as NativeEntity, layerIndex);", "number"),
        ("animationClipAddEvent", "entity: IEntity, frameIndex: number, name: string, payloadType: number, payloadInt: number, payloadFloat: number, payloadString?: string | null", "return (this.native as any).animationClipAddEvent(entity as unknown as NativeEntity, frameIndex, name, payloadType, payloadInt, payloadFloat, payloadString ?? null);", "number"),
        ("animationEventsCount", "", "return (this.native as any).animationEventsCount();", "number"),
        ("animationEventsRead", "index: number", "return (this.native as any).animationEventsRead(index) as unknown as IAnimationEventData;", "IAnimationEventData"),
    ]
    for method_name, signature, body, return_type in anim_wrappers:
        lines.append(f"  {method_name}({signature}): {return_type} {{")
        lines.append(f"    {body}")
        lines.append("  }")
        lines.append("")


def append_context_wrapper(lines):
    lines += [
        "/** Headless engine context exposing low-level networking APIs for Node-only tests. */",
        "export class GoudContext implements IGoudContext {",
        "  private native: NativeGoudContext;",
        "",
        "  constructor() {",
        "    this.native = new NativeGoudContext();",
        "  }",
        "",
        "  destroy(): boolean {",
        "    return this.native.destroy();",
        "  }",
        "",
        "  isValid(): boolean {",
        "    return this.native.isValid();",
        "  }",
        "",
        "  getNetworkCapabilities(): INetworkCapabilities {",
        "    return this.native.getNetworkCapabilities() as unknown as INetworkCapabilities;",
        "  }",
        "",
        "  networkHost(protocol: number, port: number): number {",
        "    return this.native.networkHost(protocol, port);",
        "  }",
        "",
        "  networkConnect(protocol: number, address: string, port: number): number {",
        "    return this.native.networkConnect(protocol, address, port);",
        "  }",
        "",
        "  networkConnectWithPeer(protocol: number, address: string, port: number): INetworkConnectResult {",
        "    return this.native.networkConnectWithPeer(protocol, address, port) as unknown as INetworkConnectResult;",
        "  }",
        "",
        "  networkDisconnect(handle: number): number {",
        "    return this.native.networkDisconnect(handle);",
        "  }",
        "",
        "  networkSend(handle: number, peerId: number, data: Uint8Array, channel: number): number {",
        "    return this.native.networkSend(handle, peerId, Buffer.from(data), channel);",
        "  }",
        "",
        "  networkReceive(handle: number): Uint8Array {",
        "    return this.native.networkReceive(handle);",
        "  }",
        "",
        "  networkReceivePacket(handle: number): INetworkPacket | null {",
        "    return this.native.networkReceivePacket(handle) as unknown as INetworkPacket | null;",
        "  }",
        "",
        "  networkPoll(handle: number): number {",
        "    return this.native.networkPoll(handle);",
        "  }",
        "",
        "  getNetworkStats(handle: number): INetworkStats {",
        "    return this.native.getNetworkStats(handle) as unknown as INetworkStats;",
        "  }",
        "",
        "  networkPeerCount(handle: number): number {",
        "    return this.native.networkPeerCount(handle);",
        "  }",
        "",
        "  setNetworkSimulation(handle: number, config: INetworkSimulationConfig): number {",
        "    return this.native.setNetworkSimulation(handle, {",
        "      one_way_latency_ms: config.oneWayLatencyMs,",
        "      jitter_ms: config.jitterMs,",
        "      packet_loss_percent: config.packetLossPercent,",
        "    } as unknown as INetworkSimulationConfig);",
        "  }",
        "",
        "  clearNetworkSimulation(handle: number): number {",
        "    return this.native.clearNetworkSimulation(handle);",
        "  }",
        "",
        "  setNetworkOverlayHandle(handle: number): number {",
        "    return this.native.setNetworkOverlayHandle(handle);",
        "  }",
        "",
        "  clearNetworkOverlayHandle(): number {",
        "    return this.native.clearNetworkOverlayHandle();",
        "  }",
        "}",
        "",
    ]


def _append_physics_world_methods(lines, tool):
    for method in tool.get("methods", []):
        method_name = to_camel(method["name"])
        params = method.get("params", [])
        return_type = method.get("returns", "void")
        if method.get("doc"):
            lines.append(f"  /** {method['doc']} */")
        param_signature = ", ".join(
            f"{to_camel(param['name'])}: {ts_iface_type(param['type'])}"
            for param in params
        )
        call_args = ", ".join(to_camel(param["name"]) for param in params)
        lines.append(f"  {method_name}({param_signature}): {ts_iface_type(return_type)} {{")
        if return_type == "void":
            lines.append(f"    this.native.{method_name}({call_args});")
        else:
            lines.append(f"    return this.native.{method_name}({call_args});")
        lines.append("  }")
        lines.append("")


def append_physics_world_2d_wrapper(lines):
    tool = schema["tools"]["PhysicsWorld2D"]
    if tool.get("doc"):
        lines.append(f"/** {tool['doc']} */")
    lines += [
        "export class PhysicsWorld2D implements IPhysicsWorld2D {",
        "  private native: any;",
        "",
        "  constructor(gravityX: number, gravityY: number, backend: number = 0) {",
        "    const { NativePhysicsWorld2D } = require('../../../index');",
        "    this.native = new NativePhysicsWorld2D(gravityX, gravityY, backend);",
        "  }",
        "",
    ]
    _append_physics_world_methods(lines, tool)
    lines += ["}", ""]


def append_physics_world_3d_wrapper(lines):
    tool = schema["tools"]["PhysicsWorld3D"]
    if tool.get("doc"):
        lines.append(f"/** {tool['doc']} */")
    lines += [
        "export class PhysicsWorld3D implements IPhysicsWorld3D {",
        "  private native: any;",
        "",
        "  constructor(gravityX: number, gravityY: number, gravityZ: number) {",
        "    const { NativePhysicsWorld3D } = require('../../../index');",
        "    this.native = new NativePhysicsWorld3D(gravityX, gravityY, gravityZ);",
        "  }",
        "",
    ]
    _append_physics_world_methods(lines, tool)
    lines += ["}", ""]


def append_engine_config_wrapper(lines):
    tool = schema["tools"]["EngineConfig"]
    lines += ["import type { IEngineConfig } from '../types/engine.g.js';", ""]
    if tool.get("doc"):
        lines.append(f"/** {tool['doc']} */")
    lines += [
        "export class EngineConfig implements IEngineConfig {",
        "  private native: any;",
        "",
        "  constructor() {",
        "    const { NativeEngineConfig } = require('../../../index');",
        "    this.native = new NativeEngineConfig();",
        "  }",
        "",
    ]
    for method in tool.get("methods", []):
        method_name = to_camel(method["name"])
        params = method.get("params", [])
        if method.get("doc"):
            lines.append(f"  /** {method['doc']} */")
        if method_name == "build":
            lines += [
                "  build(): GoudGame {",
                "    const ctx = this.native.build();",
                "    const game = Object.create(GoudGame.prototype);",
                "    game.native = ctx;",
                "    return game;",
                "  }",
            ]
        elif method_name == "destroy":
            lines += ["  destroy(): void {", "    this.native.destroy();", "  }"]
        else:
            param_signature = ", ".join(
                f"{to_camel(param['name'])}: {ts_iface_type(param['type'])}"
                for param in params
            )
            call_args = ", ".join(to_camel(param["name"]) for param in params)
            lines += [
                f"  {method_name}({param_signature}): EngineConfig {{",
                f"    this.native.{method_name}({call_args});",
                "    return this;",
                "  }",
            ]
        lines.append("")
    lines += ["}", ""]


def append_ui_manager_wrapper(lines):
    tool = schema["tools"]["UiManager"]
    lines += [
        "function toNativeUiNodeId(nodeId: UiNodeId): number {",
        "  return typeof nodeId === 'bigint' ? Number(nodeId) : nodeId;",
        "}",
        "",
        "export class UiManager implements IUiManager {",
        "  private native: NativeUiManager;",
        "",
        "  constructor() {",
        "    this.native = new NativeUiManager();",
        "  }",
        "",
    ]
    for method in tool.get("methods", []):
        method_name = to_camel(method["name"])
        params = method.get("params", [])
        return_type = method.get("returns", "void")
        if method.get("doc"):
            lines.append(f"  /** {method['doc']} */")
        param_defs = []
        call_args = []
        uses_ui_node_id = False
        for param in params:
            param_name = to_camel(param["name"])
            param_type = param["type"]
            if param_name in {"nodeId", "childId", "parentId"}:
                param_defs.append(f"{param_name}: UiNodeId")
                call_args.append(f"toNativeUiNodeId({param_name})")
                uses_ui_node_id = True
            elif param_type == "UiStyle":
                param_defs.append(f"{param_name}: IUiStyle")
                call_args.append(param_name)
            elif param_type in schema.get("enums", {}):
                param_defs.append(f"{param_name}: number")
                call_args.append(param_name)
            else:
                param_defs.append(f"{param_name}: {ts_iface_type(param_type)}")
                call_args.append(param_name)
        exported_return_type = (
            "UiNodeId" if method_name in {"createNode", "getParent", "getChildAt"} else ts_iface_type(return_type)
        )
        lines.append(f"  {method_name}({', '.join(param_defs)}): {exported_return_type} {{")
        if method_name == "eventRead":
            lines.append(f"    return this.native.{method_name}({', '.join(call_args)}) as unknown as IUiEvent | null;")
        elif return_type == "void":
            lines.append(f"    this.native.{method_name}({', '.join(call_args)});")
        elif uses_ui_node_id and method_name in {"createNode", "getParent", "getChildAt"}:
            lines.append(f"    return this.native.{method_name}({', '.join(call_args)}) as UiNodeId;")
        else:
            lines.append(f"    return this.native.{method_name}({', '.join(call_args)});")
        lines.append("  }")
        lines.append("")

    lines += [
        "  createPanel(): UiNodeId {",
        "    return this.createNode(0);",
        "  }",
        "",
        "  createLabel(text: string): UiNodeId {",
        "    const nodeId = this.createNode(2);",
        "    this.setLabelText(nodeId, text);",
        "    return nodeId;",
        "  }",
        "",
        "  createButton(enabled: boolean = true): UiNodeId {",
        "    const nodeId = this.createNode(1);",
        "    this.setButtonEnabled(nodeId, enabled);",
        "    return nodeId;",
        "  }",
        "",
        "  createImage(path: string): UiNodeId {",
        "    const nodeId = this.createNode(3);",
        "    this.setImageTexturePath(nodeId, path);",
        "    return nodeId;",
        "  }",
        "",
        "  createSlider(min: number, max: number, value: number, enabled: boolean = true): UiNodeId {",
        "    const nodeId = this.createNode(4);",
        "    this.setSlider(nodeId, min, max, value, enabled);",
        "    return nodeId;",
        "  }",
        "}",
        "",
    ]


def has_physics_world_2d():
    return "PhysicsWorld2D" in schema.get("tools", {}) and "PhysicsWorld2D" in mapping.get("tools", {})


def has_physics_world_3d():
    return "PhysicsWorld3D" in schema.get("tools", {}) and "PhysicsWorld3D" in mapping.get("tools", {})


def has_engine_config():
    return "EngineConfig" in schema.get("tools", {}) and "EngineConfig" in mapping.get("tools", {})


def has_ui_manager():
    return "UiManager" in schema.get("tools", {}) and "UiManager" in mapping.get("tools", {})
