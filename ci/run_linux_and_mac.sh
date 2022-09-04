#case $(uname | tr '[:upper:]' '[:lower:]') in
#  linux*)
#    export APPVEYOR_OS_NAME=linux
#    ;;
#  darwin*)
#    export APPVEYOR_OS_NAME=osx
#    ;;
#esac
#if [ "$APPVEYOR_OS_NAME" = "linux" ]; then export PATH="/home/appveyor/.cargo/bin:$PATH"; fi
#if [ "$APPVEYOR_OS_NAME" = "osx" ]; then export PATH="$HOME/.cargo/bin:$PATH"; fi
#if [ "$APPVEYOR_OS_NAME" = "linux" ]; then MIRI_NIGHTLY=nightly-$(curl -s https://rust-lang.github.io/rustup-components-history/x86_64-unknown-linux-gnu/miri); fi
#if [ "$APPVEYOR_OS_NAME" = "osx" ]; then MIRI_NIGHTLY=nightly-$(curl -s https://rust-lang.github.io/rustup-components-history/x86_64-apple-darwin/miri); fi
#echo "Installing latest nightly with Miri: $MIRI_NIGHTLY"
#rustup set profile minimal
#rustup default "$MIRI_NIGHTLY"
#rustup component add miri
# The `-Zmiri-disable-isolation` is so Miri can access the system clock
# while calling `SystemTime::now()` in one of the tests.
# The `-Zmiri-permissive-provenance` is because this crate has a few
# integer-to-pointer casts that are needed to avoid UB with ZST-related
# stuff. 
#export MIRIFLAGS="-Zmiri-disable-isolation -Zmiri-permissive-provenance"
# We run the suite once under Miri with all functionality enabled, and then once
# normally without the default features just to make sure `no_std` support has
# not been broken.
#cargo clean
#cargo miri test --features="std" || exit 1
#cargo clean
#cargo test --no-default-features || exit 1
# Uncommenting the last two lines also runs the libfuzzer fuzz target, but it's more suited for running
# locally than on CI currently due to the amount of time it takes to be effective
# cargo clean
# if [ "$APPVEYOR_OS_NAME" = "linux" ]; then cd ./fuzz && cargo install cargo-fuzz && cargo fuzz run ops; fi
