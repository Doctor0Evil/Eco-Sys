cmake_minimum_required(VERSION 3.16)
project(AirGlobePlanningSafetySecurity VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

include_directories(include src)

add_executable(airglobe_demo
    src/MainDemo.cpp
    src/EspdModel.cpp
    src/DtTrustModel.cpp
    src/KtsfModel.cpp
    src/ShardIo.cpp
)

target_compile_options(airglobe_demo PRIVATE -Wall -Wextra -O2 -march=native)

enable_testing()
add_test(NAME EspdTests COMMAND airglobe_demo --test espd)
add_test(NAME DtTests COMMAND airglobe_demo --test dt)
add_test(NAME KtsfTests COMMAND airglobe_demo --test ktsf)
