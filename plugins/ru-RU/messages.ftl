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

# Автоустановка
external-autoinstall-found = Плагин '{ $id }' предоставляет команду '{ $command }'
external-autoinstall-prompt = Установить? [y/N]
external-autoinstall-installing = Установка плагина '{ $id }'...
external-autoinstall-success = Плагин успешно установлен!
external-autoinstall-failed = Не удалось установить плагин: { $error }
external-autoinstall-disabled = Автоустановка отключена. Выполните: adi plugin install { $id }
external-autoinstall-not-found = Не найден плагин, предоставляющий команду '{ $command }'

# ============================================================================
# КОМАНДА ИНФОРМАЦИИ
# ============================================================================

info-title = Информация ADI CLI
info-version = Версия
info-config-dir = Конфигурация
info-plugins-dir = Плагины
info-registry = Реестр
info-theme = Тема
info-language = Язык
info-installed-plugins = Установленные плагины ({ $count })
info-no-plugins = Плагины не установлены
info-commands-title = Команды
info-plugin-commands = Команды плагинов:
info-cmd-info = Показать информацию CLI, версию и пути
info-cmd-start = Запустить локальный сервер ADI
info-cmd-plugin = Управление плагинами
info-cmd-run = Запустить CLI плагина
info-cmd-logs = Просмотр логов плагина
info-cmd-self-update = Обновить adi CLI

# ============================================================================
# ИНТЕРАКТИВНЫЙ ВЫБОР КОМАНД
# ============================================================================

interactive-select-command = Выберите команду

# Метки команд
interactive-cmd-info = инфо
interactive-cmd-start = старт
interactive-cmd-plugin = плагин
interactive-cmd-search = поиск
interactive-cmd-run = запуск
interactive-cmd-logs = логи
interactive-cmd-debug = отладка
interactive-cmd-self-update = самообновление
interactive-cmd-completions = дополнения
interactive-cmd-init = инициализация

# Описания команд
interactive-cmd-info-desc = Показать информацию CLI, версию, пути и установленные плагины
interactive-cmd-start-desc = Запустить локальный сервер ADI для подключения браузера
interactive-cmd-plugin-desc = Управление плагинами из реестра
interactive-cmd-search-desc = Поиск плагинов и пакетов
interactive-cmd-run-desc = Запустить CLI интерфейс плагина
interactive-cmd-logs-desc = Просмотр логов плагина в реальном времени
interactive-cmd-debug-desc = Команды отладки и диагностики
interactive-cmd-self-update-desc = Обновить adi CLI до последней версии
interactive-cmd-completions-desc = Сгенерировать дополнения для shell
interactive-cmd-init-desc = Инициализировать дополнения для shell

# Запросы аргументов
interactive-self-update-force = Принудительно обновить даже если версия последняя?
interactive-start-port = Порт
interactive-search-query = Поисковый запрос
interactive-completions-shell = Выберите shell
interactive-init-shell = Выберите shell (оставьте пустым для автоопределения)
interactive-logs-plugin-id = ID плагина (например, adi.hive)
interactive-logs-follow = Следить за выводом логов?
interactive-logs-lines = Количество строк

# Подкоманды плагинов
interactive-plugin-select = Выберите действие с плагином
interactive-plugin-list = Список доступных
interactive-plugin-installed = Список установленных
interactive-plugin-search = Поиск
interactive-plugin-install = Установить
interactive-plugin-update = Обновить
interactive-plugin-update-all = Обновить все
interactive-plugin-uninstall = Удалить
interactive-plugin-path = Показать путь
interactive-plugin-install-id = ID плагина для установки (например, adi.tasks)
interactive-plugin-update-id = ID плагина для обновления
interactive-plugin-uninstall-id = ID плагина для удаления
interactive-plugin-path-id = ID плагина

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

# ============================================================================
# ДОМЕН ОШИБОК
# ============================================================================

error-component-not-found = Компонент '{ $name }' не найден
error-installation-failed = Ошибка установки '{ $component }': { $reason }
error-dependency-missing = Зависимость '{ $dependency }', необходимая для '{ $component }', не установлена
error-config = Ошибка конфигурации: { $detail }
error-io = Ошибка ввода-вывода: { $detail }
error-serialization = Ошибка сериализации: { $detail }
error-already-installed = Компонент '{ $name }' уже установлен
error-uninstallation-failed = Ошибка удаления '{ $component }': { $reason }
error-registry = Ошибка реестра: { $detail }
error-plugin-not-found = Плагин не найден: { $id }
error-plugin-host = Ошибка хоста плагинов: { $detail }
error-service = Ошибка сервиса: { $detail }
error-other = Ошибка: { $detail }
