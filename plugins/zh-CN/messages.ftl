# ============================================================================
# 自更新域
# ============================================================================

self-update-checking = 正在检查更新...
self-update-already-latest = 您已经是最新版本 ({ $version })
self-update-new-version = 有新版本可用: { $current } → { $latest }
self-update-downloading = 正在下载更新...
self-update-extracting = 正在解压更新...
self-update-installing = 正在安装更新...
self-update-success = 成功更新到版本 { $version }
self-update-error-platform = 不支持的操作系统
self-update-error-arch = 不支持的架构
self-update-error-no-asset = 未找到平台 { $platform } 的发布资源
self-update-error-no-release = 未找到 CLI 管理器发布版本

# ============================================================================
# Shell 补全域
# ============================================================================

completions-init-start = 正在为 { $shell } 初始化 shell 补全...
completions-init-done = 完成！补全已安装到: { $path }
completions-restart-zsh = 重启 shell 或运行: source ~/.zshrc
completions-restart-bash = 重启 shell 或运行: source ~/.bashrc
completions-restart-fish = 补全在新的 fish 会话中立即生效。
completions-restart-generic = 重启 shell 以启用补全。
completions-error-no-shell = 无法检测 shell。请指定: adi init bash|zsh|fish

# ============================================================================
# 插件管理域
# ============================================================================

# 插件列表
plugin-list-title = 可用插件:
plugin-list-empty = 注册表中没有可用的插件。
plugin-installed-title = 已安装的插件:
plugin-installed-empty = 没有已安装的插件。
plugin-installed-hint = 使用以下命令安装插件: adi plugin install <plugin-id>

# 插件安装
plugin-install-downloading = 正在下载 { $id } v{ $version } ({ $platform })...
plugin-install-extracting = 正在解压到 { $path }...
plugin-install-success = 成功安装 { $id } v{ $version }!
plugin-install-already-installed = { $id } v{ $version } 已安装
plugin-install-dependency = 正在安装依赖: { $id }
plugin-install-error-platform = 插件 { $id } 不支持平台 { $platform }
plugin-install-pattern-searching = 正在搜索匹配模式 "{ $pattern }" 的插件...
plugin-install-pattern-found = 找到 { $count } 个匹配的插件
plugin-install-pattern-none = 未找到匹配模式 "{ $pattern }" 的插件
plugin-install-pattern-installing = 正在安装 { $count } 个插件...
plugin-install-pattern-success = 成功安装 { $count } 个插件!
plugin-install-pattern-failed = 安装失败:

# 插件更新
plugin-update-checking = 正在检查 { $id } 的更新...
plugin-update-already-latest = { $id } 已是最新版本 ({ $version })
plugin-update-available = 正在将 { $id } 从 { $current } 更新到 { $latest }...
plugin-update-downloading = 正在下载 { $id } v{ $version }...
plugin-update-success = 已将 { $id } 更新到 v{ $version }
plugin-update-all-start = 正在更新 { $count } 个插件...
plugin-update-all-done = 更新完成!
plugin-update-all-warning = 更新 { $id } 失败: { $error }

# 插件卸载
plugin-uninstall-prompt = 卸载插件 { $id }?
plugin-uninstall-cancelled = 已取消。
plugin-uninstall-progress = 正在卸载 { $id }...
plugin-uninstall-success = 成功卸载 { $id }!
plugin-uninstall-error-not-installed = 插件 { $id } 未安装

# ============================================================================
# 搜索域
# ============================================================================

search-searching = 正在搜索 "{ $query }"...
search-no-results = 未找到结果。
search-packages-title = 软件包:
search-plugins-title = 插件:
search-results-summary = 找到 { $packages } 个软件包和 { $plugins } 个插件

# ============================================================================
# 服务域
# ============================================================================

services-title = 已注册的服务:
services-empty = 没有已注册的服务。
services-hint = 安装插件以添加服务: adi plugin install <id>

# ============================================================================
# 运行命令域
# ============================================================================

run-title = 可运行的插件:
run-empty = 没有安装带有 CLI 接口的插件。
run-hint-install = 使用以下命令安装插件: adi plugin install <plugin-id>
run-hint-usage = 使用以下命令运行插件: adi run <plugin-id> [args...]
run-error-not-found = 未找到插件 '{ $id }' 或该插件没有 CLI 接口
run-error-no-plugins = 没有安装可运行的插件。
run-error-available = 可运行的插件:
run-error-failed = 运行插件失败: { $error }

# ============================================================================
# 外部命令域
# ============================================================================

external-error-no-command = 未提供命令
external-error-unknown = 未知命令: { $command }
external-error-no-installed = 没有安装插件命令。
external-hint-install = 使用以下命令安装插件: adi plugin install <plugin-id>
external-available-title = 可用的插件命令:
external-error-load-failed = 加载插件 '{ $id }' 失败: { $error }
external-hint-reinstall = 尝试重新安装: adi plugin install { $id }
external-error-run-failed = 运行 { $command } 失败: { $error }

# ============================================================================
# 通用/共享消息
# ============================================================================

common-version-prefix = v
common-tags-label = 标签:
common-error-prefix = 错误:
common-warning-prefix = 警告:
common-info-prefix = 信息:
common-success-prefix = 成功:
common-downloading-prefix = →
common-checkmark = ✓
common-arrow = →
