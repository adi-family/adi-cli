# ============================================================================
# 自動更新ドメイン
# ============================================================================

self-update-checking = アップデートを確認中...
self-update-already-latest = すでに最新バージョンです ({ $version })
self-update-new-version = 新しいバージョンが利用可能です: { $current } → { $latest }
self-update-downloading = アップデートをダウンロード中...
self-update-extracting = アップデートを展開中...
self-update-installing = アップデートをインストール中...
self-update-success = バージョン { $version } に正常に更新されました
self-update-error-platform = サポートされていないオペレーティングシステム
self-update-error-arch = サポートされていないアーキテクチャ
self-update-error-no-asset = プラットフォーム { $platform } 用のリリースアセットが見つかりません
self-update-error-no-release = CLIマネージャーのリリースが見つかりません

# ============================================================================
# シェル補完ドメイン
# ============================================================================

completions-init-start = { $shell } のシェル補完を初期化中...
completions-init-done = 完了！補完が以下にインストールされました: { $path }
completions-restart-zsh = シェルを再起動するか、以下を実行してください: source ~/.zshrc
completions-restart-bash = シェルを再起動するか、以下を実行してください: source ~/.bashrc
completions-restart-fish = 補完は新しいfishセッションですぐに有効になります。
completions-restart-generic = 補完を有効にするにはシェルを再起動してください。
completions-error-no-shell = シェルを検出できませんでした。指定してください: adi init bash|zsh|fish

# ============================================================================
# プラグイン管理ドメイン
# ============================================================================

# プラグイン一覧
plugin-list-title = 利用可能なプラグイン:
plugin-list-empty = レジストリに利用可能なプラグインがありません。
plugin-installed-title = インストール済みプラグイン:
plugin-installed-empty = インストールされているプラグインがありません。
plugin-installed-hint = プラグインをインストール: adi plugin install <plugin-id>

# プラグインのインストール
plugin-install-downloading = { $id } v{ $version } ({ $platform }) をダウンロード中...
plugin-install-extracting = { $path } に展開中...
plugin-install-success = { $id } v{ $version } を正常にインストールしました！
plugin-install-already-installed = { $id } v{ $version } はすでにインストールされています
plugin-install-dependency = 依存関係をインストール中: { $id }
plugin-install-error-platform = プラグイン { $id } はプラットフォーム { $platform } をサポートしていません
plugin-install-pattern-searching = パターン "{ $pattern }" に一致するプラグインを検索中...
plugin-install-pattern-found = パターンに一致する { $count } 個のプラグインが見つかりました
plugin-install-pattern-none = "{ $pattern }" に一致するプラグインが見つかりません
plugin-install-pattern-installing = { $count } 個のプラグインをインストール中...
plugin-install-pattern-success = { $count } 個のプラグインを正常にインストールしました！
plugin-install-pattern-failed = インストールに失敗しました:

# プラグインの更新
plugin-update-checking = { $id } のアップデートを確認中...
plugin-update-already-latest = { $id } はすでに最新バージョンです ({ $version })
plugin-update-available = { $id } を { $current } から { $latest } に更新中...
plugin-update-downloading = { $id } v{ $version } をダウンロード中...
plugin-update-success = { $id } を v{ $version } に更新しました
plugin-update-all-start = { $count } 個のプラグインを更新中...
plugin-update-all-done = 更新完了！
plugin-update-all-warning = { $id } の更新に失敗しました: { $error }

# プラグインのアンインストール
plugin-uninstall-prompt = プラグイン { $id } をアンインストールしますか？
plugin-uninstall-cancelled = キャンセルされました。
plugin-uninstall-progress = { $id } をアンインストール中...
plugin-uninstall-success = { $id } を正常にアンインストールしました！
plugin-uninstall-error-not-installed = プラグイン { $id } はインストールされていません

# ============================================================================
# 検索ドメイン
# ============================================================================

search-searching = "{ $query }" を検索中...
search-no-results = 結果が見つかりませんでした。
search-packages-title = パッケージ:
search-plugins-title = プラグイン:
search-results-summary = { $packages } 個のパッケージと { $plugins } 個のプラグインが見つかりました

# ============================================================================
# サービスドメイン
# ============================================================================

services-title = 登録済みサービス:
services-empty = 登録されているサービスがありません。
services-hint = サービスを追加するにはプラグインをインストール: adi plugin install <id>

# ============================================================================
# 実行コマンドドメイン
# ============================================================================

run-title = 実行可能なプラグイン:
run-empty = CLIインターフェースを持つプラグインがインストールされていません。
run-hint-install = プラグインをインストール: adi plugin install <plugin-id>
run-hint-usage = プラグインを実行: adi run <plugin-id> [args...]
run-error-not-found = プラグイン '{ $id }' が見つからないか、CLIインターフェースがありません
run-error-no-plugins = 実行可能なプラグインがインストールされていません。
run-error-available = 実行可能なプラグイン:
run-error-failed = プラグインの実行に失敗しました: { $error }

# ============================================================================
# 外部コマンドドメイン
# ============================================================================

external-error-no-command = コマンドが指定されていません
external-error-unknown = 不明なコマンド: { $command }
external-error-no-installed = プラグインコマンドがインストールされていません。
external-hint-install = プラグインをインストール: adi plugin install <plugin-id>
external-available-title = 利用可能なプラグインコマンド:
external-error-load-failed = プラグイン '{ $id }' の読み込みに失敗しました: { $error }
external-hint-reinstall = 再インストールを試してください: adi plugin install { $id }
external-error-run-failed = { $command } の実行に失敗しました: { $error }

# 自動インストール
external-autoinstall-found = プラグイン '{ $id }' がコマンド '{ $command }' を提供しています
external-autoinstall-prompt = インストールしますか？ [y/N]
external-autoinstall-installing = プラグイン '{ $id }' をインストール中...
external-autoinstall-success = プラグインのインストールに成功しました！
external-autoinstall-failed = プラグインのインストールに失敗しました: { $error }
external-autoinstall-disabled = 自動インストールが無効です。実行: adi plugin install { $id }
external-autoinstall-not-found = コマンド '{ $command }' を提供するプラグインが見つかりません

# ============================================================================
# 情報コマンド
# ============================================================================

info-title = ADI CLI 情報
info-version = バージョン
info-config-dir = 設定
info-plugins-dir = プラグイン
info-registry = レジストリ
info-theme = テーマ
info-language = 言語
info-installed-plugins = インストール済みプラグイン ({ $count })
info-no-plugins = プラグインがインストールされていません
info-commands-title = コマンド
info-plugin-commands = プラグインコマンド:
info-cmd-info = CLI情報、バージョン、パスを表示
info-cmd-start = ローカルADIサーバーを起動
info-cmd-plugin = プラグインを管理
info-cmd-run = プラグインCLIを実行
info-cmd-logs = プラグインログを表示
info-cmd-self-update = adi CLIを更新

# ============================================================================
# インタラクティブコマンド選択
# ============================================================================

interactive-select-command = コマンドを選択

# コマンドラベル
interactive-cmd-info = 情報
interactive-cmd-start = 起動
interactive-cmd-plugin = プラグイン
interactive-cmd-search = 検索
interactive-cmd-run = 実行
interactive-cmd-logs = ログ
interactive-cmd-debug = デバッグ
interactive-cmd-self-update = 自動更新
interactive-cmd-completions = 補完
interactive-cmd-init = 初期化

# コマンド説明
interactive-cmd-info-desc = CLI情報、バージョン、パス、インストール済みプラグインを表示
interactive-cmd-start-desc = ブラウザ接続用のローカルADIサーバーを起動
interactive-cmd-plugin-desc = レジストリからプラグインを管理
interactive-cmd-search-desc = プラグインとパッケージを検索
interactive-cmd-run-desc = プラグインのCLIインターフェースを実行
interactive-cmd-logs-desc = プラグインのライブログをストリーム
interactive-cmd-debug-desc = デバッグと診断コマンド
interactive-cmd-self-update-desc = adi CLIを最新バージョンに更新
interactive-cmd-completions-desc = シェル補完を生成
interactive-cmd-init-desc = シェル補完を初期化

# 引数プロンプト
interactive-self-update-force = 最新バージョンでも強制更新しますか？
interactive-start-port = ポート
interactive-search-query = 検索クエリ
interactive-completions-shell = シェルを選択
interactive-init-shell = シェルを選択（自動検出するには空のまま）
interactive-logs-plugin-id = プラグインID（例: adi.hive）
interactive-logs-follow = ログ出力を追跡しますか？
interactive-logs-lines = 行数

# プラグインサブコマンド
interactive-plugin-select = プラグインアクションを選択
interactive-plugin-list = 利用可能一覧
interactive-plugin-installed = インストール済み一覧
interactive-plugin-search = 検索
interactive-plugin-install = インストール
interactive-plugin-update = 更新
interactive-plugin-update-all = すべて更新
interactive-plugin-uninstall = アンインストール
interactive-plugin-path = パスを表示
interactive-plugin-install-id = インストールするプラグインID（例: adi.tasks）
interactive-plugin-update-id = 更新するプラグインID
interactive-plugin-uninstall-id = アンインストールするプラグインID
interactive-plugin-path-id = プラグインID

# ============================================================================
# 共通メッセージ
# ============================================================================

common-version-prefix = v
common-tags-label = タグ:
common-error-prefix = エラー:
common-warning-prefix = 警告:
common-info-prefix = 情報:
common-success-prefix = 成功:
common-downloading-prefix = →
common-checkmark = ✓
common-arrow = →

# ============================================================================
# エラードメイン
# ============================================================================

error-component-not-found = コンポーネント '{ $name }' が見つかりません
error-installation-failed = '{ $component }' のインストールに失敗しました: { $reason }
error-dependency-missing = '{ $component }' に必要な依存関係 '{ $dependency }' がインストールされていません
error-config = 設定エラー: { $detail }
error-io = IOエラー: { $detail }
error-serialization = シリアライズエラー: { $detail }
error-already-installed = コンポーネント '{ $name }' は既にインストールされています
error-uninstallation-failed = '{ $component }' のアンインストールに失敗しました: { $reason }
error-registry = レジストリエラー: { $detail }
error-plugin-not-found = プラグインが見つかりません: { $id }
error-plugin-host = プラグインホストエラー: { $detail }
error-service = サービスエラー: { $detail }
error-other = エラー: { $detail }
