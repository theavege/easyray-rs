set -euo pipefail
source '/etc/os-release'
case ${ID:?} in
    debian | ubuntu) sudo bash -c '
        apt-get update
        apt-get install -y libgtk-3-dev
    ';;
    fedora | alma) sudo apt-get install -y fox-devel ;;
esac 1> /dev/null
cargo clippy --quiet --features="ray" --example simple
cargo build --release --features="ray" --examples simple
cargo clippy --quiet --features="tui" --example console
cargo build --release --features="tui" --examples console
cargo fmt --check --all
