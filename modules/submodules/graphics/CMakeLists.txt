project(GraphicsModule)

include_directories(
    ${CMAKE_CURRENT_SOURCE_DIR}/include
)


file(GLOB GRAPHICS_SRC ${CMAKE_CURRENT_SOURCE_DIR}/src/*.cpp)

add_library(GraphicsModule STATIC ${GRAPHICS_SRC}) 