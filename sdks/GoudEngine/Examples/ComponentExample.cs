using System;
using GoudEngine.Core;
using GoudEngine.Components;
using GoudEngine.Math;

namespace GoudEngine.Examples
{
    /// <summary>
    /// Example demonstrating component usage in the GoudEngine ECS system.
    /// This shows how to create entities and attach components using the builder pattern.
    /// </summary>
    public static class ComponentExample
    {
        public static void Run()
        {
            Console.WriteLine("=== GoudEngine Component Example ===\n");

            using var context = new GoudContext();

            // Example 1: Create a simple sprite entity
            CreateSimpleSprite(context);

            // Example 2: Create a sprite with transform
            CreateSpriteWithTransform(context);

            // Example 3: Method chaining
            CreateWithMethodChaining(context);

            // Example 4: Update components
            UpdateComponentsExample(context);

            // Example 5: Query components
            QueryComponentsExample(context);

            Console.WriteLine("\nAll examples completed!");
        }

        static void CreateSimpleSprite(GoudContext context)
        {
            Console.WriteLine("Example 1: Create Simple Sprite");

            var entity = context.Spawn();
            Console.WriteLine($"  Spawned entity: {entity}");

            // Create a sprite component
            var sprite = new Sprite(textureHandle: 0)
                .WithColor(Components.Color.Red)
                .WithFlipX();

            // Add it to the entity
            try
            {
                entity.AddComponent(sprite);
                Console.WriteLine($"  Added sprite component: {sprite}");
            }
            catch (NotImplementedException)
            {
                Console.WriteLine("  Note: Component FFI not yet fully implemented");
            }

            Console.WriteLine();
        }

        static void CreateSpriteWithTransform(GoudContext context)
        {
            Console.WriteLine("Example 2: Create Sprite with Transform");

            var entity = context.Spawn();

            // Create transform at position (100, 50)
            var transform = Transform2D.FromPosition(new Vector2(100f, 50f));
            transform.RotateDegrees(45f); // Rotate 45 degrees
            transform.Scale = new Vector2(2f, 2f); // Scale 2x

            Console.WriteLine($"  Transform: {transform}");

            // Create sprite with custom anchor
            var sprite = new Sprite(123)
                .WithAnchor(0.5f, 1.0f) // Bottom-center anchor
                .WithCustomSize(new Vector2(64f, 64f));

            Console.WriteLine($"  Sprite: {sprite}");

            try
            {
                entity.AddComponent(transform);
                entity.AddComponent(sprite);
                Console.WriteLine($"  Added components to entity: {entity}");
            }
            catch (NotImplementedException)
            {
                Console.WriteLine("  Note: Component FFI not yet fully implemented");
            }

            Console.WriteLine();
        }

        static void CreateWithMethodChaining(GoudContext context)
        {
            Console.WriteLine("Example 3: Method Chaining");

            try
            {
                var entity = context.Spawn()
                    .AddComponent(Transform2D.FromPosition(new Vector2(200f, 100f)))
                    .AddComponent(new Sprite(456)
                        .WithColor(Components.Color.Blue)
                        .WithFlipY()
                        .WithSourceRect(new Rect(0, 0, 32, 32)));

                Console.WriteLine($"  Created entity with chained components: {entity}");
            }
            catch (NotImplementedException)
            {
                Console.WriteLine("  Note: Component FFI not yet fully implemented");
            }

            Console.WriteLine();
        }

        static void UpdateComponentsExample(GoudContext context)
        {
            Console.WriteLine("Example 4: Update Components");

            var entity = context.Spawn();

            try
            {
                // Add initial transform
                var transform = Transform2D.FromPosition(new Vector2(0f, 0f));
                entity.AddComponent(transform);

                // Update the transform
                transform.Translate(new Vector2(50f, 50f));
                transform.RotateDegrees(90f);
                entity.UpdateComponent(transform);

                Console.WriteLine($"  Updated transform: {transform}");

                // Get and modify sprite
                if (entity.TryGetComponent<Sprite>(out var sprite))
                {
                    sprite.Color = Components.Color.Green;
                    entity.UpdateComponent(sprite);
                    Console.WriteLine($"  Updated sprite color to green");
                }
            }
            catch (NotImplementedException)
            {
                Console.WriteLine("  Note: Component FFI not yet fully implemented");
            }

            Console.WriteLine();
        }

        static void QueryComponentsExample(GoudContext context)
        {
            Console.WriteLine("Example 5: Query Components");

            var entity = context.Spawn();

            try
            {
                // Add components
                entity.AddComponent(Transform2D.Identity);
                entity.AddComponent(Sprite.Default());

                // Check if entity has components
                Console.WriteLine($"  Has Transform2D: {entity.HasComponent<Transform2D>()}");
                Console.WriteLine($"  Has Sprite: {entity.HasComponent<Sprite>()}");

                // Try to get component
                if (entity.TryGetComponent<Transform2D>(out var transform))
                {
                    Console.WriteLine($"  Retrieved transform: {transform}");
                }

                // Remove a component
                var removed = entity.RemoveComponent<Sprite>();
                Console.WriteLine($"  Removed sprite: {removed}");
                Console.WriteLine($"  Has Sprite after removal: {entity.HasComponent<Sprite>()}");
            }
            catch (NotImplementedException)
            {
                Console.WriteLine("  Note: Component FFI not yet fully implemented");
            }

            Console.WriteLine();
        }
    }
}
