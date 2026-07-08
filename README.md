# AviUtl2 Linux 互換レイヤー

AviUtl ExEdit2（AviUtl2）を Linux 上で動作させるためのセットアップです。
**Proton GE + DXVK (Vulkan)** ベースで動作し、カスタム Wine ビルドは不要です。

## 動作確認環境

- Proton GE 11-1 (wine-staging 11.0 + DXVK v2.7.1)
- Intel Iris Xe Graphics (ADL GT2) + Mesa 26.1.4
- AviUtl2 beta52

## 必要なもの

- x86_64 Linux
- AVX2 対応 CPU（AviUtl2 本体の要件）
- X11 または Wayland 上の XWayland 環境
- Steam がインストールされていること（Proton GE の入手に使用）
- Vulkan 対応 GPU + Mesa 23.2+
- 日本語表示用の Noto Sans CJK JP フォント

```sh
# Arch 系の依存関係例
sudo pacman -S noto-fonts-cjk gst-libav gst-plugins-bad gst-plugins-ugly
```

## セットアップ

### 1. Proton GE の入手

```sh
mkdir -p ~/.local/share/Steam/compatibilitytools.d
cd ~/.local/share/Steam/compatibilitytools.d
curl -L -o GE-Proton11-1.tar.gz \
  https://github.com/GloriousEggroll/proton-ge-custom/releases/download/GE-Proton11-1/GE-Proton11-1.tar.gz
tar -xzf GE-Proton11-1.tar.gz
```

### 2. AviUtl2 と依存関係の準備

```sh
# リポジトリのルートで
./setup.sh
```

`setup.sh` は以下を行います:
1. 公式サイトから AviUtl2 beta52 をダウンロード
2. Proton GE の存在を確認
3. Wine プレフィックスを作成
4. ネイティブ `d3dcompiler_47.dll` を配置
5. 日本語フォントを設定

### 3. (推奨) dwrite パッチの適用

AviUtl2 のテキスト描画には `HitTestPoint` / `HitTestTextPosition` の実装が必要です。
以下で Proton GE の dwrite.dll にパッチを適用します:

```sh
# dwrite.dll をビルドして Proton GE に適用
cd wine-proton_11.0  # Wine-Proton 11.0 ソースを別途取得
patch -p1 < ../patches/dwrite-hittest.patch
./configure --enable-archs=x86_64 2>&1 | tail -5
make -C dlls/dwrite -j$(nproc)
chmod +w ~/.local/share/Steam/compatibilitytools.d/GE-Proton11-1/files/lib/wine/x86_64-windows/dwrite.dll
cp dlls/dwrite/x86_64-windows/dwrite.dll \
   ~/.local/share/Steam/compatibilitytools.d/GE-Proton11-1/files/lib/wine/x86_64-windows/dwrite.dll
```

## 起動

```sh
./launch-ge.sh
```

初回起動時は Wine プレフィックスの初期化と DXVK DLL のインストールが自動で行われます。
「D3D RDMs not supported.」ダイアログは `tools/dismiss-dialogs.py` が自動的に閉じます。

## プラグイン

### L-SMASH-Works

```sh
./tools/install-lsmash-works.sh
```

`.mp4` / `.mov` の読み込みに使用します。インストール後、AviUtl2 の
`設定 → 入力プラグインの設定` で `L-SMASH Works File Reader for AviUtl2` を確認してください。

### x264guiEx / x265guiEx 出力プラグイン

これらは `pfx-ge` プレフィックス作成時に自動でインストールされます。
出力時に `拡張 x264 出力(GUI) Ex` / `拡張 x265 出力(GUI) Ex` が選択できます。

## トラブルシューティング

### プレビューが黒い

X Composite / XRender が正しく動作していない可能性があります。
`libxcomposite` / `lib32-libxcomposite` がインストールされているか確認してください。

### 色がおかしい（青と赤が入れ替わる）

`system.conf` の `[YUV2RGB]` セクションが正しく設定されているか確認してください。
Intel Mesa Vulkan ドライバでは NV12 の U/V チャンネルが逆転するため、
変換行列の U/V 係数を入れ替えるワークアラウンドを適用しています。

### 起動しない・クラッシュする

```sh
# デバッグログを出力
./launch-ge.sh 2>&1 | tee debug.log
```

問題の切り分け:
- `lwinput.aui2` の読み込みで落ちる → L-SMASH Works の再インストール
- DXVK 関連のエラー → ドライバの Vulkan 対応状況を確認

## パッチ内容

| パッチ | 適用対象 | 内容 |
|---|---|---|
| `patches/dwrite-hittest.patch` | Proton GE の dwrite.dll | HitTestPoint / HitTestTextPosition の実装 |
| `patches/d3d10core-dynamic-dxgcreatedevice.patch` | Wine d3d10core | DXGID3D10CreateDevice の動的ロード対応 |

## 注意

- AviUtl2 の再配布は禁止されています。本リポジトリは実行ファイルを同梱せず、
  公式サイトからのダウンロード方式を採用しています。
- `d3dcompiler_47.dll` は Microsoft の配布物です。各自の責任で入手・使用してください。
- Wine / DXVK のパッチは暫定的なものです。

## ライセンス

スクリプト・設定ファイル類: MIT License
AviUtl2 本体の権利: 作者 KEN くん様に帰属
