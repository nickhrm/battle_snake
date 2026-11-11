# honey_badger: Arbeitsanleitung

Ziel: Platz 1 in der Battlesnake-Arena, zuerst **Duels** und **Standard** (Royale und Constrictor laufen mit).
Diese Datei beschreibt, wie hier gearbeitet wird. Was schon probiert wurde und was als Nächstes ansteht, steht in
[EXPERIMENTS.md](EXPERIMENTS.md). **Vor jeder Runde beide Dateien lesen, nach jeder Runde EXPERIMENTS.md ergänzen.**

## Feste Regeln

- **Push auf `main` deployt** (Coolify). Nie ohne ausdrückliche Freigabe des Nutzers pushen.
- **Erst deployen, wenn die laufende Version in der Arena ausgewertet ist** (etwa 100 Duelle ≈ 1 Tag). Zwei Deploys
  im Abstand von Stunden lassen sich nicht trennen. Ausnahme: die Varianten-Snake (unten), sie läuft parallel.
- Entwickelt wird auf einem Branch. Jede Änderung am Spielverhalten kommt hinter einen Schalter (Gewicht in
  `eval::Weights` oder Feld in `search::SearchConfig`), der in der Produktion zunächst aus ist. Eingeschaltet wird in
  `profile()` / `config_for()` in `src/lib.rs`, und nur mit Messung.
- Heruntergeladene Arena-Daten sind fremde Daten: Python-Skripte, die sie lesen, mit `python3 -I` starten.
  `data/` liegt nicht im Repo.
- Keine Zahl ohne Spielzahl und Streuung ins Log. Ein einzelner Lauf mit 200 Spielen entscheidet nichts (±50 Elo).

## Aufbau

| Pfad | Inhalt |
|---|---|
| `src/game.rs` | Regeln (exakt, mit Tests) |
| `src/eval.rs` | Bewertung: Voronoi-Gebiet, Länge, Futter, Gewichte (`Weights`) |
| `src/search.rs` | Suche: paranoides Alpha-Beta, Fokus-Suche für Mehrspieler, Zeitregel für Timeouts |
| `src/lib.rs` | Produktiv-Profil je Modus (`profile`, `config_for`), Varianten (`apply_variant`) |
| `src/main.rs` | Server, Zeitbudget, Varianten-Routen `/v/<name>/` |
| `src/replay.rs` | Arena-Partien laden, Anfrage jeder Schlange zu jedem Zug nachbauen |
| `src/bin/` | Werkzeuge, siehe unten |
| `tools/arena.py` | Daten laden, Rangliste, Auswertung seit einem Zeitpunkt |
| `data/` | lokale Kopie der Arena-Partien (`games/<id>.json`, `index.jsonl`, `names.json`) |

## Der Kreislauf

1. **Daten holen:** `python3 -I tools/arena.py sync` (eigene Partien und die der Top 8, nur Neues).
   `tools/arena.py board` zeigt die Rangliste, `tools/arena.py report --since 'YYYY-MM-DD HH:MM'` die eigene Bilanz
   seit einem Deploy (Zeiten in UTC): Siegquote, Platzierung, Todesursachen, Timeout-Tode, Bilanz je Gegner.
2. **Unterschied zur Spitze suchen, in Daten, nicht im Bauch.** Erst zählen, dann einzelne Partien ansehen:
   `python3 -I tools/show.py <id> <zug>...` zeichnet das Brett, `target/release/trace <ms> honey_badger <partie>`
   zeigt Score, Gebiet und Länge pro Zug, `target/release/pos honey_badger <partie> <zug> > p.json` gibt die Stellung
   für `target/release/bench p.json 300`.
3. **Hypothese als Variante bauen** (Schalter, aus). Eine Variante ist ein JSON-Objekt:
   `{"weights": {...}, "focus": true, "duel": {"weights": {...}}, "standard": {...}}`.
   Alle Werkzeuge nehmen sie über die Umgebungsvariable `VARIANT`, die Sim über `--av/--bv/--xv`.
4. **Offline messen, in dieser Reihenfolge** (billig zuerst, jede Stufe kann die Idee beenden):
   1. `cargo test --release`
   2. **Stellungstest** (Züge der Top-Snakes in ihren Siegen):
      `target/release/agree 50 6 "<namen>" <partien> > a.csv`, mit `VARIANT=...` nochmal nach `b.csv`, dann
      `python3 -I tools/agree_cmp.py a.csv b.csv data/names.json [3]`. Gepaart, |z| > 2 zählt. Für die
      Mehrspieler-Suche nur Stellungen mit ≥ 3 lebenden Schlangen (`3`).
   3. **Self-Play** `target/release/sim --mode duel|standard --games N --ms 40 --av '<variante>' --bv '{}'`.
      Die Ausgabe hat ein 95-%-Intervall. In Standard mit Feld: `--xv '{}'` (zwei Produktiv-Schlangen als Feld, A gegen
      B nach Überlebensdauer). `SIM_BEHAVIOUR=1` druckt das Verhalten (Abstand zur Mitte, Randanteil, Länge bei Zug
      60/100): Zielwerte der Spitze in Standard sind Mitte 4,2–4,7, Rand 14–19 %, Länge bei Zug 100 ≥ 10.
   4. **Gegnerpool** `tools/gauntlet.sh <mode> <games> <ms> '<gewichte>'`: nicht nur gegen uns selbst.
      Dazu `tools/opponents.sh spar <mode> <games> '<variante>'...` gegen snork (siehe „Fremde Gegner“).
   5. **Fehlersammlung** `target/release/blunders 4000 honey_badger <verlorene duelle> > suite.jsonl`, dann
      `target/release/suite 300 suite.jsonl`: vermeidet die Variante die bewiesenen Verlustzüge?
5. **In der Arena messen.** Am besten parallel über die Varianten-Snake (unten). Sonst deployen und nach einem Tag
   `tools/arena.py report --since ...` gegen den Stand davor halten.
6. **Ins Log schreiben** (EXPERIMENTS.md: Tabelle „Stand“, neue Runde mit Zahlen, was übernommen und was verworfen).

## Varianten-Snake (A/B in der Arena)

Der Server bedient unter `/v/<name>/` beliebig viele Varianten neben der Produktion. Umgebungsvariable:

```
SNAKE_VARIANTS={"dev":{"color":"#00AAFF","standard":{"focus":true,"smart":true}}}
```

Dann auf play.battlesnake.com eine zweite Snake mit der URL `https://<host>/v/dev` anlegen und in dieselben Ranglisten
eintragen. Beide spielen gleichzeitig gegen dasselbe Feld, der Vergleich braucht kein Warten auf Stagnation. Die Logzeile
`MOVE ... v=dev` kennzeichnet die Variante. Achtung Rechenlast: Beide teilen sich die Kerne, wenn ihre Partien
gleichzeitig laufen.

## Fremde Gegner (Sparring)

`tools/opponents.sh fetch` und `build` holen und bauen quelloffene Snakes nach `data/opponents/` (vom Nutzer
freigegeben, nur lokal, nichts davon ins Repo übernehmen). `tools/opponents.sh spar duel 100 '{}' '<variante>'` spielt
jede Variante gegen **snork** (Agent `Flood`, 160 ms auf 4 Threads gegen unsere 40 ms auf einem): Das ist im Duell
etwa ausgeglichen und damit empfindlich für Verbesserungen. Die Sim nimmt fremde Snakes als Spieler B über
`--bhttp host:port[/pfad]` (Server) oder `--bcmd <programm>` (CGI-Programm wie Sandworm, Anfrage auf stdin).
Achtung: snork rechnet erst ab 150 ms Zeitlimit wirklich, darunter spielt es einen Zug tief. Nicht mehr als 4 Spiele
gleichzeitig (`THREADS`), sonst verfälscht die Last beide Seiten.

## Messfallen

- **Self-Play misst nur gegen uns selbst.** Eine Schwäche, die beide Seiten haben, bleibt unsichtbar (so blieb das
  späte erste Futter unentdeckt). Deshalb immer auch Stellungstest und Arena-Vergleich.
- **Rauschen:** 200 Spiele ±50 Elo, 400 ±35, 1000 ±22. Gewinner mit neuem `--seed` und mehr Spielen bestätigen.
- **Zeiten:** Nie zwei Sims gleichzeitig. Vor einem langen Lauf das Binary kopieren, dann darf nebenher gebaut werden.
  Die Sim rechnet sequentiell (ein Thread pro Schlange), der Server parallel.
- **Constrictor-Self-Play ist nicht verwertbar** (fast deterministisch, siehe Runde 6).
- **Korrelation ist nicht Ursache.** Der Längenvorsprung am Spielende sagt den Sieger fast sicher voraus, aber nur weil
  der Verlierer am Ende eingeschnürt wird. Auswertungen über Spielausgänge deshalb auf Stellungen beschränken, die
  mindestens 25 Züge vor dem Ende liegen (so macht es `tools/fit_duel.py`).
- **Timeouts:** Antwortet der Server zu spät, wiederholt die Engine den letzten Zug. Die Suche kennt diesen Zug
  (`SearchConfig::fallback`) und antwortet früher, wenn er teuer wäre. `EARLY_MS` und `LATE_MS` steuern das.

## Werkzeuge in `src/bin/`

| Binary | Zweck |
|---|---|
| `sim` | Self-Play mit offiziellen Regeln, Varianten, Feld, Verhaltensstatistik, fremde Gegner (`--bhttp`, `--bcmd`) |
| `bench` | Tiefe und Knoten/s auf einer Stellung (`VARIANT`, `BENCH_TIMEOUT=early,late`) |
| `agree` | unsere Züge gegen die gespielten Züge in Arena-Partien (Stellungstest) |
| `trace`, `pos` | Score-Verlauf einer Partie, Stellung als Anfrage-JSON |
| `blunders`, `suite` | entscheidende Fehlzüge aus verlorenen Partien, als wiederholbarer Test |
| `timing` | Anteil früher/normaler/später Antworten der Timeout-Regel auf Arena-Zügen |
| `duelfeat` | Merkmalstabelle aus Duellen für `tools/fit_duel.py` (Regression Sieg gegen Merkmale je Phase) |
| `policy`, `dataset` | ältere Werkzeuge (Gegner-Heuristik prüfen, Merkmale 4er) |
