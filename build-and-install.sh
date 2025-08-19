#!/bin/bash

# WebAssemblyビルドとインストールの自動化スクリプト

set -e  # エラー時に停止

echo "🔨 Building WebAssembly package..."

# phase-4.1ディレクトリに移動してビルド
cd phase-4.1
wasm-pack build --target web --out-dir pkg

echo "📦 Copying binary files..."

# バイナリファイルをpkgディレクトリにコピー
cp src/*.bin pkg/

echo "📋 Binary files copied:"
ls -la pkg/*.bin

echo "🔄 Installing package to reversi-web..."

# reversi-webディレクトリに移動
cd ../reversi-web

# 既存パッケージをアンインストール
npm uninstall fl-reversi-rs

# 新しいパッケージをインストール
npm install ../phase-4.1/pkg

echo "✅ Installation complete!"

# 確認
echo "📋 Installed files:"
ls -la node_modules/fl-reversi-rs/*.bin

echo "🚀 Ready to run: npm start"
