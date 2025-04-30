cargo-nextest := require("cargo-nextest") # require cargo-nextest

[private]
default:
    just --list

# Run tests
test *test_names:
    cargo nextest run {{test_names}}
