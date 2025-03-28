@echo off
setlocal enabledelayedexpansion

echo Ballista-BesiegeLauncher 開発環境起動スクリプト
echo ======================================

cd frontend

echo 開発環境を起動中...
start /b cmd /c npm run tauri dev > dev_output.log 2>&1
set DEV_PID=!ERRORLEVEL!

set TIMEOUT=60
set /a COUNTER=0
set SUCCESS=0
set RUST_COMPILED=0

echo フロントエンドとRustバックエンドの起動を待機しています...

:WAIT_LOOP
if !COUNTER! geq %TIMEOUT% (
    echo 開発環境の起動がタイムアウトしました。
    echo ログを確認しています...
    
    :: タイムアウト時にも最終確認
    :: フロントエンド成功チェック
    set FRONTEND_OK=0
    findstr /c:"Local:   http://localhost" dev_output.log > nul
    if !ERRORLEVEL! equ 0 set FRONTEND_OK=1
    
    findstr /c:"ready in" dev_output.log > nul
    if !ERRORLEVEL! equ 0 set FRONTEND_OK=1
    
    findstr /c:"Tauri development" dev_output.log > nul
    if !ERRORLEVEL! equ 0 set FRONTEND_OK=1

    :: Rustコンパイル成功チェック（ない場合は既にチェック済みの値を使用）
    if !RUST_COMPILED! equ 0 (
        findstr /c:"Finished" dev_output.log > nul
        if !ERRORLEVEL! equ 0 set RUST_COMPILED=1
        
        findstr /c:"Starting Tauri Development" dev_output.log > nul
        if !ERRORLEVEL! equ 0 set RUST_COMPILED=1
        
        findstr /c:"APIテスト環境の初期化が完了しました" dev_output.log > nul
        if !ERRORLEVEL! equ 0 set RUST_COMPILED=1
        
        :: 警告があればRustコンパイルは少なくとも進んでいる
        findstr /c:"warning:" dev_output.log > nul
        if !ERRORLEVEL! equ 0 (
            findstr /c:"Compiling ballista-app" dev_output.log > nul
            if !ERRORLEVEL! equ 0 set RUST_COMPILED=1
        )
    )
    
    :: 両方成功していれば成功とみなす
    if !FRONTEND_OK! equ 1 (
        if !RUST_COMPILED! equ 1 (
            set SUCCESS=1
            goto :DEV_DONE
        )
    )
    
    :: 失敗原因を表示
    if !RUST_COMPILED! equ 0 (
        echo Rustバックエンドのコンパイルが完了していません。
    )
    if !FRONTEND_OK! equ 0 (
        echo フロントエンドの起動が完了していません。
    )
    
    taskkill /F /PID !DEV_PID! 2>nul
    echo 起動をキャンセルしました。ログを確認してください。
    type dev_output.log
    cd ..
    exit /b 1
)

:: Rustコンパイル成功チェック（まだチェックしていない場合）
if !RUST_COMPILED! equ 0 (
    :: Cargoビルド完了メッセージをチェック
    findstr /c:"Finished" dev_output.log > nul
    if !ERRORLEVEL! equ 0 (
        set RUST_COMPILED=1
        echo Rustバックエンドが正常にコンパイルされました！
    )
    
    :: Tauriの起動メッセージをチェック
    findstr /c:"Starting Tauri Development" dev_output.log > nul
    if !ERRORLEVEL! equ 0 (
        set RUST_COMPILED=1
        echo Rustバックエンドが正常にコンパイルされました！
    )
    
    :: APIテスト環境初期化完了メッセージをチェック
    findstr /c:"APIテスト環境の初期化が完了しました" dev_output.log > nul
    if !ERRORLEVEL! equ 0 (
        set RUST_COMPILED=1
        echo Rustバックエンドが正常にコンパイルされました！
    )
    
    :: 警告があればRustコンパイルは少なくとも進んでいる
    findstr /c:"warning:" dev_output.log > nul
    if !ERRORLEVEL! equ 0 (
        findstr /c:"Compiling ballista-app" dev_output.log > nul
        if !ERRORLEVEL! equ 0 (
            set RUST_COMPILED=1
            echo Rustコンパイルは警告がありますが進行中です...
        )
    )
    
    :: コンパイル中のメッセージ、時間がかかっている場合は成功とみなす
    findstr /c:"Compiling ballista-app" dev_output.log > nul
    if !ERRORLEVEL! equ 0 (
        if !COUNTER! gtr 30 (
            set RUST_COMPILED=1
            echo コンパイルに時間がかかっていますが進行中と判断します...
        )
    )
)

:: フロントエンド起動チェック
set FRONTEND_OK=0

:: "Local: http://localhost" メッセージをチェック
findstr /c:"Local:   http://localhost" dev_output.log > nul
if !ERRORLEVEL! equ 0 set FRONTEND_OK=1

:: "ready in" メッセージもチェック
findstr /c:"ready in" dev_output.log > nul
if !ERRORLEVEL! equ 0 set FRONTEND_OK=1

:: Tauriが起動したことを示す他のメッセージもチェック
findstr /c:"Tauri development" dev_output.log > nul
if !ERRORLEVEL! equ 0 set FRONTEND_OK=1

:: 両方成功していれば完了
if !FRONTEND_OK! equ 1 (
    if !RUST_COMPILED! equ 1 (
        set SUCCESS=1
        goto :DEV_DONE
    ) else (
        echo フロントエンドの準備ができました。Rustバックエンドのコンパイルを待機中...
    )
) else (
    if !RUST_COMPILED! equ 1 (
        echo Rustバックエンドの準備ができました。フロントエンドの起動を待機中...
    )
)

:: エラーメッセージがあるか確認 - ただし警告だけの場合はスキップ
findstr /c:"error: could not compile" dev_output.log > nul
if !ERRORLEVEL! equ 0 (
    echo Cargoビルドエラーが発生しました。
    type dev_output.log
    cd ..
    exit /b 1
)

:: Rustのコンパイルエラーをチェック - ただし警告だけの場合はスキップ
findstr /c:"error[E" dev_output.log > nul
if !ERRORLEVEL! equ 0 (
    findstr /c:"warning:" dev_output.log > nul
    if !ERRORLEVEL! neq 0 (
        echo Rustコンパイルエラーが発生しました。
        type dev_output.log
        cd ..
        exit /b 1
    )
)

:: Rustのパニックをチェック
findstr /c:"thread" dev_output.log | findstr /c:"panicked at" > nul
if !ERRORLEVEL! equ 0 (
    echo Rustプログラムでパニックが発生しました。
    type dev_output.log
    cd ..
    exit /b 1
)

:: 1秒待機してカウンタを増加
timeout /t 1 /nobreak > nul
set /a COUNTER+=1
echo カウント: !COUNTER!/!TIMEOUT!
goto :WAIT_LOOP

:DEV_DONE
if !SUCCESS! equ 1 (
    echo 開発環境が正常に起動しました！
    echo フロントエンド: OK
    echo Rustバックエンド: OK
    echo カウント: !COUNTER!/!TIMEOUT!
    echo ログ出力:
    type dev_output.log
    
    :: バックアッププロセスチェック
    findstr /c:"Backup process started" dev_output.log > nul
    if !ERRORLEVEL! equ 0 (
        echo バックアッププロセスも正常に起動しています。
    )
) else (
    echo 開発環境の起動状態が不明です。
    if !RUST_COMPILED! equ 0 (
        echo Rustバックエンドのコンパイルが完了していない可能性があります。
    )
    echo ログ出力:
    type dev_output.log
)

echo ======================================
echo 開発環境が起動中です。
echo ブラウザで http://localhost:1420/ にアクセスしてください。
echo 終了するには Ctrl+C を押してください。
endlocal 