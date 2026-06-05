# Code Inspector (Encoding Checker)

指定したフォルダ配下のファイルを走査し、文字コードと改行コードを自動判定するデスクトップアプリケーションです。

## 主な機能

- **一括判定**: フォルダ内の全ファイルを再帰的にスキャン。
- **文字コード判定**: `chardetng` ライブラリを使用した高精度な文字コード推測。
- **改行コード判定**: CRLF, LF などの改行コードを識別。
- **期待値チェック**: ユーザーが設定した期待する文字コード・改行コードと異なるファイルを抽出（NG表示）。
- **フィルタリング**: パスによる検索や、NGファイルのみの表示切り替え。
- **CSV出力**: 判定結果をCSV形式でエクスポート可能。

## 技術スタック

- **Frontend**: Svelte 5, TypeScript, Tailwind CSS v4, shadcn-svelte
- **Backend**: Tauri v2, Rust
- **Logic**: `walkdir`, `chardetng`, `encoding_rs`

## 開発環境のセットアップ

### 必須環境
- Node.js (v20以上推奨)
- Rust (stable)

### インストール
```bash
npm install
```

### 開発モードの起動
```bash
npm run tauri dev
```

### ビルド
```bash
npm run tauri build
```

## ライセンス
MIT
