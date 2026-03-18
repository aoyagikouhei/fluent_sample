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

## Web
- エンドポイント: `POST /logs`
- リクエスト形式: JSON
- 受信後の処理: 標準出力にログを表示
- リッスンポート: 8080

## docker compose
- fluentdポート: 9880:9880
- webポート: 8080:8080（デバッグ用）
- ネットワーク: デフォルトネットワーク、サービス名 `web` で接続

## 動作確認
```bash
curl -X POST http://localhost:9880/log.access \
  -H "Content-Type: application/json" \
  -d '{"path": "log1", "message": "hello"}'
```

## Rust
- エディション: 2021

# fluentd `${path}` プレースホルダの仕組み

## なぜ `${path}` でJSONのフィールドが読めるのか

fluentdでは**レコード（ログイベント）のフィールド名**と**バッファのチャンクキー**の2つの仕組みが連携して動作しています。

### ステップ1: レコードのフィールド

fluentdに送られるJSONデータ `{"path": "log1", "message": "hello"}` は、fluentd内部で**レコード**として扱われます。各キー（`path`, `message`）はレコードのフィールドになります。

### ステップ2: `<buffer path>` のチャンクキー

```conf
<buffer path>
  flush_interval 1s
</buffer>
```

`<buffer path>` の `path` は**チャンクキー**と呼ばれます。これはfluentdに対して「レコードの`path`フィールドの値ごとにバッファチャンクを分けろ」と指示しています。

つまり:
- `path=log1` のレコード → チャンク1
- `path=log2` のレコード → チャンク2

と別々のバッファに振り分けられます。

### ステップ3: `endpoint` のプレースホルダ展開

```conf
endpoint https://https-portal:443/${path}
```

`out_http`プラグインがバッファをフラッシュ（送信）する際、`${path}` は**そのチャンクのチャンクキーの値**に置換されます。

- チャンク1（`path=log1`）のフラッシュ時 → `https://https-portal:443/log1`
- チャンク2（`path=log2`）のフラッシュ時 → `https://https-portal:443/log2`

### まとめ: 3つの連携

```
JSONの "path" フィールド
        ↓
<buffer path> でチャンクキーとして登録
        ↓
${path} でフラッシュ時にチャンクキーの値に展開
```

**重要なポイント**: `<buffer path>` の宣言が**必須**です。これがないと fluentd はどのフィールドでチャンクを分けるべきか分からず、`${path}` プレースホルダも展開できません。`<buffer>` だけ（キーなし）だと全レコードが1つのチャンクにまとめられ、個別のフィールド値を参照できません。