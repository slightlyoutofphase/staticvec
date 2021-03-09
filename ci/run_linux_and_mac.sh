case $(uname | tr '[:upper:]' '[:lower:]') in
  linux*)
    export APPVEYOR_OS_NAME=linux
    ;;
  darwin*)
    export APPVEYOR_OS_NAME=osx
    ;;
esac
export PATH="$HOME/.cargo/bin:$PATH"
# source $HOME/.cargo/env
$HOME/.cargo/bin/cargo clean
if [ "$APPVEYOR_OS_NAME" = "linux" ]; then MIRI_NIGHTLY=nightly-$(curl -s https://rust-lang.github.io/rustup-components-history/x86_64-unknown-linux-gnu/miri); fi
if [ "$APPVEYOR_OS_NAME" = "osx" ]; then MIRI_NIGHTLY=nightly-$(curl -s https://rust-lang.github.io/rustup-components-history/x86_64-apple-darwin/miri); fi
echo "Installing latest nightly with Miri: $MIRI_NIGHTLY"
$HOME/.cargo/bin/rustup set profile minimal
$HOME/.cargo/bin/rustup default "$MIRI_NIGHTLY"
$HOME/.cargo/bin/rustup component add miri
# The `-Zmiri-disable-isolation` is so Miri can access the system clock
# while calling `SystemTime::now()` in one of the tests.
export MIRIFLAGS="-Zmiri-disable-isolation"
# We run the suite once under Miri with all functionality enabled, and then once
# normally without the default features just to make sure `no_std` support has
# not been broken.
$HOME/.cargo/bin/rustup miri test --features="std"
$HOME/.cargo/bin/rustup clean
$HOME/.cargo/bin/rustup test --no-default-features