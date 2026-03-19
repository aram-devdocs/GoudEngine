package com.goudengine

import org.junit.jupiter.api.Disabled
import kotlin.test.Test
import kotlin.test.assertNotNull

class Transform2DTest {

    @Test
    @Disabled("Requires native library (JNI)")
    fun testDefault() {
        val t = com.goudengine.components.Transform2D.default()
        assertNotNull(t)
    }

    @Test
    @Disabled("Requires native library (JNI)")
    fun testFromPosition() {
        val t = com.goudengine.components.Transform2D.fromPosition(100f, 200f)
        assertNotNull(t)
    }

    @Test
    @Disabled("Requires native library (JNI)")
    fun testFromScale() {
        val t = com.goudengine.components.Transform2D.fromScale(2f, 2f)
        assertNotNull(t)
    }

    @Test
    @Disabled("Requires native library (JNI)")
    fun testFromRotation() {
        val t = com.goudengine.components.Transform2D.fromRotation(1.57f)
        assertNotNull(t)
    }

    @Test
    @Disabled("Requires native library (JNI)")
    fun testNew() {
        val t = com.goudengine.components.Transform2D.new(100f, 200f, 0f, 1f, 1f)
        assertNotNull(t)
    }
}
