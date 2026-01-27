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
