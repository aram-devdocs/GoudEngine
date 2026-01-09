// GameManager.cs
// Game logic for Goud Jumper using immediate-mode rendering

using System;
using System.Collections.Generic;
using GoudEngine.Input;
using GoudEngine.Math;
using GoudEngine;

public class GameManager
{
    private readonly GoudGame game;
    private SimpleAnimator? animator;

    private bool isGoingLeft = false;

    private float playerX = 100;
    private float playerY = 100;
    private float playerVelocityY = 0;
    private const float gravity = 500f;      // Pixels per second^2
    private const float jumpStrength = 350f; // Pixels per second
    private const float moveSpeed = 200f;    // Pixels per second
    
    private readonly float groundY;          // Ground level
    private readonly float screenWidth;
    private readonly float screenHeight;
    
    private PlayerState currentState = PlayerState.Idle;

    public GameManager(GoudGame game)
    {
        this.game = game ?? throw new ArgumentNullException(nameof(game));
        
        // Screen dimensions (from GoudGame constructor)
        screenWidth = 30 * 32;  // 960
        screenHeight = 20 * 32; // 640
        groundY = screenHeight - 64; // Ground is 64 pixels from bottom
    }

    public void Initialize()
    {
        Console.WriteLine("Initializing Goud Jumper...");
        
        // Create animation configurations
        var animations = new Dictionary<string, AnimationConfig>
        {
            { PlayerState.Idle.ToString(), new AnimationConfig(
                "assets/1 Pink_Monster/Pink_Monster_Idle_4.png", 32, 32, 4, 0.2f, true) },
            { PlayerState.Walking.ToString(), new AnimationConfig(
                "assets/1 Pink_Monster/Pink_Monster_Walk_6.png", 32, 32, 6, 0.1f, true) },
            { PlayerState.Jumping.ToString(), new AnimationConfig(
                "assets/1 Pink_Monster/Pink_Monster_Jump_8.png", 32, 32, 8, 0.1f, false) },
            { PlayerState.Run.ToString(), new AnimationConfig(
                "assets/1 Pink_Monster/Pink_Monster_Run_6.png", 32, 32, 6, 0.08f, true) },
            { PlayerState.Attack1.ToString(), new AnimationConfig(
                "assets/1 Pink_Monster/Pink_Monster_Attack1_4.png", 32, 32, 4, 0.1f, false) },
            { PlayerState.Attack2.ToString(), new AnimationConfig(
                "assets/1 Pink_Monster/Pink_Monster_Attack2_6.png", 32, 32, 6, 0.1f, false) },
            { PlayerState.Death.ToString(), new AnimationConfig(
                "assets/1 Pink_Monster/Pink_Monster_Death_8.png", 32, 32, 8, 0.1f, false) },
            { PlayerState.Hurt.ToString(), new AnimationConfig(
                "assets/1 Pink_Monster/Pink_Monster_Hurt_4.png", 32, 32, 4, 0.1f, false) },
        };
        
        animator = new SimpleAnimator(game, animations);
        animator.SetAnimation(PlayerState.Idle.ToString());
        
        // Set initial player position
        playerX = screenWidth / 4;
        playerY = groundY;
        
        Console.WriteLine("Goud Jumper initialized!");
    }

    public void Start() 
    {
        // Nothing special to do on start
    }

    public void Update(float deltaTime)
    {
        UpdatePlayerPosition(deltaTime);
        animator?.Update(deltaTime);
    }
    
    public void Draw()
    {
        // Draw simple ground
        DrawGround();
        
        // Draw simple background platforms
        DrawPlatforms();
        
        // Draw player
        DrawPlayer();
    }
    
    private void DrawGround()
    {
        // Draw ground as a series of colored quads
        for (float x = 0; x < screenWidth; x += 32)
        {
            // Ground tile
            game.DrawQuad(x + 16, groundY + 32, 32, 64, new Color(0.4f, 0.3f, 0.2f, 1f));
            // Grass on top
            game.DrawQuad(x + 16, groundY + 8, 32, 16, new Color(0.2f, 0.6f, 0.2f, 1f));
        }
    }
    
    private void DrawPlatforms()
    {
        // Draw some floating platforms
        DrawPlatform(200, groundY - 100, 3);
        DrawPlatform(400, groundY - 180, 4);
        DrawPlatform(650, groundY - 120, 3);
        DrawPlatform(100, groundY - 250, 2);
        DrawPlatform(500, groundY - 300, 5);
    }
    
    private void DrawPlatform(float x, float y, int tileCount)
    {
        for (int i = 0; i < tileCount; i++)
        {
            float tileX = x + i * 32 + 16;
            game.DrawQuad(tileX, y + 16, 32, 32, new Color(0.5f, 0.4f, 0.3f, 1f));
            // Grass on top
            game.DrawQuad(tileX, y + 4, 32, 8, new Color(0.3f, 0.7f, 0.3f, 1f));
        }
    }
    
    private void DrawPlayer()
    {
        if (animator == null) return;
        
        // Get current frame info
        var (textureId, sourceRect) = animator.GetCurrentFrame();
        
        // Player sprite size (scaled 2x from 32x32 to 64x64)
        float spriteWidth = 64;
        float spriteHeight = 64;
        
        // Flip horizontally if going left (negative width)
        float displayWidth = isGoingLeft ? -spriteWidth : spriteWidth;
        
        // Draw using DrawSprite with source rectangle for sprite sheet animation
        game.DrawSprite(textureId, 
            playerX + spriteWidth / 2, 
            playerY + spriteHeight / 2, 
            displayWidth, 
            spriteHeight,
            sourceRect,  // Use the source rectangle for sprite sheet frame
            0f);
    }

    private void UpdatePlayerPosition(float deltaTime)
    {
        // Apply gravity
        playerVelocityY += gravity * deltaTime;
        playerY += playerVelocityY * deltaTime;

        // Ground collision
        if (playerY > groundY)
        {
            playerY = groundY;
            playerVelocityY = 0;
            
            // Return to idle if we were jumping
            if (currentState == PlayerState.Jumping)
            {
                SetState(PlayerState.Idle);
            }
        }
        
        // Keep player in bounds
        playerX = System.Math.Clamp(playerX, 0, screenWidth - 64);
    }

    public void HandleInput()
    {
        bool isMoving = false;
        
        // Quit on Escape
        if (game.IsKeyPressed(Keys.Escape))
        {
            game.Close();
            return;
        }

        // Jump
        if (game.IsKeyPressed(Keys.Space) && playerY >= groundY - 1)
        {
            SetState(PlayerState.Jumping);
            playerVelocityY = -jumpStrength;
            isMoving = true;
        }
        // Move left
        else if (game.IsKeyPressed(Keys.A))
        {
            if (currentState != PlayerState.Jumping)
                SetState(PlayerState.Walking);
            playerX -= moveSpeed * game.DeltaTime;
            isMoving = true;
            isGoingLeft = true;
        }
        // Move right
        else if (game.IsKeyPressed(Keys.D))
        {
            if (currentState != PlayerState.Jumping)
                SetState(PlayerState.Walking);
            playerX += moveSpeed * game.DeltaTime;
            isMoving = true;
            isGoingLeft = false;
        }
        // Attack 1
        else if (game.IsKeyJustPressed(Keys.J))
        {
            SetState(PlayerState.Attack1);
            isMoving = true;
        }
        // Attack 2
        else if (game.IsKeyJustPressed(Keys.K))
        {
            SetState(PlayerState.Attack2);
            isMoving = true;
        }

        // Return to idle if not moving and on ground
        if (!isMoving && playerY >= groundY - 1 && currentState != PlayerState.Jumping)
        {
            SetState(PlayerState.Idle);
        }
    }
    
    private void SetState(PlayerState newState)
    {
        if (currentState != newState)
        {
            currentState = newState;
            animator?.SetAnimation(newState.ToString());
        }
    }
}

// Simple animation configuration
public class AnimationConfig
{
    public string TexturePath { get; }
    public int FrameWidth { get; }
    public int FrameHeight { get; }
    public int FrameCount { get; }
    public float FrameDuration { get; }
    public bool Loop { get; }
    
    public AnimationConfig(string texturePath, int frameWidth, int frameHeight, 
        int frameCount, float frameDuration, bool loop)
    {
        TexturePath = texturePath;
        FrameWidth = frameWidth;
        FrameHeight = frameHeight;
        FrameCount = frameCount;
        FrameDuration = frameDuration;
        Loop = loop;
    }
}

// Simple animator for sprite sheet animations using RectF (normalized UV coordinates)
public class SimpleAnimator
{
    private readonly GoudGame game;
    private readonly Dictionary<string, AnimationData> animations;
    private string currentAnimation = "";
    private int currentFrame = 0;
    private float frameTimer = 0f;
    
    public SimpleAnimator(GoudGame game, Dictionary<string, AnimationConfig> configs)
    {
        this.game = game;
        animations = new Dictionary<string, AnimationData>();
        
        foreach (var kvp in configs)
        {
            var config = kvp.Value;
            ulong textureId = game.LoadTexture(config.TexturePath);
            
            // Generate frame rectangles in normalized UV coordinates
            // For horizontal sprite sheets: each frame is 1/frameCount wide
            var frames = new List<RectF>();
            float frameWidthUV = 1.0f / config.FrameCount;
            
            for (int i = 0; i < config.FrameCount; i++)
            {
                frames.Add(new RectF(
                    i * frameWidthUV, 0f,    // X, Y in UV space
                    frameWidthUV, 1.0f));    // Width, Height in UV space
            }
            
            animations[kvp.Key] = new AnimationData(
                textureId, frames, config.FrameDuration, config.Loop);
        }
    }
    
    public void SetAnimation(string name)
    {
        if (name != currentAnimation && animations.ContainsKey(name))
        {
            currentAnimation = name;
            currentFrame = 0;
            frameTimer = 0f;
        }
    }
    
    public void Update(float deltaTime)
    {
        if (string.IsNullOrEmpty(currentAnimation) || !animations.ContainsKey(currentAnimation))
            return;
            
        var anim = animations[currentAnimation];
        frameTimer += deltaTime;
        
        if (frameTimer >= anim.FrameDuration)
        {
            frameTimer = 0f;
            currentFrame++;
            
            if (currentFrame >= anim.Frames.Count)
            {
                currentFrame = anim.Loop ? 0 : anim.Frames.Count - 1;
            }
        }
    }
    
    public (ulong textureId, RectF frame) GetCurrentFrame()
    {
        if (string.IsNullOrEmpty(currentAnimation) || !animations.ContainsKey(currentAnimation))
            return (0, RectF.Full);
            
        var anim = animations[currentAnimation];
        return (anim.TextureId, anim.Frames[currentFrame]);
    }
}

// Animation data storage using RectF for normalized UV coordinates
public class AnimationData
{
    public ulong TextureId { get; }
    public List<RectF> Frames { get; }
    public float FrameDuration { get; }
    public bool Loop { get; }
    
    public AnimationData(ulong textureId, List<RectF> frames, float frameDuration, bool loop)
    {
        TextureId = textureId;
        Frames = frames;
        FrameDuration = frameDuration;
        Loop = loop;
    }
}

// Player states
public enum PlayerState
{
    Walking,
    Jumping,
    Ducking,
    Hurt,
    Attack1,
    Attack2,
    Climb,
    Death,
    Idle,
    Push,
    Run,
    Throw,
    WalkAttack,
    Custom1,
    Custom2,
    Custom3
}
