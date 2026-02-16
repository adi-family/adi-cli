# ============================================================================
# ДОМЕН САМООНОВЛЕННЯ
# ============================================================================

self-update-checking = Перевірка оновлень...
self-update-already-latest = Ви вже використовуєте останню версію ({ $version })
self-update-new-version = Доступна нова версія: { $current } → { $latest }
self-update-downloading = Завантаження оновлення...
self-update-extracting = Розпакування оновлення...
self-update-installing = Встановлення оновлення...
self-update-success = Успішно оновлено до версії { $version }
self-update-error-platform = Непідтримувана операційна система
self-update-error-arch = Непідтримувана архітектура
self-update-error-no-asset = Не знайдено ресурс релізу для платформи: { $platform }
self-update-error-no-release = Не знайдено реліз CLI менеджера

# ============================================================================
# ДОМЕН АВТОДОПОВНЕННЯ SHELL
# ============================================================================

completions-init-start = Ініціалізація автодоповнення для { $shell }...
completions-init-done = Готово! Автодоповнення встановлено в: { $path }
completions-restart-zsh = Перезапустіть shell або виконайте: source ~/.zshrc
completions-restart-bash = Перезапустіть shell або виконайте: source ~/.bashrc
completions-restart-fish = Автодоповнення активне одразу в нових сесіях fish.
completions-restart-generic = Перезапустіть shell для активації автодоповнення.
completions-error-no-shell = Не вдалося визначити shell. Вкажіть: adi init bash|zsh|fish

# ============================================================================
# ДОМЕН КЕРУВАННЯ ПЛАГІНАМИ
# ============================================================================

# Список плагінів
plugin-list-title = Доступні плагіни:
plugin-list-empty = В реєстрі немає доступних плагінів.
plugin-installed-title = Встановлені плагіни:
plugin-installed-empty = Немає встановлених плагінів.
plugin-installed-hint = Встановіть плагіни командою: adi plugin install <plugin-id>

# Встановлення плагінів
plugin-install-downloading = Завантаження { $id } v{ $version } для { $platform }...
plugin-install-extracting = Розпакування в { $path }...
plugin-install-success = Успішно встановлено { $id } v{ $version }!
plugin-install-already-installed = { $id } v{ $version } вже встановлено
plugin-install-dependency = Встановлення залежності: { $id }
plugin-install-error-platform = Плагін { $id } не підтримує платформу { $platform }
plugin-install-pattern-searching = Пошук плагінів за шаблоном "{ $pattern }"...
plugin-install-pattern-found = Знайдено { $count } плагін(ів) за шаблоном
plugin-install-pattern-none = Не знайдено плагінів за шаблоном "{ $pattern }"
plugin-install-pattern-installing = Встановлення { $count } плагін(ів)...
plugin-install-pattern-success = Успішно встановлено { $count } плагін(ів)!
plugin-install-pattern-failed = Не вдалося встановити:

# Оновлення плагінів
plugin-update-checking = Перевірка оновлень для { $id }...
plugin-update-already-latest = { $id } вже останньої версії ({ $version })
plugin-update-available = Оновлення { $id } з { $current } до { $latest }...
plugin-update-downloading = Завантаження { $id } v{ $version }...
plugin-update-success = Оновлено { $id } до v{ $version }
plugin-update-all-start = Оновлення { $count } плагін(ів)...
plugin-update-all-done = Оновлення завершено!
plugin-update-all-warning = Не вдалося оновити { $id }: { $error }

# Видалення плагінів
plugin-uninstall-prompt = Видалити плагін { $id }?
plugin-uninstall-cancelled = Скасовано.
plugin-uninstall-progress = Видалення { $id }...
plugin-uninstall-success = { $id } успішно видалено!
plugin-uninstall-error-not-installed = Плагін { $id } не встановлено

# ============================================================================
# ДОМЕН ПОШУКУ
# ============================================================================

search-searching = Пошук "{ $query }"...
search-no-results = Результатів не знайдено.
search-packages-title = Пакети:
search-plugins-title = Плагіни:
search-results-summary = Знайдено { $packages } пакет(ів) та { $plugins } плагін(ів)

# ============================================================================
# ДОМЕН СЕРВІСІВ
# ============================================================================

services-title = Зареєстровані сервіси:
services-empty = Немає зареєстрованих сервісів.
services-hint = Встановіть плагіни для додавання сервісів: adi plugin install <id>

# ============================================================================
# ДОМЕН КОМАНДИ ЗАПУСКУ
# ============================================================================

run-title = Плагіни для запуску:
run-empty = Немає встановлених плагінів з CLI інтерфейсом.
run-hint-install = Встановіть плагіни командою: adi plugin install <plugin-id>
run-hint-usage = Запустіть плагін командою: adi run <plugin-id> [args...]
run-error-not-found = Плагін '{ $id }' не знайдено або він не має CLI інтерфейсу
run-error-no-plugins = Немає встановлених плагінів для запуску.
run-error-available = Доступні плагіни:
run-error-failed = Не вдалося запустити плагін: { $error }

# ============================================================================
# ДОМЕН ЗОВНІШНІХ КОМАНД
# ============================================================================

external-error-no-command = Команду не вказано
external-error-unknown = Невідома команда: { $command }
external-error-no-installed = Немає встановлених команд плагінів.
external-hint-install = Встановіть плагіни командою: adi plugin install <plugin-id>
external-available-title = Доступні команди плагінів:
external-error-load-failed = Не вдалося завантажити плагін '{ $id }': { $error }
external-hint-reinstall = Спробуйте перевстановити: adi plugin install { $id }
external-error-run-failed = Не вдалося виконати { $command }: { $error }

# Автовстановлення
external-autoinstall-found = Плагін '{ $id }' надає команду '{ $command }'
external-autoinstall-prompt = Встановити? [y/N]
external-autoinstall-installing = Встановлення плагіна '{ $id }'...
external-autoinstall-success = Плагін успішно встановлено!
external-autoinstall-failed = Не вдалося встановити плагін: { $error }
external-autoinstall-disabled = Автовстановлення вимкнено. Виконайте: adi plugin install { $id }
external-autoinstall-not-found = Не знайдено плагін, що надає команду '{ $command }'

# ============================================================================
# КОМАНДА ІНФОРМАЦІЇ
# ============================================================================

info-title = Інформація ADI CLI
info-version = Версія
info-config-dir = Конфігурація
info-plugins-dir = Плагіни
info-registry = Реєстр
info-theme = Тема
info-language = Мова
info-installed-plugins = Встановлені плагіни ({ $count })
info-no-plugins = Плагіни не встановлені
info-commands-title = Команди
info-plugin-commands = Команди плагінів:
info-cmd-info = Показати інформацію CLI, версію та шляхи
info-cmd-start = Запустити локальний сервер ADI
info-cmd-plugin = Керувати плагінами
info-cmd-run = Запустити CLI плагіна
info-cmd-logs = Переглянути логи плагіна
info-cmd-self-update = Оновити adi CLI

# ============================================================================
# ІНТЕРАКТИВНИЙ ВИБІР КОМАНД
# ============================================================================

interactive-select-command = Оберіть команду

# Мітки команд
interactive-cmd-info = інфо
interactive-cmd-start = старт
interactive-cmd-plugin = плагін
interactive-cmd-search = пошук
interactive-cmd-run = запуск
interactive-cmd-logs = логи
interactive-cmd-debug = налагодження
interactive-cmd-self-update = самооновлення
interactive-cmd-completions = доповнення
interactive-cmd-init = ініціалізація

# Описи команд
interactive-cmd-info-desc = Показати інформацію CLI, версію, шляхи та встановлені плагіни
interactive-cmd-start-desc = Запустити локальний сервер ADI для підключення браузера
interactive-cmd-plugin-desc = Керувати плагінами з реєстру
interactive-cmd-search-desc = Пошук плагінів та пакетів
interactive-cmd-run-desc = Запустити CLI інтерфейс плагіна
interactive-cmd-logs-desc = Переглядати логи плагіна в реальному часі
interactive-cmd-debug-desc = Команди налагодження та діагностики
interactive-cmd-self-update-desc = Оновити adi CLI до останньої версії
interactive-cmd-completions-desc = Згенерувати доповнення для shell
interactive-cmd-init-desc = Ініціалізувати доповнення для shell

# Запити аргументів
interactive-self-update-force = Примусово оновити навіть якщо версія остання?
interactive-start-port = Порт
interactive-search-query = Пошуковий запит
interactive-completions-shell = Оберіть shell
interactive-init-shell = Оберіть shell (залиште порожнім для автовизначення)
interactive-logs-plugin-id = ID плагіна (наприклад, adi.hive)
interactive-logs-follow = Слідкувати за виводом логів?
interactive-logs-lines = Кількість рядків

# Підкоманди плагінів
interactive-plugin-select = Оберіть дію з плагіном
interactive-plugin-list = Список доступних
interactive-plugin-installed = Список встановлених
interactive-plugin-search = Пошук
interactive-plugin-install = Встановити
interactive-plugin-update = Оновити
interactive-plugin-update-all = Оновити всі
interactive-plugin-uninstall = Видалити
interactive-plugin-path = Показати шлях
interactive-plugin-install-id = ID плагіна для встановлення (наприклад, adi.tasks)
interactive-plugin-update-id = ID плагіна для оновлення
interactive-plugin-uninstall-id = ID плагіна для видалення
interactive-plugin-path-id = ID плагіна

# ============================================================================
# ЗАГАЛЬНІ/СПІЛЬНІ ПОВІДОМЛЕННЯ
# ============================================================================

common-version-prefix = v
common-tags-label = Теги:
common-error-prefix = Помилка:
common-warning-prefix = Попередження:
common-info-prefix = Інформація:
common-success-prefix = Успіх:
common-downloading-prefix = →
common-checkmark = ✓
common-arrow = →

# ============================================================================
# ДОМЕН ПОМИЛОК
# ============================================================================

error-component-not-found = Компонент '{ $name }' не знайдено
error-installation-failed = Помилка встановлення '{ $component }': { $reason }
error-dependency-missing = Залежність '{ $dependency }', необхідна для '{ $component }', не встановлена
error-config = Помилка конфігурації: { $detail }
error-io = Помилка введення-виведення: { $detail }
error-serialization = Помилка серіалізації: { $detail }
error-already-installed = Компонент '{ $name }' вже встановлено
error-uninstallation-failed = Помилка видалення '{ $component }': { $reason }
error-registry = Помилка реєстру: { $detail }
error-plugin-not-found = Плагін не знайдено: { $id }
error-plugin-host = Помилка хосту плагінів: { $detail }
error-service = Помилка сервісу: { $detail }
error-other = Помилка: { $detail }
