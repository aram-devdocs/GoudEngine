using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using GoudEngine;
using Xunit;

namespace GoudEngine.Tests.Runtime;

public class GeneratedValueTypeRuntimeTests
{
    public static IEnumerable<object[]> DataStructTypes()
    {
        yield return new object[] { typeof(AnimationEventData) };
        yield return new object[] { typeof(AudioCapabilities) };
        yield return new object[] { typeof(Contact) };
        yield return new object[] { typeof(FpsStats) };
        yield return new object[] { typeof(InputCapabilities) };
        yield return new object[] { typeof(NetworkCapabilities) };
        yield return new object[] { typeof(NetworkConnectResult) };
        yield return new object[] { typeof(NetworkPacket) };
        yield return new object[] { typeof(NetworkSimulationConfig) };
        yield return new object[] { typeof(NetworkStats) };
        yield return new object[] { typeof(PhysicsCapabilities) };
        yield return new object[] { typeof(PhysicsCollisionEvent2D) };
        yield return new object[] { typeof(PhysicsRaycastHit2D) };
        yield return new object[] { typeof(RenderCapabilities) };
        yield return new object[] { typeof(RenderStats) };
        yield return new object[] { typeof(UiEvent) };
        yield return new object[] { typeof(UiStyle) };
    }

    [Fact]
    public void Vec2_Color_Rect_Vec3_And_Entity_Managed_Methods_Work()
    {
        var vec = new Vec2(3f, 4f);
        Assert.Equal(5f, vec.Length(), 3);
        Assert.Equal(new Vec2(4f, 5f).ToString(), vec.Add(new Vec2(1f, 1f)).ToString());
        Assert.Equal(new Vec2(2f, 3f).ToString(), vec.Sub(Vec2.One()).ToString());
        Assert.Equal(new Vec2(6f, 8f).ToString(), vec.Scale(2f).ToString());
        Assert.Equal(7f, vec.Dot(new Vec2(1f, 1f)), 3);
        Assert.Equal(5f, vec.Distance(Vec2.Zero()), 3);
        Assert.Equal(new Vec2(1f, 0f).ToString(), new Vec2(5f, 0f).Normalize().ToString());
        Assert.Equal(Vec2.Zero().ToString(), Vec2.Zero().Normalize().ToString());
        Assert.Equal(new Vec2(2f, 3f).ToString(), new Vec2(1f, 2f).Lerp(new Vec2(3f, 4f), 0.5f).ToString());

        Assert.Equal("Vec3(0, 0, 0)", Vec3.Zero().ToString());
        Assert.Equal("Vec3(1, 1, 1)", Vec3.One().ToString());
        Assert.Equal("Vec3(0, 1, 0)", Vec3.Up().ToString());

        var color = Color.FromHex(0x3366CC).WithAlpha(0.5f);
        Assert.Equal(0.5f, color.A, 3);
        Assert.Equal(Color.FromU8(255, 0, 0, 255).ToString(), Color.Red().ToString());
        Assert.Equal("Color(1, 1, 1, 1)", Color.White().ToString());
        Assert.Equal("Color(0, 0, 0, 1)", Color.Black().ToString());
        Assert.Equal("Color(0, 0, 0, 0)", Color.Transparent().ToString());
        Assert.Equal(Color.Rgba(0.25f, 0.5f, 0.75f, 1f).ToString(), Color.Rgb(0.25f, 0.5f, 0.75f).ToString().Replace(", 1)", ", 1)"));
        Assert.Equal(new Color(0.5f, 0f, 0.5f, 1f).ToString(), Color.Red().Lerp(Color.Blue(), 0.5f).ToString());

        var rect = new Rect(10f, 20f, 30f, 40f);
        Assert.True(rect.Contains(new Vec2(25f, 35f)));
        Assert.False(rect.Contains(new Vec2(0f, 0f)));
        Assert.True(rect.Intersects(new Rect(20f, 30f, 5f, 5f)));
        Assert.False(rect.Intersects(new Rect(100f, 100f, 5f, 5f)));
        Assert.Contains("Rect(", rect.ToString(), StringComparison.Ordinal);

        var entity = new Entity(((ulong)7 << 32) | 21UL);
        Assert.Equal(21U, entity.Index);
        Assert.Equal(7U, entity.Generation);
        Assert.False(entity.IsPlaceholder);
        Assert.Equal(((ulong)7 << 32) | 21UL, entity.ToBits());
        Assert.True(Entity.Placeholder.IsPlaceholder);
        Assert.Contains("Entity(", entity.ToString(), StringComparison.Ordinal);
    }

    [Theory]
    [MemberData(nameof(DataStructTypes))]
    public void Generated_Data_Struct_Constructors_And_ToString_Execute(Type type)
    {
        var ctor = type.GetConstructors(BindingFlags.Instance | BindingFlags.Public)
            .OrderByDescending(candidate => candidate.GetParameters().Length)
            .FirstOrDefault();

        object instance = ctor == null
            ? Activator.CreateInstance(type)!
            : ctor.Invoke(ctor.GetParameters()
                .Select((parameter, index) => CreateSample(parameter.ParameterType, index + 1))
                .ToArray());

        foreach (var field in type.GetFields(BindingFlags.Instance | BindingFlags.Public))
        {
            Assert.NotNull(field.GetValue(instance));
        }

        Assert.Contains(type.Name, instance.ToString(), StringComparison.Ordinal);
    }

    [Fact]
    public void Mat3x3_Indexer_Executes()
    {
        var matrix = new Mat3x3();
        for (var i = 0; i < 9; i++)
        {
            matrix[i] = i + 0.5f;
        }

        for (var i = 0; i < 9; i++)
        {
            Assert.Equal(i + 0.5f, matrix[i], 3);
        }

        Assert.Equal("Mat3x3(...)", matrix.ToString());
    }

    private static object CreateSample(Type type, int seed)
    {
        var nullableType = Nullable.GetUnderlyingType(type);
        if (nullableType != null)
        {
            return CreateSample(nullableType, seed);
        }

        if (type == typeof(bool))
        {
            return seed % 2 == 0;
        }

        if (type == typeof(byte))
        {
            return (byte)(seed + 1);
        }

        if (type == typeof(uint))
        {
            return (uint)(seed * 3);
        }

        if (type == typeof(ulong))
        {
            return (ulong)(seed * 11);
        }

        if (type == typeof(float))
        {
            return seed + 0.25f;
        }

        if (type == typeof(string))
        {
            return $"value-{seed}";
        }

        if (type == typeof(byte[]))
        {
            return new byte[] { (byte)seed, (byte)(seed + 1) };
        }

        if (type.IsEnum)
        {
            return Enum.GetValues(type).GetValue(seed % Enum.GetValues(type).Length)!;
        }

        if (type == typeof(Color))
        {
            return new Color(0.1f * seed, 0.2f, 0.3f, 1f);
        }

        if (type == typeof(Vec2))
        {
            return new Vec2(seed, seed + 1);
        }

        if (type == typeof(Vec3))
        {
            return new Vec3(seed, seed + 1, seed + 2);
        }

        if (type.IsValueType)
        {
            var ctor = type.GetConstructors(BindingFlags.Instance | BindingFlags.Public)
                .OrderByDescending(candidate => candidate.GetParameters().Length)
                .FirstOrDefault();

            if (ctor != null)
            {
                return ctor.Invoke(ctor.GetParameters()
                    .Select((parameter, index) => CreateSample(parameter.ParameterType, seed + index + 1))
                    .ToArray());
            }

            return Activator.CreateInstance(type)!;
        }

        throw new InvalidOperationException($"No sample generator for {type.FullName}.");
    }
}
