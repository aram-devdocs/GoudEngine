using System;
using System.Diagnostics;
using System.Linq;
using System.Runtime.InteropServices;
using System.Threading;
using GoudEngine;
using Xunit;

namespace GoudEngine.Tests.Runtime;

public class ContextAndGameRuntimeTests
{
    [Fact]
    public void GoudContext_Entity_Component_Scene_And_Generic_Component_Apis_Work()
    {
        var context = new GoudContext();
        try
        {
            Assert.True(context.IsValid());
            Assert.True(context.GetNetworkCapabilities().MaxMessageSize >= 0);

            var sceneId = context.SceneCreate("coverage-main");
            var secondarySceneId = context.SceneCreate("coverage-menu");
            Assert.Equal(sceneId, context.SceneGetByName("coverage-main"));
            Assert.True(context.SceneCount() >= 2);
            _ = context.SceneGetCurrent();
            Assert.True(context.SetActiveScene(sceneId, true).Success);
            Assert.True(context.SceneSetActive(secondarySceneId, false).Success);
            Assert.True(context.SceneIsActive(sceneId));
            Assert.True(context.SceneSetCurrent(secondarySceneId).Success);
            Assert.Equal(secondarySceneId, context.SceneGetCurrent());
            Assert.True(context.SceneTransitionTo(sceneId, secondarySceneId, TransitionType.Instant, 0.01f).Success);
            _ = context.SceneTransitionProgress();
            _ = context.SceneTransitionIsActive();
            _ = context.SceneTransitionTick(0.01f);

            var entity = context.SpawnEmpty();
            var spawned = context.SpawnBatch(2);
            Assert.True(context.IsAlive(entity));
            Assert.Equal(2, spawned.Length);

            var alive = new byte[spawned.Length];
            Assert.Equal((uint)spawned.Length, context.IsAliveBatch(spawned, alive));
            Assert.All(alive, value => Assert.Equal((byte)1, value));
            Assert.True(context.EntityCount() >= 3);

            var transform = Transform2D.FromPosition(1f, 2f);
            context.AddTransform2d(entity, transform);
            _ = context.HasTransform2d(entity);
            _ = context.GetTransform2d(entity);
            context.SetTransform2d(entity, Transform2D.FromPositionRotation(5f, 6f, 0.5f));
            _ = context.GetTransform2d(entity);
            _ = context.RemoveTransform2d(entity);

            context.AddName(entity, "hero");
            _ = context.HasName(entity);
            _ = context.GetName(entity);
            _ = context.RemoveName(entity);

            var sprite = Sprite.New(7UL).WithColor(0.1f, 0.2f, 0.3f, 1f);
            context.AddSprite(entity, sprite);
            _ = context.HasSprite(entity);
            _ = context.GetSprite(entity);
            context.SetSprite(entity, sprite.WithCustomSize(32f, 16f));
            _ = context.GetSprite(entity);
            _ = context.RemoveSprite(entity);

            var typeId = BitConverter.ToUInt64(Guid.NewGuid().ToByteArray(), 0);
            Assert.True(context.ComponentRegisterType(typeId, "CoverageCounter", 4, 4));

            var data = Marshal.AllocHGlobal(4);
            try
            {
                Marshal.WriteInt32(data, 42);
                _ = context.ComponentAdd(entity, typeId, data, 4);
                _ = context.ComponentHas(entity, typeId);
                _ = context.ComponentGet(entity, typeId);
                _ = context.ComponentGetMut(entity, typeId);

                _ = context.ComponentAddBatch(spawned, typeId, data, 4);
                var hasBatch = new byte[spawned.Length];
                _ = context.ComponentHasBatch(spawned, typeId, hasBatch);
                _ = context.ComponentRemoveBatch(spawned, typeId);
                _ = context.ComponentRemove(entity, typeId);
            }
            finally
            {
                Marshal.FreeHGlobal(data);
            }

            Assert.Equal((uint)spawned.Length, context.DespawnBatch(spawned));
            Assert.True(context.Despawn(entity));
            Assert.True(context.SceneDestroy(secondarySceneId).Success);
            Assert.True(context.SceneDestroy(sceneId).Success);
        }
        finally
        {
            RuntimeTestSupport.DestroyContext(context);
        }
    }

    [Fact]
    public void Headless_GoudGame_Executes_State_Ecs_Collision_And_Utility_Apis()
    {
        var (context, game) = RuntimeTestSupport.CreateHeadlessGame("HeadlessCoverage");
        try
        {
            RuntimeTestSupport.SetPrivateField(game, "_deltaTime", 0.5f);
            RuntimeTestSupport.SetPrivateField(game, "_totalTime", 12.5d);
            RuntimeTestSupport.SetPrivateField(game, "_frameCount", 7U);

            Assert.Equal(0.5f, game.DeltaTime, 3);
            Assert.Equal(2f, game.Fps, 3);
            Assert.Equal("HeadlessCoverage", game.Title);
            Assert.Equal(12.5d, game.TotalTime, 3);
            Assert.Equal(7U, game.FrameCount);

            CallGame(() => game.WindowWidth);
            CallGame(() => game.WindowHeight);
            CallGame(() => game.ShouldClose());
            CallGame(() => game.Close());
            CallGame(() => game.BeginFrame(0.1f, 0.2f, 0.3f, 1f));
            CallGame(() => game.EndFrame());
            CallGame(() => game.LoadTexture("assets/missing.png"));
            CallGame(() => game.DestroyTexture(0));
            CallGame(() => game.LoadFont("assets/missing.ttf"));
            CallGame(() => game.DestroyFont(0));
            CallGame(() => game.DrawText(0, "coverage", 10f, 20f, color: Color.Green()));
            CallGame(() => game.DrawSprite(0, 1f, 2f, 3f, 4f, color: Color.Red()));
            CallGame(() => game.DrawQuad(1f, 2f, 3f, 4f, color: Color.Blue()));
            CallGame(() => game.IsKeyPressed(Keys.Space));
            CallGame(() => game.IsKeyJustPressed(Keys.Space));
            CallGame(() => game.IsKeyJustReleased(Keys.Space));
            CallGame(() => game.IsMouseButtonPressed(MouseButtons.Left));
            CallGame(() => game.IsMouseButtonJustPressed(MouseButtons.Left));
            CallGame(() => game.IsMouseButtonJustReleased(MouseButtons.Left));
            CallGame(() => game.GetMousePosition());
            CallGame(() => game.GetMouseDelta());
            CallGame(() => game.GetScrollDelta());

            var entity = game.SpawnEmpty();
            var spawned = game.SpawnBatch(2);
            Assert.True(game.IsAlive(entity));
            Assert.Equal(2, spawned.Length);
            Assert.True(game.EntityCount() >= 3);

            var alive = new byte[spawned.Length];
            Assert.Equal((uint)spawned.Length, game.IsAliveBatch(spawned, alive));
            Assert.All(alive, value => Assert.Equal((byte)1, value));

            var transform = Transform2D.FromPosition(3f, 4f);
            game.AddTransform2d(entity, transform);
            _ = game.HasTransform2d(entity);
            _ = game.GetTransform2d(entity);
            game.SetTransform2d(entity, Transform2D.FromPosition(8f, 9f));
            _ = game.GetTransform2d(entity);
            _ = game.RemoveTransform2d(entity);

            game.AddName(entity, "player");
            _ = game.HasName(entity);
            _ = game.GetName(entity);
            _ = game.RemoveName(entity);

            var sprite = Sprite.New(5UL).WithAnchor(0.5f, 0.5f);
            game.AddSprite(entity, sprite);
            _ = game.HasSprite(entity);
            _ = game.GetSprite(entity);
            _ = game.RemoveSprite(entity);

            var typeId = BitConverter.ToUInt64(Guid.NewGuid().ToByteArray(), 0);
            Assert.True(game.ComponentRegisterType(typeId, "GameCoverageCounter", 4, 4));

            var data = Marshal.AllocHGlobal(4);
            try
            {
                Marshal.WriteInt32(data, 17);
                _ = game.ComponentAdd(entity, typeId, data, 4);
                _ = game.ComponentHas(entity, typeId);
                _ = game.ComponentGet(entity, typeId);
                _ = game.ComponentGetMut(entity, typeId);
                _ = game.ComponentAddBatch(spawned, typeId, data, 4);
                var hasBatch = new byte[spawned.Length];
                _ = game.ComponentHasBatch(spawned, typeId, hasBatch);
                _ = game.ComponentRemoveBatch(spawned, typeId);
                _ = game.ComponentRemove(entity, typeId);
            }
            finally
            {
                Marshal.FreeHGlobal(data);
            }

            Assert.Contains("RenderCapabilities(", game.GetRenderCapabilities().ToString(), StringComparison.Ordinal);
            Assert.Contains("PhysicsCapabilities(", game.GetPhysicsCapabilities().ToString(), StringComparison.Ordinal);
            Assert.Contains("AudioCapabilities(", game.GetAudioCapabilities().ToString(), StringComparison.Ordinal);
            Assert.Contains("InputCapabilities(", game.GetInputCapabilities().ToString(), StringComparison.Ordinal);
            Assert.Contains("NetworkCapabilities(", game.GetNetworkCapabilities().ToString(), StringComparison.Ordinal);

            Assert.NotNull(game.CollisionAabbAabb(0f, 0f, 2f, 2f, 1f, 1f, 2f, 2f));
            Assert.Null(game.CollisionAabbAabb(0f, 0f, 1f, 1f, 10f, 10f, 1f, 1f));
            Assert.NotNull(game.CollisionCircleCircle(0f, 0f, 3f, 1f, 1f, 3f));
            Assert.Null(game.CollisionCircleCircle(0f, 0f, 1f, 10f, 10f, 1f));
            Assert.NotNull(game.CollisionCircleAabb(0f, 0f, 3f, 1f, 1f, 2f, 2f));
            Assert.Null(game.CollisionCircleAabb(0f, 0f, 1f, 10f, 10f, 1f, 1f));
            Assert.True(game.PointInRect(1f, 1f, 0f, 0f, 2f, 2f));
            Assert.False(game.PointInRect(5f, 5f, 0f, 0f, 2f, 2f));
            Assert.True(game.PointInCircle(0f, 0f, 0f, 0f, 2f));
            Assert.False(game.PointInCircle(10f, 10f, 0f, 0f, 2f));
            Assert.True(game.AabbOverlap(0f, 0f, 3f, 3f, 1f, 1f, 4f, 4f));
            Assert.False(game.AabbOverlap(0f, 0f, 1f, 1f, 2f, 2f, 3f, 3f));
            Assert.True(game.CircleOverlap(0f, 0f, 2f, 1f, 1f, 2f));
            Assert.False(game.CircleOverlap(0f, 0f, 1f, 5f, 5f, 1f));
            Assert.Equal(5f, game.Distance(0f, 0f, 3f, 4f), 3);
            Assert.Equal(25f, game.DistanceSquared(0f, 0f, 3f, 4f), 3);

            CallGame(() => game.Play(entity));
            CallGame(() => game.Stop(entity));
            CallGame(() => game.SetState(entity, "idle"));
            CallGame(() => game.SetParameterBool(entity, "isGrounded", true));
            CallGame(() => game.SetParameterFloat(entity, "speed", 2.5f));
            CallGame(() => game.CreateCube(0, 1f, 2f, 3f));
            CallGame(() => game.CreatePlane(0, 4f, 5f));
            CallGame(() => game.CreateSphere(0, 2f, 8));
            CallGame(() => game.CreateCylinder(0, 1f, 2f, 8));
            CallGame(() => game.SetObjectPosition(0, 1f, 2f, 3f));
            CallGame(() => game.SetObjectRotation(0, 1f, 2f, 3f));
            CallGame(() => game.SetObjectScale(0, 1f, 2f, 3f));
            CallGame(() => game.DestroyObject(0));
            CallGame(() => game.AddLight(0, 0f, 1f, 2f, 0f, -1f, 0f, 1f, 1f, 1f, 0.5f, 10f, 30f));
            CallGame(() => game.UpdateLight(0, 0, 0f, 1f, 2f, 0f, -1f, 0f, 1f, 1f, 1f, 0.5f, 10f, 30f));
            CallGame(() => game.RemoveLight(0));
            CallGame(() => game.SetCameraPosition3D(0f, 1f, 2f));
            CallGame(() => game.SetCameraRotation3D(10f, 20f, 30f));
            CallGame(() => game.ConfigureGrid(true, 10f, 10));
            CallGame(() => game.SetGridEnabled(true));
            CallGame(() => game.ConfigureSkybox(true, 0.1f, 0.2f, 0.3f, 1f));
            CallGame(() => game.ConfigureFog(true, 0.1f, 0.2f, 0.3f, 0.5f));
            CallGame(() => game.SetFogEnabled(true));
            CallGame(() => game.Render3D());
            CallGame(() => game.DrawSpriteRect(0, 1f, 2f, 3f, 4f, 0f, 0f, 0f, 16f, 16f, Color.White()));
            CallGame(() => game.SetViewport(0, 0, 320, 200));
            CallGame(() => game.EnableDepthTest());
            CallGame(() => game.DisableDepthTest());
            CallGame(() => game.ClearDepth());
            CallGame(() => game.DisableBlending());
            _ = game.MapActionKey("jump", Keys.Space);
            Assert.False(game.IsActionPressed("jump"));
            Assert.False(game.IsActionJustPressed("jump"));
            Assert.False(game.IsActionJustReleased("jump"));
            game.SetFpsOverlayEnabled(true);
            game.SetFpsUpdateInterval(0.25f);
            game.SetFpsOverlayCorner(OverlayCorner.TopRight);
            Assert.True(game.GetRenderStats().DrawCalls >= 0);
            Assert.True(game.GetFpsStats().CurrentFps >= 0f);
            CallGame(() => game.PhysicsRaycastEx(0f, 10f, 0f, -1f, 20f, uint.MaxValue));
            CallGame(() => game.PhysicsCollisionEventsCount());
            CallGame(() => game.PhysicsCollisionEventsRead(0));
            CallGame(() => game.PhysicsSetCollisionCallback(IntPtr.Zero, IntPtr.Zero));
            CallGame(() => game.AudioPlay(Array.Empty<byte>()));
            CallGame(() => game.AudioPlayOnChannel(Array.Empty<byte>(), 0));
            CallGame(() => game.AudioPlayWithSettings(Array.Empty<byte>(), 0.5f, 1f, false, 0));
            CallGame(() => game.AudioStop(0));
            CallGame(() => game.AudioPause(0));
            CallGame(() => game.AudioResume(0));
            CallGame(() => game.AudioStopAll());
            CallGame(() => game.AudioSetGlobalVolume(0.5f));
            CallGame(() => game.AudioGetGlobalVolume());
            CallGame(() => game.AudioSetChannelVolume(0, 0.75f));
            CallGame(() => game.AudioGetChannelVolume(0));
            CallGame(() => game.AudioIsPlaying(0));
            CallGame(() => game.AudioActiveCount());
            CallGame(() => game.AudioCleanupFinished());
            CallGame(() => game.AudioPlaySpatial3d(Array.Empty<byte>(), 0f, 0f, 0f, 0f, 0f, 0f, 10f, 1f));
            CallGame(() => game.AudioUpdateSpatial3d(0, 0f, 0f, 0f, 0f, 0f, 0f, 10f, 1f));
            CallGame(() => game.AudioSetListenerPosition3d(0f, 0f, 0f));
            CallGame(() => game.AudioSetSourcePosition3d(0, 0f, 0f, 0f, 10f, 1f));
            CallGame(() => game.AudioSetPlayerVolume(0, 0.5f));
            CallGame(() => game.AudioSetPlayerSpeed(0, 1.25f));
            CallGame(() => game.AudioCrossfade(0, 1, 0.5f));
            CallGame(() => game.AudioCrossfadeTo(0, Array.Empty<byte>(), 0.5f, 0));
            CallGame(() => game.AudioMixWith(0, Array.Empty<byte>(), 0.25f, 0));
            CallGame(() => game.AudioUpdateCrossfades(0.016f));
            CallGame(() => game.AudioActiveCrossfadeCount());
            CallGame(() => game.AudioActivate());
            CallGame(() => game.CheckHotSwapShortcut());

            Assert.Equal((uint)spawned.Length, game.DespawnBatch(spawned));
            Assert.True(game.Despawn(entity));
        }
        finally
        {
            RuntimeTestSupport.DestroyContext(context);
        }
    }

    [Fact]
    public void Headless_GoudGame_Network_Path_Executes_Through_Wrappers()
    {
        var (hostContext, hostGame) = RuntimeTestSupport.CreateHeadlessGame("Host");
        var (clientContext, clientGame) = RuntimeTestSupport.CreateHeadlessGame("Client");
        var hostManager = new NetworkManager(hostGame);
        var clientManager = new NetworkManager(clientGame);
        var port = RuntimeTestSupport.ReservePort();
        var hostEndpoint = hostManager.Host(NetworkProtocol.Tcp, (ushort)port);
        var clientEndpoint = clientManager.Connect(NetworkProtocol.Tcp, "127.0.0.1", (ushort)port);

        try
        {
            WaitForConnectedPeers(hostEndpoint, clientEndpoint);

            Assert.NotNull(clientEndpoint.DefaultPeerId);
            var simulationResult = clientEndpoint.SetSimulation(new NetworkSimulationConfig(5U, 0U, 0f));
            Assert.True(simulationResult is 0 or 902, $"unexpected simulation result: {simulationResult}");
            var clearSimulationResult = clientEndpoint.ClearSimulation();
            Assert.True(clearSimulationResult is 0 or 902, $"unexpected clear simulation result: {clearSimulationResult}");
            Assert.Equal(0, clientEndpoint.SetOverlayTarget());
            Assert.Equal(0, clientEndpoint.ClearOverlayTarget());

            var payload = new byte[] { 0x63, 0x73, 0x68, 0x61, 0x72, 0x70 };
            Assert.Equal(0, clientEndpoint.Send(payload, 0));

            var hostPacket = WaitForPacket(hostEndpoint, clientEndpoint);
            Assert.NotNull(hostPacket);
            Assert.Equal(payload, hostPacket!.Value.Data);

            Assert.Equal(Array.Empty<byte>(), hostGame.NetworkReceive(hostEndpoint.Handle));
            Assert.Equal(0, hostGame.NetworkSend(hostEndpoint.Handle, hostPacket.Value.PeerId, new byte[] { 0x61, 0x63, 0x6B }, 0));

            var clientPacket = WaitForPacket(clientEndpoint, hostEndpoint);
            Assert.NotNull(clientPacket);
            Assert.Equal(new byte[] { 0x61, 0x63, 0x6B }, clientPacket!.Value.Data);

            Assert.True(hostGame.GetNetworkStats(hostEndpoint.Handle).BytesReceived > 0);
            Assert.True(clientGame.GetNetworkStats(clientEndpoint.Handle).BytesSent > 0);
            Assert.True(hostGame.NetworkPeerCount(hostEndpoint.Handle) > 0);
            Assert.True(clientGame.NetworkPeerCount(clientEndpoint.Handle) > 0);

            var extraHandle = clientGame.NetworkConnect((int)NetworkProtocol.Tcp, "127.0.0.1", (ushort)port);
            Assert.True(extraHandle >= 0);
            Assert.Equal(0, clientGame.NetworkDisconnect(extraHandle));
        }
        finally
        {
            _ = clientEndpoint.Disconnect();
            _ = hostEndpoint.Disconnect();
            RuntimeTestSupport.DestroyContext(clientContext);
            RuntimeTestSupport.DestroyContext(hostContext);
        }
    }

    private static void WaitForConnectedPeers(NetworkEndpoint first, NetworkEndpoint second)
    {
        var sw = Stopwatch.StartNew();
        while (sw.Elapsed < TimeSpan.FromSeconds(5))
        {
            Pump(first, second);
            if (first.PeerCount() > 0 && second.PeerCount() > 0)
            {
                return;
            }

            Thread.Sleep(10);
        }

        throw new Xunit.Sdk.XunitException("timed out waiting for connected peers");
    }

    private static NetworkPacket? WaitForPacket(NetworkEndpoint receiver, NetworkEndpoint other)
    {
        var sw = Stopwatch.StartNew();
        while (sw.Elapsed < TimeSpan.FromSeconds(5))
        {
            Pump(receiver, other);
            var packet = receiver.Receive();
            if (packet.HasValue)
            {
                return packet;
            }

            Thread.Sleep(10);
        }

        throw new Xunit.Sdk.XunitException("timed out waiting for packet");
    }

    private static void Pump(NetworkEndpoint first, NetworkEndpoint second, int iterations = 1)
    {
        for (var i = 0; i < iterations; i++)
        {
            _ = first.Poll();
            _ = second.Poll();
            Thread.Sleep(10);
        }
    }

    private static void CallGame(Action action) => RuntimeTestSupport.CallGame(action);

    private static void CallGame<T>(Func<T> action) => RuntimeTestSupport.CallGame(action);
}
