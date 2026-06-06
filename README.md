# Code Inspector (文字コード・改行コード判定ツール)

指定したフォルダ配下のファイルを走査し、文字コード（Encoding）と改行コード（Newline）を自動判定するデスクトップアプリケーションです。

## 主な機能

- **一括判定**: 指定フォルダ内の全ファイルを再帰的にスキャンし、一覧表示します。
- **高精度な判定**: `chardetng` ライブラリを採用し、日本語（Shift_JIS, EUC-JP等）を含む多様な文字コードを推定します。
- **改行コードの識別**: CRLF、LF、CR を正確に判定します。
- **期待値チェック**: ユーザーが設定した「期待する文字コード・改行コード」と一致しないファイルを「NG」として即座に強調表示します。
- **柔軟な除外設定**: `.git` や `node_modules` などのディレクトリ、およびバイナリファイル（`.exe`, `.pdf`, `.xlsx` 等）を判定対象から除外できます。
- **検索・フィルタリング**: ファイルパスによる絞り込みや、NGファイルのみの表示が可能です。
- **CSVエクスポート**: 判定結果をCSVファイルとして保存し、Excel等で二次利用できます。

## 技術構成

- **フロントエンド**: Svelte 5, TypeScript, Tailwind CSS, shadcn-svelte
- **バックエンド**: Tauri v2, Rust
- **主要ライブラリ**: `walkdir`, `chardetng`, `encoding_rs`

## 開発環境のセットアップ

### 必須ツール
- [Node.js](https://nodejs.org/) (v20以上推奨)
- [Rust](https://www.rust-lang.org/) (stable)

### 依存関係のインストール
```bash
npm install
```

### 開発用サーバーの起動
```bash
npm run tauri dev
```

### パッケージのビルド（インストーラー作成）
```bash
npm run tauri build
```

## ライセンス
MIT
