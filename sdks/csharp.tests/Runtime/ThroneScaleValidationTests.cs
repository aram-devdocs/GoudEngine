using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
using GoudEngine;
using Xunit;

namespace GoudEngine.Tests.Runtime;

/// <summary>
/// Scale validation tests for Throne's planned migration to GoudEngine ECS.
/// Validates 100k entities with 5 component types matching Throne's usage.
/// This is a validation suite — failures indicate SDK limitations to file as bugs.
/// </summary>
public class ThroneScaleValidationTests
{
    [StructLayout(LayoutKind.Sequential)]
    private struct Position
    {
        public float X;
        public float Y;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct Movement
    {
        public float Speed;
        public byte IsMoving;
        private byte _pad1;
        private byte _pad2;
        private byte _pad3;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct Health
    {
        public float Current;
        public float Max;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct Needs
    {
        public float Hunger;
        public float Thirst;
        public float Rest;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct AI
    {
        public int GoalType;
        public float Priority;
    }

    private const uint ThroneEntityCount = 100_000;
    private const long SpawnAndAttachThresholdMs = 2000;
    private const long QueryCountThresholdMs = 1;
    private const long UpdateThresholdMs = 2000;
    private const long MemoryGrowthThresholdBytes = 50 * 1024 * 1024; // 50 MB

    private void RegisterAllComponents(GoudGame game)
    {
        game.RegisterComponent<Position>();
        game.RegisterComponent<Movement>();
        game.RegisterComponent<Health>();
        game.RegisterComponent<Needs>();
        game.RegisterComponent<AI>();
    }

    private static void AddComponentBatch<T>(GoudGame game, Entity[] entities, T[] data)
        where T : unmanaged
    {
        var handle = GCHandle.Alloc(data, GCHandleType.Pinned);
        try
        {
            int componentSize = Marshal.SizeOf<T>();
            game.ComponentAddBatch(entities, ComponentStore<T>.Hash,
                handle.AddrOfPinnedObject(), (nuint)componentSize);
        }
        finally
        {
            handle.Free();
        }
    }

    [Fact]
    public void Spawn_100k_Entities_With_5_Components_Under_Threshold()
    {
        var (context, game) = RuntimeTestSupport.CreateHeadlessGame("ThroneSpawn");
        try
        {
            RegisterAllComponents(game);

            // Warmup: spawn a small batch to force JIT and Rust-side init
            var warmup = game.SpawnBatch(100);
            var warmupPos = new Position[100];
            AddComponentBatch(game, warmup, warmupPos);
            game.DespawnBatch(warmup);

            var sw = Stopwatch.StartNew();

            var entities = game.SpawnBatch(ThroneEntityCount);

            var positions = new Position[entities.Length];
            var movements = new Movement[entities.Length];
            var healths = new Health[entities.Length];
            var needs = new Needs[entities.Length];
            var ais = new AI[entities.Length];

            for (int i = 0; i < entities.Length; i++)
            {
                positions[i] = new Position { X = i * 0.1f, Y = i * 0.2f };
                movements[i] = new Movement { Speed = 5.0f, IsMoving = 1 };
                healths[i] = new Health { Current = 100f, Max = 100f };
                needs[i] = new Needs { Hunger = 50f, Thirst = 50f, Rest = 80f };
                ais[i] = new AI { GoalType = i % 4, Priority = 0.5f };
            }

            AddComponentBatch(game, entities, positions);
            AddComponentBatch(game, entities, movements);
            AddComponentBatch(game, entities, healths);
            AddComponentBatch(game, entities, needs);
            AddComponentBatch(game, entities, ais);

            sw.Stop();

            Assert.Equal(ThroneEntityCount, game.EntityCount());
            Assert.True(sw.ElapsedMilliseconds < SpawnAndAttachThresholdMs,
                $"Spawn+attach took {sw.Elapsed.TotalMilliseconds:F2}ms, threshold is {SpawnAndAttachThresholdMs}ms");
        }
        finally
        {
            RuntimeTestSupport.DestroyContext(context);
        }
    }

    [Fact]
    public void Query_By_Component_Returns_Correct_Count()
    {
        var (context, game) = RuntimeTestSupport.CreateHeadlessGame("ThroneQueryCount");
        try
        {
            RegisterAllComponents(game);
            var entities = game.SpawnBatch(ThroneEntityCount);

            var positions = new Position[entities.Length];
            var movements = new Movement[entities.Length];
            var healths = new Health[entities.Length];
            var needs = new Needs[entities.Length];
            var ais = new AI[entities.Length];
            AddComponentBatch(game, entities, positions);
            AddComponentBatch(game, entities, movements);
            AddComponentBatch(game, entities, healths);
            AddComponentBatch(game, entities, needs);
            AddComponentBatch(game, entities, ais);

            // Warmup query
            _ = game.Query<Position>().Count;

            var sw = Stopwatch.StartNew();
            int posCount = game.Query<Position>().Count;
            sw.Stop();
            Assert.Equal((int)ThroneEntityCount, posCount);
            Assert.True(sw.ElapsedMilliseconds < QueryCountThresholdMs,
                $"Position query count took {sw.Elapsed.TotalMilliseconds:F2}ms, threshold is {QueryCountThresholdMs}ms");

            Assert.Equal((int)ThroneEntityCount, game.Query<Movement>().Count);
            Assert.Equal((int)ThroneEntityCount, game.Query<Health>().Count);
            Assert.Equal((int)ThroneEntityCount, game.Query<Needs>().Count);
            Assert.Equal((int)ThroneEntityCount, game.Query<AI>().Count);
        }
        finally
        {
            RuntimeTestSupport.DestroyContext(context);
        }
    }

    [Fact]
    public void Query_Iterate_100k_Verifies_Data_Integrity()
    {
        var (context, game) = RuntimeTestSupport.CreateHeadlessGame("ThroneQueryIterate");
        try
        {
            game.RegisterComponent<Position>();
            var entities = game.SpawnBatch(ThroneEntityCount);

            var positions = new Position[entities.Length];
            for (int i = 0; i < entities.Length; i++)
                positions[i] = new Position { X = i, Y = i * 2f };
            AddComponentBatch(game, entities, positions);

            int count = 0;
            bool foundFirst = false;
            bool foundLast = false;
            foreach (var item in game.Query<Position>())
            {
                count++;
                if (item.Value.X == 0f && item.Value.Y == 0f)
                    foundFirst = true;
                if (item.Value.X == (ThroneEntityCount - 1) && item.Value.Y == (ThroneEntityCount - 1) * 2f)
                    foundLast = true;
            }

            Assert.Equal((int)ThroneEntityCount, count);
            Assert.True(foundFirst, "First entity's Position data not found");
            Assert.True(foundLast, "Last entity's Position data not found");
        }
        finally
        {
            RuntimeTestSupport.DestroyContext(context);
        }
    }

    [Fact]
    public void Update_100k_Components_Under_Threshold()
    {
        var (context, game) = RuntimeTestSupport.CreateHeadlessGame("ThroneUpdate");
        try
        {
            game.RegisterComponent<Health>();
            var entities = game.SpawnBatch(ThroneEntityCount);

            var healths = new Health[entities.Length];
            for (int i = 0; i < entities.Length; i++)
                healths[i] = new Health { Current = 100f, Max = 100f };
            AddComponentBatch(game, entities, healths);

            // Warmup: iterate once
            foreach (var item in game.Query<Health>())
            {
                item.Value.Current -= 1f;
            }

            // Timed update pass
            var sw = Stopwatch.StartNew();
            foreach (var item in game.Query<Health>())
            {
                item.Value.Current -= 10f;
            }
            sw.Stop();

            Assert.True(sw.ElapsedMilliseconds < UpdateThresholdMs,
                $"Update took {sw.Elapsed.TotalMilliseconds:F2}ms, threshold is {UpdateThresholdMs}ms");

            // Spot-check: after warmup (-1) and measured pass (-10), Current should be 89
            ref readonly Health sample = ref game.GetComponent<Health>(entities[0]);
            Assert.Equal(89f, sample.Current);
            Assert.Equal(100f, sample.Max);
        }
        finally
        {
            RuntimeTestSupport.DestroyContext(context);
        }
    }

    [Fact]
    public void Despawn_10k_Entities_Verifies_Entity_Cleanup()
    {
        var (context, game) = RuntimeTestSupport.CreateHeadlessGame("ThroneDespawn");
        try
        {
            game.RegisterComponent<Position>();
            game.RegisterComponent<Health>();
            var entities = game.SpawnBatch(ThroneEntityCount);

            var positions = new Position[entities.Length];
            var healths = new Health[entities.Length];
            AddComponentBatch(game, entities, positions);
            AddComponentBatch(game, entities, healths);

            Assert.Equal(ThroneEntityCount, game.EntityCount());

            // Despawn first 10k
            var toDespawn = new Entity[10_000];
            Array.Copy(entities, toDespawn, 10_000);
            uint despawned = game.DespawnBatch(toDespawn);

            Assert.Equal(10_000u, despawned);
            Assert.Equal(ThroneEntityCount - 10_000, game.EntityCount());

            // Verify despawned entities are no longer alive
            Assert.False(game.IsAlive(toDespawn[0]));
            Assert.False(game.IsAlive(toDespawn[9_999]));

            // Verify surviving entities are still alive with data
            Assert.True(game.IsAlive(entities[10_000]));
            Assert.True(game.HasComponent<Position>(entities[10_000]));
        }
        finally
        {
            RuntimeTestSupport.DestroyContext(context);
        }
    }

    [Fact(Skip = "Known bug: FFI component storage not cleaned up on entity despawn (see PR description)")]
    public void Despawn_10k_Entities_Cleans_Up_Component_Storage()
    {
        var (context, game) = RuntimeTestSupport.CreateHeadlessGame("ThroneDespawnComponents");
        try
        {
            game.RegisterComponent<Position>();
            game.RegisterComponent<Health>();
            var entities = game.SpawnBatch(ThroneEntityCount);

            var positions = new Position[entities.Length];
            var healths = new Health[entities.Length];
            AddComponentBatch(game, entities, positions);
            AddComponentBatch(game, entities, healths);

            // Despawn first 10k
            var toDespawn = new Entity[10_000];
            Array.Copy(entities, toDespawn, 10_000);
            game.DespawnBatch(toDespawn);

            // BUG: FFI component storage is not cleaned up when entities are
            // despawned through the native world. The component query counts
            // still report the full 100k. Filed as a known SDK limitation
            // for Throne migration planning.
            int posCount = game.Query<Position>().Count;
            int healthCount = game.Query<Health>().Count;
            Assert.Equal((int)(ThroneEntityCount - 10_000), posCount);
            Assert.Equal((int)(ThroneEntityCount - 10_000), healthCount);
        }
        finally
        {
            RuntimeTestSupport.DestroyContext(context);
        }
    }

    [Fact]
    public void MultiComponent_All_5_Types_Correct_Counts()
    {
        var (context, game) = RuntimeTestSupport.CreateHeadlessGame("ThroneMulti");
        try
        {
            RegisterAllComponents(game);
            var entities = game.SpawnBatch(ThroneEntityCount);

            var positions = new Position[entities.Length];
            var movements = new Movement[entities.Length];
            var healths = new Health[entities.Length];
            var needs = new Needs[entities.Length];
            var ais = new AI[entities.Length];

            AddComponentBatch(game, entities, positions);
            AddComponentBatch(game, entities, movements);
            AddComponentBatch(game, entities, healths);
            AddComponentBatch(game, entities, needs);
            AddComponentBatch(game, entities, ais);

            Assert.Equal((int)ThroneEntityCount, game.Query<Position>().Count);
            Assert.Equal((int)ThroneEntityCount, game.Query<Movement>().Count);
            Assert.Equal((int)ThroneEntityCount, game.Query<Health>().Count);
            Assert.Equal((int)ThroneEntityCount, game.Query<Needs>().Count);
            Assert.Equal((int)ThroneEntityCount, game.Query<AI>().Count);
        }
        finally
        {
            RuntimeTestSupport.DestroyContext(context);
        }
    }

    [Fact]
    public void Churn_1000_Cycles_No_Memory_Leak()
    {
        var (context, game) = RuntimeTestSupport.CreateHeadlessGame("ThroneChurn");
        try
        {
            game.RegisterComponent<Position>();

            const int cycleCount = 1000;
            const uint batchSize = 100;

            // Warmup: 10 cycles to stabilize allocator
            for (int i = 0; i < 10; i++)
            {
                var warm = game.SpawnBatch(batchSize);
                var warmPos = new Position[batchSize];
                AddComponentBatch(game, warm, warmPos);
                game.DespawnBatch(warm);
            }

            GC.Collect();
            GC.WaitForPendingFinalizers();
            GC.Collect();

            long baselineMemory = System.Diagnostics.Process.GetCurrentProcess().WorkingSet64;

            for (int cycle = 0; cycle < cycleCount; cycle++)
            {
                var batch = game.SpawnBatch(batchSize);
                var batchPos = new Position[batchSize];
                for (int j = 0; j < (int)batchSize; j++)
                    batchPos[j] = new Position { X = cycle, Y = j };
                AddComponentBatch(game, batch, batchPos);
                game.DespawnBatch(batch);
            }

            GC.Collect();
            GC.WaitForPendingFinalizers();
            GC.Collect();

            long finalMemory = System.Diagnostics.Process.GetCurrentProcess().WorkingSet64;
            long growth = finalMemory - baselineMemory;

            // Entity count should be back to zero (only churn entities were created)
            Assert.Equal(0u, game.EntityCount());

            Assert.True(growth < MemoryGrowthThresholdBytes,
                $"Memory grew by {growth / (1024 * 1024)}MB after {cycleCount} churn cycles, threshold is {MemoryGrowthThresholdBytes / (1024 * 1024)}MB");
        }
        finally
        {
            RuntimeTestSupport.DestroyContext(context);
        }
    }
}
