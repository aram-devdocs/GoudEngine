using System;
using System.Diagnostics;
using System.Linq;
using System.Net;
using System.Net.Sockets;
using System.Reflection;
using System.Threading;
using GoudEngine;
using Xunit;

namespace GoudEngine.Tests.Network;

internal static class NetworkTestSupport
{
    public static object CreateNetworkManager(object target)
    {
        var managerType = typeof(GoudGame).Assembly.GetType("GoudEngine.NetworkManager");
        Assert.NotNull(managerType);
        return Activator.CreateInstance(managerType!, target)!;
    }

    public static NetworkPacket? WaitForPacket(object receiver, object other, string message)
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

    public static void WaitForConnectedPeers(object first, object second)
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

    public static void Pump(object first, object second, int iterations = 1)
    {
        for (var i = 0; i < iterations; i++)
        {
            _ = Invoke(first, "Poll");
            _ = Invoke(second, "Poll");
            Thread.Sleep(10);
        }
    }

    public static void PumpMulti(object host, object[] clients, int iterations = 1)
    {
        for (var i = 0; i < iterations; i++)
        {
            _ = Invoke(host, "Poll");
            foreach (var client in clients)
            {
                _ = Invoke(client, "Poll");
            }

            Thread.Sleep(10);
        }
    }

    public static object? GetProperty(object target, string name)
    {
        var prop = target.GetType().GetProperty(name, BindingFlags.Instance | BindingFlags.Public);
        Assert.NotNull(prop);
        return prop!.GetValue(target);
    }

    public static object Invoke(object target, string name, params object[] args)
    {
        var method = target.GetType().GetMethods(BindingFlags.Instance | BindingFlags.Public)
            .SingleOrDefault(m => MethodMatches(m, name, args));
        Assert.NotNull(method);
        return method!.Invoke(target, args)!;
    }

    public static int ReservePort()
    {
        using var listener = new TcpListener(IPAddress.Loopback, 0);
        listener.Start();
        return ((IPEndPoint)listener.LocalEndpoint).Port;
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
}
