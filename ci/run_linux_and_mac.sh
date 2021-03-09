case $(uname | tr '[:upper:]' '[:lower:]') in
  linux*)
    export APPVEYOR_OS_NAME=linux
    ;;
  darwin*)
    export APPVEYOR_OS_NAME=osx
    ;;
esac
export PATH="$HOME/.cargo/bin:$PATH"
source $HOME/.cargo/env
cargo clean
if [ "$APPVEYOR_OS_NAME" = "linux" ]; then MIRI_NIGHTLY=nightly-$(curl -s https://rust-lang.github.io/rustup-components-history/x86_64-unknown-linux-gnu/miri); fi
if [ "$APPVEYOR_OS_NAME" = "osx" ]; then MIRI_NIGHTLY=nightly-$(curl -s https://rust-lang.github.io/rustup-components-history/x86_64-apple-darwin/miri); fi
echo "Installing latest nightly with Miri: $MIRI_NIGHTLY"
rustup set profile minimal
rustup default "$MIRI_NIGHTLY"
rustup component add miri
# The `-Zmiri-disable-isolation` is so Miri can access the system clock
# while calling `SystemTime::now()` in one of the tests.
export MIRIFLAGS="-Zmiri-disable-isolation"
# We run the suite once under Miri with all functionality enabled, and then once
# normally without the default features just to make sure `no_std` support has
# not been broken.
cargo miri test --features="std"
cargo clean
cargo test --no-default-features