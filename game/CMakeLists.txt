project(GoudGameCore)


include_directories(
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${CMAKE_SOURCE_DIR}/modules/graphics/include  # Add this line
)


file(GLOB GAME_SRC ${CMAKE_CURRENT_SOURCE_DIR}/src/*.cpp)

add_library(GoudGameCore STATIC ${GAME_SRC})  # Remove `${}` around `ENGINE_SRC`