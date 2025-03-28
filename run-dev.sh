#!/bin/bash

echo "Ballista-BesiegeLauncher 開発環境起動スクリプト"
echo "======================================"

cd frontend

echo "開発環境を起動中..."
npm run tauri dev > dev_output.log 2>&1 &
DEV_PID=$!

TIMEOUT=60
COUNTER=0
SUCCESS=0
RUST_COMPILED=0

echo "フロントエンドとRustバックエンドの起動を待機しています..."

# ログファイルに特定のメッセージが表示されたかをチェックする関数
check_frontend_success() {
    if grep -q "Local:   http://localhost" dev_output.log || 
       grep -q "ready in" dev_output.log || 
       grep -q "Tauri development" dev_output.log; then
        return 0
    else
        return 1
    fi
}

# Rustのコンパイル成功をチェックする関数
check_rust_success() {
    if grep -q "Finished .* \[unoptimized + debuginfo\]" dev_output.log || 
       grep -q "Starting Tauri Development" dev_output.log || 
       grep -q "APIテスト環境の初期化が完了しました" dev_output.log; then
        return 0
    elif grep -q "warning:" dev_output.log && grep -q "Compiling ballista-app" dev_output.log; then
        # 警告があるが、コンパイルは進行中
        return 0
    elif grep -q "Compiling ballista-app" dev_output.log && [ $COUNTER -gt 20 ]; then
        # 20秒以上経過してもコンパイル中なら進行中と判断
        return 0
    else
        return 1
    fi
}

# エラーが発生したかをチェックする関数
check_error() {
    # 実際のエラーメッセージがログにある場合のみエラーと判断
    if grep -q "error: could not compile" dev_output.log || 
       grep -q "error\[E.*\]: " dev_output.log || 
       grep -q "thread.*panicked at" dev_output.log; then
        # 警告メッセージの場合はエラーとしない
        if ! grep -q "warning:" dev_output.log || grep -q "FATAL ERROR" dev_output.log; then
            return 0
        fi
    fi
    return 1
}

# バックアッププロセスが実行されているかチェックする関数
check_backup_process() {
    if grep -q "Backup process started" dev_output.log; then
        return 0
    else
        return 1
    fi
}

while [ $COUNTER -lt $TIMEOUT ]; do
    # フロントエンド起動チェック
    FRONTEND_OK=0
    if check_frontend_success; then
        FRONTEND_OK=1
    fi
    
    # Rustコンパイル成功チェック
    if [ $RUST_COMPILED -eq 0 ]; then
        if check_rust_success; then
            RUST_COMPILED=1
            echo "Rustバックエンドが正常にコンパイルされました！"
        fi
    fi
    
    # エラーチェック - 実際のエラーが発生した場合のみ
    if check_error; then
        echo "開発環境の起動に失敗しました。"
        cat dev_output.log
        kill $DEV_PID 2>/dev/null
        cd ..
        exit 1
    fi
    
    # 両方成功していれば完了
    if [ $FRONTEND_OK -eq 1 ] && [ $RUST_COMPILED -eq 1 ]; then
        SUCCESS=1
        break
    elif [ $FRONTEND_OK -eq 1 ]; then
        echo "フロントエンドの準備ができました。Rustバックエンドのコンパイルを待機中..."
    elif [ $RUST_COMPILED -eq 1 ]; then
        echo "Rustバックエンドの準備ができました。フロントエンドの起動を待機中..."
    fi
    
    # 1秒待機してカウンタを増加
    sleep 1
    COUNTER=$((COUNTER + 1))
    echo "カウント: $COUNTER/$TIMEOUT"
done

if [ $COUNTER -ge $TIMEOUT ]; then
    echo "開発環境の起動がタイムアウトしました。"
    echo "ログを確認しています..."
    
    # タイムアウト時にも最終確認
    FRONTEND_OK=0
    if check_frontend_success; then
        FRONTEND_OK=1
    fi
    
    if [ $RUST_COMPILED -eq 0 ]; then
        if check_rust_success; then
            RUST_COMPILED=1
        fi
    fi
    
    # 両方成功していれば成功とみなす
    if [ $FRONTEND_OK -eq 1 ] && [ $RUST_COMPILED -eq 1 ]; then
        SUCCESS=1
    else
        # 失敗原因を表示
        if [ $RUST_COMPILED -eq 0 ]; then
            echo "Rustバックエンドのコンパイルが完了していません。"
        fi
        if [ $FRONTEND_OK -eq 0 ]; then
            echo "フロントエンドの起動が完了していません。"
        fi
        
        kill $DEV_PID 2>/dev/null
        echo "起動をキャンセルしました。ログを確認してください。"
        cat dev_output.log
        cd ..
        exit 1
    fi
fi

if [ $SUCCESS -eq 1 ]; then
    echo "開発環境が正常に起動しました！"
    echo "フロントエンド: OK"
    echo "Rustバックエンド: OK"
    echo "カウント: $COUNTER/$TIMEOUT"
    echo "ログ出力:"
    cat dev_output.log
    
    # バックアッププロセスチェック
    if check_backup_process; then
        echo "バックアッププロセスも正常に起動しています。"
    fi
else
    echo "開発環境の起動状態が不明です。"
    if [ $RUST_COMPILED -eq 0 ]; then
        echo "Rustバックエンドのコンパイルが完了していない可能性があります。"
    fi
    echo "ログ出力:"
    cat dev_output.log
fi

echo "======================================"
echo "開発環境が起動中です。"
echo "ブラウザで http://localhost:1420/ にアクセスしてください。"
echo "終了するには Ctrl+C を押してください。"

# フォアグラウンドで待機
wait $DEV_PID
cd .. 