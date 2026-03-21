using System;
using System.Runtime.InteropServices;
using GoudEngine;
using Xunit;

namespace GoudEngine.Tests.Runtime;

/// <summary>
/// Runtime tests for the hand-written generic component API in Components.cs.
/// Requires a live engine context (headless) to exercise FFI round-trips.
/// </summary>
public class GenericComponentRuntimeTests
{
    [StructLayout(LayoutKind.Sequential)]
    private struct Health
    {
        public float Current;
        public float Max;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct Velocity
    {
        public float X;
        public float Y;
    }

    [Fact]
    public void TypeHash_Is_Deterministic()
    {
        var h1 = ComponentStore.TypeHash("Health");
        var h2 = ComponentStore.TypeHash("Health");
        Assert.Equal(h1, h2);
    }

    [Fact]
    public void TypeHash_Differs_For_Different_Names()
    {
        var h1 = ComponentStore.TypeHash("Health");
        var h2 = ComponentStore.TypeHash("Velocity");
        Assert.NotEqual(h1, h2);
    }

    [Fact]
    public void TypeHash_Matches_Known_FNV1a_Value()
    {
        // Compute expected FNV-1a 64-bit for "Health" inline.
        ulong expected = 0xcbf29ce484222325;
        foreach (byte b in System.Text.Encoding.UTF8.GetBytes("Health"))
        {
            expected ^= b;
            expected *= 0x100000001b3;
        }
        Assert.Equal(expected, ComponentStore.TypeHash("Health"));
    }

    [Fact]
    public void ComponentStoreGeneric_Hash_Matches_TypeName()
    {
        // ComponentStore<T>.Hash uses typeof(T).Name.
        Assert.Equal(ComponentStore.TypeHash("Health"), ComponentStore<Health>.Hash);
        Assert.Equal(ComponentStore.TypeHash("Velocity"), ComponentStore<Velocity>.Hash);
    }

    [Fact]
    public void RegisterComponent_Succeeds()
    {
        var (context, game) = RuntimeTestSupport.CreateHeadlessGame("GenericReg");
        try
        {
            bool registered = game.RegisterComponent<Health>();
            Assert.True(registered);

            // Second registration returns false (already registered).
            bool second = game.RegisterComponent<Health>();
            Assert.False(second);
        }
        finally
        {
            RuntimeTestSupport.DestroyContext(context);
        }
    }

    [Fact]
    public void SetComponent_And_HasComponent_RoundTrip()
    {
        var (context, game) = RuntimeTestSupport.CreateHeadlessGame("GenericSetHas");
        try
        {
            game.RegisterComponent<Velocity>();

            var entity = game.SpawnEmpty();
            Assert.False(game.HasComponent<Velocity>(entity));

            game.SetComponent(entity, new Velocity { X = 5f, Y = 10f });
            Assert.True(game.HasComponent<Velocity>(entity));
        }
        finally
        {
            RuntimeTestSupport.DestroyContext(context);
        }
    }

    [Fact]
    public void GetComponent_Returns_Correct_Values()
    {
        var (context, game) = RuntimeTestSupport.CreateHeadlessGame("GenericGet");
        try
        {
            game.RegisterComponent<Health>();

            var entity = game.SpawnEmpty();
            game.SetComponent(entity, new Health { Current = 75f, Max = 150f });

            ref readonly Health h = ref game.GetComponent<Health>(entity);
            Assert.Equal(75f, h.Current);
            Assert.Equal(150f, h.Max);
        }
        finally
        {
            RuntimeTestSupport.DestroyContext(context);
        }
    }

    [Fact]
    public void GetComponentMut_Allows_Mutation()
    {
        var (context, game) = RuntimeTestSupport.CreateHeadlessGame("GenericMut");
        try
        {
            game.RegisterComponent<Health>();

            var entity = game.SpawnEmpty();
            game.SetComponent(entity, new Health { Current = 100f, Max = 100f });

            ref Health h = ref game.GetComponentMut<Health>(entity);
            h.Current = 42f;

            // Read again to verify mutation persisted.
            ref readonly Health h2 = ref game.GetComponent<Health>(entity);
            Assert.Equal(42f, h2.Current);
            Assert.Equal(100f, h2.Max);
        }
        finally
        {
            RuntimeTestSupport.DestroyContext(context);
        }
    }

    [Fact]
    public void RemoveComponent_Works()
    {
        var (context, game) = RuntimeTestSupport.CreateHeadlessGame("GenericRem");
        try
        {
            game.RegisterComponent<Health>();

            var entity = game.SpawnEmpty();
            game.SetComponent(entity, new Health { Current = 1f, Max = 1f });
            Assert.True(game.HasComponent<Health>(entity));

            bool removed = game.RemoveComponent<Health>(entity);
            Assert.True(removed);
            Assert.False(game.HasComponent<Health>(entity));
        }
        finally
        {
            RuntimeTestSupport.DestroyContext(context);
        }
    }

    [Fact]
    public void Query_Iterates_All_Entities()
    {
        var (context, game) = RuntimeTestSupport.CreateHeadlessGame("GenericQuery");
        try
        {
            game.RegisterComponent<Health>();

            const int entityCount = 5;
            for (int i = 0; i < entityCount; i++)
            {
                var e = game.SpawnEmpty();
                game.SetComponent(e, new Health { Current = i * 10f, Max = 100f });
            }

            int count = 0;
            float sum = 0f;
            foreach (var item in game.Query<Health>())
            {
                count++;
                sum += item.Value.Current;
            }

            Assert.Equal(entityCount, count);
            // Sum of 0, 10, 20, 30, 40 = 100
            Assert.Equal(100f, sum);
        }
        finally
        {
            RuntimeTestSupport.DestroyContext(context);
        }
    }

    [Fact]
    public void Query_Empty_Returns_Zero_Items()
    {
        var (context, game) = RuntimeTestSupport.CreateHeadlessGame("GenericEmpty");
        try
        {
            game.RegisterComponent<Health>();

            int count = 0;
            foreach (var _ in game.Query<Health>())
            {
                count++;
            }

            Assert.Equal(0, count);
        }
        finally
        {
            RuntimeTestSupport.DestroyContext(context);
        }
    }
}
