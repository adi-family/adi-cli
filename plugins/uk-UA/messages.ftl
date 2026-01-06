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
