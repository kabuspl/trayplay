pkgname=instantreplay
pkgver=0.0.1
pkgrel=1
pkgdesc="Instant Replay (DVR) app for KDE using gpu-screen-recorder in the background"
arch=('x86_64', 'aarch64')
url="https://github.com/kabuspl/instantreplay"
license=('MIT')
depends=('gcc-libs', 'glibc', 'gpu-screen-recorder', 'xdg-desktop-portal-impl')
makedepends=('cargo')
source=("${pkgname}-${pkgver}.tar.gz::https://github.com/kabuspl/instantreplay/archive/v${pkgver}.tar.gz")
sha256sums=('2ee006027c5ccb46ffca5de2d310f92a9e8fb19e0b3b6892e2cdc29890b2e3f5')

prepare() {
    export RUSTUP_TOOLCHAIN=stable
    cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cargo build --frozen --release --all-features
}

check() {
    export RUSTUP_TOOLCHAIN=stable
    cargo test --frozen --all-features
}

package() {
    install -Dm0755 -t "$pkgdir/usr/bin/" "target/release/$pkgname"
    install -Dm0644 -t "$pkgdir/usr/share/instantreplay" "dist/kwin_script.js"
    install -Dm0644 -t "$pkgdir/usr/share/applications" "dist/ovh.kabus.instantreplay.desktop"
    install -Dm0644 LICENSE "$pkgdir/usr/share/licenses/${pkgname}/LICENSE"
}
