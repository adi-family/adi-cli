# ============================================================================
# ДОМЕН САМООБНОВЛЕНИЯ
# ============================================================================

self-update-checking = Проверка обновлений...
self-update-already-latest = У вас уже установлена последняя версия ({ $version })
self-update-new-version = Доступна новая версия: { $current } → { $latest }
self-update-downloading = Загрузка обновления...
self-update-extracting = Распаковка обновления...
self-update-installing = Установка обновления...
self-update-success = Успешно обновлено до версии { $version }
self-update-error-platform = Неподдерживаемая операционная система
self-update-error-arch = Неподдерживаемая архитектура
self-update-error-no-asset = Не найден ресурс релиза для платформы: { $platform }
self-update-error-no-release = Не найден релиз CLI менеджера

# ============================================================================
# ДОМЕН АВТОДОПОЛНЕНИЯ SHELL
# ============================================================================

completions-init-start = Инициализация автодополнения для { $shell }...
completions-init-done = Готово! Автодополнение установлено в: { $path }
completions-restart-zsh = Перезапустите shell или выполните: source ~/.zshrc
completions-restart-bash = Перезапустите shell или выполните: source ~/.bashrc
completions-restart-fish = Автодополнение активно сразу в новых сессиях fish.
completions-restart-generic = Перезапустите shell для активации автодополнения.
completions-error-no-shell = Не удалось определить shell. Укажите: adi init bash|zsh|fish

# ============================================================================
# ДОМЕН УПРАВЛЕНИЯ ПЛАГИНАМИ
# ============================================================================

# Список плагинов
plugin-list-title = Доступные плагины:
plugin-list-empty = В реестре нет доступных плагинов.
plugin-installed-title = Установленные плагины:
plugin-installed-empty = Нет установленных плагинов.
plugin-installed-hint = Установите плагины командой: adi plugin install <plugin-id>

# Установка плагинов
plugin-install-downloading = Загрузка { $id } v{ $version } для { $platform }...
plugin-install-extracting = Распаковка в { $path }...
plugin-install-success = Успешно установлен { $id } v{ $version }!
plugin-install-already-installed = { $id } v{ $version } уже установлен
plugin-install-dependency = Установка зависимости: { $id }
plugin-install-error-platform = Плагин { $id } не поддерживает платформу { $platform }
plugin-install-pattern-searching = Поиск плагинов по шаблону "{ $pattern }"...
plugin-install-pattern-found = Найдено { $count } плагин(ов) по шаблону
plugin-install-pattern-none = Не найдено плагинов по шаблону "{ $pattern }"
plugin-install-pattern-installing = Установка { $count } плагин(ов)...
plugin-install-pattern-success = Успешно установлено { $count } плагин(ов)!
plugin-install-pattern-failed = Не удалось установить:

# Обновление плагинов
plugin-update-checking = Проверка обновлений для { $id }...
plugin-update-already-latest = { $id } уже последней версии ({ $version })
plugin-update-available = Обновление { $id } с { $current } до { $latest }...
plugin-update-downloading = Загрузка { $id } v{ $version }...
plugin-update-success = Обновлён { $id } до v{ $version }
plugin-update-all-start = Обновление { $count } плагин(ов)...
plugin-update-all-done = Обновление завершено!
plugin-update-all-warning = Не удалось обновить { $id }: { $error }

# Удаление плагинов
plugin-uninstall-prompt = Удалить плагин { $id }?
plugin-uninstall-cancelled = Отменено.
plugin-uninstall-progress = Удаление { $id }...
plugin-uninstall-success = { $id } успешно удалён!
plugin-uninstall-error-not-installed = Плагин { $id } не установлен

# ============================================================================
# ДОМЕН ПОИСКА
# ============================================================================

search-searching = Поиск "{ $query }"...
search-no-results = Результатов не найдено.
search-packages-title = Пакеты:
search-plugins-title = Плагины:
search-results-summary = Найдено { $packages } пакет(ов) и { $plugins } плагин(ов)

# ============================================================================
# ДОМЕН СЕРВИСОВ
# ============================================================================

services-title = Зарегистрированные сервисы:
services-empty = Нет зарегистрированных сервисов.
services-hint = Установите плагины для добавления сервисов: adi plugin install <id>

# ============================================================================
# ДОМЕН КОМАНДЫ ЗАПУСКА
# ============================================================================

run-title = Запускаемые плагины:
run-empty = Нет установленных плагинов с CLI интерфейсом.
run-hint-install = Установите плагины командой: adi plugin install <plugin-id>
run-hint-usage = Запустите плагин командой: adi run <plugin-id> [args...]
run-error-not-found = Плагин '{ $id }' не найден или не имеет CLI интерфейса
run-error-no-plugins = Нет установленных запускаемых плагинов.
run-error-available = Доступные плагины:
run-error-failed = Не удалось запустить плагин: { $error }

# ============================================================================
# ДОМЕН ВНЕШНИХ КОМАНД
# ============================================================================

external-error-no-command = Команда не указана
external-error-unknown = Неизвестная команда: { $command }
external-error-no-installed = Нет установленных команд плагинов.
external-hint-install = Установите плагины командой: adi plugin install <plugin-id>
external-available-title = Доступные команды плагинов:
external-error-load-failed = Не удалось загрузить плагин '{ $id }': { $error }
external-hint-reinstall = Попробуйте переустановить: adi plugin install { $id }
external-error-run-failed = Не удалось выполнить { $command }: { $error }

# ============================================================================
# ОБЩИЕ СООБЩЕНИЯ
# ============================================================================

common-version-prefix = v
common-tags-label = Теги:
common-error-prefix = Ошибка:
common-warning-prefix = Предупреждение:
common-info-prefix = Информация:
common-success-prefix = Успех:
common-downloading-prefix = →
common-checkmark = ✓
common-arrow = →
