using System;
using Xunit;
using GoudEngine.Math;

namespace GoudEngine.Tests.Math;

public class Vector2Tests
{
    [Fact]
    public void Constructor_SetsXAndY()
    {
        var v = new Vector2(3.0f, 4.0f);
        Assert.Equal(3.0f, v.X);
        Assert.Equal(4.0f, v.Y);
    }

    [Fact]
    public void Constructor_SingleValue_SetsBothComponents()
    {
        var v = new Vector2(5.0f);
        Assert.Equal(5.0f, v.X);
        Assert.Equal(5.0f, v.Y);
    }

    [Fact]
    public void Zero_ReturnsZeroVector()
    {
        var v = Vector2.Zero;
        Assert.Equal(0.0f, v.X);
        Assert.Equal(0.0f, v.Y);
    }

    [Fact]
    public void One_ReturnsOneVector()
    {
        var v = Vector2.One;
        Assert.Equal(1.0f, v.X);
        Assert.Equal(1.0f, v.Y);
    }

    [Fact]
    public void Length_CalculatesCorrectly()
    {
        var v = new Vector2(3.0f, 4.0f);
        Assert.Equal(5.0f, v.Length);
    }

    [Fact]
    public void LengthSquared_CalculatesCorrectly()
    {
        var v = new Vector2(3.0f, 4.0f);
        Assert.Equal(25.0f, v.LengthSquared);
    }

    [Fact]
    public void Normalized_ReturnsUnitVector()
    {
        var v = new Vector2(3.0f, 4.0f);
        var normalized = v.Normalized();
        Assert.Equal(0.6f, normalized.X, 5);
        Assert.Equal(0.8f, normalized.Y, 5);
        Assert.Equal(1.0f, normalized.Length, 5);
    }

    [Fact]
    public void Normalized_ZeroVector_ReturnsZero()
    {
        var v = Vector2.Zero;
        var normalized = v.Normalized();
        Assert.Equal(Vector2.Zero, normalized);
    }

    [Fact]
    public void Dot_CalculatesCorrectly()
    {
        var a = new Vector2(1.0f, 2.0f);
        var b = new Vector2(3.0f, 4.0f);
        Assert.Equal(11.0f, a.Dot(b));
    }

    [Fact]
    public void Distance_CalculatesCorrectly()
    {
        var a = new Vector2(0.0f, 0.0f);
        var b = new Vector2(3.0f, 4.0f);
        Assert.Equal(5.0f, Vector2.Distance(a, b));
    }

    [Fact]
    public void Lerp_InterpolatesCorrectly()
    {
        var a = new Vector2(0.0f, 0.0f);
        var b = new Vector2(10.0f, 20.0f);

        var mid = Vector2.Lerp(a, b, 0.5f);
        Assert.Equal(5.0f, mid.X);
        Assert.Equal(10.0f, mid.Y);
    }

    [Fact]
    public void Lerp_ClampsTParameter()
    {
        var a = new Vector2(0.0f, 0.0f);
        var b = new Vector2(10.0f, 10.0f);

        var below = Vector2.Lerp(a, b, -1.0f);
        var above = Vector2.Lerp(a, b, 2.0f);

        Assert.Equal(a, below);
        Assert.Equal(b, above);
    }

    [Fact]
    public void Addition_AddsComponents()
    {
        var a = new Vector2(1.0f, 2.0f);
        var b = new Vector2(3.0f, 4.0f);
        var result = a + b;
        Assert.Equal(4.0f, result.X);
        Assert.Equal(6.0f, result.Y);
    }

    [Fact]
    public void Subtraction_SubtractsComponents()
    {
        var a = new Vector2(5.0f, 7.0f);
        var b = new Vector2(2.0f, 3.0f);
        var result = a - b;
        Assert.Equal(3.0f, result.X);
        Assert.Equal(4.0f, result.Y);
    }

    [Fact]
    public void ScalarMultiplication_MultipliesComponents()
    {
        var v = new Vector2(2.0f, 3.0f);
        var result = v * 2.0f;
        Assert.Equal(4.0f, result.X);
        Assert.Equal(6.0f, result.Y);
    }

    [Fact]
    public void ScalarDivision_DividesComponents()
    {
        var v = new Vector2(6.0f, 8.0f);
        var result = v / 2.0f;
        Assert.Equal(3.0f, result.X);
        Assert.Equal(4.0f, result.Y);
    }

    [Fact]
    public void Negation_NegatesComponents()
    {
        var v = new Vector2(3.0f, -4.0f);
        var result = -v;
        Assert.Equal(-3.0f, result.X);
        Assert.Equal(4.0f, result.Y);
    }

    [Fact]
    public void Equality_WorksCorrectly()
    {
        var a = new Vector2(1.0f, 2.0f);
        var b = new Vector2(1.0f, 2.0f);
        var c = new Vector2(1.0f, 3.0f);

        Assert.True(a == b);
        Assert.False(a == c);
        Assert.True(a != c);
    }

    [Fact]
    public void Deconstruct_WorksCorrectly()
    {
        var v = new Vector2(3.0f, 4.0f);
        var (x, y) = v;
        Assert.Equal(3.0f, x);
        Assert.Equal(4.0f, y);
    }

    [Fact]
    public void ToString_FormatsCorrectly()
    {
        var v = new Vector2(1.5f, 2.5f);
        Assert.Equal("(1.5, 2.5)", v.ToString());
    }
}
