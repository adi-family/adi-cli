# ============================================================================
# DOMAINE DE MISE À JOUR AUTOMATIQUE
# ============================================================================

self-update-checking = Vérification des mises à jour...
self-update-already-latest = Vous avez déjà la dernière version ({ $version })
self-update-new-version = Nouvelle version disponible : { $current } → { $latest }
self-update-downloading = Téléchargement de la mise à jour...
self-update-extracting = Extraction de la mise à jour...
self-update-installing = Installation de la mise à jour...
self-update-success = Mise à jour réussie vers la version { $version }
self-update-error-platform = Système d'exploitation non supporté
self-update-error-arch = Architecture non supportée
self-update-error-no-asset = Aucune ressource de version trouvée pour la plateforme : { $platform }
self-update-error-no-release = Aucune version du gestionnaire CLI trouvée

# ============================================================================
# DOMAINE DE COMPLÉTION SHELL
# ============================================================================

completions-init-start = Initialisation de la complétion shell pour { $shell }...
completions-init-done = Terminé ! Complétion installée dans : { $path }
completions-restart-zsh = Redémarrez votre shell ou exécutez : source ~/.zshrc
completions-restart-bash = Redémarrez votre shell ou exécutez : source ~/.bashrc
completions-restart-fish = La complétion est active immédiatement dans les nouvelles sessions fish.
completions-restart-generic = Redémarrez votre shell pour activer la complétion.
completions-error-no-shell = Impossible de détecter le shell. Veuillez spécifier : adi init bash|zsh|fish

# ============================================================================
# DOMAINE DE GESTION DES PLUGINS
# ============================================================================

# Liste des plugins
plugin-list-title = Plugins disponibles :
plugin-list-empty = Aucun plugin disponible dans le registre.
plugin-installed-title = Plugins installés :
plugin-installed-empty = Aucun plugin installé.
plugin-installed-hint = Installez des plugins avec : adi plugin install <plugin-id>

# Installation de plugins
plugin-install-downloading = Téléchargement de { $id } v{ $version } pour { $platform }...
plugin-install-extracting = Extraction dans { $path }...
plugin-install-success = { $id } v{ $version } installé avec succès !
plugin-install-already-installed = { $id } v{ $version } est déjà installé
plugin-install-dependency = Installation de la dépendance : { $id }
plugin-install-error-platform = Le plugin { $id } ne supporte pas la plateforme { $platform }
plugin-install-pattern-searching = Recherche des plugins correspondant à "{ $pattern }"...
plugin-install-pattern-found = { $count } plugin(s) trouvé(s) correspondant au motif
plugin-install-pattern-none = Aucun plugin trouvé correspondant à "{ $pattern }"
plugin-install-pattern-installing = Installation de { $count } plugin(s)...
plugin-install-pattern-success = { $count } plugin(s) installé(s) avec succès !
plugin-install-pattern-failed = Échec de l'installation :

# Mise à jour des plugins
plugin-update-checking = Vérification des mises à jour pour { $id }...
plugin-update-already-latest = { $id } est déjà à la dernière version ({ $version })
plugin-update-available = Mise à jour de { $id } de { $current } vers { $latest }...
plugin-update-downloading = Téléchargement de { $id } v{ $version }...
plugin-update-success = { $id } mis à jour vers v{ $version }
plugin-update-all-start = Mise à jour de { $count } plugin(s)...
plugin-update-all-done = Mise à jour terminée !
plugin-update-all-warning = Échec de la mise à jour de { $id } : { $error }

# Désinstallation de plugins
plugin-uninstall-prompt = Désinstaller le plugin { $id } ?
plugin-uninstall-cancelled = Annulé.
plugin-uninstall-progress = Désinstallation de { $id }...
plugin-uninstall-success = { $id } désinstallé avec succès !
plugin-uninstall-error-not-installed = Le plugin { $id } n'est pas installé

# ============================================================================
# DOMAINE DE RECHERCHE
# ============================================================================

search-searching = Recherche de "{ $query }"...
search-no-results = Aucun résultat trouvé.
search-packages-title = Paquets :
search-plugins-title = Plugins :
search-results-summary = { $packages } paquet(s) et { $plugins } plugin(s) trouvé(s)

# ============================================================================
# DOMAINE DES SERVICES
# ============================================================================

services-title = Services enregistrés :
services-empty = Aucun service enregistré.
services-hint = Installez des plugins pour ajouter des services : adi plugin install <id>

# ============================================================================
# DOMAINE DE LA COMMANDE RUN
# ============================================================================

run-title = Plugins exécutables :
run-empty = Aucun plugin avec interface CLI installé.
run-hint-install = Installez des plugins avec : adi plugin install <plugin-id>
run-hint-usage = Exécutez un plugin avec : adi run <plugin-id> [args...]
run-error-not-found = Plugin '{ $id }' non trouvé ou n'a pas d'interface CLI
run-error-no-plugins = Aucun plugin exécutable installé.
run-error-available = Plugins exécutables :
run-error-failed = Échec de l'exécution du plugin : { $error }

# ============================================================================
# DOMAINE DES COMMANDES EXTERNES
# ============================================================================

external-error-no-command = Aucune commande fournie
external-error-unknown = Commande inconnue : { $command }
external-error-no-installed = Aucune commande de plugin installée.
external-hint-install = Installez des plugins avec : adi plugin install <plugin-id>
external-available-title = Commandes de plugins disponibles :
external-error-load-failed = Échec du chargement du plugin '{ $id }' : { $error }
external-hint-reinstall = Essayez de réinstaller : adi plugin install { $id }
external-error-run-failed = Échec de l'exécution de { $command } : { $error }

# Installation automatique
external-autoinstall-found = Le plugin '{ $id }' fournit la commande '{ $command }'
external-autoinstall-prompt = Voulez-vous l'installer ? [y/N]
external-autoinstall-installing = Installation du plugin '{ $id }'...
external-autoinstall-success = Plugin installé avec succès !
external-autoinstall-failed = Échec de l'installation du plugin : { $error }
external-autoinstall-disabled = Installation automatique désactivée. Exécutez : adi plugin install { $id }
external-autoinstall-not-found = Aucun plugin trouvé fournissant la commande '{ $command }'

# ============================================================================
# COMMANDE D'INFORMATION
# ============================================================================

info-title = Informations ADI CLI
info-version = Version
info-config-dir = Configuration
info-plugins-dir = Plugins
info-registry = Registre
info-theme = Thème
info-language = Langue
info-installed-plugins = Plugins installés ({ $count })
info-no-plugins = Aucun plugin installé
info-commands-title = Commandes
info-plugin-commands = Commandes des plugins :
info-cmd-info = Afficher les infos CLI, version et chemins
info-cmd-start = Démarrer le serveur ADI local
info-cmd-plugin = Gérer les plugins
info-cmd-run = Exécuter le CLI d'un plugin
info-cmd-logs = Voir les logs du plugin
info-cmd-self-update = Mettre à jour adi CLI

# ============================================================================
# SÉLECTION INTERACTIVE DES COMMANDES
# ============================================================================

interactive-select-command = Choisissez une commande

# Libellés des commandes
interactive-cmd-info = info
interactive-cmd-start = démarrer
interactive-cmd-plugin = plugin
interactive-cmd-search = recherche
interactive-cmd-run = exécuter
interactive-cmd-logs = logs
interactive-cmd-debug = débogage
interactive-cmd-self-update = mise à jour auto
interactive-cmd-completions = complétions
interactive-cmd-init = init

# Descriptions des commandes
interactive-cmd-info-desc = Afficher les infos CLI, version, chemins et plugins installés
interactive-cmd-start-desc = Démarrer le serveur ADI local pour la connexion navigateur
interactive-cmd-plugin-desc = Gérer les plugins depuis le registre
interactive-cmd-search-desc = Rechercher des plugins et des paquets
interactive-cmd-run-desc = Exécuter l'interface CLI d'un plugin
interactive-cmd-logs-desc = Voir les logs d'un plugin en temps réel
interactive-cmd-debug-desc = Commandes de débogage et diagnostic
interactive-cmd-self-update-desc = Mettre à jour adi CLI vers la dernière version
interactive-cmd-completions-desc = Générer les complétions shell
interactive-cmd-init-desc = Initialiser les complétions shell

# Demandes d'arguments
interactive-self-update-force = Forcer la mise à jour même si déjà à la dernière version ?
interactive-start-port = Port
interactive-search-query = Requête de recherche
interactive-completions-shell = Sélectionner le shell
interactive-init-shell = Sélectionner le shell (laisser vide pour détection auto)
interactive-logs-plugin-id = ID du plugin (ex. adi.hive)
interactive-logs-follow = Suivre la sortie des logs ?
interactive-logs-lines = Nombre de lignes

# Sous-commandes de plugins
interactive-plugin-select = Sélectionner l'action du plugin
interactive-plugin-list = Lister les disponibles
interactive-plugin-installed = Lister les installés
interactive-plugin-search = Rechercher
interactive-plugin-install = Installer
interactive-plugin-update = Mettre à jour
interactive-plugin-update-all = Tout mettre à jour
interactive-plugin-uninstall = Désinstaller
interactive-plugin-path = Afficher le chemin
interactive-plugin-install-id = ID du plugin à installer (ex. adi.tasks)
interactive-plugin-update-id = ID du plugin à mettre à jour
interactive-plugin-uninstall-id = ID du plugin à désinstaller
interactive-plugin-path-id = ID du plugin

# ============================================================================
# MESSAGES COMMUNS/PARTAGÉS
# ============================================================================

common-version-prefix = v
common-tags-label = Tags :
common-error-prefix = Erreur :
common-warning-prefix = Avertissement :
common-info-prefix = Info :
common-success-prefix = Succès :
common-downloading-prefix = →
common-checkmark = ✓
common-arrow = →

# ============================================================================
# DOMAINE D'ERREURS
# ============================================================================

error-component-not-found = Composant '{ $name }' introuvable
error-installation-failed = Échec de l'installation de '{ $component }' : { $reason }
error-dependency-missing = La dépendance '{ $dependency }' requise par '{ $component }' n'est pas installée
error-config = Erreur de configuration : { $detail }
error-io = Erreur d'E/S : { $detail }
error-serialization = Erreur de sérialisation : { $detail }
error-already-installed = Le composant '{ $name }' est déjà installé
error-uninstallation-failed = Échec de la désinstallation de '{ $component }' : { $reason }
error-registry = Erreur de registre : { $detail }
error-plugin-not-found = Plugin introuvable : { $id }
error-plugin-host = Erreur de l'hôte de plugins : { $detail }
error-service = Erreur de service : { $detail }
error-other = Erreur : { $detail }
