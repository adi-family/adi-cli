# ============================================================================
# SELBSTAKTUALISIERUNGS-DOMÄNE
# ============================================================================

self-update-checking = Suche nach Updates...
self-update-already-latest = Sie haben bereits die neueste Version ({ $version })
self-update-new-version = Neue Version verfügbar: { $current } → { $latest }
self-update-downloading = Update wird heruntergeladen...
self-update-extracting = Update wird entpackt...
self-update-installing = Update wird installiert...
self-update-success = Erfolgreich auf Version { $version } aktualisiert
self-update-error-platform = Nicht unterstütztes Betriebssystem
self-update-error-arch = Nicht unterstützte Architektur
self-update-error-no-asset = Kein Release-Asset für Plattform gefunden: { $platform }
self-update-error-no-release = Kein CLI-Manager-Release gefunden

# ============================================================================
# SHELL-VERVOLLSTÄNDIGUNGS-DOMÄNE
# ============================================================================

completions-init-start = Initialisiere Shell-Vervollständigung für { $shell }...
completions-init-done = Fertig! Vervollständigung installiert in: { $path }
completions-restart-zsh = Starten Sie Ihre Shell neu oder führen Sie aus: source ~/.zshrc
completions-restart-bash = Starten Sie Ihre Shell neu oder führen Sie aus: source ~/.bashrc
completions-restart-fish = Vervollständigungen sind sofort in neuen Fish-Sitzungen aktiv.
completions-restart-generic = Starten Sie Ihre Shell neu, um Vervollständigungen zu aktivieren.
completions-error-no-shell = Shell konnte nicht erkannt werden. Bitte angeben: adi init bash|zsh|fish

# ============================================================================
# PLUGIN-VERWALTUNGS-DOMÄNE
# ============================================================================

# Plugin-Liste
plugin-list-title = Verfügbare Plugins:
plugin-list-empty = Keine Plugins in der Registry verfügbar.
plugin-installed-title = Installierte Plugins:
plugin-installed-empty = Keine Plugins installiert.
plugin-installed-hint = Installieren Sie Plugins mit: adi plugin install <plugin-id>

# Plugin-Installation
plugin-install-downloading = Lade { $id } v{ $version } für { $platform } herunter...
plugin-install-extracting = Entpacke nach { $path }...
plugin-install-success = { $id } v{ $version } erfolgreich installiert!
plugin-install-already-installed = { $id } v{ $version } ist bereits installiert
plugin-install-dependency = Installiere Abhängigkeit: { $id }
plugin-install-error-platform = Plugin { $id } unterstützt Plattform { $platform } nicht
plugin-install-pattern-searching = Suche nach Plugins mit Muster "{ $pattern }"...
plugin-install-pattern-found = { $count } Plugin(s) gefunden, die dem Muster entsprechen
plugin-install-pattern-none = Keine Plugins gefunden, die "{ $pattern }" entsprechen
plugin-install-pattern-installing = Installiere { $count } Plugin(s)...
plugin-install-pattern-success = { $count } Plugin(s) erfolgreich installiert!
plugin-install-pattern-failed = Installation fehlgeschlagen:

# Plugin-Updates
plugin-update-checking = Suche nach Updates für { $id }...
plugin-update-already-latest = { $id } ist bereits auf der neuesten Version ({ $version })
plugin-update-available = Aktualisiere { $id } von { $current } auf { $latest }...
plugin-update-downloading = Lade { $id } v{ $version } herunter...
plugin-update-success = { $id } auf v{ $version } aktualisiert
plugin-update-all-start = Aktualisiere { $count } Plugin(s)...
plugin-update-all-done = Aktualisierung abgeschlossen!
plugin-update-all-warning = Aktualisierung von { $id } fehlgeschlagen: { $error }

# Plugin-Deinstallation
plugin-uninstall-prompt = Plugin { $id } deinstallieren?
plugin-uninstall-cancelled = Abgebrochen.
plugin-uninstall-progress = Deinstalliere { $id }...
plugin-uninstall-success = { $id } erfolgreich deinstalliert!
plugin-uninstall-error-not-installed = Plugin { $id } ist nicht installiert

# ============================================================================
# SUCH-DOMÄNE
# ============================================================================

search-searching = Suche nach "{ $query }"...
search-no-results = Keine Ergebnisse gefunden.
search-packages-title = Pakete:
search-plugins-title = Plugins:
search-results-summary = { $packages } Paket(e) und { $plugins } Plugin(s) gefunden

# ============================================================================
# DIENSTE-DOMÄNE
# ============================================================================

services-title = Registrierte Dienste:
services-empty = Keine Dienste registriert.
services-hint = Installieren Sie Plugins, um Dienste hinzuzufügen: adi plugin install <id>

# ============================================================================
# RUN-BEFEHL-DOMÄNE
# ============================================================================

run-title = Ausführbare Plugins:
run-empty = Keine Plugins mit CLI-Schnittstelle installiert.
run-hint-install = Installieren Sie Plugins mit: adi plugin install <plugin-id>
run-hint-usage = Führen Sie ein Plugin aus mit: adi run <plugin-id> [args...]
run-error-not-found = Plugin '{ $id }' nicht gefunden oder hat keine CLI-Schnittstelle
run-error-no-plugins = Keine ausführbaren Plugins installiert.
run-error-available = Ausführbare Plugins:
run-error-failed = Plugin-Ausführung fehlgeschlagen: { $error }

# ============================================================================
# EXTERNE-BEFEHLE-DOMÄNE
# ============================================================================

external-error-no-command = Kein Befehl angegeben
external-error-unknown = Unbekannter Befehl: { $command }
external-error-no-installed = Keine Plugin-Befehle installiert.
external-hint-install = Installieren Sie Plugins mit: adi plugin install <plugin-id>
external-available-title = Verfügbare Plugin-Befehle:
external-error-load-failed = Laden von Plugin '{ $id }' fehlgeschlagen: { $error }
external-hint-reinstall = Versuchen Sie neu zu installieren: adi plugin install { $id }
external-error-run-failed = Ausführung von { $command } fehlgeschlagen: { $error }

# Automatische Installation
external-autoinstall-found = Plugin '{ $id }' stellt Befehl '{ $command }' bereit
external-autoinstall-prompt = Möchten Sie es installieren? [y/N]
external-autoinstall-installing = Installiere Plugin '{ $id }'...
external-autoinstall-success = Plugin erfolgreich installiert!
external-autoinstall-failed = Plugin-Installation fehlgeschlagen: { $error }
external-autoinstall-disabled = Automatische Installation deaktiviert. Führen Sie aus: adi plugin install { $id }
external-autoinstall-not-found = Kein Plugin gefunden, das Befehl '{ $command }' bereitstellt

# ============================================================================
# INFO-BEFEHL
# ============================================================================

info-title = ADI CLI Info
info-version = Version
info-config-dir = Konfiguration
info-plugins-dir = Plugins
info-registry = Registry
info-theme = Theme
info-language = Sprache
info-installed-plugins = Installierte Plugins ({ $count })
info-no-plugins = Keine Plugins installiert
info-commands-title = Befehle
info-plugin-commands = Plugin-Befehle:
info-cmd-info = CLI-Info, Version und Pfade anzeigen
info-cmd-start = Lokalen ADI-Server starten
info-cmd-plugin = Plugins verwalten
info-cmd-run = Plugin-CLI ausführen
info-cmd-logs = Plugin-Logs streamen
info-cmd-self-update = adi CLI aktualisieren

# ============================================================================
# INTERAKTIVE BEFEHLSAUSWAHL
# ============================================================================

interactive-select-command = Befehl auswählen

# Befehlsbezeichnungen
interactive-cmd-info = info
interactive-cmd-start = start
interactive-cmd-plugin = plugin
interactive-cmd-search = suche
interactive-cmd-run = ausführen
interactive-cmd-logs = logs
interactive-cmd-debug = debug
interactive-cmd-self-update = selbstaktualisierung
interactive-cmd-completions = vervollständigungen
interactive-cmd-init = init

# Befehlsbeschreibungen
interactive-cmd-info-desc = CLI-Info, Version, Pfade und installierte Plugins anzeigen
interactive-cmd-start-desc = Lokalen ADI-Server für Browser-Verbindung starten
interactive-cmd-plugin-desc = Plugins aus der Registry verwalten
interactive-cmd-search-desc = Nach Plugins und Paketen suchen
interactive-cmd-run-desc = CLI-Schnittstelle eines Plugins ausführen
interactive-cmd-logs-desc = Live-Logs eines Plugins streamen
interactive-cmd-debug-desc = Debug- und Diagnosebefehle
interactive-cmd-self-update-desc = adi CLI auf die neueste Version aktualisieren
interactive-cmd-completions-desc = Shell-Vervollständigungen generieren
interactive-cmd-init-desc = Shell-Vervollständigungen initialisieren

# Argument-Abfragen
interactive-self-update-force = Update erzwingen, auch wenn bereits auf neuester Version?
interactive-start-port = Port
interactive-search-query = Suchbegriff
interactive-completions-shell = Shell auswählen
interactive-init-shell = Shell auswählen (leer lassen für Autoerkennung)
interactive-logs-plugin-id = Plugin-ID (z.B. adi.hive)
interactive-logs-follow = Log-Ausgabe verfolgen?
interactive-logs-lines = Anzahl der Zeilen

# Plugin-Unterbefehle
interactive-plugin-select = Plugin-Aktion auswählen
interactive-plugin-list = Verfügbare auflisten
interactive-plugin-installed = Installierte auflisten
interactive-plugin-search = Suchen
interactive-plugin-install = Installieren
interactive-plugin-update = Aktualisieren
interactive-plugin-update-all = Alle aktualisieren
interactive-plugin-uninstall = Deinstallieren
interactive-plugin-path = Pfad anzeigen
interactive-plugin-install-id = Plugin-ID zum Installieren (z.B. adi.tasks)
interactive-plugin-update-id = Plugin-ID zum Aktualisieren
interactive-plugin-uninstall-id = Plugin-ID zum Deinstallieren
interactive-plugin-path-id = Plugin-ID

# ============================================================================
# GEMEINSAME NACHRICHTEN
# ============================================================================

common-version-prefix = v
common-tags-label = Tags:
common-error-prefix = Fehler:
common-warning-prefix = Warnung:
common-info-prefix = Info:
common-success-prefix = Erfolg:
common-downloading-prefix = →
common-checkmark = ✓
common-arrow = →

# ============================================================================
# FEHLER-DOMAIN
# ============================================================================

error-component-not-found = Komponente '{ $name }' nicht gefunden
error-installation-failed = Installation von '{ $component }' fehlgeschlagen: { $reason }
error-dependency-missing = Abhängigkeit '{ $dependency }', benötigt von '{ $component }', ist nicht installiert
error-config = Konfigurationsfehler: { $detail }
error-io = E/A-Fehler: { $detail }
error-serialization = Serialisierungsfehler: { $detail }
error-already-installed = Komponente '{ $name }' ist bereits installiert
error-uninstallation-failed = Deinstallation von '{ $component }' fehlgeschlagen: { $reason }
error-registry = Registry-Fehler: { $detail }
error-plugin-not-found = Plugin nicht gefunden: { $id }
error-plugin-host = Plugin-Host-Fehler: { $detail }
error-service = Dienstfehler: { $detail }
error-other = Fehler: { $detail }
