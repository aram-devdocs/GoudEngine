using System;

sealed class GameConfig
{
    public int NpcCount { get; init; } = 60;
    public int AnimalCount { get; init; } = 24;
    public int Seed { get; init; } = 42;
    public bool VariedAnims { get; init; } = false;
    public bool PhaseLock { get; init; } = false;
    public bool Profile { get; init; } = false;
    public int ProfileDuration { get; init; } = 10;
    public bool Shadows { get; init; } = false;
    public uint ShadowSize { get; init; } = 1024;
    public bool Vsync { get; init; } = true;

    public static GameConfig Parse(string[] args)
    {
        int npcs = 60;
        int animals = 24;
        int seed = 42;
        bool variedAnims = false;
        bool phaseLock = false;
        bool profile = false;
        int profileDuration = 10;
        bool shadows = false;
        uint shadowSize = 1024;
        bool vsync = true;

        for (int i = 0; i < args.Length; i++)
        {
            if (args[i] == "--npcs" && i + 1 < args.Length && int.TryParse(args[i + 1], out int n))
            {
                npcs = Math.Max(0, n);
                i++;
            }
            else if (args[i] == "--animals" && i + 1 < args.Length && int.TryParse(args[i + 1], out int a))
            {
                animals = Math.Max(0, a);
                i++;
            }
            else if (args[i] == "--seed" && i + 1 < args.Length && int.TryParse(args[i + 1], out int s))
            {
                seed = s;
                i++;
            }
            else if (args[i] == "--varied-anims")
            {
                variedAnims = true;
            }
            else if (args[i] == "--phase-lock")
            {
                phaseLock = true;
            }
            else if (args[i] == "--profile")
            {
                profile = true;
            }
            else if (args[i] == "--duration" && i + 1 < args.Length && int.TryParse(args[i + 1], out int d))
            {
                profileDuration = Math.Max(1, d);
                i++;
            }
            else if (args[i] == "--shadows")
            {
                shadows = true;
            }
            else if (args[i] == "--shadow-size" && i + 1 < args.Length && uint.TryParse(args[i + 1], out uint ss))
            {
                shadowSize = ss;
                i++;
            }
            else if (args[i] == "--no-vsync")
            {
                vsync = false;
            }
        }

        return new GameConfig
        {
            NpcCount = npcs,
            AnimalCount = animals,
            Seed = seed,
            VariedAnims = variedAnims,
            PhaseLock = phaseLock,
            Profile = profile,
            ProfileDuration = profileDuration,
            Shadows = shadows,
            ShadowSize = shadowSize,
            Vsync = vsync,
        };
    }
}
