"""Handwritten networking wrappers built on top of generated low-level APIs."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any, Optional

if TYPE_CHECKING:
    from .generated._types import NetworkPacket


class NetworkEndpoint:
    """Thin wrapper around a low-level network handle."""

    def __init__(self, backend: Any, handle: int, default_peer_id: Optional[int] = None):
        self._backend = backend
        self.handle = handle
        self.default_peer_id = default_peer_id

    def poll(self) -> int:
        return self._backend.network_poll(self.handle)

    def receive(self):
        status = self._backend.network_poll(self.handle)
        if status < 0:
            raise RuntimeError(f"goud_network_poll failed with status {status}")
        return self._backend.network_receive_packet(self.handle)

    def send(self, data, channel = 0):
        if self.default_peer_id is None:
            raise ValueError(
                "No default peer ID is set for this endpoint; use send_to(peer_id, data, channel)."
            )
        return self.send_to(self.default_peer_id, data, channel)

    def send_to(self, peer_id, data, channel = 0):
        return self._backend.network_send(self.handle, peer_id, data, channel)

    def disconnect(self):
        return self._backend.network_disconnect(self.handle)


class NetworkManager:
    """Factory for creating NetworkEndpoint wrappers from a game/context backend."""

    def __init__(self, backend: Any):
        self._backend = backend

    def host(self, protocol, port):
        handle = self._backend.network_host(protocol, port)
        if handle < 0:
            raise RuntimeError(f"goud_network_host failed with handle {handle}")
        return NetworkEndpoint(self._backend, handle, None)

    def connect(self, protocol, address, port):
        result = self._backend.network_connect_with_peer(protocol, address, port)
        return NetworkEndpoint(self._backend, result.handle, result.peer_id)
