package com.goudengine

import com.goudengine.types.Rect
import com.goudengine.types.Vec2
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class RectTest {
    @Test
    fun testConstruction() {
        val r = Rect(10f, 20f, 100f, 50f)
        assertEquals(10f, r.x)
        assertEquals(20f, r.y)
        assertEquals(100f, r.width)
        assertEquals(50f, r.height)
    }

    @Test
    fun testContainsInside() {
        val r = Rect(0f, 0f, 100f, 100f)
        assertTrue(r.contains(Vec2(50f, 50f)))
    }

    @Test
    fun testContainsOnEdge() {
        val r = Rect(0f, 0f, 100f, 100f)
        assertTrue(r.contains(Vec2(0f, 0f)))
        assertTrue(r.contains(Vec2(100f, 100f)))
    }

    @Test
    fun testContainsOutside() {
        val r = Rect(0f, 0f, 100f, 100f)
        assertFalse(r.contains(Vec2(101f, 50f)))
        assertFalse(r.contains(Vec2(-1f, 50f)))
    }

    @Test
    fun testIntersectsOverlapping() {
        val a = Rect(0f, 0f, 100f, 100f)
        val b = Rect(50f, 50f, 100f, 100f)
        assertTrue(a.intersects(b))
        assertTrue(b.intersects(a))
    }

    @Test
    fun testIntersectsNoOverlap() {
        val a = Rect(0f, 0f, 50f, 50f)
        val b = Rect(100f, 100f, 50f, 50f)
        assertFalse(a.intersects(b))
        assertFalse(b.intersects(a))
    }

    @Test
    fun testDataClassEquality() {
        val a = Rect(1f, 2f, 3f, 4f)
        val b = Rect(1f, 2f, 3f, 4f)
        assertEquals(a, b)
    }
}
