default:
    @just --list

clean:
    @echo "Cleaning space-cmd..."
    @cargo clean
    @rm -rf target

install:
    @cargo build

ci: format lint test build
    @echo "CI passed"

format:
    @cargo fmt

lint:
    @cargo clippy -- -D warnings

test:
    @cargo test

build:
    @cargo build --release

run:
    @cargo run

publish: ci
    @cargo publish

commits:
    @git --no-pager log --pretty=format:"%h | %ar | %s"
