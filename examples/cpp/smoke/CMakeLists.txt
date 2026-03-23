cmake_minimum_required(VERSION 3.16)
project(cpp_smoke LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Repository root is three levels above this directory.
get_filename_component(GOUD_ENGINE_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../.." ABSOLUTE)

# Include paths: generated header + C SDK + C++ SDK convenience layer
set(GOUD_INCLUDE_DIR "${GOUD_ENGINE_ROOT}/codegen/generated")
set(GOUD_C_SDK_DIR "${GOUD_ENGINE_ROOT}/sdks/c/include")
set(GOUD_CPP_SDK_DIR "${GOUD_ENGINE_ROOT}/sdks/cpp/include")

# Find native library
find_library(GOUD_ENGINE_LIB goud_engine
    PATHS "${GOUD_ENGINE_ROOT}/target/release"
    REQUIRED)

add_executable(cpp_smoke main.cpp)
target_include_directories(cpp_smoke PRIVATE
    "${GOUD_INCLUDE_DIR}"
    "${GOUD_C_SDK_DIR}"
    "${GOUD_CPP_SDK_DIR}"
)
target_link_libraries(cpp_smoke PRIVATE "${GOUD_ENGINE_LIB}")
