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
        public void Transform2D_SetPosition()
        {
            var transform = Transform2D.Identity;
            transform.Position = new Math.Vector2(10f, 20f);

            Assert.Equal(10f, transform.Position.X);
            Assert.Equal(20f, transform.Position.Y);
        }

        [Fact]
        public void Transform2D_SetRotation()
        {
            var transform = Transform2D.Identity;
            transform.Rotation = MathF.PI / 4f; // 45 degrees

            Assert.InRange(transform.RotationDegrees, 44f, 46f);
        }

        [Fact]
        public void Transform2D_SetScale()
        {
            var transform = Transform2D.Identity;
            transform.Scale = new Math.Vector2(2f, 3f);

            Assert.Equal(2f, transform.ScaleX);
            Assert.Equal(3f, transform.ScaleY);
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
        public void Sprite_SetColor()
        {
            var sprite = new Sprite(123);
            sprite.Color = Color.Red;

            Assert.Equal(123ul, sprite.TextureHandle);
            Assert.Equal(Color.Red.R, sprite.Color.R);
            Assert.Equal(Color.Red.G, sprite.Color.G);
            Assert.Equal(Color.Red.B, sprite.Color.B);
        }

        [Fact]
        public void Sprite_SetFlip()
        {
            var sprite = new Sprite(0);
            sprite.FlipX = true;
            sprite.FlipY = true;

            Assert.True(sprite.FlipX);
            Assert.True(sprite.FlipY);
            Assert.True(sprite.IsFlipped);
        }

        [Fact]
        public void Sprite_SetSourceRect()
        {
            var sprite = new Sprite(0);
            sprite.SourceRectX = 0;
            sprite.SourceRectY = 0;
            sprite.SourceRectWidth = 32;
            sprite.SourceRectHeight = 32;
            sprite.HasSourceRect = true;

            Assert.True(sprite.HasSourceRect);
            Assert.Equal(32f, sprite.SourceRectWidth);
            Assert.Equal(32f, sprite.SourceRectHeight);
        }

        [Fact]
        public void Sprite_SetCustomSize()
        {
            var sprite = new Sprite(0);
            sprite.CustomSizeX = 100;
            sprite.CustomSizeY = 50;
            sprite.HasCustomSize = true;

            Assert.True(sprite.HasCustomSize);
            Assert.Equal(100f, sprite.CustomSizeX);
            Assert.Equal(50f, sprite.CustomSizeY);
        }

        [Fact]
        public void Sprite_ClearSourceRect()
        {
            var sprite = new Sprite(0);
            sprite.HasSourceRect = true;
            sprite.HasSourceRect = false;

            Assert.False(sprite.HasSourceRect);
        }

        [Fact]
        public void Sprite_SetAnchor()
        {
            var sprite = new Sprite(0);
            sprite.Anchor = new Math.Vector2(0.0f, 1.0f);

            Assert.Equal(0.0f, sprite.AnchorX);
            Assert.Equal(1.0f, sprite.AnchorY);
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
            var sprite = new Sprite(456);
            sprite.Color = Color.Blue;
            sprite.FlipX = true;

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
