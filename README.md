# Slint Calendar

RustとSlintを使用して構築されたクロスプラットフォームカレンダーデスクトップアプリケーション。

## 機能

- 📅 月単位でのカレンダー表示
- ⬅️➡️ 前月・次月への移動
- 📍 今日の日付へのジャンプ
- 🎨 シンプルで使いやすいUI

## 要件

- Rust 1.70以上
- Cargo

## セットアップ

```bash
cargo build
```

## 実行

```bash
cargo run
```

## ビルド（リリース版）

```bash
cargo build --release
```

## 使用方法

1. 左右の矢印ボタンで月を移動します
2. "今日"ボタンで現在の月に戻ります

## 技術スタック

- **Rust** - システムプログラミング言語
- **Slint** - 宣言型UIフレームワーク
- **Chrono** - 日付・時刻ライブラリ

## プロジェクト構成

```
├── Cargo.toml          # プロジェクト設定
├── src/
│   └── main.rs         # メインプログラム
├── ui/
│   └── calendar.slint  # UI定義
└── README.md           # このファイル
```

## ライセンス

GPL-3.0 License

## macOS版の配布ファイルについて
macOS版の配布ファイルは、署名されていないため起動時に警告が表示される場合があります。
「“SlintCalendar”は壊れているため開けません。 ゴミ箱に入れる必要があります。」

これを回避するには、以下の手順を実行してください：
1. ターミナルウィンドウを開きDownloadsフォルダに移動します：
   ```bash
   cd ~/Downloads
   ```
2. 次のコマンドを実行してアプリケーションの属性を変更する：
   ```bash
   xattr -d com.apple.quarantine SlintCalendar.app
   ```

## 開発に関する注意

このプロジェクトはVibe Codingの実験プロジェクトです。
そのため、安定性や保守性は保証されていません。
