cmake_minimum_required(VERSION 3.16)
project(flappy_bird LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Repository root is three levels above this directory.
get_filename_component(GOUD_ENGINE_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../.." ABSOLUTE)

list(APPEND CMAKE_MODULE_PATH "${GOUD_ENGINE_ROOT}/sdks/cpp/cmake")

find_package(GoudEngine REQUIRED)

file(GLOB SOURCES "${CMAKE_CURRENT_SOURCE_DIR}/src/*.cpp")

add_executable(flappy_bird ${SOURCES})
target_link_libraries(flappy_bird PRIVATE GoudEngine::GoudEngine)
