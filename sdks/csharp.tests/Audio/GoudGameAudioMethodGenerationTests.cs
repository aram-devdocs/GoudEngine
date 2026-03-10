using System;
using System.IO;
using GoudEngine;
using Xunit;

namespace GoudEngine.Tests.Audio;

public class GoudGameAudioMethodGenerationTests
{
    [Fact]
    public void GoudGame_Has_Core_Audio_Methods()
    {
        AssertMethod("AudioPlay", typeof(long), typeof(byte[]));
        AssertMethod("AudioPlayOnChannel", typeof(long), typeof(byte[]), typeof(byte));
        AssertMethod(
            "AudioPlayWithSettings",
            typeof(long),
            typeof(byte[]),
            typeof(float),
            typeof(float),
            typeof(bool),
            typeof(byte)
        );

        AssertMethod("AudioStop", typeof(int), typeof(ulong));
        AssertMethod("AudioPause", typeof(int), typeof(ulong));
        AssertMethod("AudioResume", typeof(int), typeof(ulong));
        AssertMethod("AudioStopAll", typeof(int));
        AssertMethod("AudioSetGlobalVolume", typeof(int), typeof(float));
        AssertMethod("AudioGetGlobalVolume", typeof(float));
        AssertMethod("AudioSetChannelVolume", typeof(int), typeof(byte), typeof(float));
        AssertMethod("AudioGetChannelVolume", typeof(float), typeof(byte));
        AssertMethod("AudioIsPlaying", typeof(int), typeof(ulong));
        AssertMethod("AudioActiveCount", typeof(int));
        AssertMethod("AudioCleanupFinished", typeof(int));
        AssertMethod("AudioActivate", typeof(int));

        var generatedSource = ReadGeneratedGoudGameSource();
        var audioActivateIndex = generatedSource.IndexOf("public int AudioActivate()", StringComparison.Ordinal);
        Assert.True(audioActivateIndex >= 0, "missing AudioActivate wrapper in generated GoudGame source");
        Assert.True(
            generatedSource.IndexOf("return NativeMethods.goud_audio_activate(_ctx);", audioActivateIndex, StringComparison.Ordinal) >= 0,
            "AudioActivate must map to goud_audio_activate"
        );
        Assert.True(
            generatedSource.IndexOf(
                "return NativeMethods.goud_audio_cleanup_finished(_ctx);",
                audioActivateIndex,
                StringComparison.Ordinal
            ) == -1,
            "AudioActivate must not map to goud_audio_cleanup_finished"
        );
    }

    [Fact]
    public void GoudGame_Has_Spatial_And_Crossfade_Audio_Methods()
    {
        AssertMethod(
            "AudioPlaySpatial3d",
            typeof(long),
            typeof(byte[]),
            typeof(float),
            typeof(float),
            typeof(float),
            typeof(float),
            typeof(float),
            typeof(float),
            typeof(float),
            typeof(float)
        );
        AssertMethod(
            "AudioUpdateSpatial3d",
            typeof(int),
            typeof(ulong),
            typeof(float),
            typeof(float),
            typeof(float),
            typeof(float),
            typeof(float),
            typeof(float),
            typeof(float),
            typeof(float)
        );
        AssertMethod("AudioSetListenerPosition3d", typeof(int), typeof(float), typeof(float), typeof(float));
        AssertMethod("AudioSetSourcePosition3d", typeof(int), typeof(ulong), typeof(float), typeof(float), typeof(float), typeof(float), typeof(float));
        AssertMethod("AudioSetPlayerVolume", typeof(int), typeof(ulong), typeof(float));
        AssertMethod("AudioSetPlayerSpeed", typeof(int), typeof(ulong), typeof(float));
        AssertMethod("AudioCrossfade", typeof(int), typeof(ulong), typeof(ulong), typeof(float));
        AssertMethod("AudioCrossfadeTo", typeof(long), typeof(ulong), typeof(byte[]), typeof(float), typeof(byte));
        AssertMethod("AudioMixWith", typeof(long), typeof(ulong), typeof(byte[]), typeof(float), typeof(byte));
        AssertMethod("AudioUpdateCrossfades", typeof(int), typeof(float));
        AssertMethod("AudioActiveCrossfadeCount", typeof(int));
    }

    private static void AssertMethod(string name, Type returnType, params Type[] parameterTypes)
    {
        var method = typeof(GoudGame).GetMethod(name, parameterTypes);
        Assert.NotNull(method);
        Assert.Equal(returnType, method!.ReturnType);
    }

    private static string ReadGeneratedGoudGameSource()
    {
        var current = AppContext.BaseDirectory;
        for (var i = 0; i < 12; i++)
        {
            var candidate = Path.Combine(current, "sdks", "csharp", "generated", "GoudGame.g.cs");
            if (File.Exists(candidate))
            {
                return File.ReadAllText(candidate);
            }

            var parent = Directory.GetParent(current);
            if (parent == null)
            {
                break;
            }

            current = parent.FullName;
        }

        var cwdCandidate = Path.Combine(Directory.GetCurrentDirectory(), "sdks", "csharp", "generated", "GoudGame.g.cs");
        if (File.Exists(cwdCandidate))
        {
            return File.ReadAllText(cwdCandidate);
        }

        throw new FileNotFoundException(
            "Could not locate generated C# SDK source at sdks/csharp/generated/GoudGame.g.cs"
        );
    }
}
