project(BasicSample)

add_executable(BasicSample main.cpp)

target_link_libraries(BasicSample PRIVATE GoudEngineCore GraphicsModule)
