"""Enregistre de vraies vues du serveur pour les tests du client (tests/fixtures/*.json).

À relancer quand le format de la vue change (docs/PROTOCOLE.md) : il faut un serveur lancé
(`cargo serveur`) et la bibliothèque `websocket-client`.

    python client-godot/tests/fixtures/capture_fixtures.py

Il crée une salle contre le bot aléatoire, enregistre la première vue du draft, joue le
premier coup permis à chaque étape du draft, puis enregistre la première vue de la partie,
et la vue qui suit une pose (avec ses événements).
"""
import json
import pathlib

import websocket

URL = "ws://127.0.0.1:3000/ws"
HERE = pathlib.Path(__file__).parent


def receive(ws, wanted):
    """Le prochain message de ce type (les autres sont ignorés)."""
    while True:
        message = json.loads(ws.recv())
        if message["type"] == wanted:
            return message


def save(name, message):
    path = HERE / f"{name}.json"
    path.write_text(json.dumps(message, ensure_ascii=False, indent=1), encoding="utf-8")
    print("enregistré", path.name)


def main():
    ws = websocket.create_connection(URL)
    ws.send(json.dumps({"type": "cards"}))
    save("cards", receive(ws, "cards"))
    ws.send(json.dumps({"type": "create", "opponent": "random"}))
    receive(ws, "seated")
    state = receive(ws, "state")
    save("draft_view", state)
    while "Draft" in (state["view"]["decision"] or {}):
        if state["view"]["legal_actions"]:
            ws.send(json.dumps({"type": "play", "action": state["view"]["legal_actions"][0]}))
        state = receive(ws, "state")
    while not state["view"]["legal_actions"]:
        state = receive(ws, "state")
    save("game_view", state)
    place = next(a for a in state["view"]["legal_actions"] if isinstance(a, dict) and "Place" in a)
    ws.send(json.dumps({"type": "play", "action": place}))
    save("placed_view", receive(ws, "state"))
    ws.close()


if __name__ == "__main__":
    main()
