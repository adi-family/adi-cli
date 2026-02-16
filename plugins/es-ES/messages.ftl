# ============================================================================
# DOMINIO DE AUTOACTUALIZACIÓN
# ============================================================================

self-update-checking = Buscando actualizaciones...
self-update-already-latest = Ya tienes la última versión ({ $version })
self-update-new-version = Nueva versión disponible: { $current } → { $latest }
self-update-downloading = Descargando actualización...
self-update-extracting = Extrayendo actualización...
self-update-installing = Instalando actualización...
self-update-success = Actualizado correctamente a la versión { $version }
self-update-error-platform = Sistema operativo no soportado
self-update-error-arch = Arquitectura no soportada
self-update-error-no-asset = No se encontró recurso de lanzamiento para la plataforma: { $platform }
self-update-error-no-release = No se encontró lanzamiento del gestor CLI

# ============================================================================
# DOMINIO DE COMPLETADO DE SHELL
# ============================================================================

completions-init-start = Inicializando completado de shell para { $shell }...
completions-init-done = ¡Listo! Completado instalado en: { $path }
completions-restart-zsh = Reinicia tu shell o ejecuta: source ~/.zshrc
completions-restart-bash = Reinicia tu shell o ejecuta: source ~/.bashrc
completions-restart-fish = El completado está activo inmediatamente en nuevas sesiones de fish.
completions-restart-generic = Reinicia tu shell para habilitar el completado.
completions-error-no-shell = No se pudo detectar el shell. Por favor especifica: adi init bash|zsh|fish

# ============================================================================
# DOMINIO DE GESTIÓN DE PLUGINS
# ============================================================================

# Listado de plugins
plugin-list-title = Plugins disponibles:
plugin-list-empty = No hay plugins disponibles en el registro.
plugin-installed-title = Plugins instalados:
plugin-installed-empty = No hay plugins instalados.
plugin-installed-hint = Instala plugins con: adi plugin install <plugin-id>

# Instalación de plugins
plugin-install-downloading = Descargando { $id } v{ $version } para { $platform }...
plugin-install-extracting = Extrayendo en { $path }...
plugin-install-success = ¡{ $id } v{ $version } instalado correctamente!
plugin-install-already-installed = { $id } v{ $version } ya está instalado
plugin-install-dependency = Instalando dependencia: { $id }
plugin-install-error-platform = El plugin { $id } no soporta la plataforma { $platform }
plugin-install-pattern-searching = Buscando plugins que coincidan con "{ $pattern }"...
plugin-install-pattern-found = Encontrado(s) { $count } plugin(s) que coinciden
plugin-install-pattern-none = No se encontraron plugins que coincidan con "{ $pattern }"
plugin-install-pattern-installing = Instalando { $count } plugin(s)...
plugin-install-pattern-success = ¡{ $count } plugin(s) instalado(s) correctamente!
plugin-install-pattern-failed = Error al instalar:

# Actualización de plugins
plugin-update-checking = Buscando actualizaciones para { $id }...
plugin-update-already-latest = { $id } ya está en la última versión ({ $version })
plugin-update-available = Actualizando { $id } de { $current } a { $latest }...
plugin-update-downloading = Descargando { $id } v{ $version }...
plugin-update-success = { $id } actualizado a v{ $version }
plugin-update-all-start = Actualizando { $count } plugin(s)...
plugin-update-all-done = ¡Actualización completada!
plugin-update-all-warning = Error al actualizar { $id }: { $error }

# Desinstalación de plugins
plugin-uninstall-prompt = ¿Desinstalar plugin { $id }?
plugin-uninstall-cancelled = Cancelado.
plugin-uninstall-progress = Desinstalando { $id }...
plugin-uninstall-success = ¡{ $id } desinstalado correctamente!
plugin-uninstall-error-not-installed = El plugin { $id } no está instalado

# ============================================================================
# DOMINIO DE BÚSQUEDA
# ============================================================================

search-searching = Buscando "{ $query }"...
search-no-results = No se encontraron resultados.
search-packages-title = Paquetes:
search-plugins-title = Plugins:
search-results-summary = Encontrado(s) { $packages } paquete(s) y { $plugins } plugin(s)

# ============================================================================
# DOMINIO DE SERVICIOS
# ============================================================================

services-title = Servicios registrados:
services-empty = No hay servicios registrados.
services-hint = Instala plugins para añadir servicios: adi plugin install <id>

# ============================================================================
# DOMINIO DE COMANDO RUN
# ============================================================================

run-title = Plugins ejecutables:
run-empty = No hay plugins con interfaz CLI instalados.
run-hint-install = Instala plugins con: adi plugin install <plugin-id>
run-hint-usage = Ejecuta un plugin con: adi run <plugin-id> [args...]
run-error-not-found = Plugin '{ $id }' no encontrado o no tiene interfaz CLI
run-error-no-plugins = No hay plugins ejecutables instalados.
run-error-available = Plugins ejecutables:
run-error-failed = Error al ejecutar plugin: { $error }

# ============================================================================
# DOMINIO DE COMANDOS EXTERNOS
# ============================================================================

external-error-no-command = No se proporcionó comando
external-error-unknown = Comando desconocido: { $command }
external-error-no-installed = No hay comandos de plugins instalados.
external-hint-install = Instala plugins con: adi plugin install <plugin-id>
external-available-title = Comandos de plugins disponibles:
external-error-load-failed = Error al cargar plugin '{ $id }': { $error }
external-hint-reinstall = Intenta reinstalar: adi plugin install { $id }
external-error-run-failed = Error al ejecutar { $command }: { $error }

# Instalación automática
external-autoinstall-found = El plugin '{ $id }' proporciona el comando '{ $command }'
external-autoinstall-prompt = ¿Desea instalarlo? [y/N]
external-autoinstall-installing = Instalando plugin '{ $id }'...
external-autoinstall-success = ¡Plugin instalado correctamente!
external-autoinstall-failed = Error al instalar plugin: { $error }
external-autoinstall-disabled = Instalación automática deshabilitada. Ejecuta: adi plugin install { $id }
external-autoinstall-not-found = No se encontró plugin que proporcione el comando '{ $command }'

# ============================================================================
# COMANDO DE INFORMACIÓN
# ============================================================================

info-title = Información ADI CLI
info-version = Versión
info-config-dir = Configuración
info-plugins-dir = Plugins
info-registry = Registro
info-theme = Tema
info-language = Idioma
info-installed-plugins = Plugins instalados ({ $count })
info-no-plugins = No hay plugins instalados
info-commands-title = Comandos
info-plugin-commands = Comandos de plugins:
info-cmd-info = Mostrar información del CLI, versión y rutas
info-cmd-start = Iniciar servidor ADI local
info-cmd-plugin = Gestionar plugins
info-cmd-run = Ejecutar CLI de plugin
info-cmd-logs = Ver logs del plugin
info-cmd-self-update = Actualizar adi CLI

# ============================================================================
# SELECCIÓN INTERACTIVA DE COMANDOS
# ============================================================================

interactive-select-command = Elige un comando

# Etiquetas de comandos
interactive-cmd-info = info
interactive-cmd-start = iniciar
interactive-cmd-plugin = plugin
interactive-cmd-search = buscar
interactive-cmd-run = ejecutar
interactive-cmd-logs = logs
interactive-cmd-debug = depuración
interactive-cmd-self-update = autoactualización
interactive-cmd-completions = completado
interactive-cmd-init = init

# Descripciones de comandos
interactive-cmd-info-desc = Mostrar información del CLI, versión, rutas y plugins instalados
interactive-cmd-start-desc = Iniciar servidor ADI local para conexión de navegador
interactive-cmd-plugin-desc = Gestionar plugins del registro
interactive-cmd-search-desc = Buscar plugins y paquetes
interactive-cmd-run-desc = Ejecutar interfaz CLI de un plugin
interactive-cmd-logs-desc = Ver logs de un plugin en tiempo real
interactive-cmd-debug-desc = Comandos de depuración y diagnóstico
interactive-cmd-self-update-desc = Actualizar adi CLI a la última versión
interactive-cmd-completions-desc = Generar completado de shell
interactive-cmd-init-desc = Inicializar completado de shell

# Solicitudes de argumentos
interactive-self-update-force = ¿Forzar actualización aunque sea la última versión?
interactive-start-port = Puerto
interactive-search-query = Consulta de búsqueda
interactive-completions-shell = Seleccionar shell
interactive-init-shell = Seleccionar shell (dejar vacío para autodetección)
interactive-logs-plugin-id = ID del plugin (ej. adi.hive)
interactive-logs-follow = ¿Seguir la salida de logs?
interactive-logs-lines = Número de líneas

# Subcomandos de plugins
interactive-plugin-select = Seleccionar acción de plugin
interactive-plugin-list = Listar disponibles
interactive-plugin-installed = Listar instalados
interactive-plugin-search = Buscar
interactive-plugin-install = Instalar
interactive-plugin-update = Actualizar
interactive-plugin-update-all = Actualizar todos
interactive-plugin-uninstall = Desinstalar
interactive-plugin-path = Mostrar ruta
interactive-plugin-install-id = ID del plugin a instalar (ej. adi.tasks)
interactive-plugin-update-id = ID del plugin a actualizar
interactive-plugin-uninstall-id = ID del plugin a desinstalar
interactive-plugin-path-id = ID del plugin

# ============================================================================
# MENSAJES COMUNES/COMPARTIDOS
# ============================================================================

common-version-prefix = v
common-tags-label = Etiquetas:
common-error-prefix = Error:
common-warning-prefix = Advertencia:
common-info-prefix = Info:
common-success-prefix = Éxito:
common-downloading-prefix = →
common-checkmark = ✓
common-arrow = →

# ============================================================================
# DOMINIO DE ERRORES
# ============================================================================

error-component-not-found = Componente '{ $name }' no encontrado
error-installation-failed = Instalación fallida para '{ $component }': { $reason }
error-dependency-missing = Dependencia '{ $dependency }' requerida por '{ $component }' no está instalada
error-config = Error de configuración: { $detail }
error-io = Error de E/S: { $detail }
error-serialization = Error de serialización: { $detail }
error-already-installed = El componente '{ $name }' ya está instalado
error-uninstallation-failed = Desinstalación fallida para '{ $component }': { $reason }
error-registry = Error del registro: { $detail }
error-plugin-not-found = Plugin no encontrado: { $id }
error-plugin-host = Error del host de plugins: { $detail }
error-service = Error del servicio: { $detail }
error-other = Error: { $detail }
