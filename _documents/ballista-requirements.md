# Ballista アプリケーション要件定義書

## 1. プロジェクト概要

### 1.1 プロジェクト名
Ballista - Besiege向けモッドランチャー

### 1.2 プロジェクト概要
「Ballista」は、ゲーム「Besiege」のためのモッド管理ツールで、複数のモッドを「プリセット」として管理し、簡単に切り替えて起動できるランチャーアプリケーションです。ユーザーはプリセットを作成・共有でき、モッド管理の煩わしさを解消します。本アプリケーションはデスクトップクライアントとバックエンドサーバーで構成され、プリセット共有やユーザー管理機能を提供します。

### 1.3 目的
- Besiegeのモッド管理を簡素化する
- モッドの組み合わせをプリセットとして保存・切り替え可能にする
- コミュニティでのプリセット共有を実現する
- モッド環境の整合性を維持する

### 1.4 ターゲットユーザー
- Besiegeプレイヤー
- モッド制作者
- ゲームモディングコミュニティ

## 2. 技術スタックと全体アーキテクチャ

### 2.1 技術選定
- **アプリケーションフレームワーク**: Tauri
- **フロントエンド**:
  - 言語: TypeScript
  - フレームワーク: Vue.js
  - UIライブラリ: Vuetify
  - CSS: Tailwind CSS (検討中)
- **バックエンド**:
  - 言語: Rust (デスクトップアプリ側)
  - サーバーサイド: Bun + Elysia.js + Drizzle ORM
- **データベース**:
  - クライアント側: SQLite
  - サーバー側: PostgreSQL
- **決済プラットフォーム**: Stripe
- **コンテナ化**: Docker
- **CI/CD**: GitHub Actions

### 2.1.1 サーバーサイド技術スタック詳細

**Bun + Elysia.js + Drizzle ORM**

- **Bun**:
  - JavaScript/TypeScriptランタイムで、Node.jsより大幅に高速
  - ネイティブバンドラー、トランスパイラー、パッケージマネージャーを内蔵
  - WebSocketやFetch APIのネイティブサポート
  - Node.js互換性が高く、既存のnpmパッケージの99%以上が動作

- **Elysia.js**:
  - Bunに最適化された高速Webフレームワーク
  - タイプセーフなAPIスキーマ定義と自動バリデーション
  - デコレーターベースのエレガントな構文
  - 優れたパフォーマンスと低オーバーヘッド

- **Drizzle ORM**:
  - TypeScriptファーストのORM
  - 型安全なSQLクエリビルダー
  - マイグレーション管理とスキーマ生成
  - 軽量で高速な実行効率

### 2.2 システム構成
1. **デスクトップアプリケーション** (Tauri + Vue + Rust)
   - ローカルファイル操作
   - Modding.xml解析・編集
   - UI表示とユーザー操作
   - SQLiteローカルデータベース

2. **バックエンドサーバー** (Bun + Elysia.js)
   - RESTful API提供
   - ユーザー認証・認可
   - プリセット共有機能
   - 課金処理
   - 統計データ管理
   - PostgreSQLデータベース

### 2.3 システム間連携

- **クライアント-サーバー通信**:
  - JSON形式のREST API
  - エンドポイントごとの型定義共有（共通TypeScriptインターフェース）
  - JWTベースの認証

- **データ同期**:
  - プリセット変更時の差分同期
  - オフライン操作のキューイングと再接続時の同期
  - 衝突解決メカニズム

### 2.2 システム構成
1. **デスクトップアプリケーション** (Tauri + Vue + Rust)
   - ローカルファイル操作
   - Modding.xml解析・編集
   - UI表示とユーザー操作
   - SQLiteローカルデータベース

2. **バックエンドサーバー**
   - ユーザー認証
   - プリセット共有
   - 課金処理
   - 統計データ管理
   - PostgreSQLデータベース

## 3. 機能要件

### 3.1 中核機能: Modding.xml管理

#### 3.1.1 Modding.xml仕様
- Besiegeが最後に起動した時点でのModリストを保持
- 記録内容: UUID, ソース(ワークショップ/ローカル), ワークショップID, Modバージョン, Mod名
- 無効化するModのUUIDリストも含む

#### 3.1.2 Mod検出システム
- Steamワークショップの「Besiege」フォルダ(346010)内のMod.xmlを総検索
- 各Mod.xmlを解析してUUID, 名前, バージョンなどの情報を抽出
- 既存Modリストと比較して新規・更新Modを検出
- 自動検出タイミング: アプリ起動時、「Add mod」ボタン押下時

#### 3.1.3 独自Modデータベース
- アプリケーション独自のModリストをSQLiteで管理
- Besiegeを起動せずにMod情報を把握可能
- モッド情報の永続化と高速アクセス

#### 3.1.4 中間表現としてのJSONベースModリスト
- Vue UIとRust処理層の間にJSON形式のModリスト構造を導入
- Modding.xmlとは異なる形式だが、一対一で対応する仮想的な表現
- フロントエンドはこのJSONフォーマットを通じてのみModデータを操作
- モッド情報の直感的な表示と操作をUIレベルで実現
- モッドのグループ化、フィルタリング、ソート機能の効率化

```typescript
// ModリストのJSON構造
interface ModdingData {
  enabledMods: ModInfo[];
  disabledMods: ModInfo[];
}

interface ModInfo {
  uuid: string;
  name: string;
  version: string;
  source: "workshop" | "local";
  workshopId?: string;
  enabled: boolean;
}
```

#### 3.1.5 データフローと安全性確保の仕組み
- **GUI操作** → **Modリスト変更** → **API経由でRustに送信** → **整合性検証** → **Modding.xml適用**
- 厳格な型チェックと値のバリデーションによるデータ整合性の確保
- Rustコンポーネントによるデータの整合性チェックと検証
- 問題発生時の自動バックアップ復元機能
- どのようなエラーが発生してもModding.xmlファイルを破損させない保護機能
- ユーザー操作の衝突解決メカニズム

### 3.2 プリセット機能

#### 3.2.1 プリセット作成・管理
- 複数のModを組み合わせて保存
- プリセット名、説明、タグ設定
- お気に入り登録、カテゴリ分類
- 最終プレイ日時の記録

```typescript
// プリセットのJSON構造
interface Preset {
  id?: number;
  name: string;
  description?: string;
  category?: string;
  tags: string[];
  lastPlayed?: Date;
  mods: EnabledModInfo[];  // 有効Modのみのリスト
}

// 有効Modデータ
interface EnabledModInfo {
  uuid: string;
  name: string;
  version: string;
  source: "workshop" | "local";
  workshopId?: string;
}
```

#### 3.2.2 プリセット適用と起動
- 選択したプリセットのMod構成をModding.xmlに反映
- Besiegeを自動起動
- プリセット切替には必ずゲームの再起動が必要
- 各プリセットの総プレイ時間を記録・表示

#### 3.2.3 プリセット共有機能
- プリセットのエクスポート/インポート（JSON形式）
- オンラインでのプリセット公開・検索・ダウンロード
- いいね機能とダウンロード数による人気度評価
- エクスポートデータには必須Modリスト、メタデータ、チェックサム含む

#### 3.2.4 未インストールMod処理
- プリセット適用時に未インストールModを検出
- ワークショップリンクの提供
- 手動インストール後の再試行案内
- （将来的に）自動インストール機能の検討

### 3.3 ユーザーインターフェース

#### 3.3.1 メイン画面構成
- トップナビゲーション: Home, Presets, Browse, Mods, Settings, Help, Donate
- 左サイドバー: カテゴリ、お気に入り、フィルター
- メインコンテンツエリア: プリセットカード/Modリスト
- 右サイドバー: ユーザー情報、ログイン状態

#### 3.3.2 プリセット画面
- プリセットカード表示
- フィルター: All, Downloaded, Custom
- ソート: Name, Last Played
- グループ表示: Favorite, カテゴリ別、すべて

#### 3.3.3 Mod管理画面
- Modリスト表示
- 名前、作者、バージョン、環境(steam/local)表示
- 検索フィルター
- Add modボタン

#### 3.3.4 設定画面
- 言語設定
- 外観テーマ
- 起動オプション
- パス設定 (Ballista, Besiege, Steam Workshop)
- アカウント管理

### 3.4 オンライン機能

#### 3.4.1 ユーザー認証
- Steamアカウント連携ログイン
- JWTトークンによるセッション管理
- ステートレス認証設計

#### 3.4.2 コミュニティ機能
- プリセット共有・検索
- モッドのブラウズ
- 評価システム（いいねと総ダウンロード数のみ）
- Steamからの基本ユーザー情報と画像の同期
- ※コメント機能・詳細なユーザープロファイルは実装しない

#### 3.4.3 収益モデル
- 広告表示 (無料版)
- サブスクリプション (広告非表示)
  - 月額プラン: $3.99/月
  - 年額プラン: $39.99/年 (16%割引)
- 一回限りの寄付
  - 固定額: $5, $10, $25, $50
  - カスタム金額
- Stripe決済処理
  - Elysia.jsのタイプセーフAPIによる安全な決済処理
  - JWTトークンによる課金状態確認
  - 広告表示フラグのサーバー側検証
  - 定期的な課金状態同期

#### 3.4.4 API設計
- Elysia.jsのスキーマ検証を活用した型安全なAPI
- RESTfulエンドポイント設計
- OpenAPI/Swagger自動ドキュメント生成
- 効率的なデータ転送（ページネーション、部分更新）

## 4. 非機能要件

### 4.1 性能・使用統計収集
- 詳細なユーザー操作データ収集システム
  - プリセット作成・適用・プレイ時間の記録
  - 利用頻度の高いMod統計
  - Modの組み合わせパターン分析
  - UI操作フローデータ
- プライバシー設定オプション（設定画面でデータ送信無効化可能）
- 収集データの匿名化処理
- ビッグデータ分析基盤の整備

### 4.2 信頼性要件
- システム稼働率: 99.5%以上（月間ダウンタイム最大4時間以内）
- データバックアップ: 日次自動バックアップ、7日間保持
- 障害発生時の自動リカバリー機能

### 4.3 セキュリティ要件
- ユーザー認証・認可の適切な実装
- 決済情報の安全な処理（PCI DSS準拠）
- プリセットファイルのセキュリティチェック
- 適切なCSP, HSTSなどのセキュリティヘッダー設定

### 4.4 ユーザビリティ要件
- 直感的なUI/UX設計
- ヘルプセクションに使用方法ガイド掲載
- 明確なエラーメッセージと対処方法の提示
- 効率的な操作フローの設計
- ※初回起動時のチュートリアルは実装しない

### 4.5 拡張性・互換性要件
- Windows, macOS, Linux対応
- 将来のBesiege更新に対する柔軟な対応
- 追加機能実装のためのモジュラー設計
- 外部サービス（Steam API等）の変更への適応能力

## 5. データベース設計

### 5.1 クライアントサイドデータベース (SQLite)

#### 5.1.1 スキーマ定義

SQLiteデータベースはプリセット管理と基本設定の保存に使用されます。以下のテーブル構造を採用します：

```sql
-- プリセット基本情報
CREATE TABLE Presets (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  description TEXT,
  category TEXT,
  tags TEXT,  -- JSON形式の配列
  last_played TIMESTAMP,
  total_play_time INTEGER DEFAULT 0,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- プリセットに含まれる有効Mod情報のみ
CREATE TABLE PresetMods (
  preset_id INTEGER,
  uuid TEXT NOT NULL,      -- ModのUUID
  name TEXT NOT NULL,      -- Mod名
  version TEXT,            -- Modバージョン
  source TEXT NOT NULL,    -- 'workshop'または'local'
  workshop_id TEXT,        -- ワークショップID（該当する場合）
  PRIMARY KEY (preset_id, uuid),
  FOREIGN KEY (preset_id) REFERENCES Presets(id) ON DELETE CASCADE
);

-- バックアップ情報
CREATE TABLE Backups (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  file_path TEXT NOT NULL,      -- 元ファイルのパス
  backup_path TEXT NOT NULL,    -- バックアップファイルのパス
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  description TEXT,
  is_auto BOOLEAN DEFAULT 1
);

-- アプリケーション設定
CREATE TABLE Settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

#### 5.1.2 バックアップ管理

- バックアップローテーション機能
  - ファイルごとに保持する最大バックアップ数の設定
  - 古いバックアップの自動削除
  - 重要なバックアップの手動保護機能

#### 5.1.3 データフロー設計

- **プリセット保存時**:
  - Vue UIでユーザーが指定した有効Modリストを含むプリセットデータをJSONとしてRustに送信
  - RustがPresetsテーブルとPresetModsテーブルに保存

- **プリセット読込時**:
  - SQLiteからプリセット情報と関連するMod情報を取得
  - JSON形式に変換してVue UIに提供

- **プリセット適用時**:
  - プリセットに登録されている有効Modリストを取得
  - 現在のModding.xmlを読み込み
  - プリセットに含まれるModを有効化し、その他は無効化
  - 更新されたデータをModding.xmlに書き込み

### 5.2 サーバーデータベース (PostgreSQL with Drizzle ORM)

#### 5.2.1 スキーマ定義 (Drizzle ORM形式)
```typescript
// users.ts
import { pgTable, serial, text, timestamp, boolean, jsonb, integer } from 'drizzle-orm/pg-core';

export const users = pgTable('users', {
  id: serial('id').primaryKey(),
  steamId: text('steam_id').notNull().unique(),
  displayName: text('display_name'),
  avatarUrl: text('avatar_url'),
  createdAt: timestamp('created_at').defaultNow(),
  lastLogin: timestamp('last_login'),
  subscriptionActive: boolean('subscription_active').default(false),
  subscriptionPlan: text('subscription_plan'),
  subscriptionEndDate: timestamp('subscription_end_date'),
  // 追加されたフィールド
  uiSettings: jsonb('ui_settings').default({}),
  statsCollectionOptIn: boolean('stats_collection_opt_in').default(true),
  accountStatus: text('account_status').default('active').notNull(),
  connectionHistory: jsonb('connection_history').default([])
});

// shared_presets.ts
export const sharedPresets = pgTable('shared_presets', {
  id: serial('id').primaryKey(),
  creatorId: integer('creator_id').references(() => users.id),
  creatorName: text('creator_name').notNull(),
  name: text('name').notNull(),
  shortDescription: text('short_description'),
  mdDescription: text('md_description'),
  tags: text('tags').array(),
  downloadCount: integer('download_count').default(0),
  likeCount: integer('like_count').default(0),
  rating: integer('rating').default(0),
  createdAt: timestamp('created_at').defaultNow(),
  updatedAt: timestamp('updated_at').defaultNow(),
  // 追加されたフィールド
  isPublic: boolean('is_public').default(true),
  versionHistory: jsonb('version_history').default([]),
  launchMethod: text('launch_method').default('steam'),
  totalPlayTime: integer('total_play_time').default(0),
  thumbnail: text('thumbnail_url')
});

// preset_mods.ts
export const presetMods = pgTable('preset_mods', {
  id: serial('id').primaryKey(),
  presetId: integer('preset_id').references(() => sharedPresets.id),
  modUuid: text('mod_uuid').notNull(),
  modName: text('mod_name').notNull(),
  modVersion: text('mod_version'),
  modSource: text('mod_source').notNull(),
  workshopId: text('workshop_id')
});

// usage_statistics.ts
export const usageStatistics = pgTable('usage_statistics', {
  id: serial('id').primaryKey(),
  userId: integer('user_id').references(() => users.id),
  eventType: text('event_type').notNull(),
  eventData: jsonb('event_data'),
  timestamp: timestamp('timestamp').defaultNow(),
  modUuids: text('mod_uuids').array(),
  presetId: integer('preset_id'),
  sessionDuration: integer('session_duration')
});

// version_history.ts
export const versionHistory = pgTable('version_history', {
  id: serial('id').primaryKey(),
  presetId: integer('preset_id').references(() => sharedPresets.id),
  versionNumber: text('version_number').notNull(),
  changelog: text('changelog'),
  createdAt: timestamp('created_at').defaultNow(),
  modsList: jsonb('mods_list'),
  downloadCount: integer('download_count').default(0)
});
```

#### 5.2.2 主要テーブル一覧
- **users**: ユーザー情報、認証状態、サブスクリプション情報、UI設定、統計収集設定
- **shared_presets**: 共有プリセットとメタデータ、公開状態、バージョン履歴
- **preset_mods**: プリセットに含まれるMod情報
- **usage_statistics**: ユーザー操作ログと使用統計
- **version_history**: プリセットのバージョン履歴とチェンジログ

#### 5.2.3 Drizzle ORMの利点
- TypeScriptによる完全な型安全性
- マイグレーション管理の自動化
- 効率的なクエリビルダー
- Bunとの高い互換性と高速なパフォーマンス

## 6. 開発・運用環境

### 6.1 開発環境
#### 6.1.1 フロントエンド開発環境（ネイティブ実行）
- **必要な環境構築**:
  - Node.js 20.x以上
  - Rust（最新安定版）
  - プラットフォーム別必須コンポーネント:
    - Windows: Visual Studio Build Tools, WebView2
    - macOS: Xcode Command Line Tools
    - Linux: GTK, WebKit2GTK, その他必要なライブラリ

- **必要なコマンド**:
```bash
# フロントエンド開発環境セットアップ
cd frontend
npm install
npm run dev
```

#### 6.1.2 バックエンド開発環境（Docker）
```yaml
# バックエンド用 docker-compose.backend.yml
version: '3.8'
services:
  # サーバーサイド開発環境 (Bun + Elysia.js)
  backend:
    build: 
      context: ./backend
      dockerfile: Dockerfile.dev
    volumes:
      - ./backend:/app
      - /app/node_modules
    ports:
      - "8000:8000"
    environment:
      - NODE_ENV=development
      - DATABASE_URL=postgres://ballista:dev_password@db:5432/ballista_dev
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
      - postgres-data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

volumes:
  postgres-data:
```

### 6.2 CI/CD (GitHub Actions)
```yaml
# .github/workflows/build.yml
# バックエンド用ワークフローは維持
# フロントエンドのビルドは直接ランナー上で実行（Docker不使用）
name: Build and Release

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  # バックエンド (Bun) のビルドとテスト
  backend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Bun
        uses: oven-sh/setup-bun@v1
        with:
          bun-version: latest
          
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
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: frontend/package-lock.json
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          profile: minimal
          
      - name: Install dependencies (Ubuntu)
        if: matrix.os == 'ubuntu-latest'
        run: |
          sudo apt-get update
          sudo apt-get install -y libgtk-3-dev webkit2gtk-4.0 libappindicator3-dev
      
      - name: Build Frontend
        run: |
          cd frontend
          npm ci
          npm run build
          
      - name: Build Tauri App
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tagName: v__VERSION__
          releaseName: 'Ballista v__VERSION__'
          releaseBody: 'See the release notes for details.'
          releaseDraft: true
          prerelease: false
```

### 6.3 本番環境 (ConoHa VPS)
```yaml
# production/docker-compose.yml
version: '3.8'
services:
  # Bunバックエンド
  backend:
    image: ghcr.io/your-org/ballista-backend:latest
    restart: always
    depends_on:
      - db
    environment:
      DATABASE_URL: postgres://ballista:${DB_PASSWORD}@db:5432/ballista
      NODE_ENV: production
      JWT_SECRET: ${JWT_SECRET}
      STRIPE_SECRET_KEY: ${STRIPE_SECRET_KEY}
    ports:
      - "8000:8000"
    
  # Nginxリバースプロキシ  
  web:
    image: nginx:alpine
    restart: always
    depends_on:
      - backend
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/conf.d/default.conf
      - ./certs:/etc/nginx/certs
      
  # データベース
  db:
    image: postgres:14
    restart: always
    environment:
      POSTGRES_PASSWORD: ${DB_PASSWORD}
      POSTGRES_USER: ballista
      POSTGRES_DB: ballista
    volumes:
      - postgres-data:/var/lib/postgresql/data
      - ./backups:/backups
    ports:
      - "127.0.0.1:5432:5432"
      
  # バックアップサービス
  backup:
    image: postgres:14
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
  postgres-data:
```

### 6.4 自動アップデート
- Tauriアップデーターによるクライアント自動更新
- アップデート通知とユーザー確認
- 差分更新によるダウンロード最適化
- 単一の安定版チャンネルのみ対応

## 7. リスクと課題

### 7.1 技術的リスク
- Besiegeのアップデートによるモッド構造変更
- Steamワークショップ仕様変更への対応
- クロスプラットフォーム互換性の維持
- Bunとelysia.jsの新しさによる安定性・長期メンテナンスへの懸念

### 7.2 ユーザー体験課題
- 未インストールModの自動インストール機能の実現
- モッド競合・依存関係の検出と警告
- 大量のモッド/プリセット管理時のパフォーマンス

### 7.3 ビジネスリスク
- 収益モデルの持続可能性
- ユーザー獲得・維持戦略
- コミュニティエンゲージメントの構築

### 7.4 Bun + Elysia.js 採用に関する考慮点
- 両技術が比較的新しいことによる潜在的な問題
- 将来的なAPIやエコシステムの変更可能性
- ドキュメントやコミュニティサポートの限定性
- 開発者にとってのラーニングカーブ
- 対応策として、重要な機能の単体テスト強化とバージョン固定が必要

## 8. 開発プロセスとスケジュール

### 8.1 開発手法
- 個人開発スタイル（特定の開発手法は採用せず）
- GitFlow方式によるバージョン管理
- 必要に応じた柔軟な開発アプローチ
- GitHub issueによる課題管理

### 8.2 フェーズ区分
1. **Phase 1**: 基本機能開発（ローカルモッド管理、プリセット機能）
2. **Phase 2**: オンライン機能開発（認証、プリセット共有）
3. **Phase 3**: 収益機能実装（広告、サブスクリプション）
4. **Phase 4**: 高度な機能と最適化（通知、検索機能強化）

### 8.3 優先度設定
- **最優先**: コアモッド管理機能、Modding.xml編集
- **高優先**: プリセット機能、UI/UX、安定性
- **中優先**: オンライン共有、ユーザーアカウント
- **低優先**: 拡張機能、多言語対応

## 9. 運用・保守計画

### 9.1 運用・保守体制
- GitHub issueによるバグ報告管理
- 優先度に基づく修正対応
- 個人開発者による一元管理

### 9.2 監視・分析
- エラーレポート収集システム
- 使用統計分析ダッシュボード
- パフォーマンスモニタリング

### 9.3 コミュニティ対応
- Official Besiege Discordにおけるユーザーフィードバック収集
- Discordでの直接的なユーザーコミュニケーション
- 柔軟な機能改善の実施

## 10. 付録

### 10.1 用語集
- **Besiege**: 対象となる物理演算ベースの建築シミュレーションゲーム
- **Mod**: ゲームに追加機能を提供するカスタム拡張
- **Modding.xml**: Besiegeがモッド構成を保存するXMLファイル
- **プリセット**: ユーザーが定義したモッドの組み合わせ
- **UUID**: 各モッドを一意に識別する識別子