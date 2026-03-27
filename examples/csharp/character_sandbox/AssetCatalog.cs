using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;

enum AssetKind { Humanoid, Animal, Building, Prop }

record AssetEntry(
    string Name,
    string FullPath,
    AssetKind Kind,
    bool IsDynamic,
    float? TargetHeight,
    float FallbackScale,
    float RotationX,
    float RotationY,
    float RotationZ,
    float GroundBias,
    float MoveSpeed,
    string[] IdleClips,
    string[] WalkClips,
    string[] RunClips
);

class AssetCatalog
{
    static readonly string AssetsDir = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "assets");

    static readonly string[] HumanoidIdleClips = { "Idle_Loop", "Idle" };
    static readonly string[] HumanoidWalkClips = { "Walk_Loop", "Walk_Formal_Loop", "Jog_Fwd_Loop" };
    static readonly string[] HumanoidRunClips = { "Sprint_Loop", "Jog_Fwd_Loop", "Walk_Loop" };

    static readonly string[] AnimalIdleClips = { "Idle", "Idle_2", "Idle_HeadLow", "Eating" };
    static readonly string[] AnimalWalkClips = { "Walk", "Trot" };
    static readonly string[] AnimalRunClips = { "Gallop", "Run", "Sprint", "Jog" };

    public AssetEntry PlayerProfile { get; }
    public IReadOnlyList<AssetEntry> Humanoids { get; }
    public IReadOnlyList<AssetEntry> Animals { get; }
    public IReadOnlyList<AssetEntry> Buildings { get; }
    public IReadOnlyList<AssetEntry> Props { get; }

    public AssetCatalog()
    {
        PlayerProfile = Humanoid("Character.glb", 1.8f);

        Humanoids = new[] { PlayerProfile }
            .Where(e => File.Exists(e.FullPath))
            .ToList();

        Animals = new[]
        {
            Animal("Horse.gltf",  2.35f, 4.5f),
            Animal("Deer.gltf",   1.9f,  4.5f),
            Animal("Wolf.gltf",   1.25f, 4.0f),
            Animal("Husky.gltf",  1.15f, 4.0f),
        }.Where(e => File.Exists(e.FullPath)).ToList();

        Buildings = new[]
        {
            Building("Inn.fbx",         5.8f,  0.0125f),
            Building("House_1.fbx",     4.35f, 0.012f),
            Building("House_2.fbx",     4.35f, 0.012f),
            Building("House_3.fbx",     4.35f, 0.012f),
            Building("House_4.fbx",     4.35f, 0.012f),
            Building("Blacksmith.fbx",  5.2f,  0.0125f),
            Building("Mill.fbx",        8.0f,  0.0125f),
            Building("Sawmill.fbx",     7.5f,  0.0125f),
            Building("Stable.fbx",      6.0f,  0.0125f),
            Building("Bell_Tower.fbx", 11.0f,  0.014f),
        }.Where(e => File.Exists(e.FullPath)).ToList();

        Props = new[]
        {
            Prop("Cart.fbx",    1.7f,  0.0115f),
            Prop("Barrel.fbx",  0.95f, 0.0105f),
            Prop("Bench_1.fbx", 1.0f,  0.0105f),
            Prop("Well.fbx",    1.8f,  0.0115f),
            Prop("Fence.fbx",   1.2f,  0.012f),
            Prop("Bonfire.fbx", 0.8f,  0.0105f),
        }.Where(e => File.Exists(e.FullPath)).ToList();

        Console.WriteLine(
            $"Asset catalog: {Humanoids.Count} humanoids, {Animals.Count} animals, " +
            $"{Buildings.Count} buildings, {Props.Count} props"
        );
    }

    AssetEntry Humanoid(string filename, float height) => new(
        Name: Path.GetFileNameWithoutExtension(filename),
        FullPath: Path.Combine(AssetsDir, filename),
        Kind: AssetKind.Humanoid,
        IsDynamic: true,
        TargetHeight: height,
        FallbackScale: 1f,
        RotationX: 0f, RotationY: 0f, RotationZ: 0f,
        GroundBias: 0.02f,
        MoveSpeed: 3f,
        IdleClips: HumanoidIdleClips,
        WalkClips: HumanoidWalkClips,
        RunClips: HumanoidRunClips
    );

    AssetEntry Animal(string filename, float height, float speed) => new(
        Name: Path.GetFileNameWithoutExtension(filename),
        FullPath: Path.Combine(AssetsDir, "animals", filename),
        Kind: AssetKind.Animal,
        IsDynamic: true,
        TargetHeight: height,
        FallbackScale: 0.01f,
        RotationX: 0f, RotationY: 0f, RotationZ: 0f,
        GroundBias: 0.07f,
        MoveSpeed: speed,
        IdleClips: AnimalIdleClips,
        WalkClips: AnimalWalkClips,
        RunClips: AnimalRunClips
    );

    AssetEntry Building(string filename, float height, float scale) => new(
        Name: Path.GetFileNameWithoutExtension(filename),
        FullPath: Path.Combine(AssetsDir, "buildings", filename),
        Kind: AssetKind.Building,
        IsDynamic: false,
        TargetHeight: height,
        FallbackScale: scale,
        RotationX: -90f, RotationY: 0f, RotationZ: 0f,
        GroundBias: 0.01f,
        MoveSpeed: 0f,
        IdleClips: Array.Empty<string>(),
        WalkClips: Array.Empty<string>(),
        RunClips: Array.Empty<string>()
    );

    AssetEntry Prop(string filename, float height, float scale) => new(
        Name: Path.GetFileNameWithoutExtension(filename),
        FullPath: Path.Combine(AssetsDir, "props", filename),
        Kind: AssetKind.Prop,
        IsDynamic: false,
        TargetHeight: height,
        FallbackScale: scale,
        RotationX: -90f, RotationY: 0f, RotationZ: 0f,
        GroundBias: 0.0f,
        MoveSpeed: 0f,
        IdleClips: Array.Empty<string>(),
        WalkClips: Array.Empty<string>(),
        RunClips: Array.Empty<string>()
    );
}
