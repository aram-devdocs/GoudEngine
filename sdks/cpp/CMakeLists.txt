cmake_minimum_required(VERSION 3.14)
project(GoudEngineCpp VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# ---------------------------------------------------------------------------
# Repo-root relative paths
# ---------------------------------------------------------------------------

get_filename_component(GOUD_REPO_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../.." ABSOLUTE)

# ---------------------------------------------------------------------------
# Unified include directory
# ---------------------------------------------------------------------------
#
# The C SDK header (goud/goud.h) uses  #include "../goud_engine.h"  which
# the compiler resolves relative to the physical file location. We copy the
# C SDK headers and the generated header into a single build-tree directory
# so the relative include resolves correctly.
# ---------------------------------------------------------------------------

set(_goud_unified_dir "${CMAKE_CURRENT_BINARY_DIR}/_goud_unified_include")
file(MAKE_DIRECTORY "${_goud_unified_dir}")
file(COPY "${GOUD_REPO_ROOT}/sdks/c/include/goud"        DESTINATION "${_goud_unified_dir}")
file(COPY "${GOUD_REPO_ROOT}/codegen/generated/goud_engine.h" DESTINATION "${_goud_unified_dir}")

# ---------------------------------------------------------------------------
# Interface library target
# ---------------------------------------------------------------------------

add_library(GoudEngine INTERFACE)

target_include_directories(GoudEngine INTERFACE
    $<BUILD_INTERFACE:${_goud_unified_dir}>
    $<BUILD_INTERFACE:${GOUD_REPO_ROOT}/sdks/cpp/include>
    $<BUILD_INTERFACE:${GOUD_REPO_ROOT}/codegen/generated>
    $<INSTALL_INTERFACE:include>
)

target_compile_features(GoudEngine INTERFACE cxx_std_17)

# ---------------------------------------------------------------------------
# Options
# ---------------------------------------------------------------------------

option(GOUD_BUILD_TESTS    "Build GoudEngine C++ SDK tests"    OFF)
option(GOUD_BUILD_EXAMPLES "Build GoudEngine C++ SDK examples" OFF)

if(GOUD_BUILD_TESTS)
    enable_testing()
    if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/tests/CMakeLists.txt")
        add_subdirectory(tests)
    endif()
endif()

if(GOUD_BUILD_EXAMPLES)
    if(EXISTS "${GOUD_REPO_ROOT}/examples/cpp/cmake_example/CMakeLists.txt")
        add_subdirectory("${GOUD_REPO_ROOT}/examples/cpp/cmake_example" cmake_example_build)
    endif()
endif()

# ---------------------------------------------------------------------------
# Install rules
# ---------------------------------------------------------------------------

include(GNUInstallDirs)

install(
    DIRECTORY "${GOUD_REPO_ROOT}/sdks/cpp/include/"
    DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}"
)

install(
    DIRECTORY "${GOUD_REPO_ROOT}/sdks/c/include/"
    DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}"
)

install(
    FILES "${GOUD_REPO_ROOT}/codegen/generated/goud_engine.h"
    DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}"
)

install(
    TARGETS GoudEngine
    EXPORT GoudEngineTargets
)

install(
    EXPORT GoudEngineTargets
    NAMESPACE GoudEngine::
    DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/GoudEngine"
)

install(
    FILES
        "${CMAKE_CURRENT_SOURCE_DIR}/cmake/GoudEngineConfig.cmake"
        "${CMAKE_CURRENT_SOURCE_DIR}/cmake/FindGoudEngine.cmake"
    DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/GoudEngine"
)
