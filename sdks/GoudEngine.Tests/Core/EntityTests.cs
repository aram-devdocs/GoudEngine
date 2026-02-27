using System;
using Xunit;
using GoudEngine.Core;

namespace GoudEngine.Tests.Core
{
    /// <summary>
    /// Unit tests for the Entity wrapper class.
    /// </summary>
    public class EntityTests
    {
        // ====================================================================
        // Construction Tests
        // ====================================================================

        [Fact]
        public void Spawn_CreatesValidEntity()
        {
            using var context = new GoudContext();
            var entity = context.Spawn();

            Assert.NotNull(entity);
            Assert.False(entity.Id.IsInvalid);
            Assert.True(entity.IsAlive);
            Assert.False(entity.IsDead);
            Assert.Equal(context, entity.Context);
        }

        [Fact]
        public void SpawnBatch_CreatesMultipleEntities()
        {
            using var context = new GoudContext();
            var entities = context.SpawnBatch(10);

            Assert.Equal(10, entities.Length);
            foreach (var entity in entities)
            {
                Assert.NotNull(entity);
                Assert.True(entity.IsAlive);
                Assert.Equal(context, entity.Context);
            }
        }

        [Fact]
        public void WrapEntity_CreatesEntityFromId()
        {
            using var context = new GoudContext();
            var entityId = context.SpawnEntity();
            var entity = context.WrapEntity(entityId);

            Assert.NotNull(entity);
            Assert.Equal(entityId, entity.Id);
            Assert.True(entity.IsAlive);
        }

        [Fact]
        public void WrapEntity_WithInvalidId_ThrowsException()
        {
            using var context = new GoudContext();
            Assert.Throws<ArgumentException>(() =>
                context.WrapEntity(GoudEntityId.Invalid));
        }

        // ====================================================================
        // Lifecycle Tests
        // ====================================================================

        [Fact]
        public void Despawn_RemovesEntity()
        {
            using var context = new GoudContext();
            var entity = context.Spawn();
            uint initialCount = context.EntityCount;

            bool despawned = entity.Despawn();

            Assert.True(despawned);
            Assert.True(entity.IsDead);
            Assert.False(entity.IsAlive);
            Assert.Equal(initialCount - 1, context.EntityCount);
        }

        [Fact]
        public void Despawn_OnDeadEntity_ReturnsFalse()
        {
            using var context = new GoudContext();
            var entity = context.Spawn();

            entity.Despawn();
            bool secondDespawn = entity.Despawn();

            Assert.False(secondDespawn);
            Assert.True(entity.IsDead);
        }

        [Fact]
        public void Despawn_AfterContextDisposed_ReturnsFalse()
        {
            Entity entity;
            {
                using var context = new GoudContext();
                entity = context.Spawn();
            } // Context disposed here

            bool despawned = entity.Despawn();
            Assert.False(despawned);
        }

        [Fact]
        public void DespawnBatch_RemovesMultipleEntities()
        {
            using var context = new GoudContext();
            var entities = context.SpawnBatch(5);

            uint despawned = context.DespawnBatch(entities);

            Assert.Equal(5u, despawned);
            foreach (var entity in entities)
            {
                Assert.True(entity.IsDead);
            }
        }

        // ====================================================================
        // Liveness Tests
        // ====================================================================

        [Fact]
        public void IsAlive_ForLiveEntity_ReturnsTrue()
        {
            using var context = new GoudContext();
            var entity = context.Spawn();

            Assert.True(entity.IsAlive);
            Assert.False(entity.IsDead);
        }

        [Fact]
        public void IsAlive_ForDeadEntity_ReturnsFalse()
        {
            using var context = new GoudContext();
            var entity = context.Spawn();
            entity.Despawn();

            Assert.False(entity.IsAlive);
            Assert.True(entity.IsDead);
        }

        [Fact]
        public void AreEntitiesAlive_ChecksMultipleEntities()
        {
            using var context = new GoudContext();
            var entities = context.SpawnBatch(3);
            entities[1].Despawn();

            var aliveStatus = context.AreEntitiesAlive(entities);

            Assert.Equal(3, aliveStatus.Length);
            Assert.True(aliveStatus[0]);
            Assert.False(aliveStatus[1]);
            Assert.True(aliveStatus[2]);
        }

        // ====================================================================
        // Equality Tests
        // ====================================================================

        [Fact]
        public void Equals_SameEntityId_ReturnsTrue()
        {
            using var context = new GoudContext();
            var entity1 = context.Spawn();
            var entity2 = context.WrapEntity(entity1.Id);

            Assert.Equal(entity1, entity2);
            Assert.True(entity1.Equals(entity2));
            Assert.True(entity1 == entity2);
            Assert.False(entity1 != entity2);
        }

        [Fact]
        public void Equals_DifferentEntityIds_ReturnsFalse()
        {
            using var context = new GoudContext();
            var entity1 = context.Spawn();
            var entity2 = context.Spawn();

            Assert.NotEqual(entity1, entity2);
            Assert.False(entity1.Equals(entity2));
            Assert.False(entity1 == entity2);
            Assert.True(entity1 != entity2);
        }

        [Fact]
        public void Equals_DifferentContexts_ReturnsFalse()
        {
            using var context1 = new GoudContext();
            using var context2 = new GoudContext();
            var entity1 = context1.Spawn();
            var entity2 = context2.Spawn();

            // Even if IDs happen to match, different contexts mean different entities
            Assert.NotEqual(entity1, entity2);
        }

        [Fact]
        public void Equals_WithNull_ReturnsFalse()
        {
            using var context = new GoudContext();
            var entity = context.Spawn();

            Assert.False(entity.Equals(null));
            Assert.False(entity == null);
            Assert.True(entity != null);
        }

        [Fact]
        public void GetHashCode_ConsistentForSameEntity()
        {
            using var context = new GoudContext();
            var entity = context.Spawn();

            int hash1 = entity.GetHashCode();
            int hash2 = entity.GetHashCode();

            Assert.Equal(hash1, hash2);
        }

        [Fact]
        public void GetHashCode_DifferentForDifferentEntities()
        {
            using var context = new GoudContext();
            var entity1 = context.Spawn();
            var entity2 = context.Spawn();

            int hash1 = entity1.GetHashCode();
            int hash2 = entity2.GetHashCode();

            Assert.NotEqual(hash1, hash2);
        }

        // ====================================================================
        // String Representation Tests
        // ====================================================================

        [Fact]
        public void ToString_ForAliveEntity_ContainsStatus()
        {
            using var context = new GoudContext();
            var entity = context.Spawn();

            string str = entity.ToString();

            Assert.Contains("Entity", str);
            Assert.Contains("Alive", str);
        }

        [Fact]
        public void ToString_ForDeadEntity_ContainsStatus()
        {
            using var context = new GoudContext();
            var entity = context.Spawn();
            entity.Despawn();

            string str = entity.ToString();

            Assert.Contains("Entity", str);
            Assert.Contains("Dead", str);
        }

        // ====================================================================
        // Component Operation Tests (Placeholders)
        // ====================================================================

        [Fact]
        public void AddComponent_ThrowsNotImplementedException()
        {
            using var context = new GoudContext();
            var entity = context.Spawn();

            Assert.Throws<NotImplementedException>(() =>
                entity.AddComponent(42));
        }

        [Fact]
        public void RemoveComponent_ThrowsNotImplementedException()
        {
            using var context = new GoudContext();
            var entity = context.Spawn();

            Assert.Throws<NotImplementedException>(() =>
                entity.RemoveComponent<int>());
        }

        [Fact]
        public void HasComponent_ThrowsNotImplementedException()
        {
            using var context = new GoudContext();
            var entity = context.Spawn();

            Assert.Throws<NotImplementedException>(() =>
                entity.HasComponent<int>());
        }

        [Fact]
        public void GetComponent_ThrowsNotImplementedException()
        {
            using var context = new GoudContext();
            var entity = context.Spawn();

            Assert.Throws<NotImplementedException>(() =>
                entity.GetComponent<int>());
        }

        [Fact]
        public void TryGetComponent_ThrowsNotImplementedException()
        {
            using var context = new GoudContext();
            var entity = context.Spawn();

            Assert.Throws<NotImplementedException>(() =>
                entity.TryGetComponent<int>(out _));
        }

        [Fact]
        public void ComponentOperations_OnDeadEntity_ThrowsInvalidOperationException()
        {
            using var context = new GoudContext();
            var entity = context.Spawn();
            entity.Despawn();

            // All component operations should throw InvalidOperationException
            // for dead entities (ThrowIfDead is called before NotImplementedException)
            Assert.Throws<InvalidOperationException>(() =>
                entity.AddComponent(42));
        }

        [Fact]
        public void TryGetComponent_OnDeadEntity_ReturnsFalse()
        {
            using var context = new GoudContext();
            var entity = context.Spawn();
            entity.Despawn();

            // TryGetComponent should return false for dead entities
            bool result = entity.TryGetComponent<int>(out var component);

            Assert.False(result);
            Assert.Equal(default, component);
        }

        // ====================================================================
        // Batch Operations Tests
        // ====================================================================

        [Fact]
        public void SpawnBatch_WithZeroCount_ThrowsException()
        {
            using var context = new GoudContext();
            Assert.Throws<ArgumentOutOfRangeException>(() =>
                context.SpawnBatch(0));
        }

        [Fact]
        public void SpawnBatch_WithNegativeCount_ThrowsException()
        {
            using var context = new GoudContext();
            Assert.Throws<ArgumentOutOfRangeException>(() =>
                context.SpawnBatch(-5));
        }

        [Fact]
        public void DespawnBatch_WithEmptyArray_ReturnsZero()
        {
            using var context = new GoudContext();
            uint despawned = context.DespawnBatch(Array.Empty<Entity>());

            Assert.Equal(0u, despawned);
        }

        [Fact]
        public void DespawnBatch_WithNull_ReturnsZero()
        {
            using var context = new GoudContext();
            uint despawned = context.DespawnBatch(null!);

            Assert.Equal(0u, despawned);
        }

        [Fact]
        public void AreEntitiesAlive_WithEmptyArray_ReturnsEmptyArray()
        {
            using var context = new GoudContext();
            var results = context.AreEntitiesAlive(Array.Empty<Entity>());

            Assert.Empty(results);
        }

        // ====================================================================
        // Integration Tests
        // ====================================================================

        [Fact]
        public void EntityCount_ReflectsSpawnAndDespawn()
        {
            using var context = new GoudContext();
            uint initialCount = context.EntityCount;

            var entity1 = context.Spawn();
            Assert.Equal(initialCount + 1, context.EntityCount);

            var entity2 = context.Spawn();
            Assert.Equal(initialCount + 2, context.EntityCount);

            entity1.Despawn();
            Assert.Equal(initialCount + 1, context.EntityCount);

            entity2.Despawn();
            Assert.Equal(initialCount, context.EntityCount);
        }

        [Fact]
        public void MultipleContexts_IndependentEntityLifecycles()
        {
            using var context1 = new GoudContext();
            using var context2 = new GoudContext();

            var entity1 = context1.Spawn();
            var entity2 = context2.Spawn();

            Assert.Equal(1u, context1.EntityCount);
            Assert.Equal(1u, context2.EntityCount);

            entity1.Despawn();
            Assert.Equal(0u, context1.EntityCount);
            Assert.Equal(1u, context2.EntityCount); // Context2 unaffected
        }
    }
}
