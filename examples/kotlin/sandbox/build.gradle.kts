plugins {
    kotlin("jvm") version "1.9.22"
    application
}

group = "com.goudengine.examples"
version = "0.1.0"

repositories {
    mavenCentral()
}

dependencies {
    implementation(files("../../../sdks/kotlin/build/libs").filter { it.extension == "jar" })
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.7.3")
}

kotlin {
    jvmToolchain(17)
}

application {
    mainClass.set("MainKt")

    // Native library path for JNI loading
    applicationDefaultJvmArgs = listOf(
        "-Djava.library.path=${rootDir}/../../csharp/flappy_goud"
    )
}

tasks.named<JavaExec>("run") {
    // Working directory for asset resolution
    workingDir = file("../../csharp/flappy_goud")
}
