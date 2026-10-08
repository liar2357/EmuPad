[English](README.md)

---

Linux向けの、タッチスクリーンを仮想トラックパッドとして使うためのツールです。

# 概要

EmuPadはLinux環境向けの仮想トラックパッドです。
タッチパネルの入力を直に読み取りuInputによる仮想マウスでOSへ入力します。
指での操作だけでマウス操作が可能です。

> **_本プロジェクトはver1.0に達していません。_**
> **_不安定な動作や破壊的変更の可能性があります。_**

# 技術スタック

- 言語: Rust
- 設定記述: toml

# 対応環境

2in1ラップトップなど指で操作できるタッチスクリーンが必要です。
OSはLinuxディストリビューションであれば動作すると思われます。

## 動作確認済み環境

| Distribution | DE/WM           |
| ------------ | --------------- |
| NixOS 26.11  | Hyprland 0.56.2 |

その他の環境については今後サポートを広げていく予定です。

# インストール

現時点ではNix（flake）でのみサポートをしています。
flake.nixでURLを指定し、パッケージ定義からインストールしてください。

flake.nix

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    emu-pad.url = "github:liar2357/EmuPad";
  };

  outputs = { self, nixpkgs, emu-pad, ... }:
    {
      nixosConfigurations.my-pc = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";

        modules = [
          ./configuration.nix
        ];
      };
    };
}
```

configuration.nix

```nix
{ pkgs, emu-pad, ... }:

{
  environment.systemPackages = [
    emu-pad.packages.${pkgs.system}.default
  ];
}
```

参考: [実際の使用例](https://github.com/liar2357/Dotfiles/blob/main/flake.nix)

その他の環境の場合はお手数ですがリポジトリのクローンとセルフビルドを手動でお願いします。
別途以下をご参照ください。

- [package.nix(Nix向けパッケージ定義)](./package.nix)
- [rust.nix(Rust開発用依存パッケージ)](https://github.com/liar2357/nix-dev-common/blob/main/rust.nix)

# 使い方

本プロジェクトには`emu-pad`と`emu-padctl`の2つのバイナリが含まれています。

- emu-pad: 仮想トラックパッド本体
- emu-padctl: 一時的な設定等を操作するCLIツール

## emu-pad

`emu-pad`コマンドを実行するとCLIツールが起動し、タッチスクリーンの監視が始まり仮想マウスが生成されます。
設定ファイル(`$HOME/.config/emu-pad/config.toml`)で細かな挙動を制御できます。
項目ごとに設定がされていない場合はデフォルトの設定にフォールバックします。

> EmuPadはLinuxのデバイスファイルを直接監視するため、`/dev/input`へのアクセス権限が必要です。
> また、`uinput`を使用するため、`/dev/uinput`へのアクセス権限が必要です。
> ディストリビューションによって設定方法が異なります。

### 基本操作

| 操作          | 入力                       |
| ------------- | -------------------------- |
| 2本指スライド | マウスカーソル移動         |
| 2本指タップ   | 左クリック                 |
| 2本指長押し   | 右クリック                 |
| 3本指スライド | スクロール(マウスホイール) |
| 3本指タップ   | 中クリック                 |

### 設定項目一覧

| 項目             | 説明             | デフォルト   |
| ---------------- | ---------------- | ------------ |
| layout           | キーボード配列   | "JIS-QWERTY" |
| hold_mode        | 修飾キーの動作   | "None"       |
| default_monitor  | 表示するモニター | "auto"       |
| default_ui_view  | 起動時のUI表示   | true         |
| default_ui_place | UIの表示位置     | "Lower"      |
| ui_height        | UIの高さ         | "30%"        |
| ui_width         | UIの幅           | "100%"       |

### 設定ファイル記述例

```toml
# $HOME/.config/emu-pad/config.toml

# 設定をプロファイルとして複数定義可能
[[configs]]

# デバイスセクション
# キャプチャするタッチスクリーンについての設定
[device]

# selector
# 実際にキャプチャするデバイスを指定します。
# "path:<デバイスファイルパス>" -> パス指定(起動のたびに変わることがあるので非推奨)
# "name:<デバイス表示名>" -> 名前指定
# "vp:<ベンダーID>/<製品ID>" -> ID指定
# "auto" -> 自動選択(`/dev/input`を走査し最初にヒットしたタッチスクリーンを使用)
selector = "auto"

# polling_rate_ms
# 何ms毎にタッチ操作を検知するかを設定します(最小: 1)。
polling_rate_ms = 3

# ジェスチャーセクション
# 操作内容の解釈についての設定
[gesture]

# tap_threshold_ms
# タップ判定の基準となる時間をms単位で設定します。
# これ以下はタップとして判定されます。
tap_threshold_ms = 250

# long_press_threshold_ms
# 長押し判定の基準となる時間をms単位で設定します。
# これ以上は長押しとして判定されます。
long_press_threshold_ms = 500

# movement_threshold
# 移動判定の基準となる距離をpx単位で設定します。
# これ以上は移動として判定されます。
# 小数指定であることに注意してください。
movement_threshold = 15.0

# release_grace_threshold_ms
# ジェスチャー終了判定の基準となる時間をms単位で設定します。
# これ以上はひとまとまりのジェスチャーが終了したとして判定されます。
release_grace_threshold_ms = 100

# default_direction
# デフォルトのジェスチャー向きを設定します。
# 詳しくは後述の*ジェスチャーの認識向き*を参照
default_direction = 0

# move_sensitivity_percent
# マウスカーソル移動幅の倍率を%で指定します(最小: 1)。
move_sensitivity_percent = 80

# scroll_sensitivity_percent
# マウスホイール移動幅の倍率を%で指定します(最小: 1)。
scroll_sensitivity_percent = 50
```

### コマンドラインオプション

```bash
# バージョン情報を表示
emu-pad -V
emu-pad --version

# オプション等の使い方を表示
emu-pad -h
emu-pad --help

# 任意のファイルパスをコンフィグとして指定
emu-pad -c <path>
emu-pad --config <path>

# ログレベルを指定
emu-pad        # (標準)error以上
emu-pad -v     # warn以上
emu-pad -vv    # info以上
emu-pad -vvv   # trace以上
```

### ホットリロード

設定ファイルの変更により自動的にUIを再生成

## emu-padctl

### コマンド一覧

| 名前                     | 説明                                                 |
| ------------------------ | ---------------------------------------------------- |
| set_direction \<number\> | 認識するジェスチャーの向きを変更する                 |
| switch_profile \<index\> | 複数定義されたプロファイルを切り替える(0開始/ループ) |
| lower_ui_place           | UIの表示位置を下側にします                           |
| shutdown_app             | アプリを終了します                                   |

#### ジェスチャーの認識向き

| 番号 | 向き                           |
| ---- | ------------------------------ |
| 0    | 正位置(タッチスクリーンの標準) |
| 1    | 90°回転                        |
| 2    | 180°回転                       |
| 3    | 270°回転                       |

### 使用例

```bash
# ジェスチャーの認識向きを縦に変更
emu-padctl set_direction 3

# アプリを終了
emu-padctl shutdown_app
```

### コマンドラインオプション

```bash
# バージョン情報を表示
emu-padctl -V
emu-padctl --version

# 使い方を表示
emu-padctl -h
emu-padctl --help
```

# 余談

**EmuPad**
"Emulated Trackpad"の略
「えみゅーぱっど」と読むべし

# ライセンス

[MIT License](./LICENCE)
