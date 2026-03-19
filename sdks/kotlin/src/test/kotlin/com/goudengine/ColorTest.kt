package com.goudengine

import com.goudengine.types.Color
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotEquals

class ColorTest {
    @Test
    fun testWhiteFactory() {
        val c = Color.white()
        assertEquals(1f, c.r)
        assertEquals(1f, c.g)
        assertEquals(1f, c.b)
        assertEquals(1f, c.a)
    }

    @Test
    fun testBlackFactory() {
        val c = Color.black()
        assertEquals(0f, c.r)
        assertEquals(0f, c.g)
        assertEquals(0f, c.b)
        assertEquals(1f, c.a)
    }

    @Test
    fun testRgbFactory() {
        val c = Color.rgb(0.5f, 0.6f, 0.7f)
        assertEquals(0.5f, c.r)
        assertEquals(0.6f, c.g)
        assertEquals(0.7f, c.b)
        assertEquals(1f, c.a)
    }

    @Test
    fun testFromHex() {
        val c = Color.fromHex(0xFF0000)
        assertEquals(1f, c.r, 0.01f)
        assertEquals(0f, c.g, 0.01f)
        assertEquals(0f, c.b, 0.01f)
    }

    @Test
    fun testFromU8() {
        val c = Color.fromU8(255, 128, 0, 255)
        assertEquals(1f, c.r, 0.01f)
        assertEquals(128f / 255f, c.g, 0.01f)
        assertEquals(0f, c.b, 0.01f)
        assertEquals(1f, c.a, 0.01f)
    }

    @Test
    fun testWithAlpha() {
        val c = Color.white().withAlpha(0.5f)
        assertEquals(1f, c.r)
        assertEquals(1f, c.g)
        assertEquals(1f, c.b)
        assertEquals(0.5f, c.a)
    }

    @Test
    fun testLerp() {
        val a = Color.black()
        val b = Color.white()
        val mid = a.lerp(b, 0.5f)
        assertEquals(0.5f, mid.r, 0.01f)
        assertEquals(0.5f, mid.g, 0.01f)
        assertEquals(0.5f, mid.b, 0.01f)
    }

    @Test
    fun testDataClassEquality() {
        val a = Color(1f, 0f, 0f, 1f)
        val b = Color(1f, 0f, 0f, 1f)
        val c = Color(0f, 1f, 0f, 1f)
        assertEquals(a, b)
        assertNotEquals(a, c)
    }
}
