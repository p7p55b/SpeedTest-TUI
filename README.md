# speedtui

Client TUI pour Ookla Speedtest en Rust avec cadrans logarithmiques 10 Gbps (palier 2.5G) et télémétrie Braille temps réel.

## Installation

```bash
make user-install
```

Le binaire est copié dans `~/.local/bin/speedtui`. Vérifier que `~/.local/bin` est dans le `$PATH`.

Autres méthodes :
```bash
cargo install --path .
# ou
sudo make install
```

## Lancement

```bash
speedtui
```

## Raccourcis

| Touche | Action |
| :--- | :--- |
| `Espace` / `Entrée` | Démarrer ou relancer le test |
| `S` | Sélecteur de serveurs Ookla |
| `Haut` / `Bas` | Défiler dans la liste des serveurs |
| `Entrée` | Valider le serveur sélectionné |
| `Échap` | Fermer le sélecteur |
| `O` | Ouvrir le lien du résultat dans le navigateur |
| `C` | Interrompre le test en cours |
| `L` | Effacer l'historique local |
| `Q` | Quitter |

## Fichiers

- Historique des tests : `~/.speedtest_tui_hist.json`
- Binaire Ookla CLI : auto-téléchargé si absent dans `~/.local/share/speedtui/bin/speedtest`
