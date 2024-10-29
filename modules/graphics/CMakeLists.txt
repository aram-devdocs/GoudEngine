project(GraphicsModule)

include_directories(
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${CMAKE_SOURCE_DIR}/modules/engine/include
)

file(GLOB GRAPHICS_SRC ${CMAKE_CURRENT_SOURCE_DIR}/src/*.cpp)

add_library(GraphicsModule STATIC ${GRAPHICS_SRC})  # Remove `${}` around `GRAPHICS_SRC`
target_link_libraries(GraphicsModule PUBLIC GoudEngineCore)