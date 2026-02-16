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

# 自动安装
external-autoinstall-found = 插件 '{ $id }' 提供命令 '{ $command }'
external-autoinstall-prompt = 是否安装？[y/N]
external-autoinstall-installing = 正在安装插件 '{ $id }'...
external-autoinstall-success = 插件安装成功！
external-autoinstall-failed = 插件安装失败: { $error }
external-autoinstall-disabled = 自动安装已禁用。运行: adi plugin install { $id }
external-autoinstall-not-found = 未找到提供命令 '{ $command }' 的插件

# ============================================================================
# 信息命令
# ============================================================================

info-title = ADI CLI 信息
info-version = 版本
info-config-dir = 配置
info-plugins-dir = 插件
info-registry = 注册表
info-theme = 主题
info-language = 语言
info-installed-plugins = 已安装插件 ({ $count })
info-no-plugins = 没有安装插件
info-commands-title = 命令
info-plugin-commands = 插件命令：
info-cmd-info = 显示CLI信息、版本和路径
info-cmd-start = 启动本地ADI服务器
info-cmd-plugin = 管理插件
info-cmd-run = 运行插件CLI
info-cmd-logs = 查看插件日志
info-cmd-self-update = 更新adi CLI

# ============================================================================
# 交互式命令选择
# ============================================================================

interactive-select-command = 选择命令

# 命令标签
interactive-cmd-info = 信息
interactive-cmd-start = 启动
interactive-cmd-plugin = 插件
interactive-cmd-search = 搜索
interactive-cmd-run = 运行
interactive-cmd-logs = 日志
interactive-cmd-debug = 调试
interactive-cmd-self-update = 自更新
interactive-cmd-completions = 补全
interactive-cmd-init = 初始化

# 命令描述
interactive-cmd-info-desc = 显示CLI信息、版本、路径和已安装插件
interactive-cmd-start-desc = 启动本地ADI服务器以连接浏览器
interactive-cmd-plugin-desc = 从注册表管理插件
interactive-cmd-search-desc = 搜索插件和软件包
interactive-cmd-run-desc = 运行插件的CLI接口
interactive-cmd-logs-desc = 实时查看插件日志
interactive-cmd-debug-desc = 调试和诊断命令
interactive-cmd-self-update-desc = 将adi CLI更新到最新版本
interactive-cmd-completions-desc = 生成shell补全
interactive-cmd-init-desc = 初始化shell补全

# 参数提示
interactive-self-update-force = 即使是最新版本也要强制更新吗？
interactive-start-port = 端口
interactive-search-query = 搜索查询
interactive-completions-shell = 选择shell
interactive-init-shell = 选择shell（留空自动检测）
interactive-logs-plugin-id = 插件ID（例如 adi.hive）
interactive-logs-follow = 跟踪日志输出？
interactive-logs-lines = 行数

# 插件子命令
interactive-plugin-select = 选择插件操作
interactive-plugin-list = 列出可用
interactive-plugin-installed = 列出已安装
interactive-plugin-search = 搜索
interactive-plugin-install = 安装
interactive-plugin-update = 更新
interactive-plugin-update-all = 全部更新
interactive-plugin-uninstall = 卸载
interactive-plugin-path = 显示路径
interactive-plugin-install-id = 要安装的插件ID（例如 adi.tasks）
interactive-plugin-update-id = 要更新的插件ID
interactive-plugin-uninstall-id = 要卸载的插件ID
interactive-plugin-path-id = 插件ID

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

# ============================================================================
# 错误域
# ============================================================================

error-component-not-found = 未找到组件 '{ $name }'
error-installation-failed = '{ $component }' 安装失败: { $reason }
error-dependency-missing = '{ $component }' 所需的依赖 '{ $dependency }' 未安装
error-config = 配置错误: { $detail }
error-io = IO 错误: { $detail }
error-serialization = 序列化错误: { $detail }
error-already-installed = 组件 '{ $name }' 已安装
error-uninstallation-failed = '{ $component }' 卸载失败: { $reason }
error-registry = 注册表错误: { $detail }
error-plugin-not-found = 未找到插件: { $id }
error-plugin-host = 插件主机错误: { $detail }
error-service = 服务错误: { $detail }
error-other = 错误: { $detail }
