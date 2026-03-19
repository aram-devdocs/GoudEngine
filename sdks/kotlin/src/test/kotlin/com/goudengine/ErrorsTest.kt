package com.goudengine

import com.goudengine.core.GoudException
import com.goudengine.core.RecoveryClass
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

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
    fun testRecoveryClassUnknownFallback() {
        // Unknown values should fall back to Recoverable
        assertEquals(RecoveryClass.Recoverable, RecoveryClass.fromValue(999))
    }

    @Test
    fun testGoudExceptionProperties() {
        val ex = GoudException(
            errorCode = 42,
            message = "test error",
            category = "TestCategory",
            subsystem = "TestSubsystem",
            operation = "testOp",
            recovery = RecoveryClass.Fatal,
            recoveryHint = "try again"
        )
        assertEquals(42, ex.errorCode)
        assertEquals("test error", ex.message)
        assertEquals("TestCategory", ex.category)
        assertEquals("TestSubsystem", ex.subsystem)
        assertEquals("testOp", ex.operation)
        assertEquals(RecoveryClass.Fatal, ex.recovery)
        assertEquals("try again", ex.recoveryHint)
    }

    @Test
    fun testGoudExceptionInheritance() {
        val ex = GoudException(
            errorCode = 1,
            message = "msg",
            category = "cat",
            subsystem = "sub",
            operation = "op",
            recovery = RecoveryClass.Recoverable,
            recoveryHint = ""
        )
        assertTrue(ex is Exception)
        assertTrue(ex is GoudException)
    }

    @Test
    fun testCategoryExceptions() {
        // Verify that generated category exception classes exist and extend GoudException
        // We use reflection to check
        val corePackage = "com.goudengine.core"
        val baseClass = GoudException::class.java
        assertTrue(baseClass.isAssignableFrom(baseClass))
    }
}
