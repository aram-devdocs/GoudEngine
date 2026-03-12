using System;
using System.IO;
using System.Reflection;
using System.Runtime.CompilerServices;
using GoudEngine;
using Xunit;

namespace GoudEngine.Tests.Debugger;

public class DebuggerWrapperTests
{
    [Fact]
    public void GoudGame_And_Context_Expose_Debugger_Surface()
    {
        AssertMethod(typeof(GoudGame), "GetDebuggerSnapshotJson", typeof(string));
        AssertMethod(typeof(GoudGame), "GetDebuggerManifestJson", typeof(string));
        AssertMethod(typeof(GoudGame), "SetDebuggerProfilingEnabled", typeof(void), typeof(bool));
        AssertMethod(typeof(GoudGame), "SetDebuggerSelectedEntity", typeof(void), typeof(ulong));
        AssertMethod(typeof(GoudGame), "ClearDebuggerSelectedEntity", typeof(void));
        AssertMethod(typeof(GoudGame), "GetMemorySummary", typeof(MemorySummary));

        AssertMethod(typeof(GoudContext), "GetDebuggerSnapshotJson", typeof(string));
        AssertMethod(typeof(GoudContext), "GetDebuggerManifestJson", typeof(string));
        AssertMethod(typeof(GoudContext), "SetDebuggerProfilingEnabled", typeof(void), typeof(bool));
        AssertMethod(typeof(GoudContext), "SetDebuggerSelectedEntity", typeof(void), typeof(ulong));
        AssertMethod(typeof(GoudContext), "ClearDebuggerSelectedEntity", typeof(void));
        AssertMethod(typeof(GoudContext), "GetMemorySummary", typeof(MemorySummary));
        Assert.NotNull(typeof(GoudContext).GetConstructor(new[] { typeof(ContextConfig) }));

        AssertMethod(typeof(EngineConfig), "SetDebugger", typeof(EngineConfig), typeof(DebuggerConfig));
    }

    [Fact]
    public void Generated_Debugger_Wrappers_Map_To_Expected_Ffi()
    {
        var gameSource = ReadGeneratedSource("GoudGame.g.cs");
        var contextSource = ReadGeneratedSource("GoudContext.g.cs");
        var nativeSource = ReadGeneratedSource("NativeMethods.g.cs");

        Assert.Contains("NativeMethods.goud_debugger_get_snapshot_json(_ctx, IntPtr.Zero, (nuint)0)", gameSource, StringComparison.Ordinal);
        Assert.Contains("NativeMethods.goud_debugger_get_manifest_json(IntPtr.Zero, (nuint)0)", gameSource, StringComparison.Ordinal);
        Assert.Contains("return new MemorySummary(new MemoryCategoryStats(_summary.Rendering.CurrentBytes, _summary.Rendering.PeakBytes)", gameSource, StringComparison.Ordinal);
        Assert.Contains("NativeMethods.goud_debugger_get_manifest_json(IntPtr.Zero, (nuint)0)", contextSource, StringComparison.Ordinal);
        Assert.Contains("public GoudContext(ContextConfig config)", contextSource, StringComparison.Ordinal);
        Assert.Contains("NativeMethods.goud_context_create_with_config(ref _configFfi);", contextSource, StringComparison.Ordinal);
        Assert.Contains("public static extern int goud_debugger_get_manifest_json(IntPtr buf, nuint buf_len);", nativeSource, StringComparison.Ordinal);
        Assert.Contains("public static extern int goud_debugger_get_snapshot_json(GoudContextId context_id, IntPtr buf, nuint buf_len);", nativeSource, StringComparison.Ordinal);
        Assert.Contains("public static extern bool goud_engine_config_set_debugger(IntPtr handle, ref GoudDebuggerConfig debugger);", nativeSource, StringComparison.Ordinal);
        Assert.Contains("public static extern GoudContextId goud_context_create_with_config(ref GoudContextConfig config);", nativeSource, StringComparison.Ordinal);
    }

    [Fact]
    public void Extension_Helpers_Are_Publicly_Available()
    {
        Assert.NotNull(typeof(DebuggerJsonExtensions).GetMethod("ParseDebuggerSnapshot", new[] { typeof(GoudGame) }));
        Assert.NotNull(typeof(DebuggerJsonExtensions).GetMethod("ParseDebuggerManifest", new[] { typeof(GoudGame) }));
        Assert.NotNull(typeof(DebuggerJsonExtensions).GetMethod("ParseDebuggerSnapshot", new[] { typeof(GoudContext) }));
        Assert.NotNull(typeof(DebuggerJsonExtensions).GetMethod("ParseDebuggerManifest", new[] { typeof(GoudContext) }));
    }

    [Fact]
    public void Debugger_Public_Value_Types_Are_Usable_From_CSharp()
    {
        var debugger = new DebuggerConfig(true, true, "editor");
        var contextConfig = new ContextConfig(debugger);
        var summary = new MemorySummary(
            new MemoryCategoryStats(10, 11),
            new MemoryCategoryStats(12, 13),
            new MemoryCategoryStats(14, 15),
            new MemoryCategoryStats(16, 17),
            new MemoryCategoryStats(18, 19),
            new MemoryCategoryStats(20, 21),
            new MemoryCategoryStats(22, 23),
            new MemoryCategoryStats(24, 25),
            136,
            144
        );

        Assert.True(debugger.Enabled);
        Assert.True(debugger.PublishLocalAttach);
        Assert.Equal("editor", debugger.RouteLabel);
        Assert.True(contextConfig.Debugger.Enabled);
        Assert.Equal((ulong)10, summary.Rendering.CurrentBytes);
        Assert.Equal((ulong)25, summary.Other.PeakBytes);
        Assert.Equal((ulong)136, summary.TotalCurrentBytes);
        Assert.Equal((ulong)144, summary.TotalPeakBytes);
    }

    private static void AssertMethod(Type type, string name, Type returnType, params Type[] parameterTypes)
    {
        var method = type.GetMethod(name, parameterTypes);
        Assert.NotNull(method);
        Assert.Equal(returnType, method!.ReturnType);
    }

    private static string ReadGeneratedSource(string fileName, [CallerFilePath] string sourceFile = "")
    {
        var repoRoot = Path.GetFullPath(
            Path.Combine(Path.GetDirectoryName(sourceFile)!, "..", "..", "..")
        );
        var generatedPath = Path.Combine(repoRoot, "sdks", "csharp", "generated", fileName);
        if (File.Exists(generatedPath))
        {
            return File.ReadAllText(generatedPath);
        }

        throw new FileNotFoundException(
            $"Could not locate generated C# SDK source at {generatedPath}"
        );
    }
}
