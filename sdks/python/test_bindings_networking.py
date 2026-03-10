#!/usr/bin/env python3
"""Handwritten networking-wrapper binding tests for the Python SDK."""

from test_bindings_common import NetworkSimulationConfig, _PACKAGE_DIR, _load_module


def test_handwritten_network_wrapper_exports():
    """Validate handwritten network wrapper exports and source API shape."""
    print("Testing handwritten network wrapper exports...")

    init_src = (_PACKAGE_DIR / "__init__.py").read_text()
    networking_path = _PACKAGE_DIR / "networking.py"
    assert networking_path.exists(), "missing handwritten networking wrapper module"

    networking_src = networking_path.read_text()
    assert "from .networking import NetworkManager, NetworkEndpoint" in init_src, \
        "__init__.py must export NetworkManager and NetworkEndpoint"
    assert "class NetworkManager:" in networking_src, "missing NetworkManager class"
    assert "class NetworkEndpoint:" in networking_src, "missing NetworkEndpoint class"
    assert "def host(self, protocol, port):" in networking_src, "missing NetworkManager.host()"
    assert "def connect(self, protocol, address, port):" in networking_src, "missing NetworkManager.connect()"
    assert "def receive(self):" in networking_src, "missing NetworkEndpoint.receive()"
    assert "def send(self, data, channel = 0):" in networking_src, "missing NetworkEndpoint.send()"
    assert "def send_to(self, peer_id, data, channel = 0):" in networking_src, "missing NetworkEndpoint.send_to()"
    assert "def poll(self):" in networking_src, "missing NetworkEndpoint.poll()"
    assert "def disconnect(self):" in networking_src, "missing NetworkEndpoint.disconnect()"
    assert "def get_stats(self):" in networking_src, "missing NetworkEndpoint.get_stats()"
    assert "def peer_count(self):" in networking_src, "missing NetworkEndpoint.peer_count()"
    assert "def set_simulation(self, config):" in networking_src, "missing NetworkEndpoint.set_simulation()"
    assert "def clear_simulation(self):" in networking_src, "missing NetworkEndpoint.clear_simulation()"
    assert "def set_overlay_target(self):" in networking_src, "missing NetworkEndpoint.set_overlay_target()"
    assert "def clear_overlay_target(self):" in networking_src, "missing NetworkEndpoint.clear_overlay_target()"
    assert '"NetworkConnectResult",' in init_src, "__init__.py must export NetworkConnectResult"
    assert '"NetworkPacket",' in init_src, "__init__.py must export NetworkPacket"
    assert "network_connect_with_peer" in networking_src, "connect() must use network_connect_with_peer"

    print("  Handwritten network wrapper export tests passed")
    return True


def test_handwritten_network_wrapper_send_contract_source():
    """Validate endpoint send() and receive() wrapper contracts."""
    print("Testing handwritten network wrapper send() contract...")

    networking_path = _PACKAGE_DIR / "networking.py"
    networking_src = networking_path.read_text()
    assert "if self.default_peer_id is None:" in networking_src, \
        "send() must check for missing default peer ID"
    assert "raise ValueError" in networking_src, \
        "send() must fail clearly when no default peer ID exists"
    assert "return self.send_to(self.default_peer_id, data, channel)" in networking_src, \
        "send() must route through send_to() using default peer ID"
    assert "raise RuntimeError(f\"network_host failed with handle {handle}\")" in networking_src, \
        "host() must fail clearly when the backend returns a negative handle"

    networking_mod = _load_module("_networking", networking_path)
    NetworkManager = networking_mod.NetworkManager

    class _ConnectResult:
        def __init__(self, handle, peer_id):
            self.handle = handle
            self.peer_id = peer_id

    class _FakePacket:
        def __init__(self, peer_id, data):
            self.peer_id = peer_id
            self.data = data

    class _FakeBackend:
        def __init__(self):
            self.calls = []
            self._packet = _FakePacket(42, b"payload")
            self._stats = "stats"

        def network_host(self, protocol, port):
            self.calls.append(("network_host", protocol, port))
            return 1001

        def network_connect_with_peer(self, protocol, address, port):
            self.calls.append(("network_connect_with_peer", protocol, address, port))
            return _ConnectResult(1002, 77)

        def network_send(self, handle, peer_id, data, channel):
            self.calls.append(("network_send", handle, peer_id, data, channel))
            return 0

        def network_poll(self, handle):
            self.calls.append(("network_poll", handle))
            return 0

        def network_receive_packet(self, handle):
            self.calls.append(("network_receive_packet", handle))
            packet = self._packet
            self._packet = None
            return packet

        def network_disconnect(self, handle):
            self.calls.append(("network_disconnect", handle))
            return 0

        def get_network_stats(self, handle):
            self.calls.append(("get_network_stats", handle))
            return self._stats

        def network_peer_count(self, handle):
            self.calls.append(("network_peer_count", handle))
            return 2

        def set_network_simulation(self, handle, config):
            self.calls.append(("set_network_simulation", handle, config))
            return 0

        def clear_network_simulation(self, handle):
            self.calls.append(("clear_network_simulation", handle))
            return 0

        def set_network_overlay_handle(self, handle):
            self.calls.append(("set_network_overlay_handle", handle))
            return 0

        def clear_network_overlay_handle(self):
            self.calls.append(("clear_network_overlay_handle",))
            return 0

    backend = _FakeBackend()
    manager = NetworkManager(backend)
    host_endpoint = manager.host(2, 40000)
    assert host_endpoint.default_peer_id is None, "host() should not seed a default peer ID"

    try:
        host_endpoint.send(b"hello")
        assert False, "send() must raise ValueError when no default peer ID exists"
    except ValueError as exc:
        assert "default peer ID" in str(exc), "send() error should clearly mention default peer ID"

    connect_endpoint = manager.connect(2, "127.0.0.1", 40000)
    assert connect_endpoint.default_peer_id == 77, "connect() should seed default peer ID"
    assert connect_endpoint.send(b"hello") == 0
    assert connect_endpoint.send_to(88, b"other", 1) == 0
    assert connect_endpoint.poll() == 0
    assert connect_endpoint.get_stats() == "stats"
    assert connect_endpoint.peer_count() == 2
    sim = NetworkSimulationConfig(one_way_latency_ms=5, jitter_ms=1, packet_loss_percent=0.5)
    assert connect_endpoint.set_simulation(sim) == 0
    assert connect_endpoint.clear_simulation() == 0
    assert connect_endpoint.set_overlay_target() == 0
    assert connect_endpoint.clear_overlay_target() == 0

    packet = connect_endpoint.receive()
    assert packet is not None, "receive() should return packet when available"
    assert packet.peer_id == 42
    assert packet.data == b"payload"
    assert connect_endpoint.receive() is None, "receive() should return None when no packet exists"

    class _NegativeHostBackend:
        def network_host(self, protocol, port):
            return -17

    try:
        NetworkManager(_NegativeHostBackend()).host(2, 40001)
        assert False, "host() must raise when the backend returns a negative handle"
    except RuntimeError as exc:
        assert str(exc) == "network_host failed with handle -17", \
            "host() should surface the negative handle clearly"

    print("  Handwritten network wrapper send() contract tests passed")
    return True
