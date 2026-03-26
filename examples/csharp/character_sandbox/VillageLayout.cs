using System;

static class VillageLayout
{
    static readonly (float X, float Z)[] HumanoidWaypoints =
    {
        (-12f, 18f), (-8f, 24f), (-2f, 28f), (4f, 20f),
        (10f, 26f), (14f, 30f), (-10f, 34f), (-4f, 38f),
        (4f, 34f), (10f, 40f), (0f, 44f), (16f, 34f),
    };

    static readonly (float X, float Z)[] AnimalWaypoints =
    {
        (-18f, 12f), (-12f, 18f), (-6f, 22f), (0f, 14f),
        (6f, 14f), (12f, 22f), (18f, 18f), (22f, 12f),
    };

    public static (float x, float z) PickSpawnPoint(Random rng, int index, bool isAnimal)
    {
        if (isAnimal)
        {
            int col = index % 8;
            int row = index / 8;
            float x = -18f + col * 5f + Jitter(rng, 0.5f);
            float z = 12f + row * 4.5f + Jitter(rng, 0.5f);
            return (x, z);
        }

        int lane = index % 6;
        int rowIdx = index / 6;
        float[] laneX = { -12f, -7f, -2f, 2f, 7f, 12f };
        float bx = laneX[lane] + Jitter(rng, 0.7f);
        float bz = 18f + rowIdx * 4.5f + Jitter(rng, 0.6f);
        return (bx, bz);
    }

    public static (float x, float z) PickWanderTarget(Random rng, bool isAnimal)
    {
        var waypoints = isAnimal ? AnimalWaypoints : HumanoidWaypoints;
        var anchor = waypoints[rng.Next(waypoints.Length)];
        float jitter = isAnimal ? 2.5f : 1.75f;
        return (anchor.X + Jitter(rng, jitter), anchor.Z + Jitter(rng, jitter));
    }

    public static (float x, float z, float yaw) PickBuildingPosition(int index, Random rng)
    {
        // Two rows of buildings along village streets
        float[] streetX = { -24f, -12f, 12f, 24f };
        float[] streetYaw = { 95f, 85f, 275f, 265f };
        int col = index % streetX.Length;
        int row = index / streetX.Length;
        float x = streetX[col] + Jitter(rng, 1.5f);
        float z = 16f + row * 10f + Jitter(rng, 1.0f);
        float yaw = streetYaw[col] + Jitter(rng, 5f);
        return (x, z, yaw);
    }

    public static (float x, float z, float yaw) PickPropPosition(int index, Random rng)
    {
        // Scatter props between buildings
        float x = -20f + (index % 8) * 5.5f + Jitter(rng, 2f);
        float z = 14f + (index / 8) * 6f + Jitter(rng, 2f);
        float yaw = (float)(rng.NextDouble() * 360.0);
        return (x, z, yaw);
    }

    static float Jitter(Random rng, float half)
        => (float)(rng.NextDouble() * 2.0 * half - half);
}
