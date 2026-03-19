package com.goudengine

import com.goudengine.input.Key
import com.goudengine.input.MouseButton
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull

class EnumTest {
    @Test
    fun testKeySpaceValue() {
        assertEquals(32, Key.Space.value)
    }

    @Test
    fun testKeyFromValue() {
        val key = Key.fromValue(32)
        assertNotNull(key)
        assertEquals(Key.Space, key)
    }

    @Test
    fun testKeyFromValueUnknown() {
        val key = Key.fromValue(99999)
        assertNull(key)
    }

    @Test
    fun testMouseButtonLeft() {
        assertEquals(0, MouseButton.Left.value)
    }

    @Test
    fun testMouseButtonRight() {
        assertEquals(1, MouseButton.Right.value)
    }

    @Test
    fun testMouseButtonFromValue() {
        val btn = MouseButton.fromValue(0)
        assertNotNull(btn)
        assertEquals(MouseButton.Left, btn)
    }
}
