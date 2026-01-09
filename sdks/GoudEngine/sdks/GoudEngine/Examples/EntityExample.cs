using System;
using GoudEngine.Core;

namespace GoudEngine.Examples
{
    /// <summary>
    /// Example demonstrating the Entity wrapper API for the GoudEngine ECS system.
    /// This example shows how to spawn, despawn, and work with entities using
    /// the object-oriented Entity wrapper instead of raw IDs.
    /// </summary>
    public static class EntityExample
    {
        public static void Run()
        {
            Console.WriteLine("=== GoudEngine Entity Wrapper Example ===\n");

            // Create a new GoudEngine context
            using var context = new GoudContext();
            Console.WriteLine($"Created context: {context}\n");

            // ====================================================================
            // Example 1: Spawn a single entity using extension method
            // ====================================================================
            Console.WriteLine("Example 1: Spawn single entity");
            var player = context.Spawn();
            Console.WriteLine($"  Spawned: {player}");
            Console.WriteLine($"  Entity ID: {player.Id}");
            Console.WriteLine($"  Is Alive: {player.IsAlive}");
            Console.WriteLine();

            // ====================================================================
            // Example 2: Spawn multiple entities in a batch
            // ====================================================================
            Console.WriteLine("Example 2: Spawn entity batch");
            var enemies = context.SpawnBatch(5);
            Console.WriteLine($"  Spawned {enemies.Length} enemies:");
            foreach (var enemy in enemies)
            {
                Console.WriteLine($"    - {enemy}");
            }
            Console.WriteLine();

            // ====================================================================
            // Example 3: Check entity liveness
            // ====================================================================
            Console.WriteLine("Example 3: Check entity liveness");
            Console.WriteLine($"  Player is alive: {player.IsAlive}");
            Console.WriteLine($"  Player is dead: {player.IsDead}");

            // Check multiple entities at once
            var aliveStatus = context.AreEntitiesAlive(player, enemies[0], enemies[1]);
            Console.WriteLine($"  Batch liveness check: [{string.Join(", ", aliveStatus)}]");
            Console.WriteLine();

            // ====================================================================
            // Example 4: Despawn a single entity
            // ====================================================================
            Console.WriteLine("Example 4: Despawn single entity");
            bool despawned = enemies[0].Despawn();
            Console.WriteLine($"  Despawned enemy: {despawned}");
            Console.WriteLine($"  Enemy is now dead: {enemies[0].IsDead}");

            // Trying to despawn again returns false
            bool despawnedAgain = enemies[0].Despawn();
            Console.WriteLine($"  Despawning again: {despawnedAgain}");
            Console.WriteLine();

            // ====================================================================
            // Example 5: Despawn multiple entities in a batch
            // ====================================================================
            Console.WriteLine("Example 5: Despawn entity batch");
            uint batchDespawned = context.DespawnBatch(enemies[1], enemies[2], enemies[3]);
            Console.WriteLine($"  Batch despawned {batchDespawned} entities");
            Console.WriteLine();

            // ====================================================================
            // Example 6: Entity equality and hashing
            // ====================================================================
            Console.WriteLine("Example 6: Entity equality");
            var playerCopy = context.WrapEntity(player.Id);
            Console.WriteLine($"  player == playerCopy: {player == playerCopy}");
            Console.WriteLine($"  player.Equals(playerCopy): {player.Equals(playerCopy)}");
            Console.WriteLine($"  player != enemies[4]: {player != enemies[4]}");
            Console.WriteLine();

            // ====================================================================
            // Example 7: Entity context and total count
            // ====================================================================
            Console.WriteLine("Example 7: Entity statistics");
            Console.WriteLine($"  Total entities in context: {context.EntityCount}");
            Console.WriteLine($"  Player context: {player.Context == context}");
            Console.WriteLine();

            // ====================================================================
            // Example 8: Despawn all remaining entities
            // ====================================================================
            Console.WriteLine("Example 8: Cleanup");
            player.Despawn();
            enemies[4].Despawn(); // Only remaining enemy
            Console.WriteLine($"  Final entity count: {context.EntityCount}");
            Console.WriteLine();

            Console.WriteLine("Example complete!");
        }

        /// <summary>
        /// Example showing error handling with dead entities.
        /// </summary>
        public static void RunErrorHandlingExample()
        {
            Console.WriteLine("\n=== Entity Error Handling Example ===\n");

            using var context = new GoudContext();
            var entity = context.Spawn();

            Console.WriteLine($"Entity spawned: {entity}");

            // Despawn the entity
            entity.Despawn();
            Console.WriteLine($"Entity despawned: {entity}");

            // Trying to operate on dead entity will throw (once component ops are implemented)
            try
            {
                // This will throw NotImplementedException for now (Step 6.2.4)
                entity.HasComponent<int>();
            }
            catch (NotImplementedException)
            {
                Console.WriteLine("Component operations not yet implemented (Step 6.2.4)");
            }
            catch (InvalidOperationException ex)
            {
                Console.WriteLine($"Error: {ex.Message}");
            }

            Console.WriteLine("\nError handling example complete!");
        }
    }
}
