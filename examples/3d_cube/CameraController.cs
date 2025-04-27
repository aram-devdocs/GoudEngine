using System;
using GoudEngine;

public class CameraController
{
    private GoudGame game;

    // Camera position
    private float positionX;
    private float positionY;
    private float positionZ;

    // Camera rotation (in degrees)
    private float yaw; // Rotation around Y axis (left/right)
    private float pitch; // Rotation around X axis (up/down)
    private float roll; // Rotation around Z axis (tilt)

    // Camera settings
    private float moveSpeed;
    private float rotationSpeed;
    private float mouseSensitivity;

    // Direction vectors
    private float[] forward;
    private float[] right;
    private float[] up;

    // Mouse input
    private double lastMouseX;
    private double lastMouseY;
    private bool firstMouse;

    public CameraController(GoudGame game, float startX = 0, float startY = 10, float startZ = -15)
    {
        this.game = game;

        // Initialize position
        positionX = startX;
        positionY = startY;
        positionZ = startZ;

        // Initialize rotation
        yaw = 0;
        pitch = -30; // Start looking slightly downward
        roll = 0;

        // Set speeds
        moveSpeed = 0.5f;
        rotationSpeed = 2.0f;
        mouseSensitivity = 0.1f;

        // Initialize direction vectors
        forward = new float[3] { 0, 0, 1 };
        right = new float[3] { 1, 0, 0 };
        up = new float[3] { 0, 1, 0 };

        // Initialize mouse input
        lastMouseX = 0;
        lastMouseY = 0;
        firstMouse = true;

        // Apply initial position and rotation
        UpdateCamera();
    }

    private void UpdateVectors()
    {
        // Convert angles to radians
        float yawRad = yaw * (float)Math.PI / 180.0f;
        float pitchRad = pitch * (float)Math.PI / 180.0f;

        // Calculate new forward vector
        forward[0] = (float)(Math.Sin(yawRad) * Math.Cos(pitchRad));
        forward[1] = (float)Math.Sin(pitchRad);
        forward[2] = (float)(Math.Cos(yawRad) * Math.Cos(pitchRad));

        // Calculate right vector
        right[0] = (float)Math.Cos(yawRad);
        right[1] = 0;
        right[2] = (float)-Math.Sin(yawRad);

        // Calculate up vector (cross product of right and forward)
        up[0] = right[1] * forward[2] - right[2] * forward[1];
        up[1] = right[2] * forward[0] - right[0] * forward[2];
        up[2] = right[0] * forward[1] - right[1] * forward[0];
    }

    private void UpdateCamera()
    {
        // Set camera position directly through the 3D position method
        game.SetCameraPosition3D(positionX, positionY, positionZ);

        // Set rotation directly using the new rotation method
        game.SetCameraRotation(pitch, yaw, roll);
    }

    public void Update(float deltaTime)
    {
        bool moved = false;

        // Handle keyboard input for movement
        HandleKeyboardInput(ref moved, deltaTime);

        // Handle mouse input for rotation (could be enabled when needed)
        // HandleMouseInput(ref moved);

        if (moved)
        {
            UpdateVectors();
            UpdateCamera();
        }
    }

    private void HandleKeyboardInput(ref bool moved, float deltaTime)
    {
        float moveDelta = moveSpeed * deltaTime * 10.0f; // Scale by 10 for smoother movement

        // Forward/Backward movement using Q/E
        if (game.IsKeyPressed(81)) // Q key - move forward
        {
            positionX -= forward[0] * moveDelta;
            positionY -= forward[1] * moveDelta;
            positionZ -= forward[2] * moveDelta;
            moved = true;
        }
        if (game.IsKeyPressed(69)) // E key - move backward
        {
            positionX += forward[0] * moveDelta;
            positionY += forward[1] * moveDelta;
            positionZ += forward[2] * moveDelta;
            moved = true;
        }

        // Strafe movement using A/D
        if (game.IsKeyPressed(68)) // D key - strafe right
        {
            positionX -= right[0] * moveDelta;
            positionY -= right[1] * moveDelta;
            positionZ -= right[2] * moveDelta;
            moved = true;
        }
        if (game.IsKeyPressed(65)) // A key - strafe left
        {
            positionX += right[0] * moveDelta;
            positionY += right[1] * moveDelta;
            positionZ += right[2] * moveDelta;
            moved = true;
        }

        // Up/Down movement using W/S
        if (game.IsKeyPressed(87)) // W key - move up
        {
            positionX -= up[0] * moveDelta;
            positionY -= up[1] * moveDelta;
            positionZ -= up[2] * moveDelta;
            moved = true;
        }
        if (game.IsKeyPressed(83)) // S key - move down
        {
            positionX += up[0] * moveDelta;
            positionY += up[1] * moveDelta;
            positionZ += up[2] * moveDelta;
            moved = true;
        }

        // Handle rotation using arrow keys
        if (game.IsKeyPressed(262)) // Right arrow
        {
            yaw -= rotationSpeed * deltaTime * 100.0f;
            moved = true;
        }
        if (game.IsKeyPressed(263)) // Left arrow
        {
            yaw += rotationSpeed * deltaTime * 100.0f;
            moved = true;
        }
        if (game.IsKeyPressed(264)) // Down arrow
        {
            pitch -= rotationSpeed * deltaTime * 100.0f;
            pitch = Math.Clamp(pitch, -89.0f, 89.0f); // Prevent flipping
            moved = true;
        }
        if (game.IsKeyPressed(265)) // Up arrow
        {
            pitch += rotationSpeed * deltaTime * 100.0f;
            pitch = Math.Clamp(pitch, -89.0f, 89.0f); // Prevent flipping
            moved = true;
        }

        // Roll control using [ and ]
        if (game.IsKeyPressed(91)) // [ key - roll left
        {
            roll += rotationSpeed * deltaTime * 100.0f;
            moved = true;
        }
        if (game.IsKeyPressed(93)) // ] key - roll right
        {
            roll -= rotationSpeed * deltaTime * 100.0f;
            moved = true;
        }
    }

    private void HandleMouseInput(ref bool moved)
    {
        // Get current mouse position
        var mousePos = game.GetMousePosition();

        if (firstMouse)
        {
            lastMouseX = mousePos.x;
            lastMouseY = mousePos.y;
            firstMouse = false;
            return;
        }

        // Calculate mouse movement
        float deltaX = (float)(mousePos.x - lastMouseX);
        float deltaY = (float)(lastMouseY - mousePos.y); // Reversed to avoid inverted camera

        lastMouseX = mousePos.x;
        lastMouseY = mousePos.y;

        // Apply mouse sensitivity
        deltaX *= mouseSensitivity;
        deltaY *= mouseSensitivity;

        // Update camera rotation
        yaw += deltaX;
        pitch += deltaY;

        // Clamp pitch to prevent flipping
        pitch = Math.Clamp(pitch, -89.0f, 89.0f);

        moved = true;
    }

    // Accessor methods
    public float GetPositionX() => positionX;

    public float GetPositionY() => positionY;

    public float GetPositionZ() => positionZ;

    public float GetYaw() => yaw;

    public float GetPitch() => pitch;

    public float GetRoll() => roll;

    // Update from external source
    public void SetPosition(float x, float y, float z)
    {
        positionX = x;
        positionY = y;
        positionZ = z;
        UpdateCamera();
    }

    public void SetRotation(float newPitch, float newYaw, float newRoll)
    {
        pitch = newPitch;
        yaw = newYaw;
        roll = newRoll;
        UpdateVectors();
        UpdateCamera();
    }
}
