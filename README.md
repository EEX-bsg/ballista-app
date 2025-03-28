# Ballista

![Ballista logo](docs/images/logo.png)

Ballista（バリスタ）は、ゲーム「Besiege」向けのモッド管理ツールです。複数のモッドを「プリセット」として管理し、簡単に切り替えて起動できるランチャーアプリケーションを提供します。ユーザーはプリセットを作成・共有でき、モッド管理の煩わしさを解消します。

## ✨ 主な機能

- 🎮 **Modding.xml自動管理**: Besiegeのモッド設定ファイルを自動で解析・編集
- 📦 **プリセット機能**: 複数のモッドをプリセットとして保存・切り替え
- 🔄 **プリセット共有**: コミュニティでプリセットを共有・ダウンロード
- 🔍 **Mod検出**: SteamワークショップとローカルフォルダからMod自動検出
- 📊 **使用統計**: プレイ時間やモッド使用状況の記録（オプション）
- 🌐 **マルチプラットフォーム対応**: Windows, macOS, Linux対応

## 🏗️ システムアーキテクチャ

Ballistaは2つの主要コンポーネントで構成されています：

1. **デスクトップアプリケーション** (Tauri + Vue + Rust)
   - ローカルファイル操作
   - Modding.xml解析・編集
   - UIとユーザー操作
   - SQLiteローカルデータベース

2. **バックエンドサーバー** (Bun + Elysia.js)
   - RESTful API提供
   - ユーザー認証
   - プリセット共有機能
   - 課金処理
   - 統計データ管理
   - PostgreSQLデータベース

## 🛠️ 技術スタック

### フロントエンド（デスクトップアプリ）
- [Tauri](https://tauri.app/) - クロスプラットフォームアプリケーションフレームワーク
- [Vue.js](https://vuejs.org/) - UIフレームワーク
- [TypeScript](https://www.typescriptlang.org/) - 型付きJavaScript
- [Vuetify](https://vuetifyjs.com/) - Vueコンポーネントライブラリ
- [Tailwind CSS](https://tailwindcss.com/) - ユーティリティファーストCSSフレームワーク
- [Rust](https://www.rust-lang.org/) - ネイティブ機能実装

### バックエンド
- [Bun](https://bun.sh/) - JavaScript/TypeScriptランタイム
- [Elysia.js](https://elysiajs.com/) - Bunに最適化されたWebフレームワーク
- [Drizzle ORM](https://orm.drizzle.team/) - TypeScriptファーストORM
- [PostgreSQL](https://www.postgresql.org/) - データベース
- [Stripe](https://stripe.com/) - 決済プラットフォーム
- [Docker](https://www.docker.com/) - コンテナ化

## 🚀 開発環境のセットアップ

Ballistaの開発環境セットアップはフロントエンド（Tauriアプリ）とバックエンド（APIサーバー）で異なります。

### 前提条件

#### フロントエンド（Tauriアプリ）
- [Node.js](https://nodejs.org/) (v20.x以上)
- [Rust](https://www.rust-lang.org/tools/install) (最新の安定版)
- プラットフォーム固有の依存関係:
  - **Windows**: Visual Studio Build Tools, WebView2
  - **macOS**: Xcode Command Line Tools (`xcode-select --install`)
  - **Linux**: GTK, WebKit2GTK (`sudo apt install libgtk-3-dev webkit2gtk-4.0 libappindicator3-dev`)

#### バックエンド（APIサーバー）
- [Docker](https://www.docker.com/get-started) と [Docker Compose](https://docs.docker.com/compose/install/)
- [Git](https://git-scm.com/)

### セットアップ手順

1. リポジトリをクローンします：

```bash
git clone https://github.com/yourusername/ballista.git
cd ballista
```

2. フロントエンド開発環境をセットアップします：

```bash
cd frontend
npm install
```

3. バックエンド開発環境（Docker）を起動します：

```bash
cd ../
docker-compose -f docker-compose.backend.yml up -d
```

これにより、以下のサービスが起動します：
- バックエンドAPI（`http://localhost:8000`）
- PostgreSQLデータベース

### 開発サーバーの起動

1. フロントエンド開発サーバーを起動します：

```bash
cd frontend
npm run dev
```

2. バックエンドAPIのログを確認します：

```bash
docker-compose -f docker-compose.backend.yml logs -f backend
```

## 🏭 ビルドプロセス

### ローカルでのビルド

1. フロントエンドをビルドします：

```bash
cd frontend
npm run build
```

2. Tauriアプリをビルドします：

```bash
cd frontend
npm run tauri build
```

バイナリは `frontend/src-tauri/target/release` ディレクトリに生成されます。

### CI/CD

プロジェクトは以下のGitHub Actionsワークフローを使用しています：

- **テスト**: プルリクエストが作成されるたびに自動的にテストが実行されます
- **ビルド**: メインブランチにコードがプッシュされると、Windows、macOS、Linux向けのバイナリが生成されます
- **デプロイ**: リリースタグが作成されると、自動的にバイナリがリリースページにアップロードされます

## 🌐 本番環境デプロイ

バックエンドサーバーは、本番環境でもDockerコンテナとして実行されます。以下に本番環境へのデプロイ手順を説明します。

### バックエンド本番環境の構築

1. レンタルサーバー（VPS）にSSH接続し、Dockerとdocker-composeをインストールします：

```bash
# Dockerインストール
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Docker Composeインストール
sudo curl -L "https://github.com/docker/compose/releases/download/v2.18.1/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose
```

2. プロジェクトの`production`ディレクトリをサーバーにコピー：

```bash
scp -r production user@your-server-ip:/path/to/ballista
```

3. 環境変数ファイルを設定します：

```bash
cd /path/to/ballista
cp .env.example .env
nano .env  # 本番環境用の値を設定
```

4. 本番環境用docker-compose.ymlを使ってコンテナを起動：

```bash
docker-compose -f docker-compose.production.yml up -d
```

### 環境変数

本番環境では以下の環境変数を`.env`ファイルに設定する必要があります：

```
# データベース設定
DB_PASSWORD=strong-password-here
POSTGRES_USER=ballista
POSTGRES_DB=ballista

# JWT認証
JWT_SECRET=your-secret-key

# Stripe決済
STRIPE_SECRET_KEY=sk_live_xxxxxxxxxxxx
STRIPE_WEBHOOK_SECRET=whsec_xxxxxxxxxxxx

# その他設定
NODE_ENV=production
PORT=8000
```

### SSL/TLS設定

本番環境ではNginxをリバースプロキシとして使用し、Let's Encryptで取得した証明書でSSL/TLSを有効化します：

```bash
# Nginxの設定例
server {
    listen 80;
    server_name api.ballista-app.com;
    
    location / {
        return 301 https://$host$request_uri;
    }
}

server {
    listen 443 ssl;
    server_name api.ballista-app.com;
    
    ssl_certificate /etc/letsencrypt/live/api.ballista-app.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/api.ballista-app.com/privkey.pem;
    
    location / {
        proxy_pass http://localhost:8000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }
}
```

### 自動デプロイ

GitHubリポジトリへのプッシュをトリガーに自動デプロイするための設定：

```yaml
# .github/workflows/deploy.yml
name: Deploy to Production

on:
  push:
    branches: [ main ]
    paths:
      - 'backend/**'
      - '.github/workflows/deploy.yml'

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Deploy to Server
        uses: appleboy/ssh-action@master
        with:
          host: ${{ secrets.SERVER_HOST }}
          username: ${{ secrets.SERVER_USER }}
          key: ${{ secrets.SSH_PRIVATE_KEY }}
          script: |
            cd /path/to/ballista
            git pull
            docker-compose -f docker-compose.production.yml down
            docker-compose -f docker-compose.production.yml up -d --build
```

### データベースバックアップ

定期的なデータベースバックアップの設定：

```bash
# /etc/cron.daily/ballista-backup
#!/bin/bash
cd /path/to/ballista
docker-compose -f docker-compose.production.yml exec -T db pg_dump -U ballista -d ballista -F c > /backups/ballista_$(date +\%Y\%m\%d_\%H\%M).dump
find /backups -name "ballista_*.dump" -mtime +7 -delete
```

## 📁 プロジェクト構造

```
ballista/
├── frontend/           # フロントエンドアプリケーション（Vue.js）
│   ├── src/            # ソースコード
│   ├── src-tauri/      # Tauriネイティブコード（Rust）
│   └── ...
├── backend/            # バックエンドサーバー（Bun + Elysia.js）
│   ├── src/            # ソースコード
│   ├── migrations/     # データベースマイグレーション
│   └── ...
├── docs/               # ドキュメント
├── docker-compose.yml  # Docker Compose設定
├── .github/            # GitHub Actions設定
└── README.md           # このファイル
```

## 📄 ライセンス

このプロジェクトは[Mozilla Public License Version 2.0](LICENSE)の下で公開されています。

## 🤝 貢献

Ballistaはまだ開発の初期段階です。バグ報告、機能リクエスト、プルリクエストなどの貢献を歓迎します。

## 📞 連絡先

- Discord: [Ballistaコミュニティサーバー](https://discord.gg/your-invite-link)
- GitHub: [Issues](https://github.com/yourusername/ballista/issues)

## 作者

[Your Name]

## 謝辞

- [Besiege](https://www.besiege.spiderlinggames.co.uk/) - ゲーム本体
- [Tauri](https://tauri.app/) - デスクトップアプリケーションフレームワーク
- [Vue.js](https://vuejs.org/) - フロントエンドフレームワーク
- [Vuetify](https://vuetifyjs.com/) - Material Designコンポーネントライブラリ
