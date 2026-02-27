using System;
using Xunit;
using GoudEngine.Math;

namespace GoudEngine.Tests.Math;

public class RectangleTests
{
    [Fact]
    public void Constructor_SetsProperties()
    {
        var r = new Rectangle(10, 20, 100, 50);
        Assert.Equal(10, r.X);
        Assert.Equal(20, r.Y);
        Assert.Equal(100, r.Width);
        Assert.Equal(50, r.Height);
    }

    [Fact]
    public void Constructor_FromVectors_Works()
    {
        var r = new Rectangle(new Vector2(10, 20), new Vector2(100, 50));
        Assert.Equal(10, r.X);
        Assert.Equal(20, r.Y);
        Assert.Equal(100, r.Width);
        Assert.Equal(50, r.Height);
    }

    [Fact]
    public void Bounds_AreCorrect()
    {
        var r = new Rectangle(10, 20, 100, 50);
        Assert.Equal(10, r.Left);
        Assert.Equal(110, r.Right);
        Assert.Equal(20, r.Top);
        Assert.Equal(70, r.Bottom);
    }

    [Fact]
    public void Center_IsCorrect()
    {
        var r = new Rectangle(0, 0, 100, 50);
        var center = r.Center;
        Assert.Equal(50, center.X);
        Assert.Equal(25, center.Y);
    }

    [Fact]
    public void Area_IsCorrect()
    {
        var r = new Rectangle(0, 0, 10, 5);
        Assert.Equal(50, r.Area);
    }

    [Fact]
    public void Contains_Point_ReturnsTrue_WhenInside()
    {
        var r = new Rectangle(0, 0, 100, 100);
        Assert.True(r.Contains(50, 50));
        Assert.True(r.Contains(0, 0));
        Assert.True(r.Contains(99, 99));
    }

    [Fact]
    public void Contains_Point_ReturnsFalse_WhenOutside()
    {
        var r = new Rectangle(0, 0, 100, 100);
        Assert.False(r.Contains(-1, 50));
        Assert.False(r.Contains(100, 50)); // Edge is exclusive
        Assert.False(r.Contains(50, 100));
    }

    [Fact]
    public void Contains_Vector2_Works()
    {
        var r = new Rectangle(0, 0, 100, 100);
        Assert.True(r.Contains(new Vector2(50, 50)));
        Assert.False(r.Contains(new Vector2(150, 50)));
    }

    [Fact]
    public void Intersects_ReturnsTrue_WhenOverlapping()
    {
        var a = new Rectangle(0, 0, 100, 100);
        var b = new Rectangle(50, 50, 100, 100);
        Assert.True(a.Intersects(b));
        Assert.True(b.Intersects(a));
    }

    [Fact]
    public void Intersects_ReturnsFalse_WhenNotOverlapping()
    {
        var a = new Rectangle(0, 0, 50, 50);
        var b = new Rectangle(100, 100, 50, 50);
        Assert.False(a.Intersects(b));
    }

    [Fact]
    public void Intersects_ReturnsFalse_WhenTouching()
    {
        var a = new Rectangle(0, 0, 50, 50);
        var b = new Rectangle(50, 0, 50, 50);
        Assert.False(a.Intersects(b)); // Touching edges don't intersect
    }

    [Fact]
    public void Offset_MovesRectangle()
    {
        var r = new Rectangle(10, 20, 100, 50);
        var moved = r.Offset(5, 10);
        Assert.Equal(15, moved.X);
        Assert.Equal(30, moved.Y);
        Assert.Equal(100, moved.Width);
        Assert.Equal(50, moved.Height);
    }

    [Fact]
    public void Inflate_ExpandsRectangle()
    {
        var r = new Rectangle(50, 50, 100, 100);
        var inflated = r.Inflate(10, 10);
        Assert.Equal(40, inflated.X);
        Assert.Equal(40, inflated.Y);
        Assert.Equal(120, inflated.Width);
        Assert.Equal(120, inflated.Height);
    }

    [Fact]
    public void Intersection_ReturnsOverlappingArea()
    {
        var a = new Rectangle(0, 0, 100, 100);
        var b = new Rectangle(50, 50, 100, 100);
        var intersection = Rectangle.Intersection(a, b);

        Assert.NotNull(intersection);
        Assert.Equal(50, intersection.Value.X);
        Assert.Equal(50, intersection.Value.Y);
        Assert.Equal(50, intersection.Value.Width);
        Assert.Equal(50, intersection.Value.Height);
    }

    [Fact]
    public void Intersection_ReturnsNull_WhenNoOverlap()
    {
        var a = new Rectangle(0, 0, 50, 50);
        var b = new Rectangle(100, 100, 50, 50);
        var intersection = Rectangle.Intersection(a, b);
        Assert.Null(intersection);
    }

    [Fact]
    public void Union_ReturnsBoundingRectangle()
    {
        var a = new Rectangle(0, 0, 50, 50);
        var b = new Rectangle(100, 100, 50, 50);
        var union = Rectangle.Union(a, b);

        Assert.Equal(0, union.X);
        Assert.Equal(0, union.Y);
        Assert.Equal(150, union.Width);
        Assert.Equal(150, union.Height);
    }

    [Fact]
    public void Empty_HasZeroDimensions()
    {
        var empty = Rectangle.Empty;
        Assert.Equal(0, empty.X);
        Assert.Equal(0, empty.Y);
        Assert.Equal(0, empty.Width);
        Assert.Equal(0, empty.Height);
    }

    [Fact]
    public void Equality_WorksCorrectly()
    {
        var a = new Rectangle(10, 20, 100, 50);
        var b = new Rectangle(10, 20, 100, 50);
        var c = new Rectangle(10, 20, 100, 60);

        Assert.True(a == b);
        Assert.False(a == c);
        Assert.True(a.Equals(b));
    }
}
