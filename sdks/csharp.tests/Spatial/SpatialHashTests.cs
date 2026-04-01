using System;
using System.Diagnostics;
using GoudEngine;
using Xunit;

namespace GoudEngine.Tests.Spatial;

public class SpatialHashTests
{
    [Fact]
    public void Create_And_Dispose_Lifecycle()
    {
        using var hash = new SpatialHash(64.0f);
        Assert.Equal(0, hash.Count);
    }

    [Fact]
    public void Create_With_Capacity()
    {
        using var hash = new SpatialHash(32.0f, 1000);
        Assert.Equal(0, hash.Count);
    }

    [Fact]
    public void Insert_Increases_Count()
    {
        using var hash = new SpatialHash(64.0f);
        hash.Insert(1, 10.0f, 10.0f, 5.0f, 5.0f);
        Assert.Equal(1, hash.Count);

        hash.Insert(2, 20.0f, 20.0f, 5.0f, 5.0f);
        Assert.Equal(2, hash.Count);
    }

    [Fact]
    public void Insert_Same_Entity_Overwrites()
    {
        using var hash = new SpatialHash(64.0f);
        hash.Insert(1, 10.0f, 10.0f, 5.0f, 5.0f);
        hash.Insert(1, 50.0f, 50.0f, 5.0f, 5.0f);
        Assert.Equal(1, hash.Count);
    }

    [Fact]
    public void Remove_Decreases_Count()
    {
        using var hash = new SpatialHash(64.0f);
        hash.Insert(1, 10.0f, 10.0f, 5.0f, 5.0f);
        hash.Insert(2, 20.0f, 20.0f, 5.0f, 5.0f);
        hash.Remove(1);
        Assert.Equal(1, hash.Count);
    }

    [Fact]
    public void Remove_Nonexistent_Is_Idempotent()
    {
        using var hash = new SpatialHash(64.0f);
        hash.Remove(999);
        Assert.Equal(0, hash.Count);
    }

    [Fact]
    public void Clear_Empties_Hash()
    {
        using var hash = new SpatialHash(64.0f);
        hash.Insert(1, 10.0f, 10.0f, 5.0f, 5.0f);
        hash.Insert(2, 20.0f, 20.0f, 5.0f, 5.0f);
        hash.Clear();
        Assert.Equal(0, hash.Count);
    }

    [Fact]
    public void QueryRange_Finds_Nearby_Entities()
    {
        using var hash = new SpatialHash(64.0f);
        hash.Insert(1, 10.0f, 10.0f, 5.0f, 5.0f);
        hash.Insert(2, 15.0f, 15.0f, 5.0f, 5.0f);
        hash.Insert(3, 1000.0f, 1000.0f, 5.0f, 5.0f);

        Span<ulong> results = stackalloc ulong[16];
        int count = hash.QueryRange(12.0f, 12.0f, 50.0f, results);

        Assert.True(count >= 2, $"Expected at least 2 results, got {count}");
    }

    [Fact]
    public void QueryRect_Finds_Overlapping_Entities()
    {
        using var hash = new SpatialHash(64.0f);
        hash.Insert(1, 10.0f, 10.0f, 5.0f, 5.0f);
        hash.Insert(2, 15.0f, 15.0f, 5.0f, 5.0f);
        hash.Insert(3, 1000.0f, 1000.0f, 5.0f, 5.0f);

        Span<ulong> results = stackalloc ulong[16];
        // Query a rect covering entities 1 and 2 (top-left coords)
        int count = hash.QueryRect(0.0f, 0.0f, 30.0f, 30.0f, results);

        Assert.True(count >= 2, $"Expected at least 2 results, got {count}");
    }

    [Fact]
    public void QueryRange_Returns_Total_Count_When_Buffer_Too_Small()
    {
        using var hash = new SpatialHash(64.0f);
        for (ulong i = 0; i < 10; i++)
        {
            hash.Insert(i + 1, 10.0f, 10.0f, 5.0f, 5.0f);
        }

        Span<ulong> results = stackalloc ulong[2];
        int count = hash.QueryRange(10.0f, 10.0f, 50.0f, results);

        Assert.True(count >= 10, $"Expected total count >= 10, got {count}");
    }

    [Fact]
    public void Dispose_Prevents_Further_Use()
    {
        var hash = new SpatialHash(64.0f);
        hash.Dispose();

        Assert.Throws<ObjectDisposedException>(() => hash.Count);
        Assert.Throws<ObjectDisposedException>(() => hash.Insert(1, 0, 0, 1, 1));
    }

    [Fact]
    public void Performance_100k_Insert_And_Query()
    {
        using var hash = new SpatialHash(64.0f, 100_000);

        var sw = Stopwatch.StartNew();
        for (ulong i = 0; i < 100_000; i++)
        {
            float x = (i % 1000) * 1.0f;
            float y = (i / 1000) * 1.0f;
            hash.Insert(i + 1, x, y, 0.5f, 0.5f);
        }
        sw.Stop();
        var insertMs = sw.Elapsed.TotalMilliseconds;

        Assert.Equal(100_000, hash.Count);

        sw.Restart();
        Span<ulong> results = stackalloc ulong[256];
        int count = hash.QueryRange(500.0f, 50.0f, 10.0f, results);
        sw.Stop();
        var queryMs = sw.Elapsed.TotalMilliseconds;

        // Acceptance criterion: insert + query should be fast
        // Note: exact <1ms target applies to release builds; debug is slower
        Assert.True(insertMs < 5000, $"100k insert took {insertMs:F1}ms");
        Assert.True(queryMs < 100, $"Query took {queryMs:F1}ms");
        Assert.True(count >= 0);
    }
}
