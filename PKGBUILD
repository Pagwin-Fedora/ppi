# Maintainer: Pagwin <spam@pagwin.xyz>
pkgname='ppi-git'
_name='ppi'
pkgver='1.1.4'
pkgrel='1'
pkgdesc="Pagwin's project initializer, a program which makes setting up project scaffolding easy"
# I could set this to any or all the archictectures but I can't easily test for those so change this if needed
arch=('x86_64')

url='https://github.com/Pagwin-Fedora/ppi'
license=('MIT')

depends=('glibc' 'openssl' 'zlib' 'gcc-libs')

makedepends=('git' 'rust')

provides=('ppi')

conflicts=('ppi')

source=("git+$url")

b2sums=('SKIP')

build(){
    cd $_name
    cargo build --release
}

package(){
    cd $_name
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$_name/LICENSE"
    install -Dm755 target/release/ppi "$pkgdir/usr/bin/ppi"
}
