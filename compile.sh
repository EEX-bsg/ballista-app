#!/bin/bash

echo "Ballista-BesiegeLauncher コンパイルスクリプト"
echo "======================================"

# ビルドタイプの選択（デフォルトはリリース）
BUILD_TYPE="release"
if [ "$1" == "debug" ]; then
    BUILD_TYPE="debug"
    echo "デバッグビルドを実行します..."
else
    echo "リリースビルドを実行します... (デバッグビルドを行うには 'debug' 引数を追加してください)"
fi

# フロントエンドディレクトリに移動
cd frontend || { echo "フロントエンドディレクトリが見つかりません"; exit 1; }

# 依存関係のインストール確認
if [ ! -d "node_modules" ]; then
    echo "node_modulesが見つかりません。依存関係をインストールします..."
    npm install || { echo "依存関係のインストールに失敗しました"; cd ..; exit 1; }
fi

# Tauriアプリケーションのビルド
echo "Tauriアプリケーションをビルドしています... (${BUILD_TYPE}モード)"
if [ "$BUILD_TYPE" == "debug" ]; then
    npm run tauri build -- --debug || { echo "ビルドに失敗しました"; cd ..; exit 1; }
else
    npm run tauri build || { echo "ビルドに失敗しました"; cd ..; exit 1; }
fi

echo "ビルド成功!"

# ビルド成果物の場所を特定
if [ "$BUILD_TYPE" == "debug" ]; then
    BUNDLE_DIR="src-tauri/target/debug/bundle"
else
    BUNDLE_DIR="src-tauri/target/release/bundle"
fi

# OSに応じたビルド成果物の場所を特定
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" || "$OSTYPE" == "cygwin" ]]; then
    # Windows
    if [ -d "$BUNDLE_DIR/msi" ]; then
        echo "Windows MSIインストーラーが生成されました:"
        find "$BUNDLE_DIR/msi" -name "*.msi" -type f | while read -r file; do
            echo "- $file"
            # 絶対パスを表示
            ABSOLUTE_PATH=$(cd "$(dirname "$file")" && pwd)
            echo "  絶対パス: $ABSOLUTE_PATH/$(basename "$file")"
        done
    fi
    
    if [ -d "$BUNDLE_DIR/nsis" ]; then
        echo "Windows NSISインストーラーが生成されました:"
        find "$BUNDLE_DIR/nsis" -name "*.exe" -type f | while read -r file; do
            echo "- $file"
            # 絶対パスを表示
            ABSOLUTE_PATH=$(cd "$(dirname "$file")" && pwd)
            echo "  絶対パス: $ABSOLUTE_PATH/$(basename "$file")"
        done
    fi
elif [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    if [ -d "$BUNDLE_DIR/macos" ]; then
        echo "macOSアプリケーションが生成されました:"
        find "$BUNDLE_DIR/macos" -name "*.app" -type d | while read -r file; do
            echo "- $file"
            # 絶対パスを表示
            ABSOLUTE_PATH=$(cd "$(dirname "$file")" && pwd)
            echo "  絶対パス: $ABSOLUTE_PATH/$(basename "$file")"
        done
        
        if [ -d "$BUNDLE_DIR/dmg" ]; then
            echo "macOS DMGインストーラーが生成されました:"
            find "$BUNDLE_DIR/dmg" -name "*.dmg" -type f | while read -r file; do
                echo "- $file"
                # 絶対パスを表示
                ABSOLUTE_PATH=$(cd "$(dirname "$file")" && pwd)
                echo "  絶対パス: $ABSOLUTE_PATH/$(basename "$file")"
            done
        fi
    fi
else
    # Linux
    if [ -d "$BUNDLE_DIR/appimage" ]; then
        echo "Linux AppImageが生成されました:"
        find "$BUNDLE_DIR/appimage" -name "*.AppImage" -type f | while read -r file; do
            echo "- $file"
            # 絶対パスを表示
            ABSOLUTE_PATH=$(cd "$(dirname "$file")" && pwd)
            echo "  絶対パス: $ABSOLUTE_PATH/$(basename "$file")"
        done
    fi
    
    if [ -d "$BUNDLE_DIR/deb" ]; then
        echo "Linux DEBパッケージが生成されました:"
        find "$BUNDLE_DIR/deb" -name "*.deb" -type f | while read -r file; do
            echo "- $file"
            # 絶対パスを表示
            ABSOLUTE_PATH=$(cd "$(dirname "$file")" && pwd)
            echo "  絶対パス: $ABSOLUTE_PATH/$(basename "$file")"
        done
    fi
fi

# 元のディレクトリに戻る
cd ..

echo "======================================"
echo "ビルドが完了しました。上記の場所に生成されたファイルを確認してください。"
echo "開発環境を起動するには次のコマンドを実行してください:"
echo "./run-dev.sh"
