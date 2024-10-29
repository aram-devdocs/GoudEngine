project(EngineModule)


include_directories(
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${CMAKE_SOURCE_DIR}/modules/submodules/graphics/include  # Add this line
)


file(GLOB ENGINE_SRC ${CMAKE_CURRENT_SOURCE_DIR}/src/*.cpp)

add_library(EngineModule STATIC ${ENGINE_SRC})  # Remove `${}` around `ENGINE_SRC`