"""Précompresse l'export web du client pour le serveur.

    python outils/precompresser_web.py [dossier]

Le dossier est par défaut `client-godot/export/web`, celui de l'export Godot. À côté de
chaque gros fichier (le moteur `.wasm`, le jeu `.pck`, les scripts `.js`), ce script écrit
sa version compressée (`index.wasm.gz`…). Le serveur l'envoie telle quelle aux
navigateurs qui l'acceptent, au lieu de recompresser des dizaines de Mo à chaque visite.

À relancer après chaque export : toutes les versions compressées sont refaites, pour
qu'aucune ne reste en retard sur son original (une copie peut garder une vieille date).
"""

import gzip
import shutil
import sys
from pathlib import Path

# Les fichiers à compresser, par extension : ceux qui pèsent et se compressent bien.
EXTENSIONS = (".wasm", ".pck", ".js", ".html")
DEFAULT_FOLDER = Path("client-godot/export/web")


def main() -> int:
    folder = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_FOLDER
    if not (folder / "index.html").is_file():
        print(f"Pas d'export web dans {folder} (aucun index.html)")
        return 1
    for original in sorted(folder.iterdir()):
        if original.suffix not in EXTENSIONS or not original.is_file():
            continue
        compressed = original.with_name(original.name + ".gz")
        with original.open("rb") as source, gzip.open(compressed, "wb", compresslevel=9) as target:
            shutil.copyfileobj(source, target)
        ratio = compressed.stat().st_size / max(original.stat().st_size, 1)
        print(f"{original.name} : {ratio:.0%} de sa taille")
    return 0


if __name__ == "__main__":
    sys.exit(main())
