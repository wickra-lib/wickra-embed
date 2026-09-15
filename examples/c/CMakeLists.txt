# Build and run the wickra-embed C ABI example.
#
#   cargo build --release -p wickra-embed-c
#   cmake -S examples/c -B examples/c/build
#   cmake --build examples/c/build --config Release
#   ctest --test-dir examples/c/build -C Release --output-on-failure
#
# WICKRA_EMBED_LIB_DIR defaults to the workspace target/release directory and can
# be overridden for out-of-tree builds.

cmake_minimum_required(VERSION 3.15)
project(wickra_embed_examples C CXX)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_CXX_STANDARD 11)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(WICKRA_EMBED_LIB_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../target/release"
    CACHE PATH "Directory containing the built wickra-embed C ABI library")

set(WICKRA_EMBED_INCLUDE "${CMAKE_CURRENT_SOURCE_DIR}/../../bindings/c/include")

find_library(WICKRA_EMBED_LIB
    NAMES wickra_embed libwickra_embed
    HINTS "${WICKRA_EMBED_LIB_DIR}"
    REQUIRED)

add_executable(sma_signal sma_signal.c)
target_include_directories(sma_signal PRIVATE "${WICKRA_EMBED_INCLUDE}")
target_link_libraries(sma_signal PRIVATE "${WICKRA_EMBED_LIB}")

# On Windows the runtime DLL must sit next to the executable (there is no rpath);
# without it the loader pops a modal "DLL not found" dialog that hangs ctest. Copy
# it after build. Unix resolves the .so/.dylib via the linker's embedded path.
if(WIN32)
    add_custom_command(TARGET sma_signal POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
            "${WICKRA_EMBED_LIB_DIR}/wickra_embed.dll"
            "$<TARGET_FILE_DIR:sma_signal>"
        VERBATIM)
endif()

enable_testing()
add_test(NAME sma_signal COMMAND sma_signal)
# A missing runtime dependency must fail fast, never block the runner for hours.
set_tests_properties(sma_signal PROPERTIES TIMEOUT 60)

# The C++ wrapper (wickra_embed.hpp) gets the same sample, so a header that
# stops compiling or a wrapper that misreads a status code fails here.
add_executable(sma_signal_cpp sma_signal.cpp)
target_include_directories(sma_signal_cpp PRIVATE "${WICKRA_EMBED_INCLUDE}")
target_link_libraries(sma_signal_cpp PRIVATE "${WICKRA_EMBED_LIB}")
if(WIN32)
    add_custom_command(TARGET sma_signal_cpp POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
            "${WICKRA_EMBED_LIB_DIR}/wickra_embed.dll"
            "$<TARGET_FILE_DIR:sma_signal_cpp>"
        VERBATIM)
endif()
add_test(NAME sma_signal_cpp COMMAND sma_signal_cpp)
set_tests_properties(sma_signal_cpp PROPERTIES TIMEOUT 60)

# The golden test: every catalogue indicator fed the golden inputs through the
# C ABI, one bar at a time, compared with the expected columns bit for bit.
# GOLDEN_DIR is the repository's golden/ directory, resolved at configure time.
add_executable(golden_test golden_test.c)
target_include_directories(golden_test PRIVATE "${WICKRA_EMBED_INCLUDE}")
target_link_libraries(golden_test PRIVATE "${WICKRA_EMBED_LIB}")
get_filename_component(WICKRA_EMBED_GOLDEN "${CMAKE_CURRENT_SOURCE_DIR}/../../golden" ABSOLUTE)
target_compile_definitions(golden_test PRIVATE GOLDEN_DIR="${WICKRA_EMBED_GOLDEN}")
if(WIN32)
    add_custom_command(TARGET golden_test POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
            "${WICKRA_EMBED_LIB_DIR}/wickra_embed.dll"
            "$<TARGET_FILE_DIR:golden_test>"
        VERBATIM)
endif()
add_test(NAME golden_test COMMAND golden_test)
set_tests_properties(golden_test PROPERTIES TIMEOUT 60)
