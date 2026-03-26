using System;
using System.Collections.Generic;
using GoudEngine;

struct CrowdAgent
{
    public uint ModelId;
    public float X, Y, Z;
    public float Facing;
    public float TargetX, TargetZ;
    public float IdleTimer;
    public float MoveSpeed;
    public float SpeedMultiplier;
    public int IdleAnim, WalkAnim;
    public bool IsMoving;
    public bool IsAnimal;
    public float RotationX, YawOffset, RotationZ;
}

class CrowdSystem
{
    readonly GoudGame _game;
    readonly List<CrowdAgent> _agents = new();
    readonly List<LoadedRig> _humanoidRigs = new();
    readonly List<LoadedRig> _animalRigs = new();
    readonly Random _rng;
    readonly uint _sceneId;
    const float TurnSpeed = 360f;

    public int Count => _agents.Count;

    public CrowdSystem(
        GoudGame game, ModelLoader loader, AssetCatalog catalog,
        uint sceneId, int npcCount, int animalCount, int seed)
    {
        _game = game;
        _rng = new Random(seed);
        _sceneId = sceneId;

        foreach (var entry in catalog.Humanoids)
        {
            var rig = loader.LoadRig(entry);
            if (rig != null) _humanoidRigs.Add(rig);
        }

        foreach (var entry in catalog.Animals)
        {
            var rig = loader.LoadRig(entry);
            if (rig != null) _animalRigs.Add(rig);
        }

        for (int i = 0; i < npcCount && _humanoidRigs.Count > 0; i++)
            SpawnAgent(_humanoidRigs[i % _humanoidRigs.Count], i, false);

        for (int i = 0; i < animalCount && _animalRigs.Count > 0; i++)
            SpawnAgent(_animalRigs[i % _animalRigs.Count], i, true);

        Console.WriteLine(
            $"Crowd: {_agents.Count} agents ({npcCount} humanoids, {animalCount} animals)"
        );
    }

    public void Update(float dt)
    {
        for (int i = 0; i < _agents.Count; i++)
        {
            var agent = _agents[i];

            if (agent.IsMoving)
            {
                float dx = agent.TargetX - agent.X;
                float dz = agent.TargetZ - agent.Z;
                float dist = MathF.Sqrt(dx * dx + dz * dz);

                if (dist < 1.0f)
                {
                    agent.IsMoving = false;
                    agent.IdleTimer = 0.75f + (float)(_rng.NextDouble() * 1.75);
                    if (agent.IdleAnim >= 0)
                        _game.TransitionAnimation(agent.ModelId, agent.IdleAnim, 0.25f);
                }
                else
                {
                    float ndx = dx / dist;
                    float ndz = dz / dist;
                    float speed = agent.MoveSpeed * agent.SpeedMultiplier;

                    agent.X += ndx * speed * dt;
                    agent.Z += ndz * speed * dt;

                    float targetFacing = MathF.Atan2(ndx, ndz) * 180f / MathF.PI;
                    float faceDiff = WrapDegrees(targetFacing - agent.Facing);
                    if (MathF.Abs(faceDiff) < TurnSpeed * dt)
                        agent.Facing = targetFacing;
                    else
                        agent.Facing += MathF.Sign(faceDiff) * TurnSpeed * dt;

                    _game.SetModelPosition(agent.ModelId, agent.X, agent.Y, agent.Z);
                    _game.SetModelRotation(
                        agent.ModelId,
                        agent.RotationX,
                        agent.Facing + agent.YawOffset,
                        agent.RotationZ
                    );
                }
            }
            else
            {
                agent.IdleTimer -= dt;
                if (agent.IdleTimer <= 0f)
                {
                    (agent.TargetX, agent.TargetZ) = VillageLayout.PickWanderTarget(_rng, agent.IsAnimal);
                    agent.IsMoving = true;
                    if (agent.WalkAnim >= 0)
                        _game.TransitionAnimation(agent.ModelId, agent.WalkAnim, 0.25f);
                }
            }

            _agents[i] = agent;
        }
    }

    public void SpawnMore(int count)
    {
        var allRigs = new List<LoadedRig>();
        allRigs.AddRange(_humanoidRigs);
        allRigs.AddRange(_animalRigs);
        if (allRigs.Count == 0) return;

        int before = _agents.Count;
        for (int i = 0; i < count; i++)
        {
            var rig = allRigs[(before + i) % allRigs.Count];
            SpawnAgent(rig, before + i, rig.IsAnimal);
        }
        Console.WriteLine($"Spawned: {before} -> {_agents.Count}");
    }

    public void RemoveLast(int count)
    {
        int toRemove = Math.Min(count, _agents.Count);
        int before = _agents.Count;
        for (int i = 0; i < toRemove; i++)
        {
            int last = _agents.Count - 1;
            _game.DestroyModel(_agents[last].ModelId);
            _agents.RemoveAt(last);
        }
        if (toRemove > 0)
            Console.WriteLine($"Removed: {before} -> {_agents.Count}");
    }

    public void DestroyAll()
    {
        foreach (var agent in _agents)
            if (agent.ModelId != 0)
                _game.DestroyModel(agent.ModelId);
        _agents.Clear();
    }

    void SpawnAgent(LoadedRig rig, int index, bool isAnimal)
    {
        uint modelId = _game.InstantiateModel(rig.ModelId);
        if (modelId == 0) return;

        var (spawnX, spawnZ) = VillageLayout.PickSpawnPoint(_rng, index, isAnimal);
        var (targetX, targetZ) = VillageLayout.PickWanderTarget(_rng, isAnimal);

        var agent = new CrowdAgent
        {
            ModelId = modelId,
            X = spawnX,
            Y = rig.GroundOffsetY,
            Z = spawnZ,
            TargetX = targetX,
            TargetZ = targetZ,
            Facing = (float)(_rng.NextDouble() * 360.0),
            IsMoving = _rng.NextDouble() > 0.35,
            IdleTimer = 0f,
            MoveSpeed = rig.MoveSpeed,
            SpeedMultiplier = 0.75f + (float)(_rng.NextDouble() * 0.55),
            IdleAnim = rig.AnimCount > 0 ? rig.IdleAnim : -1,
            WalkAnim = rig.AnimCount > 0 ? rig.WalkAnim : -1,
            IsAnimal = isAnimal,
            RotationX = rig.RotationX,
            YawOffset = rig.RotationY,
            RotationZ = rig.RotationZ,
        };

        if (!agent.IsMoving)
            agent.IdleTimer = (float)(_rng.NextDouble() * 2.0 + 0.75);

        _game.SetModelPosition(modelId, agent.X, agent.Y, agent.Z);
        _game.SetModelRotation(modelId, agent.RotationX, agent.Facing + agent.YawOffset, agent.RotationZ);
        _game.SetModelScale(modelId, rig.Scale, rig.Scale, rig.Scale);
        _game.AddModelToScene(_sceneId, modelId);

        if (rig.AnimCount > 0)
        {
            int startAnim = agent.IsMoving && rig.WalkAnim >= 0 ? rig.WalkAnim : rig.IdleAnim;
            if (startAnim >= 0)
            {
                _game.PlayAnimation(modelId, startAnim, true);
                _game.SetAnimationSpeed(modelId, agent.SpeedMultiplier);
            }
        }

        _agents.Add(agent);
    }

    static float WrapDegrees(float d)
    {
        d %= 360f;
        if (d > 180f) d -= 360f;
        if (d < -180f) d += 360f;
        return d;
    }
}
