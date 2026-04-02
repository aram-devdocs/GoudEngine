using System;
using System.Runtime.InteropServices;
using GoudEngine;
using Xunit;

namespace GoudEngine.Tests.Runtime;

/// <summary>
/// Validates that GoudEngine runs headlessly with null render/window providers.
/// Covers the Throne server use case: no GPU, no display, pure ECS simulation.
/// See GitHub issue #645.
/// </summary>
public class HeadlessValidationTests
{
    [StructLayout(LayoutKind.Sequential)]
    private struct Position
    {
        public float X;
        public float Y;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct Velocity
    {
        public float Dx;
        public float Dy;
    }

    [Fact]
    public void Headless_Initialize_With_Null_Providers()
    {
        var (context, game) = RuntimeTestSupport.CreateHeadlessGame("HeadlessInit");
        try
        {
            Assert.True(context.IsValid());

            var caps = game.GetRenderCapabilities();
            Assert.Equal(0U, caps.MaxTextureUnits);
            Assert.Equal(0U, caps.MaxTextureSize);
            Assert.False(caps.SupportsInstancing);
            Assert.False(caps.SupportsCompute);
            Assert.False(caps.SupportsMsaa);

            // Render stats and window dimensions confirm no GPU activity.
            Assert.Equal(0U, game.GetRenderStats().DrawCalls);
            RuntimeTestSupport.CallGame(() => game.WindowWidth);
            RuntimeTestSupport.CallGame(() => game.WindowHeight);
        }
        finally
        {
            RuntimeTestSupport.DestroyContext(context);
        }
    }

    [Fact]
    public void Headless_Spawn_1000_Entities_With_Components()
    {
        var (context, game) = RuntimeTestSupport.CreateHeadlessGame("Headless1K");
        try
        {
            game.RegisterComponent<Position>();
            game.RegisterComponent<Velocity>();

            var spawned = game.SpawnBatch(1000);
            Assert.Equal(1000, spawned.Length);

            for (int i = 0; i < spawned.Length; i++)
            {
                game.SetComponent(spawned[i], new Position { X = i, Y = i * 2f });
                game.SetComponent(spawned[i], new Velocity { Dx = 1f, Dy = 0.5f });
            }

            Assert.True(game.EntityCount() >= 1000);

            ref readonly var pos = ref game.GetComponent<Position>(spawned[500]);
            Assert.Equal(500f, pos.X, 1);
            Assert.Equal(1000f, pos.Y, 1);

            Assert.Equal(1000U, game.DespawnBatch(spawned));
        }
        finally
        {
            RuntimeTestSupport.DestroyContext(context);
        }
    }

    [Fact]
    public void Headless_Server_Tick_Loop_100_Frames()
    {
        // The engine's built-in fixed timestep API (goud_fixed_timestep_begin/step)
        // returns false in headless mode because no WindowState exists. This is
        // correct behavior: a headless server drives its own tick rate rather than
        // relying on a display-driven frame clock. This test validates the
        // server-authoritative simulation pattern used by Throne.
        var (context, game) = RuntimeTestSupport.CreateHeadlessGame("Headless100F");
        try
        {
            game.RegisterComponent<Position>();
            game.RegisterComponent<Velocity>();

            const float dt = 1f / 60f;
            var entities = game.SpawnBatch(10);
            SetupEntities(game, entities, 1f, 0.5f);

            RunSimulation(game, entities, 100, dt);

            ref readonly var final0 = ref game.GetComponent<Position>(entities[0]);
            Assert.True(final0.X > 0f, "Position X should have advanced after 100 frames");
            Assert.True(final0.Y > 0f, "Position Y should have advanced after 100 frames");
        }
        finally
        {
            RuntimeTestSupport.DestroyContext(context);
        }
    }

    [Fact]
    public void Headless_Query_Positions_After_Simulation_Verifies_Movement()
    {
        var (context, game) = RuntimeTestSupport.CreateHeadlessGame("HeadlessMovement");
        try
        {
            game.RegisterComponent<Position>();
            game.RegisterComponent<Velocity>();

            const int entityCount = 5;
            const float dt = 1f / 60f;
            const float velX = 10f;
            const float velY = 5f;

            var entities = game.SpawnBatch((uint)entityCount);
            SetupEntities(game, entities, velX, velY);

            RunSimulation(game, entities, 60, dt);

            // After 60 frames at dt=1/60, 1 second has elapsed.
            // Expected: X = 10 * 1 = 10, Y = 5 * 1 = 5.
            for (int i = 0; i < entityCount; i++)
            {
                ref readonly var pos = ref game.GetComponent<Position>(entities[i]);
                Assert.Equal(velX, pos.X, 2);
                Assert.Equal(velY, pos.Y, 2);
            }

            Assert.Equal((uint)entityCount, game.DespawnBatch(entities));
        }
        finally
        {
            RuntimeTestSupport.DestroyContext(context);
        }
    }

    [Fact]
    public void Headless_Fixed_Timestep_Api_Is_Safe()
    {
        var (context, game) = RuntimeTestSupport.CreateHeadlessGame("HeadlessTimestep");
        try
        {
            var ctx = RuntimeTestSupport.GetContextId(context);

            // These calls should not crash even though no WindowState exists.
            game.SetFixedTimestep(1f / 60f);
            game.SetMaxFixedSteps(8);

            // In headless mode, fixed timestep begin returns false (no window state).
            Assert.False(NativeMethods.goud_fixed_timestep_begin(ctx));
            Assert.False(NativeMethods.goud_fixed_timestep_step(ctx));
            Assert.Equal(0f, NativeMethods.goud_fixed_timestep_dt(ctx));
        }
        finally
        {
            RuntimeTestSupport.DestroyContext(context);
        }
    }

    private static void SetupEntities(GoudGame game, Entity[] entities, float velX, float velY)
    {
        for (int i = 0; i < entities.Length; i++)
        {
            game.SetComponent(entities[i], new Position { X = 0f, Y = 0f });
            game.SetComponent(entities[i], new Velocity { Dx = velX, Dy = velY });
        }
    }

    private static void RunSimulation(GoudGame game, Entity[] entities, int frameCount, float dt)
    {
        for (int frame = 0; frame < frameCount; frame++)
        {
            for (int i = 0; i < entities.Length; i++)
            {
                ref readonly var vel = ref game.GetComponent<Velocity>(entities[i]);
                ref var pos = ref game.GetComponentMut<Position>(entities[i]);
                pos.X += vel.Dx * dt;
                pos.Y += vel.Dy * dt;
            }
        }
    }
}
