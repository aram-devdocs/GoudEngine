using System;
using System.Collections.Generic;
using System.Text.Json;
using GoudEngine;

internal static class Program
{
    private const string DebuggerRouteLabel = "feature-lab-csharp-headless";
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
        using var ctx = new GoudContext(
            new ContextConfig(new DebuggerConfig(true, true, DebuggerRouteLabel))
        );
        ctx.SetDebuggerProfilingEnabled(true);

        Record(
            results,
            "debugger manifest publishes the stable attach route label",
            DebuggerManifestPublishesRoute(ctx)
        );
        Record(
            results,
            "debugger snapshot json is available for the headless route",
            DebuggerSnapshotIsAvailable(ctx)
        );
        PrintManualAttachWorkflow();

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

    private static bool DebuggerManifestPublishesRoute(GoudContext ctx)
    {
        string manifestJson = ctx.GetDebuggerManifestJson();
        if (string.IsNullOrWhiteSpace(manifestJson))
        {
            return false;
        }

        using JsonDocument manifest = JsonDocument.Parse(manifestJson);
        if (!manifest.RootElement.TryGetProperty("routes", out JsonElement routes) ||
            routes.ValueKind != JsonValueKind.Array)
        {
            return false;
        }

        foreach (JsonElement route in routes.EnumerateArray())
        {
            if (!route.TryGetProperty("label", out JsonElement label) ||
                label.ValueKind != JsonValueKind.String)
            {
                continue;
            }

            bool attachable = route.TryGetProperty("attachable", out JsonElement attachFlag) &&
                              attachFlag.ValueKind == JsonValueKind.True;
            if (label.GetString() == DebuggerRouteLabel && attachable)
            {
                return true;
            }
        }

        return false;
    }

    private static bool DebuggerSnapshotIsAvailable(GoudContext ctx)
    {
        string snapshotJson = ctx.GetDebuggerSnapshotJson();
        if (string.IsNullOrWhiteSpace(snapshotJson))
        {
            return false;
        }

        using JsonDocument snapshot = JsonDocument.Parse(snapshotJson);
        return snapshot.RootElement.TryGetProperty("snapshot_version", out JsonElement version) &&
               version.ValueKind == JsonValueKind.Number &&
               version.GetInt32() >= 1 &&
               snapshot.RootElement.TryGetProperty("route_id", out JsonElement routeId) &&
               routeId.ValueKind == JsonValueKind.Object;
    }

    private static void PrintManualAttachWorkflow()
    {
        Console.WriteLine($"Debugger route label: {DebuggerRouteLabel}");
        Console.WriteLine("Manual attach workflow:");
        Console.WriteLine("1. start `cargo run -p goudengine-mcp`");
        Console.WriteLine("2. call `goudengine.list_contexts`");
        Console.WriteLine("3. call `goudengine.attach_context`");
    }
}
