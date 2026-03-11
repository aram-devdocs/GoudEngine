#!/usr/bin/env python3
"""Interactive cross-language sandbox for the Python SDK."""

from __future__ import annotations

import json
import math
import os
import sys
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SDK_ROOT = REPO_ROOT / "sdks" / "python"
sys.path.insert(0, str(SDK_ROOT))

from goud_engine import (  # noqa: E402
    Color,
    GoudContext,
    GoudGame,
    Key,
    NetworkManager,
    NetworkProtocol,
    TextAlignment,
    UiManager,
)


WINDOW_WIDTH = 1280
WINDOW_HEIGHT = 720
MOVE_SPEED = 220.0


@dataclass
class SandboxAssets:
    background: str
    sprite: str
    accent_sprite: str
    texture3d: str
    font: str
    audio: bytes


class NetworkState:
    def __init__(self, port: int):
        self.context = GoudContext()
        self.endpoint = None
        self.role = "offline"
        self.peer_count = 0
        self.last_detail = "No network provider"
        self._known_peer_id = None
        self._heartbeat_timer = 0.0
        self._started = False
        try:
            manager = NetworkManager(self.context)
            self.endpoint = manager.host(NetworkProtocol.TCP, port)
            self.role = "host"
            self.last_detail = f"Hosting localhost:{port}"
            self._started = True
        except Exception:
            try:
                manager = NetworkManager(self.context)
                self.endpoint = manager.connect(NetworkProtocol.TCP, "127.0.0.1", port)
                self.role = "client"
                self.last_detail = f"Connected to localhost:{port}"
                self._started = True
            except Exception as exc:
                self.last_detail = str(exc)

    def update(self, dt: float) -> None:
        if self.endpoint is None:
            return
        self.endpoint.poll()
        self.peer_count = self.endpoint.peer_count()
        packet = self.endpoint.receive()
        if packet is not None:
            self._known_peer_id = packet.peer_id
            self.last_detail = f"Received {len(packet.data)} bytes from peer {packet.peer_id}"
        self._heartbeat_timer += dt
        if self._heartbeat_timer < 1.0:
            return
        self._heartbeat_timer = 0.0
        payload = f"sandbox:{self.role}:{self.peer_count}".encode("utf-8")
        try:
            if self.endpoint.default_peer_id is not None:
                self.endpoint.send(payload)
            elif self._known_peer_id is not None:
                self.endpoint.send_to(self._known_peer_id, payload)
        except Exception as exc:
            self.last_detail = str(exc)

    def destroy(self) -> None:
        if self.endpoint is not None:
            try:
                self.endpoint.disconnect()
            except Exception:
                pass
        self.context.destroy()


def _load_assets() -> tuple[SandboxAssets, int]:
    manifest = json.loads(
        (REPO_ROOT / "examples" / "shared" / "sandbox" / "manifest.json").read_text(
            encoding="utf-8"
        )
    )
    assets = manifest["assets"]
    audio_path = REPO_ROOT / assets["audio"]
    return (
        SandboxAssets(
            background=str(REPO_ROOT / assets["background"]),
            sprite=str(REPO_ROOT / assets["sprite"]),
            accent_sprite=str(REPO_ROOT / assets["accent_sprite"]),
            texture3d=str(REPO_ROOT / assets["texture3d"]),
            font=str(REPO_ROOT / assets["font"]),
            audio=audio_path.read_bytes(),
        ),
        int(manifest["network_port"]),
    )


def _make_ui() -> UiManager:
    ui = UiManager()
    root = ui.create_panel()
    title = ui.create_label("Sandbox Widgets")
    button = ui.create_button(True)
    ui.set_parent(title, root)
    ui.set_parent(button, root)
    ui.set_label_text(title, "Sandbox Widgets")
    ui.set_button_enabled(button, True)
    return ui


def _try_scene_setup(context: GoudContext) -> dict[str, int]:
    scene_ids = {}
    for name in ("sandbox-2d", "sandbox-3d", "sandbox-hybrid"):
        try:
            scene_ids[name] = context.scene_create(name)
        except Exception:
            scene_ids[name] = 0
    return scene_ids


def main() -> int:
    assets, port = _load_assets()
    game = GoudGame(WINDOW_WIDTH, WINDOW_HEIGHT, "GoudEngine Sandbox - Python")
    ui = _make_ui()
    scene_context = GoudContext()
    scene_ids = _try_scene_setup(scene_context)
    network = NetworkState(port)

    background = game.load_texture(assets.background)
    sprite = game.load_texture(assets.sprite)
    accent_sprite = game.load_texture(assets.accent_sprite)
    texture3d = game.load_texture(assets.texture3d)
    font = game.load_font(assets.font)

    cube = game.create_cube(texture3d, 1.2, 1.2, 1.2)
    plane = game.create_plane(texture3d, 8.0, 8.0)
    light = game.add_light(0, 4.0, 6.0, -4.0, 0.0, -1.0, 0.0, 1.0, 0.95, 0.8, 4.0, 25.0, 0.0)
    game.set_object_position(plane, 0.0, -1.0, 0.0)
    game.configure_grid(True, 12.0, 12)

    mode_index = 0
    mode_names = ["2D", "3D", "Hybrid"]
    player_x = 250.0
    player_y = 300.0
    angle = 0.0
    elapsed = 0.0
    audio_activated = False
    smoke_seconds = float(os.environ.get("GOUD_SANDBOX_SMOKE_SECONDS", "0") or "0")

    while not game.should_close():
        dt = game.delta_time or 0.016
        elapsed += dt
        angle += dt
        if game.is_key_just_pressed(Key.ESCAPE):
            game.close()
        if game.is_key_just_pressed(Key.DIGIT1):
            mode_index = 0
        if game.is_key_just_pressed(Key.DIGIT2):
            mode_index = 1
        if game.is_key_just_pressed(Key.DIGIT3):
            mode_index = 2
        if game.is_key_pressed(Key.A) or game.is_key_pressed(Key.LEFT):
            player_x -= MOVE_SPEED * dt
        if game.is_key_pressed(Key.D) or game.is_key_pressed(Key.RIGHT):
            player_x += MOVE_SPEED * dt
        if game.is_key_pressed(Key.W) or game.is_key_pressed(Key.UP):
            player_y -= MOVE_SPEED * dt
        if game.is_key_pressed(Key.S) or game.is_key_pressed(Key.DOWN):
            player_y += MOVE_SPEED * dt
        if game.is_key_just_pressed(Key.SPACE):
            if not audio_activated:
                game.audio_activate()
                audio_activated = True
            game.audio_play(assets.audio)

        current_mode = mode_names[mode_index]
        target_scene = scene_ids.get(f"sandbox-{current_mode.lower()}", 0)
        if target_scene:
            scene_context.scene_set_current(target_scene)

        network.update(dt)
        mouse = game.get_mouse_position()

        game.begin_frame(0.07, 0.10, 0.14, 1.0)
        game.draw_sprite(background, WINDOW_WIDTH / 2, WINDOW_HEIGHT / 2, WINDOW_WIDTH, WINDOW_HEIGHT)
        game.draw_quad(210, 110, 320, 110, Color(0.05, 0.08, 0.12, 0.88))
        game.draw_quad(620, 110, 560, 110, Color(0.08, 0.12, 0.18, 0.88))
        game.draw_quad(620, 630, 560, 120, Color(0.05, 0.08, 0.12, 0.90))
        game.draw_quad(mouse.x, mouse.y, 14, 14, Color(0.95, 0.85, 0.20, 0.95))

        if current_mode in ("2D", "Hybrid"):
            game.draw_quad(920, 260, 180, 40, Color(0.20, 0.55, 0.95, 0.80))
            game.draw_sprite(sprite, player_x, player_y, 64, 64, angle * 0.25)
            game.draw_sprite(accent_sprite, 1040, 420, 72, 240, 0.0)

        if current_mode in ("3D", "Hybrid"):
            game.set_camera_position3_d(0.0, 3.0, -9.5)
            game.set_camera_rotation3_d(-10.0, angle * 20.0, 0.0)
            game.set_object_position(cube, 0.0, 1.0 + 0.35 * math.sin(angle * 2.0), 0.0)
            game.set_object_rotation(cube, 0.0, angle * 55.0, 0.0)
            game.render3_d()

        caps = game.get_render_capabilities()
        physics = game.get_physics_capabilities()
        audio = game.get_audio_capabilities()
        network_caps = None
        try:
            network_caps = game.get_network_capabilities()
        except Exception:
            network_caps = None

        lines = [
            "GoudEngine Sandbox",
            f"Mode: {current_mode}  (1/2/3 to switch)",
            "Movement: WASD / Arrows",
            "Audio: SPACE",
            f"Mouse marker: ({mouse.x:.0f}, {mouse.y:.0f})",
            f"Render caps: tex={caps.max_texture_size} instancing={caps.supports_instancing}",
            f"Physics caps: joints={physics.supports_joints} maxBodies={physics.max_bodies}",
            f"Audio caps: spatial={audio.supports_spatial} channels={audio.max_channels}",
            f"Scene count: {scene_context.scene_count()} current={scene_context.scene_get_current()}",
            f"UI nodes: {ui.node_count()}",
            f"Network role: {network.role} peers={network.peer_count}",
            f"Network detail: {network.last_detail}",
            f"Network caps: {network_caps.max_connections if network_caps else 'unsupported'}",
        ]

        for index, line in enumerate(lines):
            game.draw_text(
                font,
                line,
                40,
                40 + index * 22,
                18,
                TextAlignment.LEFT,
                0,
                1.0,
                0,
                Color.white(),
            )

        ui.update()
        ui.render()
        game.end_frame()
        if smoke_seconds > 0.0 and elapsed >= smoke_seconds:
            game.close()

    network.destroy()
    ui.destroy()
    scene_context.destroy()
    game.destroy()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
