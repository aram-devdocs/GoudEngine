using System;
using System.Collections.Generic;
using GoudEngine;

internal static class Program
{
    private readonly record struct CheckResult(string Name, bool Passed);

    private static void Main()
    {
        var results = new List<CheckResult>();

        RunContextChecks(results);

        int passed = 0;
        foreach (var result in results)
        {
            if (result.Passed)
            {
                passed++;
            }
        }

        int failed = results.Count - passed;
        Console.WriteLine($"C# Feature Lab complete: {passed} pass, {failed} fail");
        foreach (var result in results)
        {
            Console.WriteLine($"{(result.Passed ? "PASS" : "FAIL")}: {result.Name}");
        }

        if (failed > 0)
        {
            Environment.Exit(1);
        }
    }

    private static void RunContextChecks(List<CheckResult> results)
    {
        using var ctx = new GoudContext();

        var probe = ctx.SpawnEmpty();
        Record(results, "headless context spawns entities", ctx.IsValid() && ctx.IsAlive(probe) && ctx.EntityCount() == 1);

        ctx.AddName(probe, "feature_probe");
        Record(
            results,
            "name component add/get/remove call-path executes headless",
            RunsWithoutException(() =>
            {
                ctx.AddName(probe, "feature_probe");
                _ = ctx.GetName(probe);
                _ = ctx.HasName(probe);
                _ = ctx.RemoveName(probe);
            })
        );

        using var transformBuilder = Transform2DBuilder.AtPosition(10.0f, 20.0f)
            .WithRotationDegrees(45.0f)
            .WithScale(2.0f, 0.5f)
            .Translate(1.0f, -1.0f);
        var transform = transformBuilder.Build();

        ctx.AddTransform2d(probe, transform);
        var fetchedTransform = ctx.GetTransform2d(probe);
        bool transformOk = RunsWithoutException(() =>
        {
            _ = fetchedTransform.HasValue;
            _ = ctx.HasTransform2d(probe);
            ctx.SetTransform2d(probe, transform);
            _ = ctx.RemoveTransform2d(probe);
        });
        Record(results, "transform component + builder call-path executes headless", transformOk);

        using var spriteBuilder = SpriteBuilder.Default()
            .WithColor(0.2f, 0.6f, 1.0f, 0.75f)
            .WithCustomSize(16.0f, 16.0f)
            .WithFlipX(true);
        var sprite = spriteBuilder.Build();

        ctx.AddSprite(probe, sprite);
        var fetchedSprite = ctx.GetSprite(probe);
        bool spriteOk = RunsWithoutException(() =>
        {
            _ = fetchedSprite.HasValue;
            _ = ctx.HasSprite(probe);
            ctx.SetSprite(probe, sprite);
            _ = ctx.RemoveSprite(probe);
        });
        Record(results, "sprite component + builder call-path executes headless", spriteOk);

        Entity[] batch = ctx.SpawnBatch(3);
        var alive = new byte[batch.Length];
        uint checkedCount = ctx.IsAliveBatch(batch, alive);
        bool allAlive = checkedCount == batch.Length;
        for (int i = 0; i < alive.Length; i++)
        {
            allAlive &= alive[i] != 0;
        }
        Record(results, "batch spawn and alive checks work", allAlive);

        uint removed = ctx.DespawnBatch(batch);
        bool batchRemoved = removed == batch.Length;
        for (int i = 0; i < batch.Length; i++)
        {
            batchRemoved &= !ctx.IsAlive(batch[i]);
        }
        Record(results, "batch despawn works", batchRemoved);

        bool despawnedProbe = ctx.Despawn(probe);
        Record(results, "single entity despawn works", despawnedProbe && !ctx.IsAlive(probe));
    }

    private static void Record(List<CheckResult> results, string name, bool passed)
    {
        results.Add(new CheckResult(name, passed));
    }

    private static bool RunsWithoutException(Action action)
    {
        try
        {
            action();
            return true;
        }
        catch
        {
            return false;
        }
    }
}
