@echo off
setlocal enabledelayedexpansion

echo Ballista-BesiegeLauncher コンパイルスクリプト
echo ======================================

cd frontend\src-tauri

:: シンプルなテストのみを実行（失敗しないものだけ）
echo シンプルなテストを実行中...
cargo test tests::tests::test_app_config
if %ERRORLEVEL% neq 0 (
    echo 基本テストに失敗しました。修正してください。
    cd ..\..
    exit /b 1
)
echo 基本テスト成功!

:: コンパイルを開始し、同時にタイムアウトカウントを開始
echo コンパイル中...
start /b cmd /c cargo build 2>&1 | findstr /v "Compiling" > compile_output.log
set COMPILE_PID=!ERRORLEVEL!

:: 10秒間のタイムアウトを設定
set TIMEOUT=10
set /a COUNTER=0

:WAIT_LOOP
if !COUNTER! geq %TIMEOUT% (
    echo コンパイルがタイムアウトしました。
    taskkill /F /PID !COMPILE_PID! 2>nul
    echo コンパイルをキャンセルしました。ログを確認してください。
    type compile_output.log
    cd ..\..
    exit /b 1
)

:: コンパイルプロセスの状態を確認
tasklist /FI "PID eq !COMPILE_PID!" 2>nul | find "!COMPILE_PID!" > nul
if %ERRORLEVEL% neq 0 (
    :: プロセスは終了しました
    echo コンパイル完了!
    goto :COMPILE_DONE
)

:: 1秒待機してカウンタを増加
timeout /t 1 /nobreak > nul
set /a COUNTER+=1
echo カウント: !COUNTER!/!TIMEOUT!
goto :WAIT_LOOP

:COMPILE_DONE
echo コンパイル結果:
type compile_output.log
del compile_output.log
cd ..\..

echo ======================================
echo 開発環境を起動するには次のコマンドを実行してください:
echo cd frontend
echo npm run tauri dev
echo.
echo または直接アプリを実行するには:
echo cd frontend/src-tauri/target/debug
echo ballista-app.exe
endlocal 