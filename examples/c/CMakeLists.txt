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
project(wickra_embed_examples C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

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
