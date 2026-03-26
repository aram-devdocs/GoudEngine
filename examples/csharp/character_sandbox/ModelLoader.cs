using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using GoudEngine;

record LoadedRig(
    uint ModelId,
    float Scale,
    float GroundOffsetY,
    float RotationX,
    float RotationY,
    float RotationZ,
    int AnimCount,
    int IdleAnim,
    int WalkAnim,
    int RunAnim,
    float MoveSpeed,
    bool IsAnimal
);

class ModelLoader
{
    readonly GoudGame _game;
    readonly List<uint> _sourceIds = new();

    public ModelLoader(GoudGame game) => _game = game;

    public IReadOnlyList<uint> SourceIds => _sourceIds;

    public LoadedRig? LoadRig(AssetEntry entry)
    {
        if (!File.Exists(entry.FullPath))
        {
            Console.WriteLine($"Asset not found: {entry.FullPath}");
            return null;
        }

        uint modelId = _game.LoadModel(entry.FullPath);
        if (modelId == 0)
        {
            Console.WriteLine($"Failed to load model: {entry.FullPath}");
            return null;
        }

        BoundingBox3D bounds = _game.GetModelBoundingBox(modelId);
        float scale = FitToHeight(bounds, entry.FallbackScale, entry.TargetHeight);
        float groundY = ComputeGroundOffset(bounds, scale, entry.GroundBias);
        int animCount = _game.GetAnimationCount(modelId);
        var (idle, walk, run) = ResolveAnimations(modelId, animCount, entry);

        _sourceIds.Add(modelId);
        Console.WriteLine(
            $"Loaded {entry.Name}: scale={scale:F4} groundY={groundY:F3} anims={animCount} " +
            $"idle={idle} walk={walk} run={run}"
        );

        return new LoadedRig(
            modelId, scale, groundY,
            entry.RotationX, entry.RotationY, entry.RotationZ,
            animCount, idle, walk, run,
            entry.MoveSpeed, entry.Kind == AssetKind.Animal
        );
    }

    public uint LoadStatic(AssetEntry entry)
    {
        if (!File.Exists(entry.FullPath)) return 0;

        uint modelId = _game.LoadModel(entry.FullPath);
        if (modelId == 0) return 0;

        _sourceIds.Add(modelId);
        return modelId;
    }

    public void DestroyAll()
    {
        foreach (uint id in _sourceIds)
            _game.DestroyModel(id);
    }

    static float FitToHeight(BoundingBox3D bounds, float fallback, float? target)
    {
        if (target is null) return fallback;
        float rawH = MathF.Max(bounds.MaxY - bounds.MinY, 0.0001f);
        float fitted = target.Value / rawH;
        return float.IsFinite(fitted) && fitted > 0f ? fitted : fallback;
    }

    static float ComputeGroundOffset(BoundingBox3D bounds, float scale, float bias)
        => (-bounds.MinY * scale) + bias;

    (int idle, int walk, int run) ResolveAnimations(uint modelId, int count, AssetEntry entry)
    {
        if (count == 0) return (-1, -1, -1);

        var nameMap = new Dictionary<string, int>();
        for (int i = 0; i < count; i++)
        {
            string raw = _game.GetAnimationName(modelId, i);
            nameMap[NormalizeName(raw)] = i;
        }

        int idle = FindBestMatch(nameMap, entry.IdleClips);
        int walk = FindBestMatch(nameMap, entry.WalkClips);
        int run = FindBestMatch(nameMap, entry.RunClips);

        // Fallback chain: if walk/run not found, fall back to idle
        if (walk < 0) walk = idle;
        if (run < 0) run = walk;

        return (idle, walk, run);
    }

    static int FindBestMatch(Dictionary<string, int> nameMap, string[] preferred)
    {
        // Exact match first
        foreach (string name in preferred)
        {
            string norm = NormalizeName(name);
            if (nameMap.TryGetValue(norm, out int idx))
                return idx;
        }

        // Substring match
        foreach (string name in preferred)
        {
            string norm = NormalizeName(name);
            foreach (var (key, idx) in nameMap)
            {
                if (key.Contains(norm) || norm.Contains(key))
                    return idx;
            }
        }

        return -1;
    }

    static string NormalizeName(string raw)
    {
        string name = raw;
        int pipe = name.LastIndexOf('|');
        if (pipe >= 0) name = name[(pipe + 1)..];
        return name.Replace(" ", "").Replace("-", "").Replace("_", "").ToLowerInvariant();
    }
}
