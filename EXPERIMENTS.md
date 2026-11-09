# Experiment-Log: Weg auf Platz 1

Was probiert wurde, wie es ausging und was als Nächstes ansteht. **Vor jeder neuen Runde lesen,
nach jeder Runde ergänzen.** Neueste Einträge oben in jedem Abschnitt.

## Stand

Vor jedem Deploy eine Zeile mit Wertung und Spielzahl eintragen. Die Wertung läuft träge nach und
hängt vom Gegnerfeld ab. Für die Bewertung eines Commits zählen deshalb die Spiele **seit** seinem
Deploy: Siegquote, Platzierung und die Metrik, die er verbessern soll. Self-Play-Elo ist nur die
Vorhersage. Commits im Abstand von wenigen Stunden lassen sich in der Arena nicht trennen.

| Stand (UTC) | Commit | Standard | Duels | Royale | Constrictor | Notiz |
|---|---|---|---|---|---|---|
| 11-03 | vor Runde 1 | 52.3 (93 Spiele) | 36.8 (92) | – | – | |
| 11-05 ~07:45 | `83f811b` | 52.7 (#21, 259) | 36.0 (#14, 252) | – | – | Runde 2, erstes Futter bei Zug 10/16 |
| 11-05 08:44 | `95f79b6` deployt | – | – | – | – | Futter-Gradient, `eat_early`; 4er-Erstfutter danach bei Zug 2 |
| 11-05 10:50 | `4d079b7` deployt | 52.8 (273) | 35.4 (267) | – | – | Leeres-Brett-Fix; Duell-Erstfutter danach bei Zug 2 |
| 11-05 11:25 | `421b1d9` deployt | – | – | – | – | Constrictor-Profil; Royale und Constrictor kurz zuvor beigetreten |
| 11-05 11:37 | `36f0069` deployt | 53.0 (274) | 35.4 (269) | 17.4 (3) | 17.0 (3) | `parity` 0.5 |
| 11-05 13:10 | Messung, kein Deploy | 54.3 (#16, 281) | 35.9 (#15, 274) | in Platzierung (9) | in Platzierung (9) | seit dem Leeres-Brett-Fix: Standard 8/9 Siege, Duel 6/8 |
| 11-05 15:10 | Basis vor Royale-Profil | 54.2 (#17, 290) | 37.2 (#12, 283) | 28.0 (#17, 17) | 18.7 (#21, 18) | zum Vergleich nach dem Deploy; Metriken: Hungertode, Platzierung im Royale |
| 11-06 09:25 | Basis vor Duell-Fix | 57.4 (#10, 368) | 40.1 (#10, 360) | 33.1 (#14, 98) | 20.9 (#18, 98) | zum Vergleich nach dem Deploy; Metriken: Duell-Siegquote, `gg`-Serien ohne Sieg (bisher 54 von 90 Duellen) |
| 11-06 ~09:30–09:41 | `0357c78` deployt | – | – | – | – | Zeit aus dem Verhalten erschlossen, siehe Runde 6 |
| 11-06 10:20 | Messung, kein Deploy | 57,7 (#10, 374) | 40,4 (#10, 364) | 33,6 (#14, 102) | 20,9 (101) | global 152,9 (#10), Spitze Snakes Alive 184,7 |
| 11-09 08:50 | Basis vor Duell-Abklingen | 55,8 (#14, 670) | 41,9 (#8, 656) | 35,2 (396) | 21,2 (411) | seit `0357c78`: Duell 145–149 (49 %), Standard 27 % Siege; Podium Duell 48,7, Standard 61,3 |

Die globale Rangliste summiert die beste Wertung pro Leaderboard (fehlendes Board = 0).

Arena-Profil: <https://arena.battlesnake.com/battlesnakes/837dbb1c-2781-4e13-8b33-7d1b9c75f7b1/profile>
(Spielhistorie mit `?page=N`).

## Arbeitsablauf

1. **Spiele laden.** Die Arena hat eine Engine-API:
   - `https://arena.battlesnake.com/api/games/{id}` liefert Metadaten und den letzten Frame.
   - `https://arena.battlesnake.com/api/games/{id}/frames?offset=0&limit=100` liefert die Frames
     seitenweise. Pro Schlange gibt es `Body`, `Health`, `Latency`, `Death` (`Cause`, `Turn`,
     `EliminatedBy`) und `Error`. Ein Timeout steht als `Latency` 500 mit Error
     „A response was not received…“ drin.
   - Die Spiel-IDs stehen als `/games/<uuid>`-Links im HTML der Profilseite.
2. **Auswerten.** Todesursache und Zug, Timeouts, Latenz-Histogramm, eigene Länge im Vergleich
   zum längsten Gegner bei Zug 25/50/100/150, getrennt nach Sieg und Niederlage.
3. **Stellung nachrechnen.** Frame in Engine-JSON umwandeln (Format wie `tests/fixtures/*.json`),
   dann `target/release/bench <datei> 300` aufrufen. Das zeigt Tiefe, Score pro Zug und Knoten/s.
4. **Varianten testen.** Mit `tools/tune.sh <mode> <games> 10 '<basis>' '<variante>'…`
   (Self-Play, 10 ms pro Zug).
   - **Rauschen:** Bei 200 Spielen liegt es bei etwa ±50 Elo, bei 400 Spielen bei etwa ±35 Elo.
     Alles darunter ist nicht signifikant. Gewinner mit mehr Spielen bestätigen.
   - Ein 4er-Spiel dauert etwa 1,1 s (200 Spiele ≈ 4 min), ein Duell etwa 0,55 s.
   - Mehrere Sims gleichzeitig verfälschen die Zeiten. **Nicht neu bauen, während eine Sim läuft**,
     denn `tune.sh` startet das Binary pro Variante neu.
   - Die Sim nutzt die **sequentielle** Suche. Änderungen an der Parallelsuche lassen sich nur
     mit `bench` (Tiefe, Knoten/s) beurteilen.
   - Self-Play misst nur gegen uns selbst. Ob es gegen die echten Gegner hilft, zeigt erst die
     Arena.
   - `--a`/`--b` überschreiben jetzt das Produktiv-Profil des Modus (`profile()` in `src/lib.rs`).
   - Sims mit kopiertem Binary starten, dann darf nebenher gebaut werden (mit eigenem
     `CARGO_TARGET_DIR`). Bei Fehlern, die erst tief wirken, auch mit 40 ms testen.

## Beobachtungen aus Arena-Spielen

- **2026-11-06 (893 Spiele, davon 347 seit dem Beitritt zu Royale/Constrictor): Serverlast und Timeouts.**
  - Der Matchmaker startet alle 14,4 Minuten 4–5 Spiele innerhalb von etwa 7 s. Etwa 80 % der Zeit
    läuft bei uns gar kein Spiel, die Last kommt in kurzen Schüben.
  - Timeouts hängen nicht an der Parallelität: 11,3 / 5,8 / 8,5 / 6,1 / 0 pro 10k Züge bei 1 / 2 / 3 / 4 / 5
    Spielen (p99-Latenz 387–390 ms). Vor dem Beitritt 8,6, danach 7,6. Bei einem Timeout bleiben die anderen
    eigenen Spiele normal (Median 381 ms), nur 1 Paar Timeouts in verschiedenen Spielen liegt unter 5 s
    auseinander. Die Aussetzer treffen also einzelne Anfragen, nicht den ganzen Server.
  - Anteil der Rechenzüge: Duell 42,8 %, Standard 34,4 %, Royale 20,9 %, Constrictor 1,9 %.
  - Rechenzeit in Self-Play (je 200 Spiele, ±50 Elo): halbe Zeit (40 gegen 20 ms) Standard −24, Duell −65;
    viertel Zeit (40 gegen 10 ms) Standard −74, Duell −129.
  - Latenz-Rand: 121 Züge ≥ 400 ms, davon 58 ≥ 500 ms. Wir antworten mit p50 381 ms am spätesten im Feld
    (NMC² 319 ms bei 0 Timeouts in 3949 Zügen, corrosion 350 ms, Kaisel 250 ms).
  - 9 von 347 Spielen endeten innerhalb von 2 Zügen nach einem Timeout. Darunter Duell `b212cf8f`, zwei
    Timeouts in Zug 1–2 um 20:26 UTC, vermutlich Neustart beim Deploy (Zeitpunkt nicht belegt).
  - Offen: Die Suche rechnet immer bis zur Deadline, verlorene Tiefe ist deshalb in der Latenz unsichtbar.
    Dafür braucht es die `MOVE`-Logs (`depth`, `nodes`) und die Kernzahl des Servers.
- **2026-11-05 (512 Spiele, Vergleich mit allen Gegnern):**
  - Erstes Futter im Median bei Zug 10 (4er) bzw. 16 (Duell), bei fast allen Gegnern bei Zug 2.
    Duell-Länge bei Zug 50: 5,8 gegen 7–9. Self-Play konnte das nie zeigen, beide Seiten
    hatten denselben Fehler. **Darum immer auch gegen die echten Gegner vergleichen.**
  - 12 Hungertode in 4er-Spielen, mit Länge 4–7 bei Zug 100–150 und Futter wenige Felder entfernt.
  - Ursachen: siehe Runde 3. Fast alle `self-collision`-Tode sind Einsperren 5–15 Züge vorher.
  - Timeouts: 0,05 % der Züge, ähnlich wie bei anderen starken Snakes, jeder vierte betrifft auch
    den Gegner. Etwa 10 von 170 Niederlagen fielen auf einen Timeout-Zug.
- **2026-11-03 (137 Spiele, 91 Siege):**
  - Etwa 5 Niederlagen sind genau im Timeout-Zug passiert (die Engine lässt die Schlange dann
    geradeaus fahren, meist in die Wand). Die Latenz lag fast immer bei 430–440 ms bei einem
    Limit von 500 ms. Es gab 32 Timeouts auf etwa 17 000 Züge, also seltene, große Spikes.
    Die normale Netzlatenz liegt bei etwa 20 ms.
  - Wir sind fast immer **kürzer** als der längste Gegner. Bei Zug 100 sind es in 4er-Spielen
    etwa −5 (Niederlagen) bzw. −3 (Siege), in verlorenen Duellen etwa −2. Die Gegner fressen
    früh viel aggressiver.
  - Typischer Tod in 4er-Spielen: Ein größerer Gegner läuft parallel neben uns an der Wand
    entlang, drängt uns in die Ecke, und es folgt ein Kopf-an-Kopf oder wir sind eingesperrt.
    Die Suche (4er-Spiel: Tiefe 5–7) sieht das erst, wenn es schon verloren ist.
  - Duell-Niederlagen enden meist mit `self-collision` im Endspiel: Wir verlieren den Platz,
    nachdem wir das Futter-Rennen verloren haben.
  - Züge mit etwa 20 ms Latenz sind normal. Dann gibt es nur einen sicheren Zug, oder Sieg bzw.
    Niederlage ist schon bewiesen.

## Experimente

Elo-Werte sind aus Self-Play, Variante (A) gegen die Basis (B). ✅ übernommen, ❌ verworfen,
➖ neutral bzw. nicht übernommen.

### Runde 7 (2026-11-09): Arena seit `0357c78` und Längen-Abklingen im Duell

Arena seit dem Deploy (11-06 09:45 bis 11-09 08:00, 295 Duelle, 299 Standard-Partien). Die Siegquoten fielen (Duell 67 % → 49 %,
Standard 39 % → 27 %), vor allem weil das Matchmaking jetzt stärkere Gegner zuteilt (Duell: 50 Spiele gegen die Top 3, vorher 6).
Duell gegen die Spitze: Bwukinator 0–15, Odiodin 3–14, rattler 3–19, NMC² 9–9, AlphaSlinky 12–15.

Befunde Duell:
- In 72 von 149 Niederlagen blieb der Score (Suche mit 50 ms, `agree`) bis weniger als 25 Züge vor dem Ende positiv. 32 davon mit
  mindestens +5 Länge, mehrere mit +17 bis +37. Beispiel `24487783`: Zug 362, Länge 38 gegen 17, Score +250 in Tiefe 17, danach
  schließt der Gegner die Tür. Die Längenterme (bis 265 Punkte) überdecken jeden Platzverlust (±30–50).
- 41 Niederlagen gegen die Top 5 waren schon ab Zug 2–50 dauerhaft im Minus. Die Top 5 liegen bis Zug 20 im Schnitt 0,6 Länge vorn
  (gegen den Rest liegen wir vorn). Offen.
- Die Niederlage ist im Median 15 Züge vor dem Tod bewiesen (`ouch` oder erzwungen), der Fehler liegt also jenseits des Horizonts.
- Latenz: p50 381 ms, p99,9 446 ms, 94 Timeouts in 162k Zügen. Mehr Budget (400 ms) brächte etwa 28 Timeouts mehr für +0,15 Halbzüge. ➖

Befunde Standard (noch ohne Änderung):
- Randanteil in Zug 0–25: 41 % gegen 10–23 % bei allen starken Gegnern, Abstand zur Mitte der höchste im Feld. 171 von 217 Toden
  innerhalb eines Feldes von der Wand. Alle 67 Kopf-an-Kopf-Tode erzwungen und gegen größere Köpfe.
- Länge bei Zug 100: −3,4 zum Längsten, schlechtester Wert im Feld. 18 Hungertode, davon 6 mit Länge 4 bei Zug 102.
- Vermutung: Die paranoide Suche hält Rand und Ecke für sicher, weil dort weniger Gegner gemeinsam angreifen können.

Was die Spitze macht: Ihr Code ist nicht öffentlich. `snork` (früher Platz 1–2, MIT, nur gelesen, nicht ausgeführt) lässt den
Längenvorteil mit der Zugzahl verfallen (`size_adv_decay`) und wertet später fast nur den Platzanteil.

Neue Gewichte: `length_half`/`length_min` (Abklingen aller Längenterme), `eat_want`, `area_ratio`, `center`. Die Sim kann mit
`--x <json>` in 4er-Partien zwei Feld-Schlangen setzen und wertet dann A gegen B nach Überlebensdauer (`--xreach`, `--xgreedy`).

Duell, gegen das Produktiv-Profil:

| Variante | ms | Spiele | Elo |
|---|---|---|---|
| `length_half` 150, `length_min` 0.3 | 20 | 160 / 300 (zwei Seeds) | **+103 / +88** |
| `length_half` 100 / 0.2, 250 / 0.3, 150 / 0.5 | 20 | je 300 | +63 / +62 / +58 |
| `eat_want` 1 | 20 | 300 | +13 ➖ |
| `area_ratio` 1 / 3 | 20 | je 300 | +35 / +60 |
| `opp_area` 1.2 + `length` 10 | 20 | 300 | +92 |
| auf Basis 150/0.3: + `opp_area` 1.2 + `length` 10 / + `area_ratio` 3 / + `opp_area` 1.2 | 20 | je 300 | +6 / +7 / +6 |
| auf Basis 150/0.3: alle drei zusammen (D+) | 20 | 300 | +42 |
| **150/0.3 (D)** | 40 | 300 | **+78** |
| D+ | 40 | 300 | +102 |

Gegnerpool (je 200, 10 ms): Basis 0,513, **D 0,622** (self 0,642, hungry 0,652, territory 0,585, cautious 0,550, long 0,682),
D+ 0,590 (territory 0,472). Stellungstest (Top-5-Züge in gewonnenen Duellen, 50 ms, n = 5197): Übereinstimmung 63,4 % → 64,5 %,
Fehlgriffe 3,0 % → **1,2 %**, ab Zug 250: 59,2 % → 61,4 %, Fehlgriffe 4,8 % → 1,6 %.

✅ D ins Duell-Profil übernommen (wirkt auch im 4er-Endspiel mit einem Gegner). D+ verworfen: verliert gegen „territory“, kein
Vorteil im Stellungstest erwartet. Arena-Metriken für die Bewertung: Siegquote gegen Rang 1–8, Niederlagen mit Längenvorsprung ≥ 5.

### Runde 6 (2026-11-06): Tempo und Testumgebung

**Regel:** Erst wieder pushen, wenn die aktuelle Version genug gespielt hat und in der Arena stagniert. Nur so lässt sich
sehen, ob eine neue Änderung wirklich besser ist. Lokale Kandidaten warten bis dahin.

**Offen:**
1. Frühestens ab 11-07 ~09:30 UTC (etwa 100 Duelle seit dem Deploy) `0357c78` bewerten: Siegquote (vorher 67 %), `gg`-Serien
   ohne Sieg (54 von 90), Bilanz gegen die Top 5 (0–9). Pro Takt läuft etwa ein Duell.
2. `MOVE`-Logs vom Server holen: Tiefe bei einem Spiel gegen vier gleichzeitige Spiele. Der Server hat 8 Threads.
3. Duel-Kandidat `opp_area` 1.2 + `length` 10 mit neuem Seed bei 40 ms bestätigen.
4. Constrictor-Messung reparieren (siehe unten), dann Hazard-Term für Royale und Kammerbaum für Constrictor.
5. Danach Punkt 3 und 4 unten.

**Deploy von `0357c78` belegt:** `bench` des alten Stands `70e89ce` und des neuen auf allen Zügen der Duelle. Vor dem Push rief der
Server in allen 20 Stellungen `gg`, in denen nur der alte Stand einen Sieg sieht (19 davon in `6bf4f36a`). Nach dem Push
in keiner von 3 (`786ac65a` Zug 234 und 515, `3429c8f0` Zug 96). Bei 300 ms bleibt das stabil: alt Tiefe 3 und `999970`,
neu Tiefe 12–15 und Score 82–268. Spätester Zeitpunkt ist `786ac65a` Zug 234 (Start 09:39:27).

Timeout-Cluster, unbelegt: Um 09:25 gab es Timeouts in drei Spielen bei Zug 20–21, 26 und 36–40. Geschätzt aus Startzeit und
Zugnummer liegen sie innerhalb weniger Sekunden, zwei andere Spiele im selben Takt blieben frei. Das Log vom 11-06 fand
solche Häufungen bisher kaum, die genauen Zeiten fehlen noch.

Tempo (Bench, 300 ms, 43 Arena-Stellungen, lokal):

| Suche | Duel Tiefe | Duel Knoten | Standard Tiefe | Standard Knoten |
|---|---|---|---|---|
| sequenziell | 11,1 | 154k | 6,83 | 108k |
| parallel (`main`) | 11,3 | 349k | 6,87 | 223k |
| PV-Splitting (Branch `pv-split`) | 11,3 | 206k | 6,91 | 224k |

Sequenziell je Verdopplung der Rechenzeit: Duel etwa +0,7 Züge, Standard etwa +0,3 Züge (75 bis 1200 ms). Tempo ist in
Standard kein Hebel. `pv-split` ist korrekt (gleiche Werte wie sequenziell bis Tiefe 7), bringt aber keine Tiefe, im
schmalen Duell-Baum arbeiten höchstens zwei Threads. ➖ geparkt, ungemessen in Elo. Mehr Threads helfen also nicht, nur schnellere Kerne.

Gegnerpool (`tools/gauntlet.sh`, 10 ms, Commit `0357c78`, Score der Basis):

| Stil | Duel (je 200) | Standard (je 150) |
|---|---|---|
| self | 0,497 | 0,520 |
| hungry | 0,583 | 0,553 |
| territory | **0,400** | 0,510 |
| cautious | 0,500 | 0,443 |
| long | 0,578 | 0,593 |
| Mittel | 0,512 | 0,524 |

Stellungstest (`tools/fetch_top.py`, `agree 100 8 all`, `tools/top_report.py`), Top-5-Züge in gewonnenen Partien. Ein früherer Lauf
nur im Duel: 63,6 %, Fehlgriffe 2,3 % (n = 2078). Vollständiger Lauf mit `0357c78`, 219 Partien:

| Modus | n | Übereinstimmung | Fehlgriffe | Zug < 25 | ab Zug 250 |
|---|---|---|---|---|---|
| Duel | 4236 | 62,8 % | 2,3 % | 72,0 % | 59,9 % |
| Standard | 2520 | 63,1 % | 2,1 % | 63,8 % | 61,3 % |

Kandidat Duel (das bisherige `opp_area` 0.6 wurde nur in 4er-Spielen getestet, Runde 2), Elo gegen die Produktiv-Gewichte:

| Variante | Spiele | ms | Elo |
|---|---|---|---|
| `opp_area` 1.2 + `length` 10 | 800 (zwei Seeds) | 10 | +60 (Score 0,587) |
| `opp_area` 1.2 + `length` 10 | 300 | 40 | **+94** |
| `opp_area` 1.2 | 600 | 10 | +48 |
| `length` 10 | 600 | 10 | +27 |

Gegnerpool mit `opp_area` 1.2 + `length` 10 (je 200, 10 ms): self 0,555, hungry 0,662, territory 0,470, cautious 0,542, long
0,700, Mittel **0,586** (Basis 0,512). Stellungstest gepaart (n = 4236): Übereinstimmung 62,6 % gegen 62,8 %, nur Kandidat 272,
nur Basis 281, Fehlgriffe 2,1 % gegen 2,3 %, also neutral. Zug < 25: 68,8 % gegen 72,0 %, Fehlgriffe 1,1 % gegen 0,4 %.

`opp_area` 1.2 allein: 10 ms +48 (600), 40 ms +31 (600), Gegnerpool (je 200) self 0,530, hungry 0,552, territory 0,480, cautious
0,550, long 0,595, Mittel 0,541. Stellungstest gepaart: 63,0 % gegen 62,8 %, Fehlgriffe 2,9 % gegen 2,3 %.

**Stand:** Das Paar `opp_area` 1.2 + `length` 10 liegt lokal vor `opp_area` allein (40 ms +94 gegen +31, Pool 0,586 gegen 0,541,
Fehlgriffe 2,1 % gegen 2,9 %). `length` 10 allein war bei 10 ms nur +27. Einwand: Zug < 25 wird im Stellungstest schlechter, und
Self-Play zeigt Futter-Schwächen nicht (Runde 3). Vor dem Deploy das Paar mit neuem Seed und 600 Spielen bei 40 ms bestätigen.
✅ Kandidat, nicht deployt.

Gewichts-Serie (10 ms, gegen das Produktiv-Profil des Modus, je ein Seed):

| Board | Spiele | Variante und Elo |
|---|---|---|
| Duel | 600 | `opp_area` 0.9 +21, 1.5 +47, 2.0 +49; `longest_bonus` 30 −4 |
| Royale | 500 | `hazard_cell` 0.5 +9, `edge` 1.5 −8, `trap` 20 −16, `opp_area` 0.9 −8, `food_dist` 2.5 −4, `health` 0.45 +12 ➖ |
| Standard | 500 | `opp_area_sum` 0.3 +9, 0.05 +3; `trap` 18 ±0, `edge` 4.5 −4, `longest_bonus` 30 +24 ➖ |

Royale und Standard reagieren auf keine dieser Gewichte, dort braucht es neue Terme.

**Constrictor-Self-Play ist so nicht verwertbar.** Die Werte springen, obwohl jeder Lauf 1000–3000 Spiele hat:

| Variante | 10 ms | 40 ms |
|---|---|---|
| `opp_area` 0.7 | −121 (1000) | |
| `opp_area` 1.5 | +131 (1000), +68 (3000, neuer Seed) | **−116** (1000) |
| `opp_area` 2.0 | −6 (3000) | −23 (1000) |
| `opp_area` 3.0 | **+240** (3000) | |
| 1.5 + `opp_area_sum` 0.6 | −30 (3000) | |
| 1.5 + `parity` 0.25 | +16 (3000) | |
| 1.5 + `opp_area_sum` 0.6 + `parity` 0.25 | −9 (3000) | |
| 2.0 + `parity` 0.25 | +41 (3000) | |

Dazu schwankt der Remis-Anteil zwischen 0,1 % und 49 %, und die Vorzeichen kippen zwischen 10 und 40 ms. Vermutung, nicht geprüft:
Ohne Futter ist das Spiel fast deterministisch, wenige Eröffnungslinien entscheiden, und die effektive Spielzahl liegt weit unter
der nominellen. Das stellt auch den Constrictor-Eintrag `opp_area` 1.0 (+68) aus Runde 3 in Frage. ❌ Keine Constrictor-Gewichte
übernehmen, bevor das geklärt ist: erst die Zahl verschiedener Partien zählen, dann mit zufälligen Startzügen oder gegen den
Gegnerpool messen.

Nächste Ideen für Duel:
- **Punkt 3:** Kipppunkte der Top-5-Niederlagen suchen (Score bleibt ab dort unter −30) und vergleichen: Fluchtwege, Engstellen, sicheres Gebiet.
- **Punkt 4:** Bei nahen Köpfen an der Wurzel eine 3×3-Tabelle rechnen und als Spiel mit gleichzeitigen Zügen lösen.
  Zuerst messen, wie oft das den Zug ändert. Abbrechen, falls selten.
- **Punkt 5:** Erst nach dem Arena-Urteil. Kammerbaum wie bei Tron-KIs: Gebiet an Artikulationspunkten zerlegen, nur den
  besten Pfad werten. Teuer, also nur bei vollem Brett rechnen.

Ein neuer Term braucht: Sieg im Self-Play, besseres Mittel im Gegnerpool, Stellungstest nicht schlechter.

### Runde 5 (2026-11-06): Falsche Siegbeweise und Stagnation

Die Arena-Wertung ist flach geworden (Delta pro 100 Spiele seit 11-05 14:00, 95 %-CI): Standard +2,9
(−0,4 bis +6,3), Duel +4,3 (+1,0 bis +7,6), Royale +12 (−2 bis +27), Constrictor +5 (−10 bis +20).
Duel seit dem Leeres-Brett-Fix: 0–9 gegen die Top 5 (ein Spiel fiel durch 2 Timeouts in Zug 1–2),
60–21 gegen den Rest.

**Fehler in `src/search.rs` (`min_node`):** Gegner außerhalb von `reach = 2*depth+1` bekommen nur
`default_move`. Stirbt so ein Gegner in der Suche, gilt das als Sieg, die Vertiefung stoppt, und der Server
antwortet in 20 ms mit `gg`. Beispiel `d3087934` Zug 434: Tiefe 5, 95 Knoten, `999950`, bei 44 gegen 19
Länge, 16 Züge später tot. Seit dem Fix hatten 54 von 90 Duellen, 30 von 92 Standard-, 26 von 94 Royale- und
26 von 94 Constrictor-Spielen `gg`-Serien, die nicht zum sofortigen Sieg führten. Nicht jede davon muss von
`reach` kommen (neues Futter kann einen echten Beweis kippen). 10 von 30 Duell-Niederlagen hatten `gg`.

Self-Play bei 40 ms gegen die Basis. Übernommen ist nur `duel_full`, und zwar ohne Schalter: Es gilt, sobald bei Suchbeginn
nur noch ein Gegner lebt, also auch im Endspiel von 4er-Partien. Dieser Fall ist im Self-Play nicht einzeln gemessen.
`soft_win` bleibt draußen, weil es in 4er-Partien nichts Messbares bringt.

| Variante | Modus | Spiele | Elo |
|---|---|---|---|
| im Duell alle Gegnerzüge rechnen (`duel_full`) | Duel | 400 | **+78** |
| Sieg nur bei vollständig aufgezählten Gegnerzügen (`soft_win`) | Standard | 300 | +20 |
| `soft_win` | Royale | 300 | −10 |
| `soft_win` | Constrictor | 600 | +5 |

Weitere Befunde seit dem Fix:
- Duel: 10 von 30 Niederlagen mit mindestens +3 Länge bei der letzten Entscheidung, 8 mit mindestens +10.
  Gegen die Top 5 lag der Score oft schon 90–190 Züge vor dem Tod dauerhaft unter −30.
- Bis Zug 200 sagt der Such-Score das Ergebnis nicht besser voraus als die Längendifferenz (AUC 0,66–0,71).
- Standard: Bei Niederlagen −4,3 Länge bei Zug 100, bei Siegen −0,4. 43 von 56 Toden lagen mindestens 3 zurück.
- Royale: 42 von 50 Toden durch Hazard oder Hunger, im Median mit 55 sicheren Feldern. Etwa 10 Tode genau
  7 Züge nach einem Futter im Hazard, 6-mal hatte der letzte Gegner dabei höchstens 5 HP. Hazard-Anteil der Züge
  12,3 % gegen 7,7 % bei den Gegnern. Das Royale-Profil verschob Hunger- zu Hazard-Toden, ein Rückschritt ist
  bei gleicher Gegnerstärke nicht nachweisbar.

Nächste Schritte: `duel_full` übernehmen und einzeln in der Arena bewerten, dann Royale-Bewertungsterm für die
Rückkehr aus dem Hazard (Gesundheit gegen Distanz zum sicheren Feld), danach Futter in Standard. Gewichts-Tuning
ist ausgereizt (Runde 4), gegen die Top 5 braucht es vermutlich eine bessere Bewertung, keine neuen Gewichte.

### Runde 4 (2026-11-05): Hypothesen aus den Spielen der Top-Snakes

Vergleich unserer Suche mit 434 Spielen der Spitze (`src/bin/agree.rs`): Wir stimmen in etwa 60 %
der Züge überein, etwa 5 % sind große Abweichungen. Muster dort: Die Spitze lässt Futter bei hoher
Gesundheit liegen (HP > 50 in 93 %, Median-Zug ~350, unabhängig vom Längenvorsprung), jagt kleinere
Köpfe häufiger, läuft öfter zum eigenen Schwanz und im Royale öfter zur Mitte. Die Muster stammen aus
Spielen gegen stärkere Gegner, Self-Play kann sie also nur begrenzt bestätigen.

Self-Play bei 10 ms gegen die Basis, 500 Spiele Duell, je 400 Spiele Standard und Royale. Rauschen
etwa ±35. Alle Gewichte wurden wieder entfernt.

| Variante | Duell | Standard | Royale | Ergebnis |
|---|---|---|---|---|
| `want_length_lead` 2 | +8 | −2 | | ➖ |
| Längen-Term bei Vorsprung × 0.3 | −1 | −10 | | ➖ |
| Eat-Bonus nur bei Bedarf | +4 | +9 | | ➖ |
| beides zusammen | −8 | +23 | | ➖ |
| Jagd 5 pro kleinerem Kopf in Distanz ≤ 2 | +4 | −5 | +3 | ➖ |
| Jagd 12 | +5 | +4 | | ➖ |
| Abstand zum eigenen Schwanz 0.5 | +10 | −17 | | ➖ |
| Mitte 1 | −29 | +3 | −11 | ➖ |
| Mitte 2.5 | | | −10 | ➖ |

Arena-Partie `6022c965` (Royale): Hungertod in Zug 74 bei Länge 4, obwohl Futter lag. Die Züge lagen nur
0,2 Punkte auseinander, auch der Schritt in die Gefahrenzone. Daraus entstand ein eigenes
Royale-Profil, gegen die Basis:

| Variante | Royale | Status |
|---|---|---|
| `hunger_health` 50 / 70 | +14 / +62 (je 400) | Tendenz |
| `health` 0.3 / 0.6 | +34 / +47 (je 400) | Tendenz |
| `food_dist` 3 | +23 (400) | ➖ |
| `health` 0.3 + `hunger_health` 50 / 70 | +27 / +23 (je 800) | |
| `health` 0.6 / 1.0 + `hunger_health` 70 | +44 / +47 (je 800) | |
| `health` 0.3 + `hunger_health` 101 | +44 (800) | |
| `hunger_health` 90 mit `health` 0.02 / 0.1 / 0.3 / 0.6 | +50 / +49 / +65 / +29 (je 800) | |
| `health` 0.3 + `hunger_health` 90 + `food_dist` 3 | +40 (800) | ➖ |
| **Fertiges Profil `health` 0.3, `hunger_health` 90, gegen den alten Stand, 40 ms** | **+58** (400) | ✅ nur Royale |

Erkennung: `game.map == "royale"` oder Gefahrenfelder auf dem Brett. Die Einstellung
`shrinkEveryNTurns` nutzen wir nicht: Der Simulator sendet sie in jedem Modus, und für die echten
Arena-Anfragen ist das nicht geprüft. Für
Standard und Duell brachte `hunger_health` 50 in Runde 3 nichts.

Constrictor, Kopf-an-Kopf-Tausch statt Niederlage (`mutual`): −150 zuerst +27 (600 Spiele), dann
−34 (3000 Spiele); −250 / −600 / −2000: +6 / +10 / +3 (je 3000). ❌ Lehre: Ein Ausreißer in 600
Spielen trägt keine Entscheidung, also immer mit neuem Seed und mehr Spielen wiederholen.

### Runde 3 (2026-11-05): Futter

| Änderung | Ergebnis | Status |
|---|---|---|
| Leeres Brett ist kein Verhungern (`food_empty`): Die Suche spawnt kein Futter, das letzte Futter zu fressen kostete deshalb −200 und den maximalen Distanz-Malus | 4er: **+301**, Duell: **+142** (10 ms), **+166** (40 ms) | ✅ gegen den Stand mit dem Futter-Gradienten |
| Gradient auch zu umkämpftem Futter (`food_contest` 3) statt nur zu Voronoi-eigenem | 4er: +110 | ✅ |
| Bonus fürs frühe Fressen in der Suche (`eat_early` 8, Abfall 0,85 pro Halbzug), gegen Aufschub am Horizont | zusammen mit dem Gradienten 4er: **+153**, Duell: **+110**; allein 4er: +19 | ✅ |
| `hunger_health` 50 | 4er: −33, Duell: −11 | ❌ |
| `want_length_lead` 6 | 4er: −19, Duell: +9 | ➖ |
| `eat_early` 16 | 4er: +12, Duell: +3 | ➖ |
| `food_contest` 1.5 | 4er: +29, Duell: −12 | ➖ |
| `food_dist` 3 | 4er: +38, Duell: +2 | ➖ auf neuer Basis nachtesten |
| Best-Reply-Search (ein Gegner weicht ab statt aller gemeinsamen Antworten) | 4er: −53 | ❌ |
| Transposition Table auf Pfad-Hashes zur Zugsortierung | keine Knotenersparnis bis Tiefe 13/8 | ❌ Killer sortieren schon fast perfekt (ca. 1,6 Kinder pro Halbzug). Der Hebel ist die Bewertung. |
| Gewichte per logistischer Regression aus Arena-Spielen (`src/bin/dataset.rs`) | 60 % Trefferquote im Duell, in 4er-Spielen kollinear und mit kippenden Vorzeichen | ❌ keine brauchbaren Gewichte |

| `parity` 0.5: Schachbrett-Schranke halb auf das Gebiet anrechnen | Duell +25 (1000 Spiele), +12 (40 ms, 400), 4er +25 (800), Royale +16 (500) | ✅ einzeln knapp, zusammen klar positiv |
| `parity` 1.0 | 4er: −40 | ❌ |
| Constrictor: `opp_area` 1.0, `opp_area_sum` 0.3 | +68 (1000 Spiele) | ✅ nur Constrictor-Profil |
| Constrictor: zusätzlich `parity` 1.0 | fällt auf +20 | ❌ |
| Constrictor: `opp_alive` 0 | −113 | ❌ |
| `food_owned` 5 | 4er: −17, Duell: −11 | ❌ |
| `length_each` 8 | 4er: −3 | ❌ |
| `mutual` −150 / −400 (Kamikaze-Tausch nicht als vollen Verlust werten) | 4er: +5 / +22, Duell: +6 / −17, Royale: −23 | ➖ bleibt aus |
| `food_dist` 3 auf neuer Basis | 4er: +12, Duell: +26 | ➖ |
| `hazard_cell` 0.1 / 0.5 | Royale: −12 / −9 | ➖ |

Die Zeilen ab `food_contest`-Gradient bis zur logistischen Regression liefen noch mit dem Leeres-Brett-Fehler, die Tests ab `parity` schon ohne.

### Runde 2 (2026-11-03): Suche und 4er-Gewichte

| Änderung | Ergebnis | Status |
|---|---|---|
| Geteiltes Alpha zwischen den parallelen Root-Threads, Unterscheidung exakter Werte und Schranken bei der Zugwahl | `bench`: +1 Halbzug (4er 5→6, Duell 11→12) | ✅ |
| Eigene `Arc<Rules>`-Kopie pro Such-Thread, `advance` ohne `Arc`-Clone | `bench`: +10–18 % Knoten/s | ✅ |
| `edge` 3 + `trap` 12 | 4er: +26 Elo (300 Spiele), einzeln +38/+40 (200 Spiele); Duell: 0 (400 Spiele) | ✅ nur im Mehrspieler-Profil (`src/lib.rs`) |
| `edge` 3 allein (Duell) | +15 (400 Spiele) | ➖ |
| `trap` 12 allein (Duell) | −14 (400 Spiele) | ➖ |
| `opp_area` 0.6 → 0.8 | 4er: −47 | ❌ |
| `max_relevant` 3 → 2 (nur 2 Gegner voll durchrechnen) | 4er: +3 | ➖ |
| Reichweite der Gegner begrenzen (`max_reach` 5 / 7) | 4er: −133 / −61 | ❌ deutlich schlechter. Weit entfernte Gegner müssen voll mitgerechnet werden. |

### Runde 1 (2026-11-03): Timeouts und Länge 

| Änderung | Ergebnis | Status |
|---|---|---|
| `MAX_BUDGET_MS` 420 → 360 | gegen Timeout-Spikes (etwa 100 ms Puffer statt 60) | ✅ Arena-Effekt noch prüfen |
| `want_length_lead` 2 → 4 | 4er: +53, Duell: +10 | ✅ |
| `length` 15 → 25, `length_cap` 8 → 10 | 4er: +57, Duell: +80 | ✅ |
| beides kombiniert | 4er: +53 (200 Spiele), Duell: +84 (300 Spiele) | ✅ |
| `food_dist` 1.5 → 3 | 4er: −13 | ❌ |
| `want_length_lead` 6 + `length` 25 / `cap` 12 | 4er: −7 | ❌ |
| `length` 35 / `cap` 12 (gegen die neue Basis) | 4er: +7, Duell: +10 | ➖ Sättigung |
| `longest_bonus` 15 → 30 (gegen die neue Basis) | Duell: +31 (300 Spiele) | ➖ unsicher, ggf. mit 600 Spielen nachtesten |

## Profiling-Zahlen (4er-Stellung, Ryzen mit 12 Threads)

- `evaluate` etwa 640 ns, davon Voronoi etwa 600 ns
- `clone` + `advance` etwa 380 ns, davon `clone` etwa 90 ns
- Die Blätter dominieren: Pro Blatt fallen etwa 1 µs an.

## Offene Ideen / nächste Schritte

Grob nach erwartetem Nutzen sortiert.

1. **Arena-Wirkung von Runde 1+2 prüfen:** Timeouts weg? Längendifferenz bei Zug 100 besser?
   Rating-Verlauf?
2. **Evaluation beschleunigen:** Voronoi braucht etwa 600 ns. Ideen: inkrementell rechnen,
   bei großen Bodies früher abbrechen, `advance` mit Bitboard-Belegung statt Body-Iteration,
   `VecDeque` durch einen festen Ringpuffer ersetzen (spart die Allokationen beim Clone).
3. **4er-Paranoid entschärfen:** Alle Gegner kooperieren gegen uns, das ist sehr pessimistisch.
   Alternativen: Max-N oder „Best-Reply-Search“ (nur der gefährlichste Gegner antwortet voll).
   Achtung: Weniger Gegner voll durchrechnen hat bisher nicht geholfen (Reach-Cap ❌,
   `max_relevant` 2 ➖).
4. **Abdrängen an der Wand erkennen:** Ein Eval-Term für einen größeren Gegnerkopf, der parallel
   neben uns an der Wand läuft, oder für Fluchtkorridore (zum Beispiel Größe der Region, die wir
   *vor* allen anderen erreichen, nach 2–3 Schritten).
5. **Futter-Rennen im Duell:** Die Gegner wachsen früh schneller. Die Hungerschwelle
   (`hunger_health`) und eine Futter-Kontrolle (Futter nahe dem Gegner wegnehmen) testen.
6. **`longest_bonus` 30** mit mehr Spielen nachtesten (+31 im Duell, unsicher).
7. **Transposition Table:** bisher bewusst nicht gebaut. Gleiche Stellungen sind selten, weil
   sich die Bodies unterscheiden, und die Killer-Züge übernehmen die Sortierung schon.
   Vielleicht nur zur Zugsortierung.
8. **Zeitmanagement:** Wenn ein Spiel einen Timeout hatte, für den Rest des Spiels ein größeres
   Sicherheitspolster nehmen. Prüfen, ob `MAX_BUDGET_MS` in Coolify gesetzt ist (es überschreibt
   den Default).
