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
