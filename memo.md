- `src/components/<component>/index.htmx` と `style.css` でUI定義
- `src/components/common.css` で共通トークン（色・余白・フォント）管理
- 最終的に mochiOS/Kagami 上で描画
- デザイン作業はブラウザ側でも確認しやすくする

- **HTML/HTMX パース**: `html5ever`
- **CSS パース**: `cssparser`
- **セレクタ照合**: `selectors`
- **CSS計算（後で）**: `lightningcss`
- **小規模TOML設定**: `toml`

- **実行時パースではなく、ビルド時パースを基本にする**
  - 理由: mochiOS側ランタイムを軽く保てる
  - `build.rs` で `*.htmx`/`*.css` を中間表現（独自バイナリ or Rust生成コード）へ変換
  - 実行時は「解釈済みUIツリー」を読むだけ

## ディレクトリ案
```text
src/components/
  common.css
  button/
    index.htmx
    style.css
  card/
    index.htmx
    style.css
```

## 最初のMVP範囲
1. `index.htmx` からタグ/属性/テキストを抽出
2. `style.css` + `common.css` を読み、以下だけ対応
   - `color`, `background-color`, `width`, `height`, `padding`, `margin`, `border`
3. クラスセレクタ（`.class`）のみ適用
4. ViewKit既存ピクセルレンダラで矩形と文字を描画

## HTMXの扱い
- 初期は **`hx-*` 属性を保存するだけ**（動作はしない）
- 将来イベント系（click/input）とIPCをつないで段階導入

## 次アクション
1. `viewkit_parser` モジュール追加（build-time専用）
2. `html5ever` + `cssparser` + `selectors` をまず dev/build 側で導入
3. 1コンポーネント（例: button）だけ end-to-end で描画確認
