package com.goudengine

import com.goudengine.core.GoudException
import com.goudengine.core.GoudContextException
import com.goudengine.core.GoudResourceException
import com.goudengine.core.GoudGraphicsException
import com.goudengine.core.RecoveryClass
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNotNull

class ErrorsTest {
    @Test
    fun testRecoveryClassValues() {
        assertEquals(0, RecoveryClass.Recoverable.value)
        assertEquals(1, RecoveryClass.Fatal.value)
        assertEquals(2, RecoveryClass.Degraded.value)
    }

    @Test
    fun testRecoveryClassFromValue() {
        assertEquals(RecoveryClass.Recoverable, RecoveryClass.fromValue(0))
        assertEquals(RecoveryClass.Fatal, RecoveryClass.fromValue(1))
        assertEquals(RecoveryClass.Degraded, RecoveryClass.fromValue(2))
    }

    @Test
    fun testRecoveryClassFromValueUnknownFallsBackToRecoverable() {
        assertEquals(RecoveryClass.Recoverable, RecoveryClass.fromValue(999))
    }

    @Test
    fun testGoudExceptionProperties() {
        val ex = GoudException(
            errorCode = 100,
            message = "test error",
            category = "Test",
            subsystem = "unit",
            operation = "test_op",
            recovery = RecoveryClass.Fatal,
            recoveryHint = "restart"
        )
        assertEquals(100, ex.errorCode)
        assertEquals("test error", ex.message)
        assertEquals("Test", ex.category)
        assertEquals("unit", ex.subsystem)
        assertEquals("test_op", ex.operation)
        assertEquals(RecoveryClass.Fatal, ex.recovery)
        assertEquals("restart", ex.recoveryHint)
    }

    @Test
    fun testGoudExceptionIsException() {
        val ex = GoudException(
            errorCode = 1,
            message = "test",
            category = "Test",
            subsystem = "unit",
            operation = "op",
            recovery = RecoveryClass.Recoverable,
            recoveryHint = ""
        )
        assertIs<Exception>(ex)
    }

    @Test
    fun testContextExceptionInheritsGoudException() {
        val ex = GoudContextException(
            errorCode = 200,
            message = "context error",
            subsystem = "context",
            operation = "create",
            recovery = RecoveryClass.Fatal,
            recoveryHint = "check config"
        )
        assertIs<GoudException>(ex)
        assertEquals("Context", ex.category)
    }

    @Test
    fun testResourceExceptionInheritsGoudException() {
        val ex = GoudResourceException(
            errorCode = 300,
            message = "resource error",
            subsystem = "texture",
            operation = "load",
            recovery = RecoveryClass.Recoverable,
            recoveryHint = "check path"
        )
        assertIs<GoudException>(ex)
        assertEquals("Resource", ex.category)
    }

    @Test
    fun testGraphicsExceptionCategory() {
        val ex = GoudGraphicsException(
            errorCode = 400,
            message = "gl error",
            subsystem = "renderer",
            operation = "draw",
            recovery = RecoveryClass.Degraded,
            recoveryHint = "fallback renderer"
        )
        assertEquals("Graphics", ex.category)
        assertEquals(RecoveryClass.Degraded, ex.recovery)
    }
}
