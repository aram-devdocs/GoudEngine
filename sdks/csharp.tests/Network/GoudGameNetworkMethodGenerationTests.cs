using System;
using System.IO;
using System.Runtime.CompilerServices;
using GoudEngine;
using Xunit;

namespace GoudEngine.Tests.Network;

public class GoudGameNetworkMethodGenerationTests
{
    [Fact]
    public void GoudGame_Has_Network_Methods_With_Primitive_Signatures()
    {
        AssertMethod("NetworkHost", typeof(long), typeof(int), typeof(ushort));
        AssertMethod("NetworkConnect", typeof(long), typeof(int), typeof(string), typeof(ushort));
        AssertMethod("NetworkDisconnect", typeof(int), typeof(long));
        AssertMethod("NetworkSend", typeof(int), typeof(long), typeof(ulong), typeof(byte[]), typeof(byte));
        AssertMethod("NetworkReceive", typeof(byte[]), typeof(long));
        AssertMethod("NetworkPoll", typeof(int), typeof(long));
        AssertMethod("GetNetworkStats", typeof(NetworkStats), typeof(long));
        AssertMethod("NetworkPeerCount", typeof(int), typeof(long));
        AssertMethod("SetNetworkSimulation", typeof(int), typeof(long), typeof(NetworkSimulationConfig));
        AssertMethod("ClearNetworkSimulation", typeof(int), typeof(long));
        AssertMethod("SetNetworkOverlayHandle", typeof(int), typeof(long));
        AssertMethod("ClearNetworkOverlayHandle", typeof(int));
    }

    [Fact]
    public void GoudGame_Network_Methods_Map_To_Expected_Ffi()
    {
        var generatedSource = ReadGeneratedGoudGameSource();

        Assert.Contains("return NativeMethods.goud_network_host(_ctx, protocol, port);", generatedSource, StringComparison.Ordinal);
        Assert.Contains("return NativeMethods.goud_network_connect(_ctx, protocol,", generatedSource, StringComparison.Ordinal);
        Assert.Contains("var _status = NativeMethods.goud_network_get_stats_v2(_ctx, handle, ref _stats);", generatedSource, StringComparison.Ordinal);
        Assert.Contains("throw new InvalidOperationException($\"goud_network_get_stats_v2 failed with status {_status}.\");", generatedSource, StringComparison.Ordinal);
        Assert.Contains("return NativeMethods.goud_network_set_overlay_handle(_ctx, handle);", generatedSource, StringComparison.Ordinal);
        Assert.Contains("return NativeMethods.goud_network_clear_overlay_handle(_ctx);", generatedSource, StringComparison.Ordinal);
    }

    private static void AssertMethod(string name, Type returnType, params Type[] parameterTypes)
    {
        var method = typeof(GoudGame).GetMethod(name, parameterTypes);
        Assert.NotNull(method);
        Assert.Equal(returnType, method!.ReturnType);
    }

    private static string ReadGeneratedGoudGameSource([CallerFilePath] string sourceFile = "")
    {
        var repoRoot = Path.GetFullPath(
            Path.Combine(Path.GetDirectoryName(sourceFile)!, "..", "..", "..")
        );
        var generatedPath = Path.Combine(repoRoot, "sdks", "csharp", "generated", "GoudGame.g.cs");
        if (File.Exists(generatedPath))
        {
            return File.ReadAllText(generatedPath);
        }

        throw new FileNotFoundException(
            $"Could not locate generated C# SDK source at {generatedPath}"
        );
    }
}
