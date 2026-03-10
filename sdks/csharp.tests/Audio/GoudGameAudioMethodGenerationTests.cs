using System;
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
}
