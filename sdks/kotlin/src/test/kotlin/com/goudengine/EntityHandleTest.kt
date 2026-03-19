package com.goudengine

import com.goudengine.core.EntityHandle
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class EntityHandleTest {
    @Test
    fun testIndex() {
        val handle = EntityHandle(42L)
        assertEquals(42, handle.index)
    }

    @Test
    fun testGeneration() {
        // generation is upper 32 bits
        val bits = (3L shl 32) or 7L
        val handle = EntityHandle(bits)
        assertEquals(7, handle.index)
        assertEquals(3, handle.generation)
    }

    @Test
    fun testPlaceholder() {
        val placeholder = EntityHandle.PLACEHOLDER
        assertTrue(placeholder.isPlaceholder)
        assertEquals(-1L, placeholder.id)
    }

    @Test
    fun testNotPlaceholder() {
        val handle = EntityHandle(42L)
        assertFalse(handle.isPlaceholder)
    }

    @Test
    fun testToString() {
        val bits = (2L shl 32) or 5L
        val handle = EntityHandle(bits)
        assertEquals("Entity(5v2)", handle.toString())
    }
}
