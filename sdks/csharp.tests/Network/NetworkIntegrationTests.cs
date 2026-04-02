using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Threading;
using GoudEngine;
using Xunit;
using Xunit.Abstractions;
using static GoudEngine.Tests.Network.NetworkTestSupport;

namespace GoudEngine.Tests.Network;

/// <summary>
/// Integration tests validating C# networking SDK for Throne server/client architecture.
///
/// Observed characteristics (localhost, macOS arm64, debug build):
/// - TCP bidirectional: 4KB payloads delivered reliably with correct stats tracking.
/// - TCP throughput: 10k x 32-byte messages delivered with zero loss.
/// - UDP basic: 5-message exchange reliable on localhost with send-retry for provider warmup.
/// - UDP throughput: 99.5%+ delivery on localhost; sends may return ERR_PROVIDER_OPERATION_FAILED (602)
///   when the provider is saturated — the test tracks accepted vs attempted sends.
/// - Multi-client: 3 simultaneous TCP clients connect and exchange messages reliably.
/// - Disconnect/reconnect: TCP reconnect on the same port works; new connection gets a new peer ID.
///
/// Known limitations:
/// - UDP sends can fail with error 602 when tests run back-to-back due to global network registry
///   state. Tests use send-retry loops to handle this.
/// - The NetworkEndpoint wrapper does not expose per-peer stats; only aggregate stats are available.
/// </summary>
public class NetworkIntegrationTests
{
    private readonly ITestOutputHelper _output;

    public NetworkIntegrationTests(ITestOutputHelper output)
    {
        _output = output;
    }

    [Fact]
    public void Udp_Host_Connect_Send_Receive()
    {
        using var hostContext = new GoudContext();
        using var clientContext = new GoudContext();

        var port = ReservePort();
        var hostManager = CreateNetworkManager(hostContext);
        var clientManager = CreateNetworkManager(clientContext);
        var hostEndpoint = Invoke(hostManager, "Host", NetworkProtocol.UDP, (ushort)port);
        var clientEndpoint = Invoke(clientManager, "Connect", NetworkProtocol.UDP, "127.0.0.1", (ushort)port);

        try
        {
            WaitForConnectedPeers(hostEndpoint, clientEndpoint);
            // UDP providers may need extra time after connection is established
            Pump(hostEndpoint, clientEndpoint, iterations: 5);

            // Send 5 distinct messages from client to host
            var messages = new List<byte[]>();
            for (var i = 0; i < 5; i++)
            {
                var msg = new byte[] { (byte)(0x10 + i), (byte)(0x20 + i), (byte)(0x30 + i) };
                messages.Add(msg);
                // Retry send if provider is temporarily busy
                var sendResult = -1;
                var retrySw = Stopwatch.StartNew();
                while (sendResult != 0 && retrySw.Elapsed < TimeSpan.FromSeconds(2))
                {
                    sendResult = (int)Invoke(clientEndpoint, "Send", msg, (byte)0);
                    if (sendResult != 0)
                    {
                        Thread.Sleep(20);
                        Pump(hostEndpoint, clientEndpoint);
                    }
                }
                Assert.Equal(0, sendResult);
                Thread.Sleep(20);
            }

            // Receive all 5 on host
            var received = new List<NetworkPacket>();
            var sw = Stopwatch.StartNew();
            while (received.Count < 5 && sw.Elapsed < TimeSpan.FromSeconds(10))
            {
                Pump(hostEndpoint, clientEndpoint);
                var packet = (NetworkPacket?)Invoke(hostEndpoint, "Receive");
                if (packet.HasValue)
                {
                    received.Add(packet.Value);
                }

                Thread.Sleep(10);
            }

            Assert.Equal(5, received.Count);

            for (var i = 0; i < 5; i++)
            {
                Assert.Equal(messages[i], received[i].Data);
                Assert.NotEqual(0UL, received[i].PeerId);
            }

            Assert.True((int)Invoke(hostEndpoint, "PeerCount") > 0);
            Assert.True((int)Invoke(clientEndpoint, "PeerCount") > 0);
        }
        finally
        {
            _ = Invoke(clientEndpoint, "Disconnect");
            _ = Invoke(hostEndpoint, "Disconnect");
        }
    }

    [Fact]
    public void Tcp_Bidirectional_Large_Payload()
    {
        using var hostContext = new GoudContext();
        using var clientContext = new GoudContext();

        var port = ReservePort();
        var hostManager = CreateNetworkManager(hostContext);
        var clientManager = CreateNetworkManager(clientContext);
        var hostEndpoint = Invoke(hostManager, "Host", NetworkProtocol.Tcp, (ushort)port);
        var clientEndpoint = Invoke(clientManager, "Connect", NetworkProtocol.Tcp, "127.0.0.1", (ushort)port);

        try
        {
            WaitForConnectedPeers(hostEndpoint, clientEndpoint);

            // 4KB payloads
            var clientPayload = new byte[4096];
            var hostPayload = new byte[4096];
            new Random(42).NextBytes(clientPayload);
            new Random(99).NextBytes(hostPayload);

            // Client sends to host
            Assert.Equal(0, (int)Invoke(clientEndpoint, "Send", clientPayload, (byte)0));

            var hostPacket = WaitForPacket(hostEndpoint, clientEndpoint, "host should receive 4KB payload");
            Assert.NotNull(hostPacket);
            Assert.Equal(clientPayload, hostPacket!.Value.Data);

            // Host sends back to client
            Assert.Equal(0, (int)Invoke(hostEndpoint, "SendTo", hostPacket.Value.PeerId, hostPayload, (byte)0));

            var clientPacket = WaitForPacket(clientEndpoint, hostEndpoint, "client should receive 4KB payload");
            Assert.NotNull(clientPacket);
            Assert.Equal(hostPayload, clientPacket!.Value.Data);

            // Verify stats reflect data transfer
            Pump(hostEndpoint, clientEndpoint, iterations: 5);
            var hostStats = (NetworkStats)Invoke(hostEndpoint, "GetStats");
            var clientStats = (NetworkStats)Invoke(clientEndpoint, "GetStats");

            Assert.True(hostStats.BytesReceived >= 4096);
            Assert.True(hostStats.BytesSent >= 4096);
            Assert.True(clientStats.BytesReceived >= 4096);
            Assert.True(clientStats.BytesSent >= 4096);
        }
        finally
        {
            _ = Invoke(clientEndpoint, "Disconnect");
            _ = Invoke(hostEndpoint, "Disconnect");
        }
    }

    [Fact]
    public void Multiple_Clients_Connect_To_Server()
    {
        using var hostContext = new GoudContext();
        using var client1Context = new GoudContext();
        using var client2Context = new GoudContext();
        using var client3Context = new GoudContext();

        var port = ReservePort();
        var hostManager = CreateNetworkManager(hostContext);
        var hostEndpoint = Invoke(hostManager, "Host", NetworkProtocol.Tcp, (ushort)port);

        var clientManagers = new[]
        {
            CreateNetworkManager(client1Context),
            CreateNetworkManager(client2Context),
            CreateNetworkManager(client3Context),
        };

        var clientEndpoints = new object[3];
        for (var i = 0; i < 3; i++)
        {
            clientEndpoints[i] = Invoke(clientManagers[i], "Connect", NetworkProtocol.Tcp, "127.0.0.1", (ushort)port);
        }

        try
        {
            // Wait for all 3 clients to connect
            var sw = Stopwatch.StartNew();
            while (sw.Elapsed < TimeSpan.FromSeconds(10))
            {
                PumpMulti(hostEndpoint, clientEndpoints);
                if ((int)Invoke(hostEndpoint, "PeerCount") >= 3)
                {
                    break;
                }

                Thread.Sleep(10);
            }

            Assert.True((int)Invoke(hostEndpoint, "PeerCount") >= 3, "host should have 3 peers");

            // Each client sends a unique message
            for (var i = 0; i < 3; i++)
            {
                var msg = new byte[] { (byte)(0xA0 + i), (byte)(0xB0 + i) };
                Assert.Equal(0, (int)Invoke(clientEndpoints[i], "Send", msg, (byte)0));
            }

            // Host receives all 3 messages
            var received = new List<NetworkPacket>();
            sw = Stopwatch.StartNew();
            while (received.Count < 3 && sw.Elapsed < TimeSpan.FromSeconds(10))
            {
                PumpMulti(hostEndpoint, clientEndpoints);
                var packet = (NetworkPacket?)Invoke(hostEndpoint, "Receive");
                if (packet.HasValue)
                {
                    received.Add(packet.Value);
                }

                Thread.Sleep(10);
            }

            Assert.Equal(3, received.Count);

            // Verify each message is unique
            var dataSet = received.Select(p => BitConverter.ToString(p.Data)).ToHashSet();
            Assert.Equal(3, dataSet.Count);

            // Host broadcasts to all clients
            var broadcast = new byte[] { 0xFF, 0xFE };
            foreach (var pkt in received)
            {
                Assert.Equal(0, (int)Invoke(hostEndpoint, "SendTo", pkt.PeerId, broadcast, (byte)0));
            }

            // Each client receives the broadcast
            for (var i = 0; i < 3; i++)
            {
                var clientPacket = WaitForPacket(clientEndpoints[i], hostEndpoint, $"client {i} should receive broadcast");
                Assert.NotNull(clientPacket);
                Assert.Equal(broadcast, clientPacket!.Value.Data);
            }
        }
        finally
        {
            for (var i = 0; i < 3; i++)
            {
                _ = Invoke(clientEndpoints[i], "Disconnect");
            }

            _ = Invoke(hostEndpoint, "Disconnect");
        }
    }

    [Fact]
    public void Client_Disconnect_And_Reconnect()
    {
        using var hostContext = new GoudContext();
        using var clientContext = new GoudContext();

        var port = ReservePort();
        var hostManager = CreateNetworkManager(hostContext);
        var clientManager = CreateNetworkManager(clientContext);
        var hostEndpoint = Invoke(hostManager, "Host", NetworkProtocol.Tcp, (ushort)port);
        var clientEndpoint = Invoke(clientManager, "Connect", NetworkProtocol.Tcp, "127.0.0.1", (ushort)port);

        try
        {
            WaitForConnectedPeers(hostEndpoint, clientEndpoint);

            // First exchange
            var msg1 = new byte[] { 0x01, 0x02 };
            Assert.Equal(0, (int)Invoke(clientEndpoint, "Send", msg1, (byte)0));
            var pkt1 = WaitForPacket(hostEndpoint, clientEndpoint, "host should receive first message");
            Assert.NotNull(pkt1);
            Assert.Equal(msg1, pkt1!.Value.Data);

            // Disconnect client
            _ = Invoke(clientEndpoint, "Disconnect");
            Thread.Sleep(200);

            // Poll host to process disconnect event (same endpoint for both args is intentional)
            for (var i = 0; i < 10; i++)
            {
                _ = Invoke(hostEndpoint, "Poll");
                Thread.Sleep(10);
            }

            // Reconnect on same port
            clientEndpoint = Invoke(clientManager, "Connect", NetworkProtocol.Tcp, "127.0.0.1", (ushort)port);
            WaitForConnectedPeers(hostEndpoint, clientEndpoint);

            // Second exchange after reconnect
            var msg2 = new byte[] { 0x03, 0x04 };
            Assert.Equal(0, (int)Invoke(clientEndpoint, "Send", msg2, (byte)0));
            var pkt2 = WaitForPacket(hostEndpoint, clientEndpoint, "host should receive message after reconnect");
            Assert.NotNull(pkt2);
            Assert.Equal(msg2, pkt2!.Value.Data);
        }
        finally
        {
            _ = Invoke(clientEndpoint, "Disconnect");
            _ = Invoke(hostEndpoint, "Disconnect");
        }
    }

    [Fact]
    public void Tcp_Throughput_10k_Messages()
    {
        using var hostContext = new GoudContext();
        using var clientContext = new GoudContext();

        var port = ReservePort();
        var hostManager = CreateNetworkManager(hostContext);
        var clientManager = CreateNetworkManager(clientContext);
        var hostEndpoint = Invoke(hostManager, "Host", NetworkProtocol.Tcp, (ushort)port);
        var clientEndpoint = Invoke(clientManager, "Connect", NetworkProtocol.Tcp, "127.0.0.1", (ushort)port);

        const int messageCount = 10_000;
        const int messageSize = 32;

        try
        {
            WaitForConnectedPeers(hostEndpoint, clientEndpoint);

            var payload = new byte[messageSize];
            new Random(7).NextBytes(payload);

            var sw = Stopwatch.StartNew();

            // Send all messages
            for (var i = 0; i < messageCount; i++)
            {
                var result = (int)Invoke(clientEndpoint, "Send", payload, (byte)0);
                Assert.Equal(0, result);

                // Pump periodically to avoid buffer overflow
                if (i % 100 == 0)
                {
                    Pump(hostEndpoint, clientEndpoint);
                }
            }

            // Receive all messages — drain all buffered packets per poll cycle.
            // TCP guarantees delivery so we rely on the timeout, not empty-poll count.
            var receivedCount = 0;
            var receiveTimeout = Stopwatch.StartNew();
            while (receivedCount < messageCount && receiveTimeout.Elapsed < TimeSpan.FromSeconds(30))
            {
                _ = Invoke(hostEndpoint, "Poll");
                _ = Invoke(clientEndpoint, "Poll");

                var gotAny = false;
                while (true)
                {
                    var packet = (NetworkPacket?)Invoke(hostEndpoint, "Receive");
                    if (!packet.HasValue) break;
                    receivedCount++;
                    gotAny = true;
                }

                if (!gotAny)
                {
                    Thread.Sleep(5);
                }
            }

            sw.Stop();

            _output.WriteLine($"TCP received: {receivedCount}/{messageCount}");

            Assert.Equal(messageCount, receivedCount);

            var elapsedMs = sw.ElapsedMilliseconds;
            var throughput = messageCount * 1000.0 / Math.Max(elapsedMs, 1);
            var bandwidthKBps = (messageCount * (double)messageSize) / Math.Max(elapsedMs, 1);

            _output.WriteLine($"TCP throughput: {throughput:F0} msg/s, {bandwidthKBps:F1} KB/s, {elapsedMs} ms total");
        }
        finally
        {
            _ = Invoke(clientEndpoint, "Disconnect");
            _ = Invoke(hostEndpoint, "Disconnect");
        }
    }

    [Fact]
    public void Udp_Throughput_10k_Messages()
    {
        using var hostContext = new GoudContext();
        using var clientContext = new GoudContext();

        var port = ReservePort();
        var hostManager = CreateNetworkManager(hostContext);
        var clientManager = CreateNetworkManager(clientContext);
        var hostEndpoint = Invoke(hostManager, "Host", NetworkProtocol.UDP, (ushort)port);
        var clientEndpoint = Invoke(clientManager, "Connect", NetworkProtocol.UDP, "127.0.0.1", (ushort)port);

        const int messageCount = 10_000;
        const int messageSize = 32;
        const double minDeliveryRate = 0.995;

        try
        {
            WaitForConnectedPeers(hostEndpoint, clientEndpoint);

            var payload = new byte[messageSize];
            new Random(13).NextBytes(payload);

            var sw = Stopwatch.StartNew();
            var receivedCount = 0;
            var sentCount = 0;

            // Send in batches with aggressive polling to avoid buffer overflow
            for (var i = 0; i < messageCount; i++)
            {
                var sendResult = (int)Invoke(clientEndpoint, "Send", payload, (byte)0);
                if (sendResult == 0)
                {
                    sentCount++;
                }

                // Poll and drain every 50 messages to keep buffers clear
                if (i % 50 == 0)
                {
                    _ = Invoke(hostEndpoint, "Poll");
                    _ = Invoke(clientEndpoint, "Poll");
                    while (true)
                    {
                        var pkt = (NetworkPacket?)Invoke(hostEndpoint, "Receive");
                        if (!pkt.HasValue) break;
                        receivedCount++;
                    }
                }
            }

            // Drain remaining packets
            var emptyPolls = 0;
            var receiveTimeout = Stopwatch.StartNew();
            while (receiveTimeout.Elapsed < TimeSpan.FromSeconds(10))
            {
                _ = Invoke(hostEndpoint, "Poll");
                _ = Invoke(clientEndpoint, "Poll");

                var gotAny = false;
                while (true)
                {
                    var pkt = (NetworkPacket?)Invoke(hostEndpoint, "Receive");
                    if (!pkt.HasValue) break;
                    receivedCount++;
                    gotAny = true;
                }

                if (!gotAny)
                {
                    emptyPolls++;
                    if (emptyPolls > 50) break;
                    Thread.Sleep(5);
                }
                else
                {
                    emptyPolls = 0;
                }
            }

            sw.Stop();

            // Require that most sends succeeded
            Assert.True(sentCount > messageCount * 0.9,
                $"Only {sentCount}/{messageCount} sends succeeded — provider may be saturated");

            var deliveryRate = sentCount > 0 ? (double)receivedCount / sentCount : 0;

            _output.WriteLine($"UDP throughput: {receivedCount}/{sentCount} delivered ({deliveryRate:P2}), {sentCount}/{messageCount} sends accepted");
            _output.WriteLine($"Elapsed: {sw.ElapsedMilliseconds} ms");

            Assert.True(
                deliveryRate >= minDeliveryRate,
                $"UDP delivery rate {deliveryRate:P2} is below required {minDeliveryRate:P2} ({receivedCount}/{sentCount})"
            );
        }
        finally
        {
            _ = Invoke(clientEndpoint, "Disconnect");
            _ = Invoke(hostEndpoint, "Disconnect");
        }
    }

    [Fact]
    public void Serialization_Round_Trip()
    {
        using var hostContext = new GoudContext();
        using var clientContext = new GoudContext();

        var port = ReservePort();
        var hostManager = CreateNetworkManager(hostContext);
        var clientManager = CreateNetworkManager(clientContext);
        var hostEndpoint = Invoke(hostManager, "Host", NetworkProtocol.Tcp, (ushort)port);
        var clientEndpoint = Invoke(clientManager, "Connect", NetworkProtocol.Tcp, "127.0.0.1", (ushort)port);

        try
        {
            WaitForConnectedPeers(hostEndpoint, clientEndpoint);

            // Serialize structured data
            var entityId = 42u;
            var posX = 123.456f;
            var posY = 789.012f;
            var name = "TestEntity";
            var active = true;

            byte[] serialized;
            using (var ms = new MemoryStream())
            using (var writer = new BinaryWriter(ms))
            {
                writer.Write(entityId);
                writer.Write(posX);
                writer.Write(posY);
                writer.Write(name);
                writer.Write(active);
                writer.Flush();
                serialized = ms.ToArray();
            }

            Assert.Equal(0, (int)Invoke(clientEndpoint, "Send", serialized, (byte)0));

            var hostPacket = WaitForPacket(hostEndpoint, clientEndpoint, "host should receive serialized data");
            Assert.NotNull(hostPacket);

            // Deserialize and verify
            using (var ms = new MemoryStream(hostPacket!.Value.Data))
            using (var reader = new BinaryReader(ms))
            {
                Assert.Equal(entityId, reader.ReadUInt32());
                Assert.Equal(posX, reader.ReadSingle());
                Assert.Equal(posY, reader.ReadSingle());
                Assert.Equal(name, reader.ReadString());
                Assert.Equal(active, reader.ReadBoolean());
            }
        }
        finally
        {
            _ = Invoke(clientEndpoint, "Disconnect");
            _ = Invoke(hostEndpoint, "Disconnect");
        }
    }
}
