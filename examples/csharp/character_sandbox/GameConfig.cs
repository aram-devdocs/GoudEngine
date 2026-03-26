using System;

sealed class GameConfig
{
    public int NpcCount { get; init; } = 60;
    public int AnimalCount { get; init; } = 24;
    public int Seed { get; init; } = 42;
    public bool VariedAnims { get; init; } = false;
    public bool PhaseLock { get; init; } = false;

    public static GameConfig Parse(string[] args)
    {
        int npcs = 60;
        int animals = 24;
        int seed = 42;
        bool variedAnims = false;
        bool phaseLock = false;

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
        }

        return new GameConfig
        {
            NpcCount = npcs,
            AnimalCount = animals,
            Seed = seed,
            VariedAnims = variedAnims,
            PhaseLock = phaseLock,
        };
    }
}
