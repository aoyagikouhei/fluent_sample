# 開発ルール
- コードを修正した後は必ず `docker compose up -d --build` で再ビルド・起動し、curlでログ送信 → `docker compose logs web` で動作確認を行うこと

# fluentdのサンプルプロジェクト
docker　composeを利用したfluentdのサンプルプロジェクトです
docker composeで管理するサービスは以下の2つになります。
- fluentd
- fluendからログを受け取るWeb

ホストからfluentdにcurlでログを送って、Webで受け取れるか確認します。

# Webの仕様
webはRustのaxumで作成する。
エンドポイントはPOSTでログを受けとる
認証のためにヘッダーでx-api-keyという名前でpasswordという値がある場合にのみ受け取る

# ディレクトリー構造
apps
  |- fluentd
    |- Dockerfile
    |- etc
      |- fluentd.conf
  |- web
    |-src
    |-Cargo.toml
docker-compose.yml

# デフォルト設定

## fluentd
- 入力: `in_http` プラグイン、ポート 9880
- 出力: `out_http` プラグインでWebサービスへHTTP POST転送
- 環境変数 `ENDPOINT_URL_PREFIX`: 出力先URLのプレフィックス（デフォルト: `https://https-portal:443`）

## Web
- エンドポイント: `POST /{path}`（ワイルドカード）
- リクエスト形式: JSON
- 受信後の処理: 標準出力にログを表示
- リッスンポート: 8080

## docker compose
- fluentdポート: 9880:9880
- webポート: 8080:8080（デバッグ用）
- ネットワーク: デフォルトネットワーク、サービス名 `web` で接続

## 動作確認
```bash
# curlでHTTP経由でログ送信
curl -X POST http://localhost:9880/log.access \
  -H "Content-Type: application/json" \
  -d '{"message": "hello"}'

# fluent-catでforward経由でログ送信
echo '{"message":"hello from cli"}' | docker compose exec -T fluentd fluent-cat log.access

# 送信後のログ確認
docker compose logs web

# 別ターミナルでリアルタイム監視（送信前に起動しておくと便利）
docker compose logs -f web
```

## Rust
- エディション: 2021

# fluentd `${tag[1]}` プレースホルダの仕組み

## なぜタグからエンドポイントが決まるのか

fluentdでは**タグ**と**バッファのチャンクキー**の2つの仕組みが連携して動作しています。

### ステップ1: タグの決定

`in_http` プラグインでは、URLパスがタグになります。例えば `http://localhost:9880/log.access` にPOSTすると、タグは `log.access` になります。タグはドット区切りで、`tag[0]` = `log`、`tag[1]` = `access` となります。

### ステップ2: `<buffer tag>` のチャンクキー

```conf
<buffer tag>
  flush_interval 1s
</buffer>
```

`<buffer tag>` の `tag` は**チャンクキー**と呼ばれます。これはfluentdに対して「タグの値ごとにバッファチャンクを分けろ」と指示しています。

つまり:
- `log.access` のレコード → チャンク1
- `log.error` のレコード → チャンク2

と別々のバッファに振り分けられます。

### ステップ3: `endpoint` のプレースホルダ展開

```conf
endpoint https://https-portal:443/${tag[1]}
```

`out_http`プラグインがバッファをフラッシュ（送信）する際、`${tag[1]}` は**そのチャンクのタグの2番目の部分**に置換されます。

- チャンク1（`log.access`）のフラッシュ時 → `https://https-portal:443/access`
- チャンク2（`log.error`）のフラッシュ時 → `https://https-portal:443/error`

### まとめ: 3つの連携

```
curlのURLパス /log.access → タグ "log.access"
        ↓
<buffer tag> でタグをチャンクキーとして登録
        ↓
${tag[1]} でフラッシュ時にタグの2番目の部分に展開
```

**重要なポイント**: `<buffer tag>` の宣言が**必須**です。これがないと fluentd はタグでチャンクを分けず、`${tag[1]}` プレースホルダも展開できません。また、`<match log.*>` のパターンにより `log.` で始まるタグのみがマッチするため、不正なタグは転送されません。
