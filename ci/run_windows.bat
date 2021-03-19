cargo clean
cargo test --features="std" --target=%TARGET%
cargo test --no-default-features --target=%TARGET%
