plugins {
    kotlin("jvm") version "1.9.22"
    application
}

group = "com.goudengine.examples"
version = "1.0.0"

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
}

tasks.named<JavaExec>("run") {
    workingDir = file("../../csharp/flappy_goud")
    jvmArgs = listOf("-Djava.library.path=${file("../../../target/release")}")
}
