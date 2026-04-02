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

    /// <summary>
    /// Calls an action that may fail in headless/provider-limited environments.
    /// Swallows expected exceptions so coverage tests can exercise wrapper lines
    /// without requiring a real GPU or window.
    /// </summary>
    public static void CallGame(Action action)
    {
        try
        {
            action();
        }
        catch (Exception ex) when (ex is EntryPointNotFoundException or InvalidOperationException or GoudException)
        {
            // Headless/provider-limited environments can reject some game-only calls.
        }
    }

    /// <inheritdoc cref="CallGame(Action)"/>
    public static void CallGame<T>(Func<T> action)
    {
        try
        {
            _ = action();
        }
        catch (Exception ex) when (ex is EntryPointNotFoundException or InvalidOperationException or GoudException)
        {
            // Headless/provider-limited environments can reject some game-only calls.
        }
    }
}
