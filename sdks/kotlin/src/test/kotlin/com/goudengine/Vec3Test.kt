package com.goudengine

import com.goudengine.types.Vec3
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class Vec3Test {
    @Test
    fun testConstruction() {
        val v = Vec3(1f, 2f, 3f)
        assertEquals(1f, v.x)
        assertEquals(2f, v.y)
        assertEquals(3f, v.z)
    }

    @Test
    fun testZeroFactory() {
        val v = Vec3.zero()
        assertEquals(0f, v.x)
        assertEquals(0f, v.y)
        assertEquals(0f, v.z)
    }

    @Test
    fun testOneFactory() {
        val v = Vec3.one()
        assertEquals(1f, v.x)
        assertEquals(1f, v.y)
        assertEquals(1f, v.z)
    }

    @Test
    fun testUpFactory() {
        val v = Vec3.up()
        assertEquals(0f, v.x)
        assertEquals(1f, v.y)
        assertEquals(0f, v.z)
    }

    @Test
    fun testDataClassEquality() {
        val a = Vec3(1f, 2f, 3f)
        val b = Vec3(1f, 2f, 3f)
        assertEquals(a, b)
        assertTrue(a == b)
    }

    @Test
    fun testDataClassCopy() {
        val a = Vec3(1f, 2f, 3f)
        val b = a.copy(z = 5f)
        assertEquals(1f, b.x)
        assertEquals(2f, b.y)
        assertEquals(5f, b.z)
    }
}
