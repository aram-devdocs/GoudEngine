package com.goudengine

import com.goudengine.input.Key
import com.goudengine.input.MouseButton
import com.goudengine.core.RendererType
import com.goudengine.core.OverlayCorner
import com.goudengine.core.DebuggerStepKind
import com.goudengine.animation.PlaybackMode
import com.goudengine.physics.BodyType
import com.goudengine.physics.ShapeType
import com.goudengine.physics.PhysicsBackend2D
import com.goudengine.core.RenderBackendKind
import com.goudengine.core.WindowBackendKind
import com.goudengine.animation.EasingType
import com.goudengine.network.NetworkProtocol
import com.goudengine.animation.TransitionType
import com.goudengine.core.TextAlignment
import com.goudengine.core.TextDirection
import com.goudengine.core.BlendMode
import com.goudengine.animation.EventPayloadType
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class AllEnumsTest {

    private inline fun <reified E> assertRoundTrip(
        entries: Array<E>,
        getValue: (E) -> Int,
        fromValue: (Int) -> E?
    ) where E : Enum<E> {
        assertTrue(entries.isNotEmpty(), "${E::class.simpleName} should have entries")
        for (entry in entries) {
            val value = getValue(entry)
            val recovered = fromValue(value)
            assertNotNull(recovered, "${E::class.simpleName}.fromValue($value) returned null")
            assertEquals(entry, recovered, "Round-trip failed for ${E::class.simpleName}.$entry")
        }
    }

    @Test
    fun testKeyRoundTrip() {
        assertRoundTrip(Key.entries.toTypedArray(), { it.value }, Key::fromValue)
    }

    @Test
    fun testMouseButtonRoundTrip() {
        assertRoundTrip(MouseButton.entries.toTypedArray(), { it.value }, MouseButton::fromValue)
    }

    @Test
    fun testRendererTypeRoundTrip() {
        assertRoundTrip(RendererType.entries.toTypedArray(), { it.value }, RendererType::fromValue)
    }

    @Test
    fun testOverlayCornerRoundTrip() {
        assertRoundTrip(OverlayCorner.entries.toTypedArray(), { it.value }, OverlayCorner::fromValue)
    }

    @Test
    fun testDebuggerStepKindRoundTrip() {
        assertRoundTrip(DebuggerStepKind.entries.toTypedArray(), { it.value }, DebuggerStepKind::fromValue)
    }

    @Test
    fun testPlaybackModeRoundTrip() {
        assertRoundTrip(PlaybackMode.entries.toTypedArray(), { it.value }, PlaybackMode::fromValue)
    }

    @Test
    fun testBodyTypeRoundTrip() {
        assertRoundTrip(BodyType.entries.toTypedArray(), { it.value }, BodyType::fromValue)
    }

    @Test
    fun testShapeTypeRoundTrip() {
        assertRoundTrip(ShapeType.entries.toTypedArray(), { it.value }, ShapeType::fromValue)
    }

    @Test
    fun testPhysicsBackend2DRoundTrip() {
        assertRoundTrip(PhysicsBackend2D.entries.toTypedArray(), { it.value }, PhysicsBackend2D::fromValue)
    }

    @Test
    fun testRenderBackendKindRoundTrip() {
        assertRoundTrip(RenderBackendKind.entries.toTypedArray(), { it.value }, RenderBackendKind::fromValue)
    }

    @Test
    fun testWindowBackendKindRoundTrip() {
        assertRoundTrip(WindowBackendKind.entries.toTypedArray(), { it.value }, WindowBackendKind::fromValue)
    }

    @Test
    fun testEasingTypeRoundTrip() {
        assertRoundTrip(EasingType.entries.toTypedArray(), { it.value }, EasingType::fromValue)
    }

    @Test
    fun testNetworkProtocolRoundTrip() {
        assertRoundTrip(NetworkProtocol.entries.toTypedArray(), { it.value }, NetworkProtocol::fromValue)
    }

    @Test
    fun testTransitionTypeRoundTrip() {
        assertRoundTrip(TransitionType.entries.toTypedArray(), { it.value }, TransitionType::fromValue)
    }

    @Test
    fun testTextAlignmentRoundTrip() {
        assertRoundTrip(TextAlignment.entries.toTypedArray(), { it.value }, TextAlignment::fromValue)
    }

    @Test
    fun testTextDirectionRoundTrip() {
        assertRoundTrip(TextDirection.entries.toTypedArray(), { it.value }, TextDirection::fromValue)
    }

    @Test
    fun testBlendModeRoundTrip() {
        assertRoundTrip(BlendMode.entries.toTypedArray(), { it.value }, BlendMode::fromValue)
    }

    @Test
    fun testEventPayloadTypeRoundTrip() {
        assertRoundTrip(EventPayloadType.entries.toTypedArray(), { it.value }, EventPayloadType::fromValue)
    }

    @Test
    fun testSpecificKeyValues() {
        assertEquals(32, Key.Space.value)
        assertEquals(256, Key.Escape.value)
        assertEquals(257, Key.Enter.value)
    }

    @Test
    fun testSpecificMouseButtonValues() {
        assertEquals(0, MouseButton.Left.value)
        assertEquals(1, MouseButton.Right.value)
        assertEquals(2, MouseButton.Middle.value)
    }

    @Test
    fun testAllEnumsNonEmpty() {
        assertTrue(Key.entries.isNotEmpty())
        assertTrue(MouseButton.entries.isNotEmpty())
        assertTrue(RendererType.entries.isNotEmpty())
        assertTrue(OverlayCorner.entries.isNotEmpty())
        assertTrue(DebuggerStepKind.entries.isNotEmpty())
        assertTrue(PlaybackMode.entries.isNotEmpty())
        assertTrue(BodyType.entries.isNotEmpty())
        assertTrue(ShapeType.entries.isNotEmpty())
        assertTrue(PhysicsBackend2D.entries.isNotEmpty())
        assertTrue(RenderBackendKind.entries.isNotEmpty())
        assertTrue(WindowBackendKind.entries.isNotEmpty())
        assertTrue(EasingType.entries.isNotEmpty())
        assertTrue(NetworkProtocol.entries.isNotEmpty())
        assertTrue(TransitionType.entries.isNotEmpty())
        assertTrue(TextAlignment.entries.isNotEmpty())
        assertTrue(TextDirection.entries.isNotEmpty())
        assertTrue(BlendMode.entries.isNotEmpty())
        assertTrue(EventPayloadType.entries.isNotEmpty())
    }
}
