cargo clean
if [ "$TRAVIS_OS_NAME" = "linux" ]; then MIRI_NIGHTLY=nightly-$(curl -s https://rust-lang.github.io/rustup-components-history/x86_64-unknown-linux-gnu/miri); fi
if [ "$TRAVIS_OS_NAME" = "osx" ]; then MIRI_NIGHTLY=nightly-$(curl -s https://rust-lang.github.io/rustup-components-history/x86_64-apple-darwin/miri); fi
echo "Installing latest nightly with Miri: $MIRI_NIGHTLY"
rustup set profile minimal
rustup default "$MIRI_NIGHTLY"
rustup component add miri
# The `-Zmiri-disable-isolation` is so Miri can access the system clock
# while calling `SystemTime::now()` in one of the tests.
export MIRIFLAGS="-Zmiri-disable-isolation"
cargo miri test --features="std"
