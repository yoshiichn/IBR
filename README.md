# IBR
I`m Busy Reveiwing.

あ～レビュー忙しいわ～～～～～～～～～～を解決するためのアプリケーション。

## Develop
```bash
git clone https://github.com/yoshiichn/IBR.git ibr
cd ibr
curl --proto '=https' --tlsv1.3 https://sh.rustup.rs -sSf | sh
# select 1 in console
source $HOME/.cargo/env
# cargo install
sudo apt install cargo build-essential clang
# After changing the code, build it with the commands defined in the Makefile.
cargo install --force cargo-make
cargo make build
cargo make serve
# let's check it out in your web browser.
```
