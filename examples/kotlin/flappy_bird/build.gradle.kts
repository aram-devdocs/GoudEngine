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
    implementation(fileTree("../../../sdks/kotlin/build/libs") { include("*.jar") })
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.7.3")
}

kotlin {
    jvmToolchain(17)
}

application {
    mainClass.set("MainKt")

    // Share assets with the C# flappy_goud example
    applicationDefaultJvmArgs = listOf(
        "-Djava.library.path=${rootDir}/../../csharp/flappy_goud"
    )
}

tasks.named<JavaExec>("run") {
    workingDir = file("../../csharp/flappy_goud")
}
