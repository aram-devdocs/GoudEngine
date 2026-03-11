using System;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Net;
using System.Net.Sockets;
using System.Reflection;
using System.Runtime.CompilerServices;
using System.Threading;
using GoudEngine;
using Xunit;

namespace GoudEngine.Tests.Network;

public class NetworkWrapperTests
{
    [Fact]
    public void NetworkWrappers_Are_Public_And_Expose_Expected_Api()
    {
        var assembly = typeof(GoudGame).Assembly;
        var managerType = assembly.GetType("GoudEngine.NetworkManager");
        var endpointType = assembly.GetType("GoudEngine.NetworkEndpoint");

        Assert.NotNull(managerType);
        Assert.NotNull(endpointType);

        Assert.NotNull(managerType!.GetConstructor(new[] { typeof(GoudGame) }));
        Assert.NotNull(managerType.GetConstructor(new[] { typeof(GoudContext) }));

        AssertMethod(managerType, "Host", endpointType!, typeof(NetworkProtocol), typeof(ushort));
        AssertMethod(managerType, "Connect", endpointType, typeof(NetworkProtocol), typeof(string), typeof(ushort));

        AssertProperty(endpointType, "Handle", typeof(long));
        AssertProperty(endpointType, "DefaultPeerId", typeof(ulong?));

        AssertMethod(endpointType, "Receive", typeof(NetworkPacket?), Type.EmptyTypes);
        AssertMethod(endpointType, "Send", typeof(int), typeof(byte[]), typeof(byte));
        AssertMethod(endpointType, "SendTo", typeof(int), typeof(ulong), typeof(byte[]), typeof(byte));
        AssertMethod(endpointType, "Poll", typeof(int), Type.EmptyTypes);
        AssertMethod(endpointType, "Disconnect", typeof(int), Type.EmptyTypes);
        AssertMethod(endpointType, "GetStats", typeof(NetworkStats), Type.EmptyTypes);
        AssertMethod(endpointType, "PeerCount", typeof(int), Type.EmptyTypes);
        AssertMethod(endpointType, "SetSimulation", typeof(int), typeof(NetworkSimulationConfig));
        AssertMethod(endpointType, "ClearSimulation", typeof(int), Type.EmptyTypes);
        AssertMethod(endpointType, "SetOverlayTarget", typeof(int), Type.EmptyTypes);
        AssertMethod(endpointType, "ClearOverlayTarget", typeof(int), Type.EmptyTypes);
    }

    [Fact]
    public void Host_Endpoint_Send_Without_Default_Peer_Fails_Clearly()
    {
        using var context = new GoudContext();
        var port = ReservePort();
        var manager = CreateNetworkManager(context);
        var endpoint = Invoke(manager, "Host", NetworkProtocol.Tcp, (ushort)port);

        var ex = Assert.Throws<TargetInvocationException>(
            () => Invoke(endpoint, "Send", new byte[] { 0x01 }, (byte)0)
        );

        Assert.IsType<InvalidOperationException>(ex.InnerException);
        Assert.Contains(
            "default peer",
            ex.InnerException!.Message,
            StringComparison.OrdinalIgnoreCase
        );

        _ = Invoke(endpoint, "Disconnect");
    }

    [Fact]
    public void Host_Source_Guards_Against_Negative_Host_Handle()
    {
        var source = ReadNetworkManagerSource();

        Assert.Contains("if (handle < 0)", source, StringComparison.Ordinal);
        Assert.Contains(
            "throw new InvalidOperationException($\"NetworkHost failed with handle {handle}.\");",
            source,
            StringComparison.Ordinal
        );
    }

    [Fact]
    public void NetworkWrappers_Can_Exchange_Loopback_Tcp_Packets()
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
            var defaultPeerId = (ulong?)GetProperty(clientEndpoint, "DefaultPeerId");
            Assert.True(defaultPeerId.HasValue);
            Assert.NotEqual(0UL, defaultPeerId.Value);

            var ping = new byte[] { 0x70, 0x69, 0x6E, 0x67 };
            var pong = new byte[] { 0x70, 0x6F, 0x6E, 0x67 };

            WaitForConnectedPeers(hostEndpoint, clientEndpoint);
            Assert.Equal(0, (int)Invoke(clientEndpoint, "Send", ping, (byte)0));

            var hostPacket = WaitForPacket(hostEndpoint, clientEndpoint, "host should receive ping");
            Assert.NotNull(hostPacket);
            Assert.Equal(ping, hostPacket!.Value.Data);
            Assert.NotEqual(0UL, hostPacket.Value.PeerId);

            Assert.Equal(0, (int)Invoke(hostEndpoint, "SendTo", hostPacket.Value.PeerId, pong, (byte)0));

            var clientPacket = WaitForPacket(clientEndpoint, hostEndpoint, "client should receive pong");
            Assert.NotNull(clientPacket);
            Assert.Equal(pong, clientPacket!.Value.Data);
            Assert.Equal(defaultPeerId.Value, clientPacket.Value.PeerId);

            Pump(hostEndpoint, clientEndpoint, iterations: 10);

            Assert.True((int)Invoke(hostEndpoint, "PeerCount") > 0);
            Assert.True((int)Invoke(clientEndpoint, "PeerCount") > 0);

            var hostStats = (NetworkStats)Invoke(hostEndpoint, "GetStats");
            var clientStats = (NetworkStats)Invoke(clientEndpoint, "GetStats");

            Assert.True(hostStats.BytesReceived > 0);
            Assert.True(clientStats.BytesSent > 0);
            Assert.True(clientStats.BytesReceived > 0);
        }
        finally
        {
            _ = Invoke(clientEndpoint, "Disconnect");
            _ = Invoke(hostEndpoint, "Disconnect");
        }
    }

    private static object CreateNetworkManager(object target)
    {
        var managerType = typeof(GoudGame).Assembly.GetType("GoudEngine.NetworkManager");
        Assert.NotNull(managerType);
        return Activator.CreateInstance(managerType!, target)!;
    }

    private static NetworkPacket? WaitForPacket(object receiver, object other, string message)
    {
        var sw = Stopwatch.StartNew();
        while (sw.Elapsed < TimeSpan.FromSeconds(5))
        {
            Pump(receiver, other);
            var packet = (NetworkPacket?)Invoke(receiver, "Receive");
            if (packet.HasValue)
            {
                return packet;
            }

            Thread.Sleep(10);
        }

        throw new Xunit.Sdk.XunitException(message);
    }

    private static string ReadNetworkManagerSource([CallerFilePath] string sourceFile = "")
    {
        var repoRoot = Path.GetFullPath(
            Path.Combine(Path.GetDirectoryName(sourceFile)!, "..", "..", "..")
        );
        var sourcePath = Path.Combine(repoRoot, "sdks", "csharp", "NetworkManager.cs");
        if (File.Exists(sourcePath))
        {
            return File.ReadAllText(sourcePath);
        }

        throw new FileNotFoundException(
            $"Could not locate generated C# wrapper source at {sourcePath}"
        );
    }

    private static void WaitForConnectedPeers(object first, object second)
    {
        var sw = Stopwatch.StartNew();
        while (sw.Elapsed < TimeSpan.FromSeconds(5))
        {
            Pump(first, second);
            var firstPeerCount = (int)Invoke(first, "PeerCount");
            var secondPeerCount = (int)Invoke(second, "PeerCount");
            if (firstPeerCount > 0 && secondPeerCount > 0)
            {
                return;
            }

            Thread.Sleep(10);
        }

        throw new Xunit.Sdk.XunitException("timed out waiting for connected peers");
    }

    private static void Pump(object first, object second, int iterations = 1)
    {
        for (var i = 0; i < iterations; i++)
        {
            _ = Invoke(first, "Poll");
            _ = Invoke(second, "Poll");
            Thread.Sleep(10);
        }
    }

    private static object? GetProperty(object target, string name)
    {
        var prop = target.GetType().GetProperty(name, BindingFlags.Instance | BindingFlags.Public);
        Assert.NotNull(prop);
        return prop!.GetValue(target);
    }

    private static object Invoke(object target, string name, params object[] args)
    {
        var method = target.GetType().GetMethods(BindingFlags.Instance | BindingFlags.Public)
            .SingleOrDefault(m => MethodMatches(m, name, args));
        Assert.NotNull(method);
        return method!.Invoke(target, args)!;
    }

    private static bool MethodMatches(MethodInfo method, string name, object[] args)
    {
        if (!string.Equals(method.Name, name, StringComparison.Ordinal))
        {
            return false;
        }

        var parameters = method.GetParameters();
        if (parameters.Length != args.Length)
        {
            return false;
        }

        for (var i = 0; i < parameters.Length; i++)
        {
            if (args[i] is null)
            {
                if (parameters[i].ParameterType.IsValueType
                    && Nullable.GetUnderlyingType(parameters[i].ParameterType) is null)
                {
                    return false;
                }

                continue;
            }

            if (!parameters[i].ParameterType.IsInstanceOfType(args[i])
                && parameters[i].ParameterType != args[i].GetType())
            {
                return false;
            }
        }

        return true;
    }

    private static void AssertMethod(Type type, string name, Type returnType, params Type[] parameterTypes)
    {
        var method = type.GetMethod(name, parameterTypes);
        Assert.NotNull(method);
        Assert.Equal(returnType, method!.ReturnType);
    }

    private static void AssertProperty(Type type, string name, Type propertyType)
    {
        var prop = type.GetProperty(name, BindingFlags.Instance | BindingFlags.Public);
        Assert.NotNull(prop);
        Assert.Equal(propertyType, prop!.PropertyType);
    }

    private static int ReservePort()
    {
        using var listener = new TcpListener(IPAddress.Loopback, 0);
        listener.Start();
        return ((IPEndPoint)listener.LocalEndpoint).Port;
    }
}
