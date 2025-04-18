---

# env-exec は、指定された .toml ファイルから環境設定（ディレクトリパスや環境変数）を読み取り、それを一時的に適用した状態で指定のシェルやコマンドを実行するツールです。

## このツールを利用することで、システム全体に影響を与えることなく、柔軟なカスタム環境を構築できます。特に、ツールチェーンの設定やプログラムのテスト環境に便利です。

---

# 基本的な機能

1. .toml ファイルから設定を読み取る

   paths に記載されたディレクトリパスを一時的に Path 環境変数に追加します。

   envs に記載されたカスタム環境変数を一時的に適用します。

2. 現在の Path 環境変数を拡張

   現在のシステム環境の Path に対して、読み取ったパスを一時的に追加します。

3. 指定されたコマンドやシェルを新しい環境で実行

   カスタム環境を適用した状態で任意のコマンドやシェルを実行します。

4. 元の環境に影響を与えない

   プログラム終了後、Path 環境変数およびカスタム環境変数は元に戻ります。

---

.tml ファイル形式

TOML形式の環境設定ファイル例：

version = "1.0"

# 一時的に追加するPathリスト
paths = [ "C:\\Users\\hogehoge\\test1\\", "C:\\Users\\hogehoge\\test2\\", "C:\\Users\\hogehoge\\test3\\", "C:\\Users\\hogehoge\\test4\\" ]

# カスタム環境変数のリスト
envs = [ ["TEST_ENV", "100"], ["ANOTHER_ENV", "example_value"] ]

---

# 引数の説明

1. <env_file>: TOML形式の設定ファイルのパス（例: config.toml）

2. <shell>: 実行するシェルの実行可能ファイル（例: cmd, powershell, bash）

3. [command...]: 実行したいコマンドや、シェルに渡す引数（例: "/C echo %PATH%"）

---

# 使い方の例

## 簡単な実行例：

eec.exe config.toml cmd "/C echo %PATH%"

または Bash の場合：

eec.exe config.toml bash "-c" "echo $PATH"

---
