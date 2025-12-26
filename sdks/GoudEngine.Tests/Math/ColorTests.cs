using System;
using Xunit;
using GoudEngine.Math;

namespace GoudEngine.Tests.Math;

public class ColorTests
{
    [Fact]
    public void Constructor_SetsComponents()
    {
        var c = new Color(0.5f, 0.6f, 0.7f, 0.8f);
        Assert.Equal(0.5f, c.R);
        Assert.Equal(0.6f, c.G);
        Assert.Equal(0.7f, c.B);
        Assert.Equal(0.8f, c.A);
    }

    [Fact]
    public void Constructor_DefaultsAlphaToOne()
    {
        var c = new Color(0.5f, 0.6f, 0.7f);
        Assert.Equal(1.0f, c.A);
    }

    [Fact]
    public void Constructor_ClampsValues()
    {
        var c = new Color(-0.5f, 1.5f, 0.5f, 2.0f);
        Assert.Equal(0.0f, c.R);
        Assert.Equal(1.0f, c.G);
        Assert.Equal(0.5f, c.B);
        Assert.Equal(1.0f, c.A);
    }

    [Fact]
    public void FromBytes_ConvertsCorrectly()
    {
        var c = Color.FromBytes(255, 128, 0, 255);
        Assert.Equal(1.0f, c.R, 2);
        Assert.InRange(c.G, 0.5f, 0.51f);
        Assert.Equal(0.0f, c.B);
        Assert.Equal(1.0f, c.A);
    }

    [Fact]
    public void FromHex_ParsesSixDigitHex()
    {
        var c = Color.FromHex("#FF0000");
        Assert.Equal(1.0f, c.R);
        Assert.Equal(0.0f, c.G);
        Assert.Equal(0.0f, c.B);
        Assert.Equal(1.0f, c.A);
    }

    [Fact]
    public void FromHex_ParsesEightDigitHex()
    {
        var c = Color.FromHex("00FF0080");
        Assert.Equal(0.0f, c.R);
        Assert.Equal(1.0f, c.G);
        Assert.Equal(0.0f, c.B);
        Assert.InRange(c.A, 0.5f, 0.51f);
    }

    [Fact]
    public void FromHex_ThrowsOnInvalidFormat()
    {
        Assert.Throws<ArgumentException>(() => Color.FromHex("FF"));
    }

    [Fact]
    public void PresetColors_AreCorrect()
    {
        Assert.Equal(new Color(1, 1, 1), Color.White);
        Assert.Equal(new Color(0, 0, 0), Color.Black);
        Assert.Equal(new Color(1, 0, 0), Color.Red);
        Assert.Equal(new Color(0, 1, 0), Color.Green);
        Assert.Equal(new Color(0, 0, 1), Color.Blue);
        Assert.Equal(new Color(0, 0, 0, 0), Color.Transparent);
    }

    [Fact]
    public void ToArray_ReturnsRGBComponents()
    {
        var c = new Color(0.1f, 0.2f, 0.3f, 0.4f);
        var arr = c.ToArray();
        Assert.Equal(3, arr.Length);
        Assert.Equal(0.1f, arr[0]);
        Assert.Equal(0.2f, arr[1]);
        Assert.Equal(0.3f, arr[2]);
    }

    [Fact]
    public void ToArrayWithAlpha_ReturnsAllComponents()
    {
        var c = new Color(0.1f, 0.2f, 0.3f, 0.4f);
        var arr = c.ToArrayWithAlpha();
        Assert.Equal(4, arr.Length);
        Assert.Equal(0.4f, arr[3]);
    }

    [Fact]
    public void ToVector3_ReturnsRGBAsVector()
    {
        var c = new Color(0.1f, 0.2f, 0.3f);
        var v = c.ToVector3();
        Assert.Equal(0.1f, v.X);
        Assert.Equal(0.2f, v.Y);
        Assert.Equal(0.3f, v.Z);
    }

    [Fact]
    public void WithAlpha_CreatesNewColorWithAlpha()
    {
        var c = new Color(1.0f, 0.0f, 0.0f);
        var transparent = c.WithAlpha(0.5f);
        Assert.Equal(1.0f, transparent.R);
        Assert.Equal(0.5f, transparent.A);
    }

    [Fact]
    public void Lerp_InterpolatesColors()
    {
        var a = Color.Black;
        var b = Color.White;
        var mid = Color.Lerp(a, b, 0.5f);
        Assert.Equal(0.5f, mid.R);
        Assert.Equal(0.5f, mid.G);
        Assert.Equal(0.5f, mid.B);
    }

    [Fact]
    public void ScalarMultiplication_MultipliesRGB()
    {
        var c = new Color(0.5f, 0.5f, 0.5f);
        var result = c * 2.0f;
        Assert.Equal(1.0f, result.R); // Clamped
        Assert.Equal(1.0f, result.G);
        Assert.Equal(1.0f, result.B);
    }

    [Fact]
    public void Deconstruct_WorksCorrectly()
    {
        var c = new Color(0.1f, 0.2f, 0.3f, 0.4f);
        var (r, g, b, a) = c;
        Assert.Equal(0.1f, r);
        Assert.Equal(0.2f, g);
        Assert.Equal(0.3f, b);
        Assert.Equal(0.4f, a);
    }
}
