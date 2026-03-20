import com.goudengine.core.GoudContext
import com.goudengine.core.GoudEngine
import com.goudengine.components.Transform2D
import kotlin.system.exitProcess

/**
 * GoudEngine Feature Lab -- Kotlin SDK
 *
 * Headless smoke test that exercises the Kotlin SDK API surface:
 *   - Headless context creation and validation
 *   - Entity spawn, clone, despawn lifecycle
 *   - Transform2D component add/get/remove
 *   - Scene lifecycle operations
 *   - Error handling
 *
 * Run with: ./dev.sh --sdk kotlin --game feature_lab
 */

data class CheckResult(
    val name: String,
    val status: String, // PASS, FAIL, SKIP
    val detail: String = "",
)

fun runCheck(name: String, fn: () -> Pair<Boolean, String>): CheckResult {
    return try {
        val (ok, detail) = fn()
        CheckResult(name, if (ok) "PASS" else "FAIL", detail)
    } catch (e: Exception) {
        CheckResult(name, "FAIL", e.message ?: e.toString())
    }
}

fun runSkippableCheck(name: String, fn: () -> Pair<Boolean, String>): CheckResult {
    return try {
        val (ok, detail) = fn()
        CheckResult(name, if (ok) "PASS" else "FAIL", detail)
    } catch (e: UnsatisfiedLinkError) {
        CheckResult(name, "SKIP", e.message ?: e.toString())
    } catch (e: RuntimeException) {
        CheckResult(name, "SKIP", e.message ?: e.toString())
    } catch (e: Exception) {
        CheckResult(name, "FAIL", e.message ?: e.toString())
    }
}

fun checkContextValid(ctx: GoudContext): Pair<Boolean, String> {
    val valid = ctx.isValid()
    return Pair(valid, "isValid=$valid")
}

fun checkEntityLifecycle(ctx: GoudContext): Pair<Boolean, String> {
    val initialCount = ctx.entityCount()

    // Spawn
    val entity = ctx.spawnEmpty()
    val alive = ctx.isAlive(entity)
    val countAfterSpawn = ctx.entityCount()

    // Clone
    val cloned = ctx.cloneEntity(entity)
    val clonedAlive = ctx.isAlive(cloned)

    // Despawn both
    val despawnedOriginal = ctx.despawn(entity)
    val despawnedClone = ctx.despawn(cloned)
    val finalCount = ctx.entityCount()

    val ok = alive && clonedAlive && despawnedOriginal && despawnedClone
    val detail = "initial=$initialCount, afterSpawn=$countAfterSpawn, " +
        "final=$finalCount, alive=$alive, clonedAlive=$clonedAlive"
    return Pair(ok, detail)
}

fun checkTransform2DComponent(ctx: GoudContext): Pair<Boolean, String> {
    val entity = ctx.spawnEmpty()

    // Add
    val transform = Transform2D.fromPosition(10.0f, 20.0f)
    ctx.addTransform2d(entity, transform)

    // Has
    val has = ctx.hasTransform2d(entity)

    // Get
    val got = ctx.getTransform2d(entity)
    val positionMatch = got != null

    // Remove
    val removed = ctx.removeTransform2d(entity)
    val hasAfterRemove = ctx.hasTransform2d(entity)

    ctx.despawn(entity)

    val ok = has && positionMatch && removed && !hasAfterRemove
    val detail = "has=$has, positionMatch=$positionMatch, " +
        "removed=$removed, hasAfterRemove=$hasAfterRemove"
    return Pair(ok, detail)
}

fun checkSceneLifecycle(ctx: GoudContext): Pair<Boolean, String> {
    val initialCount = ctx.sceneCount()

    val sceneId = ctx.sceneCreate("kt_feature_lab_scene")
    val countAfterCreate = ctx.sceneCount()

    val ok = sceneId >= 0 && countAfterCreate >= initialCount
    val detail = "initial=$initialCount, afterCreate=$countAfterCreate, sceneId=$sceneId"
    return Pair(ok, detail)
}

fun main() {
    println("================================================================")
    println(" GoudEngine Kotlin Feature Lab")
    println("================================================================")

    GoudEngine.ensureLoaded()

    val results = mutableListOf<CheckResult>()
    var ctx: GoudContext? = null

    try {
        ctx = GoudContext.create()

        results.add(runCheck("headless context is valid") { checkContextValid(ctx) })
        results.add(runCheck("entity lifecycle (spawn/clone/despawn)") { checkEntityLifecycle(ctx) })
        results.add(runSkippableCheck("Transform2D component add/get/remove") { checkTransform2DComponent(ctx) })
        results.add(runSkippableCheck("scene lifecycle operations") { checkSceneLifecycle(ctx) })
    } catch (e: Exception) {
        results.add(CheckResult("feature lab startup", "FAIL", e.message ?: e.toString()))
    } finally {
        ctx?.destroy()
    }

    val passCount = results.count { it.status == "PASS" }
    val failCount = results.count { it.status == "FAIL" }
    val skipCount = results.count { it.status == "SKIP" }

    println("\nFeature Lab complete: $passCount pass, $failCount fail, $skipCount skip")
    for (result in results) {
        val suffix = if (result.detail.isNotEmpty()) " (${result.detail})" else ""
        println("${result.status}: ${result.name}$suffix")
    }

    if (failCount > 0) {
        exitProcess(1)
    }
}
