#!/usr/bin/env python3
"""Runtime networking tests for handwritten Python wrappers."""

import socket
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))


class _SkipTest(Exception):
    pass


def _reserve_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def _wait_for_packet(endpoint, companion_endpoint=None, timeout_sec=3.0):
    deadline = time.monotonic() + timeout_sec
    while time.monotonic() < deadline:
        endpoint.poll()
        if companion_endpoint is not None:
            companion_endpoint.poll()
        packet = endpoint.receive()
        if packet is not None:
            return packet
        time.sleep(0.01)
    return None


def _wait_for_peer_counts(host_endpoint, client_endpoint, timeout_sec=3.0):
    deadline = time.monotonic() + timeout_sec
    while time.monotonic() < deadline:
        host_endpoint.poll()
        client_endpoint.poll()
        if host_endpoint.peer_count() > 0 and client_endpoint.peer_count() > 0:
            return True
        time.sleep(0.01)
    return False


def _test_send_requires_default_peer():
    from goud_engine import GoudContext, NetworkManager, NetworkProtocol

    ctx = GoudContext()
    endpoint = None
    try:
        endpoint = NetworkManager(ctx).host(NetworkProtocol.TCP, _reserve_port())
    except RuntimeError as exc:
        ctx.destroy()
        raise _SkipTest(f"TCP host unavailable: {exc}") from exc

    try:
        try:
            endpoint.send(b"missing-default-peer")
            raise AssertionError("send() should fail when endpoint has no default peer ID")
        except ValueError as exc:
            if "default peer ID" not in str(exc):
                raise AssertionError(
                    f"send() should mention default peer ID, got: {exc}"
                ) from exc
    finally:
        endpoint.disconnect()
        ctx.destroy()


def _test_tcp_loopback_roundtrip():
    from goud_engine import GoudContext, NetworkManager, NetworkProtocol

    host_ctx = GoudContext()
    client_ctx = GoudContext()
    host_endpoint = None
    client_endpoint = None
    try:
        port = _reserve_port()
        host_endpoint = NetworkManager(host_ctx).host(NetworkProtocol.TCP, port)
        client_endpoint = NetworkManager(client_ctx).connect(
            NetworkProtocol.TCP, "127.0.0.1", port
        )

        if client_endpoint.default_peer_id is None:
            raise AssertionError("connect() should seed a default peer ID")
        if not _wait_for_peer_counts(host_endpoint, client_endpoint):
            raise AssertionError("host/client did not report connected peers in time")

        client_payload = b"python-loopback-client"
        if client_endpoint.send(client_payload) < 0:
            raise AssertionError("client send() returned a negative status")

        host_packet = _wait_for_packet(host_endpoint, client_endpoint, timeout_sec=4.0)
        if host_packet is None:
            raise AssertionError("host did not receive client payload in time")
        if host_packet.data != client_payload:
            raise AssertionError(
                f"host payload mismatch: expected {client_payload!r}, got {host_packet.data!r}"
            )

        host_payload = b"python-loopback-host"
        if host_endpoint.send_to(host_packet.peer_id, host_payload) < 0:
            raise AssertionError("host send_to() returned a negative status")

        client_packet = _wait_for_packet(client_endpoint, host_endpoint, timeout_sec=4.0)
        if client_packet is None:
            raise AssertionError("client did not receive host payload in time")
        if client_packet.data != host_payload:
            raise AssertionError(
                f"client payload mismatch: expected {host_payload!r}, got {client_packet.data!r}"
            )
        if client_packet.peer_id != client_endpoint.default_peer_id:
            raise AssertionError(
                f"client peer ID mismatch: expected {client_endpoint.default_peer_id}, got {client_packet.peer_id}"
            )

        for _ in range(10):
            host_endpoint.poll()
            client_endpoint.poll()
            time.sleep(0.01)

        host_stats = host_endpoint.get_stats()
        client_stats = client_endpoint.get_stats()
        if host_stats.bytes_received <= 0:
            raise AssertionError("host stats should record received bytes")
        if client_stats.bytes_sent <= 0:
            raise AssertionError("client stats should record sent bytes")
        if client_stats.bytes_received <= 0:
            raise AssertionError("client stats should record received bytes")
    except RuntimeError as exc:
        raise _SkipTest(f"TCP loopback unavailable: {exc}") from exc
    finally:
        if client_endpoint is not None:
            client_endpoint.disconnect()
        if host_endpoint is not None:
            host_endpoint.disconnect()
        client_ctx.destroy()
        host_ctx.destroy()


def main() -> int:
    try:
        from goud_engine import GoudContext  # noqa: F401
    except Exception as exc:
        print(f"ERROR: Python networking loopback tests require native library: {exc}")
        return 1

    tests = [
        ("send_requires_default_peer", _test_send_requires_default_peer),
        ("tcp_loopback_roundtrip", _test_tcp_loopback_roundtrip),
    ]
    failed = 0
    print("=" * 60)
    print(" GoudEngine Python Networking Loopback Tests")
    print("=" * 60)
    for name, test in tests:
        try:
            test()
            print(f"PASS {name}")
        except _SkipTest as skip_exc:
            print(f"SKIP {name}: {skip_exc}")
        except Exception as exc:
            failed += 1
            print(f"FAIL {name}: {exc}")

    print("=" * 60)
    print(f" Results: {len(tests) - failed} passed/skipped, {failed} failed")
    print("=" * 60)
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
