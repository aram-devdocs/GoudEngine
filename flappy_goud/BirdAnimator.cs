// BirdAnimator.cs

using System;
using System.Collections.Generic;

using CsBindgen;
public class BirdAnimator
{
    private readonly GoudGame game;
    // private readonly should be a dict of paths of the textures and their respective ids
    private Dictionary<string, uint> spritePaths;

    private int currentFrame;
    private float animationTime;
    private readonly float frameDuration;
    private float initialX;
    private float initialY;

    public uint SpriteId { get; private set; }
    // public uint TextureId { get; private set; }

    public BirdAnimator(GoudGame game, float x, float y, float frameDuration = 0.1f)
    {
        this.game = game;
        this.frameDuration = frameDuration;
        animationTime = 0;
        currentFrame = 0;
        initialX = x;
        initialY = y;
        spritePaths = new Dictionary<string, uint>();



    }

    public void Initialize()
    {

        spritePaths = new Dictionary<string, uint>
        {
            { "assets/sprites/bluebird-downflap.png", game.CreateTexture("assets/sprites/bluebird-downflap.png") },
            { "assets/sprites/bluebird-midflap.png", game.CreateTexture("assets/sprites/bluebird-midflap.png") },
            { "assets/sprites/bluebird-upflap.png", game.CreateTexture("assets/sprites/bluebird-upflap.png") }
        };


        // Set up the initial sprite
        SpriteId = game.AddSprite(new SpriteCreateDto { x = initialX, y = initialY, texture_id = spritePaths["assets/sprites/bluebird-downflap.png"] });

        Console.WriteLine($"Debug bird_sprite_id: {SpriteId}");



    }

    public void Update(float deltaTime, float x, float y, float rotation)
    {
        animationTime += deltaTime;
        if (animationTime >= frameDuration)
        {
            currentFrame = (currentFrame + 1) % spritePaths.Count;
            animationTime = 0;

        }

        var textureKey = new List<string>(spritePaths.Keys)[currentFrame];
        game.UpdateSprite(SpriteId, new SpriteUpdateDto
        {
            x = x,
            y = y,
            rotation = rotation,
            texture_id = spritePaths[textureKey]
        });




    }

    public void Reset()
    {
        currentFrame = 0;
        animationTime = 0;
        game.UpdateSprite(SpriteId, new SpriteUpdateDto { x = initialX, y = initialY, texture_id = spritePaths["assets/sprites/bluebird-downflap.png"] });
    }
}