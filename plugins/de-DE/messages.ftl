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
