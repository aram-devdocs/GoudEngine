using System;
using GoudEngine;
using Xunit;

namespace GoudEngine.Tests;

public class GoudGameAnimationMethodGenerationTests
{
    [Fact]
    public void GoudGame_HasPlayAndStopMethods()
    {
        AssertMethod("Play", typeof(int), typeof(Entity));
        AssertMethod("Stop", typeof(int), typeof(Entity));
    }

    [Fact]
    public void GoudGame_HasSetStateMethod()
    {
        AssertMethod("SetState", typeof(int), typeof(Entity), typeof(string));
    }

    [Fact]
    public void GoudGame_HasSetParameterMethods()
    {
        AssertMethod(
            "SetParameterBool",
            typeof(int),
            typeof(Entity),
            typeof(string),
            typeof(bool)
        );
        AssertMethod(
            "SetParameterFloat",
            typeof(int),
            typeof(Entity),
            typeof(string),
            typeof(float)
        );
    }

    private static void AssertMethod(string name, Type returnType, params Type[] parameterTypes)
    {
        var method = typeof(GoudGame).GetMethod(name, parameterTypes);
        Assert.NotNull(method);
        Assert.Equal(returnType, method!.ReturnType);
    }
}
