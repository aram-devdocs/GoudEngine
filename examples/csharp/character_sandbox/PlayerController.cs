using System;
using GoudEngine;

class PlayerController
{
    readonly GoudGame _game;
    readonly LoadedRig _rig;

    float _x, _y, _z;
    float _facing;

    float _cameraPitch = 25f;
    float _cameraYaw = 0f;
    float _cameraDistance = 12f;
    const float CameraHeight = 6f;
    const float CameraRotSpeed = 90f;
    const float MinCameraDistance = 2f;
    const float MaxCameraDistance = 50f;
    const float ZoomSpeed = 15f;

    const float WalkSpeed = 4f;
    const float RunSpeed = 9f;
    const float TurnSpeed = 720f;
    const float AnimBlendTime = 0.25f;

    enum AnimState { Idle, Walk, Run }
    AnimState _currentAnim = AnimState.Idle;

    public PlayerController(GoudGame game, LoadedRig rig, uint sceneId)
    {
        _game = game;
        _rig = rig;
        _y = rig.GroundOffsetY;

        game.SetModelPosition(rig.ModelId, _x, _y, _z);
        game.SetModelRotation(rig.ModelId, rig.RotationX, _facing + rig.RotationY, rig.RotationZ);
        game.SetModelScale(rig.ModelId, rig.Scale, rig.Scale, rig.Scale);
        game.AddModelToScene(sceneId, rig.ModelId);

        if (rig.AnimCount > 0 && rig.IdleAnim >= 0)
            game.PlayAnimation(rig.ModelId, rig.IdleAnim, true);
    }

    public void Update(float dt)
    {
        HandleCameraInput(dt);
        var (moveX, moveZ, moving, running) = HandleMovementInput();

        float speed = running ? RunSpeed : WalkSpeed;
        _x += moveX * speed * dt;
        _z += moveZ * speed * dt;

        if (moving)
            SmoothRotateToward(MathF.Atan2(moveX, moveZ) * 180f / MathF.PI, dt);

        _game.SetModelPosition(_rig.ModelId, _x, _y, _z);
        _game.SetModelRotation(_rig.ModelId, _rig.RotationX, _facing + _rig.RotationY, _rig.RotationZ);

        AnimState desired = !moving ? AnimState.Idle : (running ? AnimState.Run : AnimState.Walk);
        if (desired != _currentAnim && _rig.AnimCount > 0)
        {
            int targetIndex = desired switch
            {
                AnimState.Walk => _rig.WalkAnim,
                AnimState.Run  => _rig.RunAnim,
                _              => _rig.IdleAnim,
            };
            if (targetIndex >= 0)
                _game.TransitionAnimation(_rig.ModelId, targetIndex, AnimBlendTime);
            _currentAnim = desired;
        }
    }

    public void UpdateCamera()
    {
        float yawRad = _cameraYaw * MathF.PI / 180f;
        float pitchRad = _cameraPitch * MathF.PI / 180f;

        float camX = _x - MathF.Sin(yawRad) * MathF.Cos(pitchRad) * _cameraDistance;
        float camY = _y + MathF.Sin(pitchRad) * _cameraDistance + CameraHeight;
        float camZ = _z - MathF.Cos(yawRad) * MathF.Cos(pitchRad) * _cameraDistance;

        _game.SetCameraPosition3D(camX, camY, camZ);

        float lookX = _x - camX;
        float lookY = (_y + 1.5f) - camY;
        float lookZ = _z - camZ;
        float lookDist = MathF.Sqrt(lookX * lookX + lookZ * lookZ);

        _game.SetCameraRotation3D(
            MathF.Atan2(lookY, lookDist) * 180f / MathF.PI,
            MathF.Atan2(lookX, lookZ) * 180f / MathF.PI,
            0f
        );
    }

    void HandleCameraInput(float dt)
    {
        if (_game.IsKeyPressed(Keys.Left))
            _cameraYaw += CameraRotSpeed * dt;
        if (_game.IsKeyPressed(Keys.Right))
            _cameraYaw -= CameraRotSpeed * dt;
        if (_game.IsKeyPressed(Keys.Up))
            _cameraPitch = Math.Clamp(_cameraPitch + CameraRotSpeed * dt, 5f, 80f);
        if (_game.IsKeyPressed(Keys.Down))
            _cameraPitch = Math.Clamp(_cameraPitch - CameraRotSpeed * dt, 5f, 80f);
        // Q/E to zoom in/out
        if (_game.IsKeyPressed(Keys.Q))
            _cameraDistance = Math.Clamp(_cameraDistance - ZoomSpeed * dt, MinCameraDistance, MaxCameraDistance);
        if (_game.IsKeyPressed(Keys.E))
            _cameraDistance = Math.Clamp(_cameraDistance + ZoomSpeed * dt, MinCameraDistance, MaxCameraDistance);
    }

    (float mx, float mz, bool moving, bool running) HandleMovementInput()
    {
        float yawRad = _cameraYaw * MathF.PI / 180f;
        float fwdX = MathF.Sin(yawRad);
        float fwdZ = MathF.Cos(yawRad);
        float rgtX = MathF.Cos(yawRad);
        float rgtZ = -MathF.Sin(yawRad);

        float moveX = 0f, moveZ = 0f;
        bool moving = false;

        if (_game.IsKeyPressed(Keys.W)) { moveX += fwdX; moveZ += fwdZ; moving = true; }
        if (_game.IsKeyPressed(Keys.S)) { moveX -= fwdX; moveZ -= fwdZ; moving = true; }
        if (_game.IsKeyPressed(Keys.A)) { moveX += rgtX; moveZ += rgtZ; moving = true; }
        if (_game.IsKeyPressed(Keys.D)) { moveX -= rgtX; moveZ -= rgtZ; moving = true; }

        float len = MathF.Sqrt(moveX * moveX + moveZ * moveZ);
        if (len > 0.001f) { moveX /= len; moveZ /= len; }

        bool running = _game.IsKeyPressed(Keys.LeftShift) || _game.IsKeyPressed(Keys.RightShift);
        return (moveX, moveZ, moving, running);
    }

    void SmoothRotateToward(float target, float dt)
    {
        float diff = WrapDegrees(target - _facing);
        if (MathF.Abs(diff) < TurnSpeed * dt)
            _facing = target;
        else
            _facing += MathF.Sign(diff) * TurnSpeed * dt;
    }

    static float WrapDegrees(float d)
    {
        d %= 360f;
        if (d > 180f) d -= 360f;
        if (d < -180f) d += 360f;
        return d;
    }
}
