package com.goudengine

import kotlin.test.Test
import kotlin.test.assertTrue

/**
 * Verifies that coroutine extension functions compile correctly.
 * Actual JNI-based coroutine execution requires the native library
 * and is not tested here.
 */
class CoroutinesTest {
    @Test
    fun testCoroutineExtensionsExist() {
        // Verify the suspend extension functions are accessible at compile time.
        // We import them to ensure they exist and compile.
        val cls = Class.forName("com.goudengine.core.CoroutinesKt")
        val methods = cls.declaredMethods.map { it.name }
        assertTrue("loadTextureAsync" in methods, "loadTextureAsync extension should exist")
        assertTrue("loadFontAsync" in methods, "loadFontAsync extension should exist")
    }
}
