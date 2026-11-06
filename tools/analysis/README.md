# Einmal-Skripte der Arena-Analyse vom 2026-11-06

Rohe Skripte, unverändert aus der Analyse übernommen. Daten liegen nicht im Repo.

`<root>` ist ein Arbeitsordner außerhalb des Repos. Skripte, die Downloads lesen, mit `python3 -I` starten.

| Skript | Zweck | Aufruf |
|---|---|---|
| `fetch.py` | Partien aus `<root>/pages/*.html`, gekürzt | `fetch.py <root>` |
| `traj.py` | Wertungsverlauf aus `<root>/entries/<board>.<seite>.html` | `traj.py <root>/entries` |
| `fetchraw.py` | Rohpartien nach `<root>/raw/` | `fetchraw.py <root>` |
| `h2h.py`, `dyn.py`, `roy.py`, `roy2.py`, `gg.py`, `forced.py` | Direktvergleich, Länge, Royale, falsche `gg`-Serien, Serien vor dem Tod | `<skript> <root>` |
| `ag.py` | Score-Verlauf aus `agree` | `ag.py <table.csv> <root>` |
| `ends.py`, `walk.py` | Endstellungen zeichnen, Stellungen mit `bench` nachrechnen | siehe Kopf der Skripte |
| `mksuite.py`, `depth.py`, `ttd.py` | Stellungsprobe, Tiefe je Build, Tiefe je Rechenzeit | `depth.py <bench> <suite> [seq par]` |
