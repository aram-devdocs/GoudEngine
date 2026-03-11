using System;
using System.Reflection;
using GoudEngine;
using Xunit;

namespace GoudEngine.Tests.Runtime;

public class ErrorAndDiagnosticRuntimeTests
{
    [Fact]
    public void DiagnosticMode_And_GoudException_Helpers_Execute()
    {
        DiagnosticMode.SetEnabled(true);
        _ = DiagnosticMode.IsEnabled;
        _ = DiagnosticMode.LastBacktrace;
        DiagnosticMode.SetEnabled(false);

        var lastError = GoudException.FromLastError();
        if (lastError != null)
        {
            Assert.IsAssignableFrom<GoudException>(lastError);
        }

        var categoryFromCode = typeof(GoudException).GetMethod("CategoryFromCode", BindingFlags.NonPublic | BindingFlags.Static);
        Assert.NotNull(categoryFromCode);
        Assert.Equal("Context", categoryFromCode!.Invoke(null, new object[] { 1 }));
        Assert.Equal("Resource", categoryFromCode.Invoke(null, new object[] { 100 }));
        Assert.Equal("Graphics", categoryFromCode.Invoke(null, new object[] { 200 }));
        Assert.Equal("Entity", categoryFromCode.Invoke(null, new object[] { 300 }));
        Assert.Equal("Input", categoryFromCode.Invoke(null, new object[] { 400 }));
        Assert.Equal("System", categoryFromCode.Invoke(null, new object[] { 500 }));
        Assert.Equal("Provider", categoryFromCode.Invoke(null, new object[] { 600 }));
        Assert.Equal("Internal", categoryFromCode.Invoke(null, new object[] { 900 }));
        Assert.Equal("Unknown", categoryFromCode.Invoke(null, new object[] { -1 }));

        var createTyped = typeof(GoudException).GetMethod("CreateTyped", BindingFlags.NonPublic | BindingFlags.Static);
        Assert.NotNull(createTyped);

        Assert.IsType<GoudContextException>(InvokeCreateTyped(createTyped!, 1, "Context"));
        Assert.IsType<GoudResourceException>(InvokeCreateTyped(createTyped!, 100, "Resource"));
        Assert.IsType<GoudGraphicsException>(InvokeCreateTyped(createTyped!, 200, "Graphics"));
        Assert.IsType<GoudEntityException>(InvokeCreateTyped(createTyped!, 300, "Entity"));
        Assert.IsType<GoudInputException>(InvokeCreateTyped(createTyped!, 400, "Input"));
        Assert.IsType<GoudSystemException>(InvokeCreateTyped(createTyped!, 500, "System"));
        Assert.IsType<GoudProviderException>(InvokeCreateTyped(createTyped!, 600, "Provider"));
        Assert.IsType<GoudInternalException>(InvokeCreateTyped(createTyped!, 900, "Internal"));
        Assert.IsType<GoudException>(InvokeCreateTyped(createTyped!, -1, "Unknown"));
    }

    private static GoudException InvokeCreateTyped(MethodInfo createTyped, int code, string category)
    {
        return Assert.IsAssignableFrom<GoudException>(createTyped.Invoke(
            null,
            new object[] { code, "message", category, "subsystem", "operation", RecoveryClass.Degraded, "hint" }
        ));
    }
}
