# Display Switcher

Windowsのディスプレイ設定（マルチモニター構成）をコマンドラインから瞬時に保存・復元・切り替えができる、軽量で高速なRust製CLIツールです。

ノートPCと外部モニターを共有している環境などで、毎回Windowsのディスプレイ設定を開いて手動で「切断」「拡張」を切り替える煩わしさを解消するために開発されました。Elgato Stream Deckなどのマクロデバイスとの連携に最適化されています。

## ✨ 主な機能

* **プロファイルの保存と復元:** 現在のディスプレイ構成（ON/OFF、解像度、配置など）をそのまま凍結して保存し、いつでも一瞬で復元できます。
* **バッチファイルの自動生成:** プロファイルを保存すると同時に、それを実行するための `.bat` ファイルを自動生成します。Stream Deckなどの外部デバイスからパスを指定するだけで簡単に呼び出せます。
* **モニターの個別制御:** 指定したモニターのTarget IDを使って、個別にON/OFFを切り替えることも可能です。
* **超高速動作:** OSの低レイヤーAPI（Windows API）を直接叩き、生のメモリデータをバイナリ（`.dat`）として保存・読み込みするため、変換のオーバーヘッドがなく一瞬で切り替わります。

## 🚀 インストールとビルド

このプロジェクトをビルドするには、[Rust](https://www.rust-lang.org/) のビルド環境が必要です。

```bash
# リポジトリのクローン
git clone [https://github.com/yourusername/display-switcher.git](https://github.com/yourusername/display-switcher.git)
cd display-switcher

# 最適化されたリリースビルドの作成
cargo build --release
