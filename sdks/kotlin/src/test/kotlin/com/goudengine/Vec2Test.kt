package com.goudengine

import com.goudengine.types.Vec2
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class Vec2Test {
    @Test
    fun testAdd() {
        val a = Vec2(1f, 2f)
        val b = Vec2(3f, 4f)
        val r = a.add(b)
        assertEquals(4f, r.x)
        assertEquals(6f, r.y)
    }

    @Test
    fun testSub() {
        val a = Vec2(5f, 7f)
        val b = Vec2(2f, 3f)
        val r = a.sub(b)
        assertEquals(3f, r.x)
        assertEquals(4f, r.y)
    }

    @Test
    fun testScale() {
        val v = Vec2(2f, 3f)
        val r = v.scale(2f)
        assertEquals(4f, r.x)
        assertEquals(6f, r.y)
    }

    @Test
    fun testLength() {
        val v = Vec2(3f, 4f)
        assertEquals(5f, v.length(), 0.001f)
    }

    @Test
    fun testNormalize() {
        val v = Vec2(3f, 4f)
        val n = v.normalize()
        assertEquals(1f, n.length(), 0.001f)
        assertEquals(0.6f, n.x, 0.001f)
        assertEquals(0.8f, n.y, 0.001f)
    }

    @Test
    fun testNormalizeZero() {
        val v = Vec2.zero()
        val n = v.normalize()
        assertEquals(0f, n.x)
        assertEquals(0f, n.y)
    }

    @Test
    fun testDot() {
        val a = Vec2(1f, 0f)
        val b = Vec2(0f, 1f)
        assertEquals(0f, a.dot(b))

        val c = Vec2(2f, 3f)
        val d = Vec2(4f, 5f)
        assertEquals(23f, c.dot(d))
    }

    @Test
    fun testDistance() {
        val a = Vec2(0f, 0f)
        val b = Vec2(3f, 4f)
        assertEquals(5f, a.distance(b), 0.001f)
    }

    @Test
    fun testLerp() {
        val a = Vec2(0f, 0f)
        val b = Vec2(10f, 20f)
        val mid = a.lerp(b, 0.5f)
        assertEquals(5f, mid.x, 0.001f)
        assertEquals(10f, mid.y, 0.001f)
    }

    @Test
    fun testFactories() {
        assertEquals(Vec2(0f, 0f), Vec2.zero())
        assertEquals(Vec2(1f, 1f), Vec2.one())
        assertEquals(Vec2(0f, 1f), Vec2.up())
        assertEquals(Vec2(0f, -1f), Vec2.down())
        assertEquals(Vec2(-1f, 0f), Vec2.left())
        assertEquals(Vec2(1f, 0f), Vec2.right())
    }

    @Test
    fun testDataClassEquality() {
        val a = Vec2(1f, 2f)
        val b = Vec2(1f, 2f)
        assertEquals(a, b)
        assertTrue(a == b)
    }
}
