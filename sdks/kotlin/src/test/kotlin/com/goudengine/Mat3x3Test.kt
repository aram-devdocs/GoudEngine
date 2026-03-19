package com.goudengine

import com.goudengine.internal.Mat3x3
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull

class Mat3x3Test {
    @Test
    fun testDefaultConstruction() {
        val m = Mat3x3()
        assertNotNull(m)
    }

    @Test
    fun testIdentityConstruction() {
        val identity = floatArrayOf(1f, 0f, 0f, 0f, 1f, 0f, 0f, 0f, 1f)
        val m = Mat3x3(identity)
        assertNotNull(m.m)
    }

    @Test
    fun testArrayStorage() {
        val values = floatArrayOf(1f, 0f, 0f, 0f, 1f, 0f, 0f, 0f, 1f)
        val m = Mat3x3(values)
        // The m array should hold 9 elements (3x3 matrix)
        assertEquals(9, m.m.size)
    }
}
