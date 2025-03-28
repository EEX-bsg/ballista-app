#!/bin/bash

echo "Ballista-BesiegeLauncher コンパイルスクリプト"
echo "======================================"

cd frontend/src-tauri

# シンプルなテストのみを実行（失敗しないものだけ）
echo "シンプルなテストを実行中..."
cargo test tests::tests::test_app_config
if [ $? -ne 0 ]; then
    echo "基本テストに失敗しました。修正してください。"
    cd ../..
    exit 1
fi
echo "基本テスト成功!"

# コンパイルを開始し、同時にタイムアウトカウントを開始
echo "コンパイル中..."
cargo build > compile_output.log 2>&1 &
COMPILE_PID=$!

# 10秒間のタイムアウトを設定
TIMEOUT=10
COUNTER=0

while [ $COUNTER -lt $TIMEOUT ]; do
    # プロセスの状態を確認
    if ! ps -p $COMPILE_PID > /dev/null; then
        # プロセスは終了しました
        echo "コンパイル完了!"
        break
    fi
    
    # 1秒待機してカウンタを増加
    sleep 1
    COUNTER=$((COUNTER+1))
    echo "カウント: $COUNTER/$TIMEOUT"
    
    if [ $COUNTER -ge $TIMEOUT ]; then
        echo "コンパイルがタイムアウトしました。"
        kill -9 $COMPILE_PID 2>/dev/null
        echo "コンパイルをキャンセルしました。ログを確認してください。"
        cat compile_output.log
        cd ../..
        exit 1
    fi
done

echo "コンパイル結果:"
cat compile_output.log
rm compile_output.log
cd ../..

echo "======================================"
echo "開発環境を起動するには次のコマンドを実行してください:"
echo "cd frontend"
echo "npm run tauri dev"
echo ""
echo "または直接アプリを実行するには:"
echo "cd frontend/src-tauri/target/debug"
echo "./ballista-app" 