project(GoudEngineCore)

include_directories(${CMAKE_CURRENT_SOURCE_DIR}/include)

file(GLOB ENGINE_SRC ${CMAKE_CURRENT_SOURCE_DIR}/src/*.cpp)

add_library(GoudEngineCore STATIC ${ENGINE_SRC})  # Remove `${}` around `ENGINE_SRC`