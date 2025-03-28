# Ballista 開発/ビルド環境構築ガイド

このドキュメントは、READMEに記載された開発環境とビルドプロセスを実際に構築するための詳細手順を解説します。このガイドラインに従って必要なファイルを作成することで、開発環境の構築から本番環境へのデプロイまでを実現できます。

## 目次

1. [プロジェクト初期構築](#1-プロジェクト初期構築)
2. [開発環境の設定](#2-開発環境の設定)
3. [本番環境用Dockerの設定](#3-本番環境用dockerの設定)
4. [フロントエンド開発環境の構築](#4-フロントエンド開発環境の構築)
5. [バックエンド開発環境の構築](#5-バックエンド開発環境の構築)
6. [GitHub Actionsによるビルド](#6-github-actionsによるビルド)
7. [必要なファイルとその役割](#7-必要なファイルとその役割)
8. [考慮事項と注意点](#8-考慮事項と注意点)

## 1. プロジェクト初期構築

はじめに、以下のディレクトリ構造を作成します：

```
ballista/
├── frontend/                # フロントエンドアプリケーション
├── backend/                 # バックエンドサーバー
├── docs/                    # ドキュメント
│   └── images/              # 画像ファイル
├── docker/                  # Docker関連ファイル（バックエンド用）
│   ├── backend/             # バックエンド用Docker設定
│   └── production/          # 本番環境用Docker設定
├── .github/                 # GitHub関連ファイル
│   └── workflows/           # GitHub Actionsワークフロー定義
├── .gitignore               # Gitの除外設定
└── docker-compose.backend.yml # バックエンド開発環境用Docker Compose設定
```

### .gitignoreの作成

```
# .gitignore

# 依存関係
node_modules/
target/
dist/

# ビルド成果物
/frontend/dist/
/frontend/src-tauri/target/

# 環境設定
.env
.env.local
.env.*.local

# ログ
*.log
npm-debug.log*
yarn-debug.log*
yarn-error.log*

# エディタ設定
.idea/
.vscode/
*.swp
*.swo

# OS固有のファイル
.DS_Store
Thumbs.db

# バックアップファイル
*.bak
*.backup

# データベース
*.sqlite
/data
```

## 2. 開発環境の設定

### 2.1 バックエンド開発環境用Docker Compose

```yaml
# docker-compose.backend.yml
version: '3.9'
services:
  # バックエンド開発環境 (Bun + Elysia.js)
  backend:
    build: 
      context: ./docker/backend
      dockerfile: Dockerfile.dev
    volumes:
      - ./backend:/app
      - /app/node_modules
    ports:
      - "8000:8000"
    environment:
      - NODE_ENV=development
      - DATABASE_URL=postgres://ballista:dev_password@db:5432/ballista_dev
      - JWT_SECRET=dev_secret_key
    depends_on:
      - db
  
  # データベース
  db:
    image: postgres:16
    environment:
      POSTGRES_PASSWORD: dev_password
      POSTGRES_USER: ballista
      POSTGRES_DB: ballista_dev
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

volumes:
  postgres_data:
```

### 2.2 バックエンド用Dockerfile

```dockerfile
# docker/backend/Dockerfile.dev
FROM oven/bun:1.1

WORKDIR /app

COPY package.json ./
RUN bun install

# ポート公開
EXPOSE 8000

# 開発サーバー起動コマンド
CMD ["bun", "run", "dev"]
```

## 3. 本番環境用Dockerの設定

### 3.1 docker-compose.production.yml

```yaml
# docker-compose.production.yml
version: '3.9'
services:
  # バックエンドAPIサーバー
  backend:
    build:
      context: ./docker/backend
      dockerfile: Dockerfile.prod
    restart: always
    depends_on:
      - db
    environment:
      - NODE_ENV=production
      - DATABASE_URL=postgres://ballista:${DB_PASSWORD}@db:5432/ballista
      - JWT_SECRET=${JWT_SECRET}
      - STRIPE_SECRET_KEY=${STRIPE_SECRET_KEY}
      - PORT=8000
    ports:
      - "127.0.0.1:8000:8000"  # localhostからのみアクセス可能
    
  # Nginxリバースプロキシ  
  web:
    image: nginx:stable-alpine
    restart: always
    depends_on:
      - backend
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./docker/production/nginx.conf:/etc/nginx/conf.d/default.conf
      - ./docker/production/certs:/etc/nginx/certs
      - ./docker/production/www:/var/www/html
      
  # PostgreSQLデータベース
  db:
    image: postgres:16
    restart: always
    environment:
      POSTGRES_PASSWORD: ${DB_PASSWORD}
      POSTGRES_USER: ballista
      POSTGRES_DB: ballista
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./backups:/backups
    ports:
      - "127.0.0.1:5432:5432"  # localhostからのみアクセス可能
      
  # バックアップサービス
  backup:
    image: postgres:14
    restart: always
    depends_on:
      - db
    volumes:
      - ./backups:/backups
    environment:
      PGPASSWORD: ${DB_PASSWORD}
    command: |
      /bin/bash -c 'while true; do
        pg_dump -h db -U ballista -d ballista -F c -f /backups/backup_`date +%Y%m%d_%H%M`.dump;
        find /backups -name "backup_*.dump" -mtime +7 -delete;
        sleep 86400;
      done'

volumes:
  postgres_data:
```

### 3.2 バックエンド本番用Dockerfile

```dockerfile
# docker/backend/Dockerfile.prod
FROM oven/bun:1.1 AS builder

WORKDIR /app

# 依存関係のインストール
COPY package.json bun.lockb ./
RUN bun install --production

# ソースコードのコピーとビルド
COPY . .
RUN bun run build

# 実行イメージ
FROM oven/bun:1.1

WORKDIR /app

# ビルド済みのアプリとproduction依存関係のコピー
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/package.json ./

# 実行ユーザーの設定
USER bun

# ヘルスチェック
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8000/health || exit 1

# ポートの公開
EXPOSE 8000

# アプリケーションの起動
CMD ["bun", "run", "start"]
```

### 3.3 Nginx設定ファイル

```nginx
# docker/production/nginx.conf
server {
    listen 80;
    server_name api.ballista-app.com;
    
    # HTTP -> HTTPS リダイレクト
    location / {
        return 301 https://$host$request_uri;
    }
    
    # Let's Encrypt証明書の更新用
    location /.well-known/acme-challenge/ {
        root /var/www/html;
    }
}

server {
    listen 443 ssl;
    server_name api.ballista-app.com;
    
    # SSL設定
    ssl_certificate /etc/nginx/certs/fullchain.pem;
    ssl_certificate_key /etc/nginx/certs/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_prefer_server_ciphers on;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:DHE-RSA-AES128-GCM-SHA256:DHE-RSA-AES256-GCM-SHA384;
    
    # セキュリティヘッダー
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Content-Security-Policy "default-src 'self'; script-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; font-src 'self' data:; connect-src 'self';" always;
    
    # プロキシ設定
    location / {
        proxy_pass http://backend:8000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
        
        # タイムアウト設定
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }
}
```

## 4. フロントエンド開発環境の構築

### 4.1 必要な開発ツールのインストール

#### Windows

1. **Node.js**のインストール (v20.x以上)
   - [Node.js公式サイト](https://nodejs.org/)からLTS版をダウンロードしてインストール

2. **Rust**のインストール
   - [rustup.rs](https://rustup.rs/)からrustupをダウンロードしてインストール
   - インストール後は次のコマンドでRustが正しくインストールされたか確認:
   ```
   rustc --version
   cargo --version
   ```

3. **Visual Studio Build Tools**のインストール
   - [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)をダウンロード
   - インストール時に「C++によるデスクトップ開発」ワークロードを選択

4. **WebView2**のインストール
   - 最新のEdgeブラウザがインストールされている場合は不要
   - それ以外の場合は[WebView2ランタイム](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)をインストール

#### macOS

1. **Xcode Command Line Tools**のインストール
   ```bash
   xcode-select --install
   ```

2. **Node.js**のインストール (v20.x以上)
   - [Node.js公式サイト](https://nodejs.org/)からLTS版をダウンロード
   - または、Homebrewを使用してインストール:
   ```bash
   brew install node
   ```

3. **Rust**のインストール
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

#### Linux (Ubuntu/Debian)

1. **必要なライブラリのインストール**
   ```bash
   sudo apt update
   sudo apt install -y libgtk-3-dev webkit2gtk-4.0 libappindicator3-dev librsvg2-dev libsoup-2.4-dev libjavascriptcoregtk-4.0-dev build-essential curl wget libssl-dev pkg-config
   ```

2. **Node.js**のインストール (v20.x以上)
   ```bash
   curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
   sudo apt install -y nodejs
   ```

3. **Rust**のインストール
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

### 4.2 フロントエンドのセットアップ

フロントエンドのpackage.jsonは以下のようになります：

```json
{
  "name": "ballista-frontend",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc --noEmit && vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.4.0",
    "vue": "^3.5.13",
    "vue-router": "^4.5.0",
    "vuetify": "^3.7.19",
    "axios": "^1.8.4",
    "pinia": "^3.0.1"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.4.0",
    "@types/node": "^18.0.0",
    "@vitejs/plugin-vue": "^5.2.3",
    "autoprefixer": "^10.4.14",
    "postcss": "^8.4.27",
    "tailwindcss": "^3.3.3",
    "typescript": "^5.8.2",
    "vite": "^6.2.3",
    "vue-tsc": "^2.2.8"
  }
}
```

### 4.3 Tauriの設定ファイル (src-tauri/tauri.conf.json)

```json
{
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devPath": "http://localhost:3000",
    "distDir": "../dist",
    "withGlobalTauri": true
  },
  "package": {
    "productName": "Ballista",
    "version": "0.1.0"
  },
  "tauri": {
    "allowlist": {
      "all": false,
      "shell": {
        "all": false,
        "open": true
      },
      "dialog": {
        "all": true
      },
      "fs": {
        "all": false,
        "readFile": true,
        "writeFile": true,
        "readDir": true,
        "createDir": true,
        "exists": true
      },
      "path": {
        "all": true
      },
      "process": {
        "all": false,
        "exit": true,
        "relaunch": true
      },
      "window": {
        "all": true
      }
    },
    "bundle": {
      "active": true,
      "targets": "all",
      "identifier": "com.ballista.app",
      "icon": [
        "icons/32x32.png",
        "icons/128x128.png",
        "icons/128x128@2x.png",
        "icons/icon.icns",
        "icons/icon.ico"
      ],
      "resources": [],
      "copyright": "© 2023-2025 Ballista",
      "category": "Utility",
      "shortDescription": "Besiege mod launcher",
      "longDescription": "Ballista is a mod launcher for Besiege that simplifies mod management."
    },
    "security": {
      "csp": "default-src 'self'; img-src 'self' asset: https://asset.localhost; style-src 'self' 'unsafe-inline'"
    },
    "updater": {
      "active": true,
      "dialog": true,
      "endpoints": [
        "https://releases.ballista-app.com/{{target}}/{{current_version}}"
      ],
      "pubkey": ""
    },
    "windows": [
      {
        "title": "Ballista",
        "width": 1200,
        "height": 800,
        "minWidth": 800,
        "minHeight": 600,
        "resizable": true,
        "fullscreen": false,
        "decorations": true,
        "center": true
      }
    ]
  }
}
```

### 4.4 Rustファイル (src-tauri/src/main.rs)

```rust
// src-tauri/src/main.rs
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod mod_manager;

use tauri::{CustomMenuItem, Menu, MenuItem, Submenu};

fn main() {
    // メニューの作成
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let close = CustomMenuItem::new("close".to_string(), "Close");
    let file_menu = Submenu::new("File", Menu::new().add_item(quit));
    let edit_menu = Submenu::new(
        "Edit",
        Menu::new()
            .add_native_item(MenuItem::Copy)
            .add_native_item(MenuItem::Paste)
            .add_native_item(MenuItem::Cut)
            .add_item(close),
    );

    let menu = Menu::new()
        .add_submenu(file_menu)
        .add_submenu(edit_menu);

    // アプリケーションの起動
    tauri::Builder::default()
        .menu(menu)
        .on_menu_event(|event| match event.menu_item_id() {
            "quit" => {
                std::process::exit(0);
            }
            "close" => {
                event.window().close().unwrap();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            mod_manager::scan_mods,
            mod_manager::update_modding_xml,
            mod_manager::get_local_presets,
            mod_manager::create_preset,
            mod_manager::delete_preset,
            mod_manager::apply_preset,
            mod_manager::launch_besiege
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 4.5 Modding.xml管理モジュール (src-tauri/src/mod_manager.rs)

```rust
// src-tauri/src/mod_manager.rs
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct Mod {
    uuid: String,
    name: String,
    version: Option<String>,
    source: String,
    workshop_id: Option<String>,
    local_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Preset {
    id: u32,
    name: String,
    mods: Vec<String>, // Mod UUIDs
    category: Option<String>,
    tags: Vec<String>,
    last_played: Option<String>,
    total_play_time: u64,
}

#[tauri::command]
pub fn scan_mods(workshop_path: &str) -> Vec<Mod> {
    // 実際の実装ではSteamワークショップのパスを検索してModを見つける
    println!("Scanning mods at: {}", workshop_path);
    
    // テスト用のダミーデータ
    vec![
        Mod {
            uuid: "uuid1".to_string(),
            name: "Sample Mod 1".to_string(),
            version: Some("1.0.0".to_string()),
            source: "workshop".to_string(),
            workshop_id: Some("12345".to_string()),
            local_path: None,
        },
        Mod {
            uuid: "uuid2".to_string(),
            name: "Sample Mod 2".to_string(),
            version: Some("2.1.0".to_string()),
            source: "local".to_string(),
            workshop_id: None,
            local_path: Some("C:/mods/sample2".to_string()),
        },
    ]
}

#[tauri::command]
pub fn update_modding_xml(path: &str, active_mods: Vec<String>, disabled_mods: Vec<String>) -> bool {
    println!("Updating Modding.xml at: {}", path);
    println!("Active mods: {:?}", active_mods);
    println!("Disabled mods: {:?}", disabled_mods);
    
    // 実際の実装ではModding.xmlファイルを更新する
    true
}

#[tauri::command]
pub fn get_local_presets() -> Vec<Preset> {
    // SQLiteからローカルプリセットを読み込む
    // テスト用のダミーデータ
    vec![
        Preset {
            id: 1,
            name: "Default Preset".to_string(),
            mods: vec!["uuid1".to_string(), "uuid2".to_string()],
            category: Some("General".to_string()),
            tags: vec!["basic".to_string()],
            last_played: Some("2023-07-15T10:30:00Z".to_string()),
            total_play_time: 3600,
        }
    ]
}

#[tauri::command]
pub fn create_preset(name: &str, mods: Vec<String>, category: Option<String>, tags: Vec<String>) -> Preset {
    println!("Creating preset: {}", name);
    
    // 実際の実装ではSQLiteにプリセットを保存する
    Preset {
        id: 2,
        name: name.to_string(),
        mods,
        category,
        tags,
        last_played: None,
        total_play_time: 0,
    }
}

#[tauri::command]
pub fn delete_preset(id: u32) -> bool {
    println!("Deleting preset: {}", id);
    
    // 実際の実装ではSQLiteからプリセットを削除する
    true
}

#[tauri::command]
pub fn apply_preset(preset_id: u32, modding_xml_path: &str) -> bool {
    println!("Applying preset {} to Modding.xml at: {}", preset_id, modding_xml_path);
    
    // 実際の実装ではプリセットのModリストを読み込み、Modding.xmlを更新する
    true
}

#[tauri::command]
pub fn launch_besiege(path: &str, args: Vec<String>) -> bool {
    println!("Launching Besiege at: {} with args: {:?}", path, args);
    
    // 実際の実装ではBesiegeを起動する
    // let status = Command::new(path).args(&args).spawn();
    // status.is_ok()
    true
}
```

### 4.6 フロントエンド開発サーバーの起動

```bash
cd frontend
npm install
npm run dev
```

Tauriの開発サーバーが起動します。これにより、フロントエンドのVueアプリケーションがTauriアプリケーションとして実行されます。

## 5. バックエンド開発環境の構築

バックエンド開発環境はDockerを使用します。

### 5.1 環境変数ファイル (.env.example)

```
# 開発環境設定
NODE_ENV=development
PORT=8000

# データベース設定
DATABASE_URL=postgres://ballista:dev_password@db:5432/ballista_dev

# JWT認証
JWT_SECRET=dev_secret_key

# Stripe決済（開発用キー）
STRIPE_SECRET_KEY=sk_test_xxxxxxxxxxxx
STRIPE_WEBHOOK_SECRET=whsec_xxxxxxxxxxxx

# アプリケーション設定
APP_URL=http://localhost:3000
API_URL=http://localhost:8000
```

### 5.2 メインアプリケーションファイル (src/index.ts)

```typescript
// src/index.ts
import { Elysia } from "elysia";
import { cors } from "@elysiajs/cors";
import { jwt } from "@elysiajs/jwt";
import { swagger } from "@elysiajs/swagger";
import { db } from "./db";
import { authRoutes } from "./routes/auth";
import { presetsRoutes } from "./routes/presets";
import { usersRoutes } from "./routes/users";
import { paymentsRoutes } from "./routes/payments";
import { statsRoutes } from "./routes/stats";

// ポート設定
const PORT = process.env.PORT ? parseInt(process.env.PORT) : 8000;

// アプリケーション設定
const app = new Elysia()
  // ミドルウェア
  .use(cors({
    origin: process.env.NODE_ENV === 'production' 
      ? ['https://ballista-app.com', 'https://www.ballista-app.com'] 
      : '*',
    methods: ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'OPTIONS'],
    allowedHeaders: ['Content-Type', 'Authorization'],
    credentials: true,
  }))
  .use(jwt({
    name: 'jwt',
    secret: process.env.JWT_SECRET || 'dev_secret',
  }))
  .use(swagger({
    documentation: {
      info: {
        title: 'Ballista API',
        version: '0.1.0',
      },
      tags: [
        { name: 'auth', description: 'Authentication endpoints' },
        { name: 'presets', description: 'Preset management' },
        { name: 'users', description: 'User management' },
        { name: 'payments', description: 'Payment processing' },
        { name: 'stats', description: 'Usage statistics' },
      ],
    },
  }))
  
  // ルート
  .use(authRoutes)
  .use(presetsRoutes)
  .use(usersRoutes)
  .use(paymentsRoutes)
  .use(statsRoutes)
  
  // ヘルスチェック
  .get('/health', () => ({ 
    status: 'ok', 
    timestamp: new Date().toISOString(),
    env: process.env.NODE_ENV
  }))
  
  // グローバルエラーハンドラー
  .onError(({ code, error, set }) => {
    console.error(`Error [${code}]:`, error);
    
    if (code === 'NOT_FOUND') {
      set.status = 404;
      return { error: 'Not Found', status: 404 };
    }
    
    if (code === 'VALIDATION') {
      set.status = 400;
      return { error: 'Validation Error', details: error.message, status: 400 };
    }
    
    if (code === 'UNKNOWN') {
      set.status = 500;
      return { error: 'Internal Server Error', status: 500 };
    }
    
    return { error: error.message, status: set.status };
  })
  
  // サーバー起動
  .listen(PORT);

console.log(`🚀 Server running at ${app.server?.hostname}:${PORT}`);

// プロセス終了時のクリーンアップ
process.on('SIGINT', async () => {
  console.log('Shutting down gracefully...');
  // DB接続のクローズなど必要なクリーンアップ
  process.exit(0);
});

export type App = typeof app;
```

### 5.3 バックエンド環境の起動

```bash
docker-compose -f docker-compose.backend.yml up -d
```

これにより、バックエンドAPIサーバー（Bun + Elysia.js）とPostgreSQLデータベースが起動します。

### 5.4 バックエンドのログ確認

```bash
docker-compose -f docker-compose.backend.yml logs -f backend
```

### 5.5 バックエンドコンテナへの接続（必要な場合）

```bash
docker-compose -f docker-compose.backend.yml exec backend sh
```

## 6. GitHub Actionsによるビルド

### 6.1 Tauriアプリのビルドワークフロー (.github/workflows/build.yml)

```yaml
# .github/workflows/build.yml
name: Build and Release

on:
  push:
    branches: [ main ]
    paths:
      - 'frontend/**'
      - '.github/workflows/build.yml'
  pull_request:
    branches: [ main ]
  release:
    types: [created]

jobs:
  # バックエンド (Bun) のビルドとテスト
  backend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Bun
        uses: oven-sh/setup-bun@v1
        with:
          bun-version: 1.1.x
          
      - name: Install dependencies
        run: cd backend && bun install
        
      - name: Run tests
        run: cd backend && bun test
        
      - name: Build
        run: cd backend && bun run build

  # フロントエンド + Tauriアプリのビルド
  build-tauri:
    needs: backend
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        include:
          - os: ubuntu-latest
            platform: linux
          - os: windows-latest
            platform: windows
          - os: macos-latest
            platform: macos
        
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: frontend/package-lock.json
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
          
      - name: Install dependencies (Ubuntu)
        if: matrix.os == 'ubuntu-latest'
        run: |
          sudo apt-get update
          sudo apt-get install -y libgtk-3-dev webkit2gtk-4.0 libappindicator3-dev libwebkit2gtk-4.0-dev libssl-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev
      
      - name: Install dependencies (macOS)
        if: matrix.os == 'macos-latest'
        run: |
          brew install webkit2png
      
      - name: Install frontend dependencies
        run: |
          cd frontend
          npm ci
          
      - name: Build Tauri App
        uses: tauri-apps/tauri-action@v0.6.1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          projectPath: frontend
          tagName: v__VERSION__
          releaseName: 'Ballista v__VERSION__'
          releaseBody: 'See the release notes for details.'
          releaseDraft: true
          prerelease: false
```

### 6.2 バックエンドのデプロイワークフロー (.github/workflows/deploy-backend.yml)

```yaml
# .github/workflows/deploy-backend.yml
name: Deploy Backend

on:
  push:
    branches: [ main ]
    paths:
      - 'backend/**'
      - 'docker/backend/**'
      - 'docker/production/**'
      - '.github/workflows/deploy-backend.yml'

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Login to GitHub Container Registry
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
          
      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3
          
      - name: Build and push backend Docker image
        uses: docker/build-push-action@v5
        with:
          context: .
          file: ./docker/backend/Dockerfile.prod
          push: true
          tags: ghcr.io/${{ github.repository }}/ballista-backend:latest
          cache-from: type=gha
          cache-to: type=gha,mode=max
          
      - name: Deploy to Server
        uses: appleboy/ssh-action@master
        with:
          host: ${{ secrets.SERVER_HOST }}
          username: ${{ secrets.SERVER_USER }}
          key: ${{ secrets.SSH_PRIVATE_KEY }}
          script: |
            cd /var/www/ballista
            docker-compose pull
            docker-compose up -d
            docker system prune -af
```

## 7. 必要なファイルとその役割

以下は、このセットアップ全体に必要なファイルとその役割の一覧です：

### プロジェクトルート
- `README.md` - プロジェクト概要と使用方法
- `.gitignore` - Gitの除外設定
- `docker-compose.backend.yml` - バックエンド開発環境のDocker設定
- `docker-compose.production.yml` - 本番環境のDocker設定
- `.env.example` - 環境変数のテンプレート

### フロントエンド
- `frontend/package.json` - 依存関係と実行スクリプト
- `frontend/src-tauri/` - Tauriのネイティブコード
- `frontend/src/` - Vue.jsアプリケーションのソースコード
- `frontend/vite.config.ts` - Viteビルド設定

### バックエンド
- `backend/package.json` - 依存関係と実行スクリプト
- `backend/src/` - Elysiaアプリケーションのソースコード
- `backend/src/db/` - データベーススキーマと接続設定
- `backend/src/routes/` - APIエンドポイント

### Docker設定
- `docker/backend/Dockerfile.dev` - バックエンド開発環境
- `docker/backend/Dockerfile.prod` - バックエンド本番環境
- `docker/production/nginx.conf` - Nginx設定

### GitHub Actions
- `.github/workflows/build.yml` - アプリケーションビルドワークフロー
- `.github/workflows/deploy-backend.yml` - バックエンドデプロイワークフロー

## 8. 考慮事項と注意点

### 開発環境での注意点

1. **Tauriのクロスプラットフォーム開発**
   - WindowsでTauri開発を行う場合、Visual Studio Build ToolsとWebView2が必要
   - macOSではXcode Command Line Toolsが必要
   - Linuxでは複数のライブラリインストールが必要

2. **開発時のデバッガー利用**
   - VS CodeなどのIDEを使用する場合、Rustとの連携デバッグが可能
   - `.vscode/launch.json`を適切に設定すると便利

3. **環境変数の管理**
   - フロントエンドとバックエンドで異なる環境変数を適切に管理
   - `.env.local`ファイルをプロジェクトルートに作成し、ローカル環境用の設定を行う

### ビルドプロセスでの注意点

1. **クロスプラットフォームビルドの複雑さ**
   - ローカルでは自分の環境向けのビルドのみ可能
   - すべてのプラットフォーム向けビルドはGitHub Actionsで行う

2. **コード署名**
   - macOS/Windowsでの配布には、アプリの署名が必要
   - 実際のリリース前に署名証明書の取得を検討

3. **自動更新機能**
   - Tauriの自動更新機能を使う場合、更新サーバーの構築が必要
   - 署名付き更新ファイルを配布するインフラの準備

### 本番環境でのデプロイにおける注意点

1. **セキュリティ**
   - 常に最新のセキュリティアップデートを適用
   - Nginxの適切な設定（HTTPS、セキュリティヘッダー）
   - 定期的なセキュリティ監査

2. **バックアップ**
   - 重要なデータ（特にPostgreSQLデータベース）の定期バックアップ
   - バックアップの自動化と監視

3. **監視とロギング**
   - 適切な監視ツールの導入（UptimeRobot、Prometheusなど）
   - 集中ログ管理の検討（ELKスタック、Logtail）

4. **スケーリング**
   - ユーザー増加に備えたスケーリング戦略の検討
   - 将来的には複数インスタンス+ロードバランサーの構成も視野に