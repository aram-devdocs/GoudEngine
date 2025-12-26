using System;
using Xunit;
using GoudEngine.Math;

namespace GoudEngine.Tests.Math;

public class Vector3Tests
{
    [Fact]
    public void Constructor_SetsComponents()
    {
        var v = new Vector3(1.0f, 2.0f, 3.0f);
        Assert.Equal(1.0f, v.X);
        Assert.Equal(2.0f, v.Y);
        Assert.Equal(3.0f, v.Z);
    }

    [Fact]
    public void Constructor_SingleValue_SetsAllComponents()
    {
        var v = new Vector3(5.0f);
        Assert.Equal(5.0f, v.X);
        Assert.Equal(5.0f, v.Y);
        Assert.Equal(5.0f, v.Z);
    }

    [Fact]
    public void Constructor_FromVector2_SetsXYAndZ()
    {
        var v = new Vector3(new Vector2(1.0f, 2.0f), 3.0f);
        Assert.Equal(1.0f, v.X);
        Assert.Equal(2.0f, v.Y);
        Assert.Equal(3.0f, v.Z);
    }

    [Fact]
    public void StaticVectors_AreCorrect()
    {
        Assert.Equal(new Vector3(0, 0, 0), Vector3.Zero);
        Assert.Equal(new Vector3(1, 1, 1), Vector3.One);
        Assert.Equal(new Vector3(0, 1, 0), Vector3.Up);
        Assert.Equal(new Vector3(0, -1, 0), Vector3.Down);
        Assert.Equal(new Vector3(1, 0, 0), Vector3.Right);
        Assert.Equal(new Vector3(-1, 0, 0), Vector3.Left);
        Assert.Equal(new Vector3(0, 0, -1), Vector3.Forward);
        Assert.Equal(new Vector3(0, 0, 1), Vector3.Backward);
    }

    [Fact]
    public void Length_CalculatesCorrectly()
    {
        var v = new Vector3(2.0f, 3.0f, 6.0f);
        Assert.Equal(7.0f, v.Length);
    }

    [Fact]
    public void XY_ReturnsVector2()
    {
        var v = new Vector3(1.0f, 2.0f, 3.0f);
        var xy = v.XY;
        Assert.Equal(1.0f, xy.X);
        Assert.Equal(2.0f, xy.Y);
    }

    [Fact]
    public void Cross_CalculatesCorrectly()
    {
        var a = new Vector3(1.0f, 0.0f, 0.0f);
        var b = new Vector3(0.0f, 1.0f, 0.0f);
        var cross = a.Cross(b);
        Assert.Equal(0.0f, cross.X);
        Assert.Equal(0.0f, cross.Y);
        Assert.Equal(1.0f, cross.Z);
    }

    [Fact]
    public void Dot_CalculatesCorrectly()
    {
        var a = new Vector3(1.0f, 2.0f, 3.0f);
        var b = new Vector3(4.0f, 5.0f, 6.0f);
        Assert.Equal(32.0f, a.Dot(b));
    }

    [Fact]
    public void Distance_CalculatesCorrectly()
    {
        var a = Vector3.Zero;
        var b = new Vector3(2.0f, 3.0f, 6.0f);
        Assert.Equal(7.0f, Vector3.Distance(a, b));
    }

    [Fact]
    public void Lerp_InterpolatesCorrectly()
    {
        var a = Vector3.Zero;
        var b = new Vector3(10.0f, 20.0f, 30.0f);

        var mid = Vector3.Lerp(a, b, 0.5f);
        Assert.Equal(5.0f, mid.X);
        Assert.Equal(10.0f, mid.Y);
        Assert.Equal(15.0f, mid.Z);
    }

    [Fact]
    public void Operators_WorkCorrectly()
    {
        var a = new Vector3(1.0f, 2.0f, 3.0f);
        var b = new Vector3(4.0f, 5.0f, 6.0f);

        var sum = a + b;
        Assert.Equal(new Vector3(5.0f, 7.0f, 9.0f), sum);

        var diff = b - a;
        Assert.Equal(new Vector3(3.0f, 3.0f, 3.0f), diff);

        var scaled = a * 2.0f;
        Assert.Equal(new Vector3(2.0f, 4.0f, 6.0f), scaled);

        var divided = b / 2.0f;
        Assert.Equal(new Vector3(2.0f, 2.5f, 3.0f), divided);

        var negated = -a;
        Assert.Equal(new Vector3(-1.0f, -2.0f, -3.0f), negated);
    }

    [Fact]
    public void Deconstruct_WorksCorrectly()
    {
        var v = new Vector3(1.0f, 2.0f, 3.0f);
        var (x, y, z) = v;
        Assert.Equal(1.0f, x);
        Assert.Equal(2.0f, y);
        Assert.Equal(3.0f, z);
    }
}
