cmake_minimum_required(VERSION 3.14)
project(cmake_example LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# ---------------------------------------------------------------------------
# Locate GoudEngine via the SDK's find module
# ---------------------------------------------------------------------------

get_filename_component(GOUD_ENGINE_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../.." ABSOLUTE)
list(APPEND CMAKE_MODULE_PATH "${GOUD_ENGINE_ROOT}/sdks/cpp/cmake")

find_package(GoudEngine REQUIRED)

# ---------------------------------------------------------------------------
# Example executable
# ---------------------------------------------------------------------------

add_executable(cmake_example main.cpp)
target_link_libraries(cmake_example PRIVATE GoudEngine::GoudEngine)
