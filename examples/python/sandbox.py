#!/usr/bin/env python3
"""Interactive cross-language sandbox for the Python SDK."""

from __future__ import annotations

import json
import math
import os
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Callable


REPO_ROOT = Path(__file__).resolve().parents[2]
SDK_ROOT = REPO_ROOT / "sdks" / "python"


def _normalize_goud_engine_lib() -> None:
    raw = os.environ.get("GOUD_ENGINE_LIB", "").strip()
    if not raw:
        return
    path = Path(raw)
    if not path.is_dir():
        return
    suffix = {
        "darwin": "libgoud_engine.dylib",
        "linux": "libgoud_engine.so",
        "win32": "goud_engine.dll",
    }.get(sys.platform)
    if not suffix:
        return
    candidate = path / suffix
    if candidate.exists():
        os.environ["GOUD_ENGINE_LIB"] = str(candidate)


_normalize_goud_engine_lib()
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
class HudRect:
    x: float
    y: float
    width: float
    height: float


@dataclass
class OverviewTextLayout:
    x: float
    title_y: float
    tagline_y: float


@dataclass
class StatusTextLayout:
    x: float
    title_y: float
    max_width: float


@dataclass
class NextTextLayout:
    x: float
    title_y: float


@dataclass
class SceneLabelLayout:
    x: float
    y: float
    max_width: float


@dataclass
class HudLayout:
    overview_panel: HudRect
    status_panel: HudRect
    next_panel: HudRect
    scene_badge: HudRect
    overview_text: OverviewTextLayout
    status_text: StatusTextLayout
    next_text: NextTextLayout
    scene_label: SceneLabelLayout


@dataclass
class SandboxAssets:
    background: str
    sprite: str
    accent_sprite: str
    texture3d: str
    font: str
    audio: bytes


@dataclass
class SandboxHud:
    overview_title: str
    status_title: str
    next_steps_title: str
    tagline: str
    overview: list[str]
    next_steps: list[str]


@dataclass
class SandboxContract:
    overview_items: list[str]
    status_rows: list[str]
    next_step_items: list[str]
    next_step_dynamic_rows: list[str]
    layout: HudLayout
    web_networking_blocker: str
    web_renderer_blocker: str


@dataclass
class SceneInfo:
    key: str
    mode: str
    label: str


@dataclass
class SandboxManifest:
    title: str
    network_port: int
    packet_version: str
    assets: SandboxAssets
    hud: SandboxHud
    contract: SandboxContract
    scenes: list[SceneInfo]
    web_networking_copy: str
    web_renderer_copy: str


def _env_flag(name: str) -> bool:
    return os.environ.get(name, "").strip().lower() in {"1", "true", "yes", "on"}


def _env_role(name: str) -> str:
    role = os.environ.get(name, "auto").strip().lower()
    return role if role in {"auto", "host", "client"} else "auto"


def _load_manifest() -> SandboxManifest:
    manifest = json.loads(
        (REPO_ROOT / "examples" / "shared" / "sandbox" / "manifest.json").read_text(
            encoding="utf-8"
        )
    )
    assets = manifest["assets"]
    hud = manifest["hud"]
    contract_path = REPO_ROOT / manifest.get("contract", "examples/shared/sandbox/contract.json")
    contract = json.loads(contract_path.read_text(encoding="utf-8"))
    layout = contract["layout"]
    network = manifest.get("network", {})
    audio_path = REPO_ROOT / assets["audio"]
    scenes = [
        SceneInfo(
            key=str(scene["key"]),
            mode=str(scene["mode"]),
            label=str(scene["label"]),
        )
        for scene in manifest["scenes"]
    ]
    return SandboxManifest(
        title=str(manifest["title"]),
        network_port=int(
            os.environ.get(
                "GOUD_SANDBOX_NETWORK_PORT",
                network.get("port", manifest.get("network_port", 38491)),
            )
        ),
        packet_version=str(network.get("packet_version", "v1")),
        assets=SandboxAssets(
            background=str(REPO_ROOT / assets["background"]),
            sprite=str(REPO_ROOT / assets["sprite"]),
            accent_sprite=str(REPO_ROOT / assets["accent_sprite"]),
            texture3d=str(REPO_ROOT / assets["texture3d"]),
            font=str(REPO_ROOT / assets["font"]),
            audio=audio_path.read_bytes(),
        ),
        hud=SandboxHud(
            overview_title=str(hud["overview_title"]),
            status_title=str(hud["status_title"]),
            next_steps_title=str(hud["next_steps_title"]),
            tagline=str(hud["tagline"]),
            overview=[str(line) for line in hud["overview"]],
            next_steps=[str(line) for line in hud["next_steps"]],
        ),
        contract=SandboxContract(
            overview_items=[str(line) for line in contract["overview_items"]],
            status_rows=[str(line) for line in contract["status_rows"]],
            next_step_items=[str(line) for line in contract["next_step_items"]],
            next_step_dynamic_rows=[str(line) for line in contract["next_step_dynamic_rows"]],
            layout=HudLayout(
                overview_panel=HudRect(**layout["overview_panel"]),
                status_panel=HudRect(**layout["status_panel"]),
                next_panel=HudRect(**layout["next_panel"]),
                scene_badge=HudRect(**layout["scene_badge"]),
                overview_text=OverviewTextLayout(**layout["overview_text"]),
                status_text=StatusTextLayout(**layout["status_text"]),
                next_text=NextTextLayout(**layout["next_text"]),
                scene_label=SceneLabelLayout(**layout["scene_label"]),
            ),
            web_networking_blocker=str(contract["web_blockers"]["networking"]),
            web_renderer_blocker=str(contract["web_blockers"]["renderer"]),
        ),
        scenes=scenes,
        web_networking_copy=str(manifest["capability_gates"]["web_networking"]),
        web_renderer_copy=str(manifest["capability_gates"]["web_renderer"]),
    )


class NetworkState:
    def __init__(self, port: int, preferred_role: str, exit_on_peer: bool):
        self.context = GoudContext()
        self.endpoint = None
        self.role = "offline"
        self.label = "solo"
        self.peer_count = 0
        self.last_detail = "No network provider"
        self.has_remote_state = False
        self.remote_x = 0.0
        self.remote_y = 0.0
        self.remote_mode = "2D"
        self.remote_label = "waiting"
        self.exit_on_peer = exit_on_peer
        self.exit_requested = False
        self._known_peer_id = None
        self._heartbeat_timer = 0.0
        self._preferred_role = preferred_role
        self._start(port)

    def _start(self, port: int) -> None:
        if self._preferred_role == "host":
            self._start_host(port)
            return
        if self._preferred_role == "client":
            self._start_client(port)
            return
        try:
            self._start_host(port)
        except Exception:
            self._start_client(port)

    def _start_host(self, port: int) -> None:
        manager = NetworkManager(self.context)
        self.endpoint = manager.host(NetworkProtocol.TCP, port)
        self.role = "host"
        self.label = "waiting"
        self.last_detail = f"Hosting localhost:{port}"

    def _start_client(self, port: int) -> None:
        manager = NetworkManager(self.context)
        self.endpoint = manager.connect(NetworkProtocol.TCP, "127.0.0.1", port)
        self.role = "client"
        self.label = "connected"
        self.last_detail = f"Connected to localhost:{port}"

    def update(self, dt: float, player_x: float, player_y: float, mode: str, packet_version: str) -> None:
        if self.endpoint is None:
            return
        self.endpoint.poll()
        self.peer_count = self.endpoint.peer_count()
        if self.role == "host" and self.peer_count <= 0:
            self.label = "waiting"
        elif self.role != "offline":
            self.label = "connected"
        packet = self.endpoint.receive()
        if packet is not None:
            self._known_peer_id = packet.peer_id
            parsed = self._parse_payload(packet.data)
            if parsed is not None:
                self.has_remote_state = True
                self.remote_x, self.remote_y, self.remote_mode, self.remote_label = parsed
                self.last_detail = f"Peer {packet.peer_id} synced in {self.remote_mode} mode"
            else:
                self.last_detail = f"Received {len(packet.data)} bytes from peer {packet.peer_id}"
        self._heartbeat_timer += dt
        if self._heartbeat_timer < 1.0:
            if self.exit_on_peer and self.has_remote_state:
                self.exit_requested = True
            return
        self._heartbeat_timer = 0.0
        payload = (
            f"sandbox|{packet_version}|{self.role}|{mode}|{player_x:0.1f}|{player_y:0.1f}|{self.label}"
        ).encode("utf-8")
        try:
            if self.endpoint.default_peer_id is not None:
                self.endpoint.send(payload)
            elif self._known_peer_id is not None:
                self.endpoint.send_to(self._known_peer_id, payload)
        except Exception as exc:
            self.last_detail = str(exc)
        if self.exit_on_peer and self.has_remote_state:
            self.exit_requested = True

    def _parse_payload(self, payload: bytes) -> tuple[float, float, str, str] | None:
        text = payload.decode("utf-8", errors="ignore")
        parts = text.split("|")
        if len(parts) == 7 and parts[0] == "sandbox" and parts[1] == "v1":
            _, _, _role, mode, x_raw, y_raw, label = parts
            try:
                return float(x_raw), float(y_raw), mode, label
            except ValueError:
                return None
        if len(parts) == 5 and parts[0] == "sandbox":
            _, _role, x_raw, y_raw, mode = parts
            try:
                return float(x_raw), float(y_raw), mode, "connected"
            except ValueError:
                return None
        return None

    def destroy(self) -> None:
        if self.endpoint is not None:
            try:
                self.endpoint.disconnect()
            except Exception:
                pass
        self.context.destroy()


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
    for scene in ("sandbox-2d", "sandbox-3d", "sandbox-hybrid"):
        try:
            scene_ids[scene] = context.scene_create(scene)
        except Exception:
            scene_ids[scene] = 0
    return scene_ids


def _draw_lines(
    game: GoudGame,
    font: int,
    lines: list[str],
    x: float,
    y: float,
    sizes: list[float],
    color: Color,
    max_width: float = 0.0,
    advance: Callable[[float], float] | None = None,
) -> None:
    current_y = y
    for index, line in enumerate(lines):
        size = sizes[index] if index < len(sizes) else sizes[-1]
        game.draw_text(
            font,
            line,
            x,
            current_y,
            size,
            TextAlignment.LEFT,
            max_width,
            1.0,
            0,
            color,
        )
        if advance is not None:
            current_y += advance(size)
        else:
            current_y += 26 if size >= 18 else 21


def _overview_advance(size: float) -> float:
    if size >= 38:
        return 44
    if size >= 24:
        return 30
    return 24


def _status_advance(size: float) -> float:
    return 38 if size >= 30 else 24


def _next_advance(size: float) -> float:
    return 38 if size >= 32 else 25


def _max_width_from_panel(panel: HudRect, text_x: float, margin: float) -> float:
    right = panel.x + panel.width * 0.5
    max_width = right - text_x - margin
    return max_width if max_width > 64 else 64


def _status_row(
    row: str,
    manifest: SandboxManifest,
    current_mode: str,
    mouse_x: float,
    mouse_y: float,
    caps,
    physics,
    audio,
    network: NetworkState,
) -> str:
    if row == "scene":
        return f"Scene: {manifest.scenes[[scene.mode for scene in manifest.scenes].index(current_mode)].label} ({manifest.scenes[[scene.mode for scene in manifest.scenes].index(current_mode)].key} to switch)"
    if row == "mouse":
        return f"Mouse marker: ({mouse_x:.0f}, {mouse_y:.0f})"
    if row == "render_caps":
        return f"Render caps: tex={caps.max_texture_size} instancing={caps.supports_instancing}"
    if row == "physics_caps":
        return f"Physics caps: joints={physics.supports_joints} maxBodies={physics.max_bodies}"
    if row == "audio_caps":
        return f"Audio caps: spatial={audio.supports_spatial} channels={audio.max_channels}"
    if row == "scene_count":
        return f"Scene count: {len(manifest.scenes)} active mode={current_mode}"
    if row == "target":
        return "Target: desktop"
    if row == "network_role":
        return f"Network role: {network.role} peers={network.peer_count} label={network.label}"
    if row == "network_detail":
        return f"Network detail: {network.last_detail}"
    return row


def _next_step_row(row: str, manifest: SandboxManifest, network: NetworkState, audio_activated: bool) -> str:
    if row == "audio_status":
        return f"Audio status: {'active' if audio_activated else 'press SPACE to activate'}"
    if row == "network_probe":
        return (
            f"Peer sprite live at ({network.remote_x:.0f}, {network.remote_y:.0f})"
            if network.has_remote_state
            else "Networking: open a second native sandbox to confirm peer sync."
        )
    return row


def main() -> int:
    manifest = _load_manifest()
    game = GoudGame(WINDOW_WIDTH, WINDOW_HEIGHT, "GoudEngine Sandbox - Python")
    ui = _make_ui()
    scene_context = GoudContext()
    _try_scene_setup(scene_context)
    network = NetworkState(
        manifest.network_port,
        _env_role("GOUD_SANDBOX_NETWORK_ROLE"),
        _env_flag("GOUD_SANDBOX_EXIT_ON_PEER"),
    )

    background = game.load_texture(manifest.assets.background)
    sprite = game.load_texture(manifest.assets.sprite)
    accent_sprite = game.load_texture(manifest.assets.accent_sprite)
    texture3d = game.load_texture(manifest.assets.texture3d)
    font = game.load_font(manifest.assets.font)

    cube = game.create_cube(texture3d, 1.2, 1.2, 1.2)
    plane = game.create_plane(texture3d, 8.0, 8.0)
    game.add_light(0, 4.0, 6.0, -4.0, 0.0, -1.0, 0.0, 1.0, 0.95, 0.80, 5.0, 28.0, 0.0)
    game.add_light(0, -3.5, 3.5, -2.0, 0.0, -0.65, 0.35, 0.70, 0.85, 1.0, 2.5, 18.0, 0.0)
    game.add_light(0, 0.0, 2.4, 7.0, 0.0, -0.25, -1.0, 0.55, 0.65, 0.90, 1.8, 20.0, 0.0)
    game.set_object_position(plane, 0.0, -1.2, 2.5)
    game.configure_grid(True, 12.0, 12)

    mode_index = 0
    mode_names = [scene.mode for scene in manifest.scenes]
    scene_labels = {scene.mode: scene.label for scene in manifest.scenes}
    player_x = 250.0
    player_y = 300.0
    angle = 0.0
    elapsed = 0.0
    audio_activated = False
    smoke_seconds = float(os.environ.get("GOUD_SANDBOX_SMOKE_SECONDS", "0") or "0")
    expect_peer = _env_flag("GOUD_SANDBOX_EXPECT_PEER")

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
            game.audio_play(manifest.assets.audio)

        current_mode = mode_names[mode_index]
        network.update(dt, player_x, player_y, current_mode, manifest.packet_version)
        mouse = game.get_mouse_position()

        game.begin_frame(0.07, 0.10, 0.14, 1.0)

        if current_mode in ("3D", "Hybrid"):
            game.enable_depth_test()
            game.set_camera_position3_d(0.0, 2.2, -7.0 if current_mode == "3D" else -7.8)
            game.set_camera_rotation3_d(-7.0, 0.0 if current_mode == "3D" else 8.0, 0.0)
            game.set_object_position(cube, 0.85, 1.2 + 0.26 * math.sin(angle * 2.0), 2.1)
            game.set_object_rotation(cube, 20.0, angle * 46.0, 0.0)
            game.render3_d()
            game.disable_depth_test()

        if current_mode == "2D":
            game.draw_sprite(background, WINDOW_WIDTH / 2, WINDOW_HEIGHT / 2, WINDOW_WIDTH, WINDOW_HEIGHT)
            game.draw_sprite(sprite, player_x, player_y, 64, 64, angle * 0.25)
            game.draw_sprite(accent_sprite, 1040, 420, 72, 240, 0.0)
            game.draw_quad(920.0, 260.0, 180.0, 40.0, Color(0.20, 0.55, 0.95, 0.80))

        if current_mode == "Hybrid":
            game.draw_quad(640.0, 360.0, 1280.0, 720.0, Color(0.08, 0.17, 0.24, 0.10))
            game.draw_quad(640.0, 654.0, 1280.0, 132.0, Color(0.03, 0.10, 0.12, 0.18))
            game.draw_sprite(sprite, player_x, player_y, 72, 72, angle * 0.25)
            game.draw_sprite(accent_sprite, 1044, 420, 78, 250, 0.0)
            game.draw_quad(920.0, 260.0, 180.0, 40.0, Color(0.20, 0.55, 0.95, 0.62))

        if current_mode in ("2D", "Hybrid"):
            if network.has_remote_state:
                game.draw_quad(
                    network.remote_x,
                    network.remote_y - 48,
                    108,
                    20,
                    Color(0.96, 0.70, 0.20, 0.92),
                )
                game.draw_text(
                    font,
                    f"Peer {network.remote_mode}",
                    network.remote_x - 40,
                    network.remote_y - 54,
                    14,
                    TextAlignment.LEFT,
                    0,
                    1.0,
                    0,
                    Color(0.04, 0.05, 0.08, 1.0),
                )
                game.draw_sprite(sprite, network.remote_x, network.remote_y, 52, 52, -angle * 0.18)

        layout = manifest.contract.layout
        panel_alpha = 0.62 if current_mode in ("3D", "Hybrid") else 0.92
        bottom_alpha = 0.70 if current_mode in ("3D", "Hybrid") else 0.94
        game.draw_quad(
            layout.overview_panel.x,
            layout.overview_panel.y,
            layout.overview_panel.width,
            layout.overview_panel.height,
            Color(0.05, 0.08, 0.12, panel_alpha),
        )
        game.draw_quad(
            layout.status_panel.x,
            layout.status_panel.y,
            layout.status_panel.width,
            layout.status_panel.height,
            Color(0.08, 0.12, 0.18, panel_alpha),
        )
        game.draw_quad(
            layout.next_panel.x,
            layout.next_panel.y,
            layout.next_panel.width,
            layout.next_panel.height,
            Color(0.05, 0.08, 0.12, bottom_alpha),
        )
        game.draw_quad(
            layout.scene_badge.x,
            layout.scene_badge.y,
            layout.scene_badge.width,
            layout.scene_badge.height,
            Color(0.20, 0.55, 0.95, 0.84),
        )
        game.draw_quad(mouse.x, mouse.y, 14, 14, Color(0.95, 0.85, 0.20, 0.95))

        caps = game.get_render_capabilities()
        physics = game.get_physics_capabilities()
        audio = game.get_audio_capabilities()
        network_caps = None
        try:
            network_caps = game.get_network_capabilities()
        except Exception:
            network_caps = None

        overview_max_width = _max_width_from_panel(layout.overview_panel, layout.overview_text.x, 24.0)
        next_max_width = _max_width_from_panel(layout.next_panel, layout.next_text.x, 24.0)
        overview_lines = [
            manifest.hud.overview_title,
            manifest.hud.tagline,
            *manifest.contract.overview_items,
        ]
        status_lines = [
            manifest.hud.status_title,
            *[
                _status_row(
                    row,
                    manifest,
                    current_mode,
                    mouse.x,
                    mouse.y,
                    caps,
                    physics,
                    audio,
                    network,
                )
                for row in manifest.contract.status_rows
            ],
        ]
        next_step_lines = [
            manifest.hud.next_steps_title,
            *manifest.contract.next_step_items,
            *[
                _next_step_row(row, manifest, network, audio_activated)
                for row in manifest.contract.next_step_dynamic_rows
            ],
        ]

        _draw_lines(
            game,
            font,
            overview_lines,
            layout.overview_text.x,
            layout.overview_text.title_y,
            [40, 24, 19],
            Color.white(),
            overview_max_width,
            _overview_advance,
        )
        _draw_lines(
            game,
            font,
            status_lines,
            layout.status_text.x,
            layout.status_text.title_y,
            [30, 18],
            Color(0.94, 0.97, 1.0, 1.0),
            layout.status_text.max_width,
            _status_advance,
        )
        _draw_lines(
            game,
            font,
            next_step_lines,
            layout.next_text.x,
            layout.next_text.title_y,
            [32, 19],
            Color(0.94, 0.97, 1.0, 1.0),
            next_max_width,
            _next_advance,
        )
        game.draw_text(
            font,
            scene_labels[current_mode],
            layout.scene_label.x,
            layout.scene_label.y,
            20,
            TextAlignment.LEFT,
            layout.scene_label.max_width,
            1.10,
            0,
            Color.white(),
        )

        ui.update()
        ui.render()
        game.end_frame()

        if network.exit_requested:
            game.close()
        if smoke_seconds > 0.0 and elapsed >= smoke_seconds:
            game.close()

    network.destroy()
    ui.destroy()
    scene_context.destroy()
    game.destroy()

    if expect_peer and not network.has_remote_state:
        print("Expected peer discovery before exit, but no remote peer state arrived.", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
