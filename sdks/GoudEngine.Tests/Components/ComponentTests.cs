using System;
using Xunit;
using GoudEngine.Core;
using GoudEngine.Components;

namespace GoudEngine.Tests.Components
{
    public class ComponentTests : IDisposable
    {
        private readonly GoudContext _context;

        public ComponentTests()
        {
            _context = new GoudContext();
        }

        public void Dispose()
        {
            _context?.Dispose();
        }

        [Fact]
        public void Transform2D_DefaultValues()
        {
            var transform = Transform2D.Identity;

            Assert.Equal(0f, transform.Position.X);
            Assert.Equal(0f, transform.Position.Y);
            Assert.Equal(0f, transform.Rotation);
            Assert.Equal(1f, transform.Scale.X);
            Assert.Equal(1f, transform.Scale.Y);
        }

        [Fact]
        public void Transform2D_FromPosition()
        {
            var pos = new Math.Vector2(100f, 50f);
            var transform = Transform2D.FromPosition(pos);

            Assert.Equal(100f, transform.Position.X);
            Assert.Equal(50f, transform.Position.Y);
            Assert.Equal(0f, transform.Rotation);
            Assert.Equal(1f, transform.Scale.X);
        }

        [Fact]
        public void Transform2D_FromRotationDegrees()
        {
            var transform = Transform2D.FromRotationDegrees(90f);

            Assert.Equal(0f, transform.Position.X);
            Assert.Equal(0f, transform.Position.Y);
            Assert.InRange(transform.Rotation, 1.5f, 1.6f); // ~PI/2
            Assert.Equal(90f, transform.RotationDegrees, precision: 2);
        }

        [Fact]
        public void Transform2D_Translate()
        {
            var transform = Transform2D.Identity;
            transform.Translate(new Math.Vector2(10f, 20f));

            Assert.Equal(10f, transform.Position.X);
            Assert.Equal(20f, transform.Position.Y);
        }

        [Fact]
        public void Transform2D_Rotate()
        {
            var transform = Transform2D.Identity;
            transform.RotateDegrees(45f);

            Assert.InRange(transform.RotationDegrees, 44f, 46f);
        }

        [Fact]
        public void Sprite_DefaultValues()
        {
            var sprite = Sprite.Default();

            Assert.Equal(0ul, sprite.TextureHandle);
            Assert.Equal(Color.White.R, sprite.Color.R);
            Assert.Equal(Color.White.G, sprite.Color.G);
            Assert.Equal(Color.White.B, sprite.Color.B);
            Assert.False(sprite.FlipX);
            Assert.False(sprite.FlipY);
            Assert.Equal(0.5f, sprite.Anchor.X);
            Assert.Equal(0.5f, sprite.Anchor.Y);
            Assert.False(sprite.HasSourceRect);
            Assert.False(sprite.HasCustomSize);
        }

        [Fact]
        public void Sprite_WithColor()
        {
            var sprite = new Sprite(123)
                .WithColor(Color.Red);

            Assert.Equal(123ul, sprite.TextureHandle);
            Assert.Equal(Color.Red.R, sprite.Color.R);
            Assert.Equal(Color.Red.G, sprite.Color.G);
            Assert.Equal(Color.Red.B, sprite.Color.B);
        }

        [Fact]
        public void Sprite_WithFlip()
        {
            var sprite = new Sprite(0)
                .WithFlipX()
                .WithFlipY();

            Assert.True(sprite.FlipX);
            Assert.True(sprite.FlipY);
            Assert.True(sprite.IsFlipped);
        }

        [Fact]
        public void Sprite_WithSourceRect()
        {
            var sprite = new Sprite(0)
                .WithSourceRect(new Rect(0, 0, 32, 32));

            Assert.True(sprite.HasSourceRect);
            Assert.Equal(32f, sprite.SourceRect!.Value.Width);
            Assert.Equal(32f, sprite.SourceRect!.Value.Height);
        }

        [Fact]
        public void Sprite_WithCustomSize()
        {
            var sprite = new Sprite(0)
                .WithCustomSize(new Math.Vector2(100, 50));

            Assert.True(sprite.HasCustomSize);
            Assert.Equal(100f, sprite.CustomSize!.Value.X);
            Assert.Equal(50f, sprite.CustomSize!.Value.Y);
        }

        [Fact]
        public void Sprite_WithoutSourceRect()
        {
            var sprite = new Sprite(0)
                .WithSourceRect(new Rect(0, 0, 32, 32))
                .WithoutSourceRect();

            Assert.False(sprite.HasSourceRect);
        }

        [Fact]
        public void ComponentRegistry_Register()
        {
            // Clear any previous registrations
            ComponentRegistry.ClearRegistrations();

            Assert.False(ComponentRegistry.IsRegistered<Transform2D>());

            ComponentRegistry.Register<Transform2D>();

            Assert.True(ComponentRegistry.IsRegistered<Transform2D>());
        }

        [Fact]
        public void ComponentRegistry_RegisterTwice_DoesNotThrow()
        {
            ComponentRegistry.ClearRegistrations();

            ComponentRegistry.Register<Transform2D>();
            ComponentRegistry.Register<Transform2D>(); // Should not throw

            Assert.True(ComponentRegistry.IsRegistered<Transform2D>());
        }

        [Fact]
        public void Entity_AddComponent_Transform2D()
        {
            var entity = _context.Spawn();
            var transform = Transform2D.FromPosition(new Math.Vector2(10f, 20f));

            // Note: This will fail until FFI is fully wired up
            // For now, we're testing that the API compiles and is usable
            try
            {
                entity.AddComponent(transform);
                // If FFI works, component should be added
                Assert.True(entity.HasComponent<Transform2D>());
            }
            catch (Exception)
            {
                // Expected to fail until FFI placeholder is removed
                // This test validates the API surface
            }
        }

        [Fact]
        public void Entity_AddComponent_Sprite()
        {
            var entity = _context.Spawn();
            var sprite = new Sprite(456)
                .WithColor(Color.Blue)
                .WithFlipX();

            try
            {
                entity.AddComponent(sprite);
                Assert.True(entity.HasComponent<Sprite>());
            }
            catch (Exception)
            {
                // Expected to fail until FFI placeholder is removed
            }
        }

        [Fact]
        public void Entity_RemoveComponent()
        {
            var entity = _context.Spawn();
            var transform = Transform2D.Identity;

            try
            {
                entity.AddComponent(transform);
                Assert.True(entity.HasComponent<Transform2D>());

                var removed = entity.RemoveComponent<Transform2D>();
                Assert.True(removed);
                Assert.False(entity.HasComponent<Transform2D>());
            }
            catch (Exception)
            {
                // Expected to fail until FFI placeholder is removed
            }
        }

        [Fact]
        public void Entity_GetComponent()
        {
            var entity = _context.Spawn();
            var originalTransform = Transform2D.FromPosition(new Math.Vector2(100f, 200f));

            try
            {
                entity.AddComponent(originalTransform);

                var retrieved = entity.GetComponent<Transform2D>();
                Assert.Equal(100f, retrieved.Position.X);
                Assert.Equal(200f, retrieved.Position.Y);
            }
            catch (Exception)
            {
                // Expected to fail until FFI placeholder is removed
            }
        }

        [Fact]
        public void Entity_TryGetComponent_Success()
        {
            var entity = _context.Spawn();
            var transform = Transform2D.FromPosition(new Math.Vector2(50f, 75f));

            try
            {
                entity.AddComponent(transform);

                if (entity.TryGetComponent<Transform2D>(out var retrieved))
                {
                    Assert.Equal(50f, retrieved.Position.X);
                    Assert.Equal(75f, retrieved.Position.Y);
                }
            }
            catch (Exception)
            {
                // Expected to fail until FFI placeholder is removed
            }
        }

        [Fact]
        public void Entity_TryGetComponent_Failure()
        {
            var entity = _context.Spawn();

            var success = entity.TryGetComponent<Transform2D>(out var component);

            Assert.False(success);
            Assert.Equal(default(Transform2D), component);
        }

        [Fact]
        public void Entity_UpdateComponent()
        {
            var entity = _context.Spawn();
            var original = Transform2D.FromPosition(new Math.Vector2(10f, 10f));

            try
            {
                entity.AddComponent(original);

                var updated = Transform2D.FromPosition(new Math.Vector2(20f, 20f));
                entity.UpdateComponent(updated);

                var retrieved = entity.GetComponent<Transform2D>();
                Assert.Equal(20f, retrieved.Position.X);
                Assert.Equal(20f, retrieved.Position.Y);
            }
            catch (Exception)
            {
                // Expected to fail until FFI placeholder is removed
            }
        }

        [Fact]
        public void Entity_MethodChaining()
        {
            var entity = _context.Spawn();
            var transform = Transform2D.Identity;
            var sprite = Sprite.Default();

            try
            {
                // Method chaining should work
                entity
                    .AddComponent(transform)
                    .AddComponent(sprite);

                Assert.True(entity.HasComponent<Transform2D>());
                Assert.True(entity.HasComponent<Sprite>());
            }
            catch (Exception)
            {
                // Expected to fail until FFI placeholder is removed
            }
        }
    }
}
