using System;
using System.Net;
using System.Net.Sockets;
using System.Reflection;
using GoudEngine;
using Xunit;

namespace GoudEngine.Tests.Runtime;

internal static class RuntimeTestSupport
{
    public static (GoudContext Context, GoudGame Game) CreateHeadlessGame(string title = "Headless")
    {
        var context = new GoudContext();
        var ctor = typeof(GoudGame).GetConstructor(
            BindingFlags.Instance | BindingFlags.NonPublic,
            binder: null,
            new[] { typeof(GoudContextId), typeof(string) },
            modifiers: null
        );
        Assert.NotNull(ctor);

        var ctx = GetContextId(context);
        var game = Assert.IsType<GoudGame>(ctor!.Invoke(new object[] { ctx, title }));
        return (context, game);
    }

    public static GoudContextId GetContextId(GoudContext context)
    {
        var ctxField = typeof(GoudContext).GetField("_ctx", BindingFlags.Instance | BindingFlags.NonPublic);
        Assert.NotNull(ctxField);
        return Assert.IsType<GoudContextId>(ctxField!.GetValue(context));
    }

    public static void DestroyContext(GoudContext context)
    {
        _ = context.Destroy();
    }

    public static int ReservePort()
    {
        var listener = new TcpListener(IPAddress.Loopback, 0);
        listener.Start();
        var port = ((IPEndPoint)listener.LocalEndpoint).Port;
        listener.Stop();
        return port;
    }

    public static void SetPrivateField<TTarget, TValue>(TTarget target, string name, TValue value)
    {
        var field = typeof(TTarget).GetField(name, BindingFlags.Instance | BindingFlags.NonPublic);
        Assert.NotNull(field);
        field!.SetValue(target, value);
    }
}
