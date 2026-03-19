package com.goudengine

import com.goudengine.core.GoudGame
import kotlin.test.Test
import kotlin.test.assertTrue

class CoroutinesTest {
    @Test
    fun testLoadTextureAsyncExists() {
        // Check via reflection that the suspend extension function exists
        val extensions = Class.forName("com.goudengine.core.CoroutinesKt")
        val methods = extensions.declaredMethods.map { it.name }
        assertTrue(
            methods.contains("loadTextureAsync"),
            "loadTextureAsync suspend extension should exist"
        )
    }

    @Test
    fun testLoadFontAsyncExists() {
        val extensions = Class.forName("com.goudengine.core.CoroutinesKt")
        val methods = extensions.declaredMethods.map { it.name }
        assertTrue(
            methods.contains("loadFontAsync"),
            "loadFontAsync suspend extension should exist"
        )
    }
}
