cmake_minimum_required(VERSION 3.16)
project(c_smoke LANGUAGES C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Repository root is three levels above this directory.
get_filename_component(GOUD_ENGINE_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../.." ABSOLUTE)

# Include paths: generated header + C SDK convenience layer
set(GOUD_INCLUDE_DIR "${GOUD_ENGINE_ROOT}/codegen/generated")
set(GOUD_C_SDK_DIR "${GOUD_ENGINE_ROOT}/sdks/c/include")

# Find native library
find_library(GOUD_ENGINE_LIB goud_engine
    PATHS "${GOUD_ENGINE_ROOT}/target/release"
    REQUIRED)

add_executable(c_smoke main.c)
target_include_directories(c_smoke PRIVATE
    "${GOUD_INCLUDE_DIR}"
    "${GOUD_C_SDK_DIR}"
)
target_link_libraries(c_smoke PRIVATE "${GOUD_ENGINE_LIB}")

# Link math library on Unix
if(UNIX)
    target_link_libraries(c_smoke PRIVATE m)
endif()
