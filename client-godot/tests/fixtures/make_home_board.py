"""Fabrique la position de l'aperçu de l'accueil (assets/demo/home_board.json).

C'est la vraie vue de début de partie enregistrée pour les tests (game_view.json), où l'on
pose quelques pions, comme sur la maquette de l'accueil : le format reste celui du serveur.
À relancer après capture_fixtures.py, quand le format de la vue change.

    python client-godot/tests/fixtures/make_home_board.py
"""
import json
import pathlib

HERE = pathlib.Path(__file__).parent
TARGET = HERE.parent.parent / "assets" / "demo" / "home_board.json"

# Les pions de l'aperçu, en (colonne, rangée) des cases jouables : trois rouges en colonne,
# que les blancs encadrent.
PIONS = [
    ((5, 2), "One"), ((5, 3), "One"), ((5, 4), "One"),
    ((6, 2), "Two"), ((6, 3), "Two"), ((3, 3), "Two"), ((5, 5), "Two"),
]


def main():
    view = json.loads((HERE / "game_view.json").read_text(encoding="utf-8"))["view"]
    size = view["board"]["size"]
    wall = view["config"]["wall_thickness"]
    cells = view["board"]["cells"]
    for identity, ((col, row), owner) in enumerate(PIONS):
        index = (row + wall) * size + col + wall
        cells[index] = {"pion": {"id": identity, "owner": owner}, "tomb": None}
    view["legal_actions"] = []
    TARGET.write_text(json.dumps({"view": view}, ensure_ascii=False, indent=1), encoding="utf-8")
    print("enregistré", TARGET.name)


if __name__ == "__main__":
    main()
