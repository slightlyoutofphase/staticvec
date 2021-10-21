cargo clean
cargo run --example main_demo && cargo run --features="std serde_support serde_json_support" --example serde_support_demo && cargo run --example string_demo
REM cargo test --features="std" --target=%TARGET%
REM cargo test --no-default-features --target=%TARGET%
