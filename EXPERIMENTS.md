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
| 11-09 10:15 | `7536b4c` deployt | – | – | – | – | Duell: Längen-Abklingen (Halbwertszeit 150, min. 30 %) |
| 11-11 15:40 | Messung, kein Deploy | 55,8 (#15, 906) | 42,4 (#8, 893) | – | – | seit `7536b4c`: Duell 108–120 (47 %), Top 5 10–57, Standard 30 % Siege, 32 % Letzter; Spitze Duell 52,3, Standard 64,5 |
| 11-11 19:00 | Basis vor Fokus-Suche | 56,1 (#14, 919) | 42,8 (#8, 908) | – | – | Standard seit `7536b4c`: 32 % Letzter, Rand 38 %, Länge bei Zug 100: 8,6; Tode im Timeout-Zug 11-08 bis 11-11: 33 in 1568 Partien |
| 11-11 19:10 | Runde 8 deployt (Push, Build einige Minuten später) | – | – | – | – | Timeout-Regel (alle Modi), Standard: Fokus-Suche + `smart`. Auswerten mit `report --since '2026-11-11 19:30'` |

Die globale Rangliste summiert die beste Wertung pro Leaderboard (fehlendes Board = 0).

Arena-Profil: <https://arena.battlesnake.com/battlesnakes/837dbb1c-2781-4e13-8b33-7d1b9c75f7b1/profile>
(Spielhistorie mit `?page=N`).

## Arbeitsablauf

Der Kreislauf (Daten holen, Unterschied zur Spitze messen, Variante bauen, Testleiter, Arena) steht mit allen Befehlen
in [CLAUDE.md](CLAUDE.md). Hier nur die Zahlen, die man dafür braucht:

- **Daten:** `python3 -I tools/arena.py sync` legt alles in `data/` ab (eigene Partien und die der Top 8). Die Arena
  liefert etwa 40 Partien pro Minute. `tools/arena.py report --since '<UTC>'` ist die Bilanz seit einem Deploy.
- **Rauschen im Self-Play:** 200 Spiele ±50 Elo, 400 ±35, 1000 ±22. Die Sim druckt das 95-%-Intervall.
- **Dauer:** Ein Duell bei 40 ms etwa 2 s pro Kern, eine 4er-Partie bei 40 ms etwa 40–50 s pro Kern (200 Partien
  ≈ 8 min auf 16 Threads).
- **Nicht zwei Sims gleichzeitig**, und vor langen Läufen das Binary kopieren (dann darf nebenher gebaut werden).
- Die Sim rechnet sequentiell. Änderungen an der Parallelsuche zeigt nur `bench` (Tiefe, Knoten/s).
- Mit `--av/--bv/--xv` nutzt die Sim pro Stellung das Produktiv-Profil plus Variante, wie der Server (auch den Wechsel
  ins Duell-Profil im 4er-Endspiel). `--a/--b` setzen feste Gewichte für die ganze Partie (älterer Weg).
- **Stellungstest:** `agree` + `tools/agree_cmp.py` (gepaart). Der Lauf über 538 Standard-Partien mit 50 ms dauert
  1,5 min. Zwei Läufe derselben Variante unterscheiden sich um etwa ±0,3 Punkte (zeitgesteuerte Suche).

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

### Runde 9 (2026-11-11 abends): fremde Snakes als Sparringspartner

Der Nutzer hat erlaubt, quelloffene Snakes lokal zu bauen. Aufbau und Bedienung: CLAUDE.md, „Fremde Gegner“,
`tools/opponents.sh`. Die Sim kann jetzt neben `--bhttp` (mit Pfad) auch CGI-Programme (`--bcmd`).

- **snork** (`wrenger/snork`, Rust, MIT, Stand 2024-08; laut README früher Platz 1–2 der Arena). Agent `Flood`:
  MaxN-Suche über eine Flood-Fill-Bewertung, 5–6 Züge tief in 160 ms.
  - Zeitlimit ≤ 150 ms: snork spielt nur einen Zug tief, wir gewinnen 39–1 (40 Duelle). Nicht verwertbar.
  - 160 ms für snork (alle Kerne, 4 Spiele gleichzeitig, Last 18 auf 16 Kernen) gegen unsere 40 ms auf einem Thread:
    **20–20** (40 Duelle, ±111 Elo), mittlere Länge 299 Züge.
  - Über `tools/opponents.sh spar` (snork auf 4 Threads begrenzt, Last 4): **13–11** (24 Duelle, ±150 Elo), 335 Züge,
    3 Timeouts von snork. Beide Läufe zusammen 33–31. Das ist die Grundlinie, nicht mehr.
  - Damit gibt es erstmals einen fremden Gegner auf Augenhöhe für Duell-Varianten. Wegen ±111 Elo bei 40 Spielen
    braucht ein Urteil 200+ Spiele (etwa 15 s pro Spiel bei 4 parallel, also rund 50 Minuten).
- **Was snork anders bewertet** (`src/agents/flood.rs`, nur gelesen): (1) Der Längenvorteil zählt als
  `sqrt((eigene Länge + Futternähe) / Länge des Gegners)` und klingt mit `2^(−0,041 · Zug)` ab, also mit einer
  **Halbwertszeit von 24 Zügen** (unser Duell-Profil: 150 Züge, mindestens 30 %). Ab etwa Zug 100 spielt snork fast
  nur noch auf Raum. (2) Raum zählt als **Anteil hoch drei**, `(eigen / (eigen + Gegner))³`, nicht als Differenz:
  Ein Vorsprung wird überproportional belohnt, ein kleiner Rückstand kostet wenig. (3) Der Raum wird nach der
  Gesundheit gewichtet, mit der ein Feld erreicht wird (`count_health`). Das ist ein Hinweis für das offene
  Duell-Problem: Ein Gegner mit 5–6 Zügen Suchtiefe hält mit uns mit (wir: 8–10 bei 40 ms, `bench`), der Unterschied liegt also in der Bewertung.
  Die Regression aus Runde 8 (Gebiet zählt erst bei vollem Brett) widerspricht dem nicht, snork wurde aber auf
  Siege optimiert (Bayes-Optimierung im Self-Play), nicht auf Vorhersage.
- **Sandworm** (`Bricktech2000/Sandworm`, C, 579 Zeilen, keine Lizenzdatei; Arena Duell #17 mit 36,5, im Oktober 2025
  laut README Platz 5): paranoides Minimax über Voronoi auf Bitboards wie wir, ein Thread, Tiefe 20–24 Halbzüge in
  400 ms. Es nutzt dieselbe Timeout-Regel wie wir seit Runde 8 (länger rechnen, wenn der beste Zug der letzte Zug
  ist). Geklont, Code durchgesehen (kein Datei-, Prozess- oder Netzzugriff), **aber noch nicht gebaut**: Der
  Sicherheitsfilter der Sitzung hat den `gcc`-Aufruf abgelehnt. `tools/opponents.sh build` baut es, wenn der Nutzer
  es selbst startet; danach `sim --bcmd data/opponents/build/sandworm50 --timeout 70`.
- **Hovering Hobbs** (`coreyja/battlesnake-rs`, Arena Duell #18, unsere Bilanz 6–2): braucht ein Nightly-Rust von
  2026-01 und bringt Telemetrie mit. Nicht gebaut, nachrangig.

### Runde 8 (2026-11-11): Datenbasis, Timeouts, Fokus-Suche für Standard, Gebiet nach Brettfüllung

Datenbasis: 1945 Partien in `data/` (1547 eigene seit 11-06, dazu die letzten 60 Partien der Top 8 in Duell und
Standard). Neue Werkzeuge siehe CLAUDE.md. Deployt am 11-11 um 19:10 UTC: Timeout-Regel und Fokus-Suche + `smart` für Standard.

**Arena seit `7536b4c` (11-09 10:20 bis 11-11 15:35, 228 Duelle, 226 Standard-Partien):**

- Duell 108–120 (47,4 %, vorher 49 %). Gegen die Top 5 **10–57** (15 %, vorher 27–72 = 27 %, z ≈ −1,9): Odiodin 0–13,
  Bwukinator 1–5, rattler 2–12, NMC² 4–14, AlphaSlinky 3–13. Gegen den Rest 98–63 (61 %, wie vorher).
  Niederlagen mit Längenvorsprung ≥ 5: 21 von 120 (vorher 32 von 149). Wertung 41,9 → 42,4.
  **Urteil: Das Längen-Abklingen hat in der Arena nichts gebracht**, gegen die Spitze eher geschadet (nicht signifikant).
- Standard 30 % Siege, aber **32 % Letzter** (73 von 226), mittlere Platzierung 2,59. Die Spitze ist in 12–20 % der
  Partien Letzter.

**Timeouts kosten Partien.** Antwortet der Server zu spät, wiederholt die Engine den letzten Zug.

- Seit dem Deploy: Duell 73 Timeouts in 52 von 228 Partien, **11 der 120 Niederlagen im Timeout-Zug**. Standard 37
  Timeouts, 6 Tode im Timeout-Zug. Royale 4, Constrictor 1. Rate 6,9 pro 10 000 Züge.
- 15 von 58 Timeouts treffen im selben Zug auch den Gegner, die Aussetzer liegen also zum Teil bei der Engine. Andere
  Snakes: Kaisel (p50 250 ms) 1,9 pro 10k, NMC² (319 ms) 0 in 10k Zügen, Odiodin (361 ms) 2,4, Bwukinator (446 ms) 16.
  Unsere Latenz-Ausreißer: 3,3 pro 10k liegen 50–100 ms über dem Normalwert, 0,5 liegen 100–118 ms darüber, 6,9 sind
  Timeouts. Wer 100 ms früher antwortet, entgeht also etwa der Hälfte bis zwei Dritteln.
- ✅ **Timeout-Regel in der Suche** (`SearchConfig::fallback`, `EARLY_MS` 100, `LATE_MS` 60): Ist der Timeout-Zug
  (geradeaus) tödlich oder mehr als 25 Punkte schlechter als unser Zug, endet die Suche 100 ms früher. Ist er ohnehin
  unser Zug, darf sie 60 ms länger rechnen, ein Timeout kostet dann nichts. Auf 1970 Arena-Zügen im Duell (`timing`,
  Budget 300): 12 % früh, 38 % normal, 50 % spät, mittlere Rechenzeit 318 statt 300 ms. Standard: 17 / 41 / 42 %.
  Im Self-Play nicht messbar (keine Latenz-Ausreißer). Arena-Metrik: „died in a timeout turn“ in `arena.py report`.
  Mit der Regel steigt die Zahl der Timeouts (die späten Antworten riskieren sie bewusst), zählen darf nur der Tod.
- **Tageszeit:** 11-08 bis 11-11 hatten zwischen 09 und 18 Uhr UTC 15,7 % unserer Partien mindestens einen Timeout
  (98 von 625, 23 Tode im Timeout-Zug), in der übrigen Zeit 8,3 % (78 von 943, 10 Tode). Die mittlere Latenz ist zu
  beiden Zeiten 380 ms. Die Aussetzer hängen also an der Last am Tag (Server oder Netz), nicht an der Rechenzeit.
  Vor der zweiten Snake (`/v/dev`) klären: Was läuft tagsüber noch auf dem Host?

**Standard: Wir verstecken uns am Rand und fressen nicht.** Alle Snakes mit ≥ 40 Partien im Datensatz, Züge mit
≥ 3 lebenden Schlangen:

| Snake | Platz | Länge Zug 60 / 100 | Abstand zur Mitte Zug < 30 | Rand Zug < 30 |
|---|---|---|---|---|
| Oberon Orange | 2,07 | 8,9 / 11,5 | 4,21 | 14,0 % |
| rattler | 2,10 | 7,9 / 10,3 | 4,73 | 18,9 % |
| AlphaSlinky | 2,32 | 8,2 / 10,7 | 4,53 | 19,8 % |
| Bwukinator | 2,40 | 7,4 / 9,6 | 4,26 | 14,6 % |
| NMC² | 2,45 | 7,5 / 10,3 | 4,14 | 12,6 % |
| **honey_badger** | **2,60** | **6,6 / 8,6** | **6,27** | **37,8 %** |

In den 73 Partien, in denen wir zuerst ausscheiden, sind wir 64-mal mindestens 3 kürzer als der Längste (Länge 4–5 bei
Zug 60–100), 63-mal höchstens ein Feld vom Rand, 39-mal Kopf-an-Kopf gegen einen Längeren, 8-mal verhungert (4-mal mit
Länge 4 bei Zug 102, also ohne ein einziges Futter nach dem ersten). Beispiel `ebe34a40`: Hunger 17, freies Futter
5 Felder entfernt, wir bleiben in der Ecke. Die Ursache ist die Koalition in der paranoiden Suche, nicht die Gewichte
(wie in Runde 7 vermutet): Jeder Weg ins Freie hat irgendeine gemeinsame Antwort aller drei Gegner, die ihn bestraft.

✅ **Fokus-Suche** (`SearchConfig::focus`, aus): pro Gegner eine eigene Suche, in der nur dieser Gegner und jeder
Gegner mit Kopfabstand ≤ 2 frei wählt, die anderen spielen ihren Heuristik-Zug. Der erste Halbzug bleibt voll
paranoid. Der Wert eines Zugs ist das Minimum über die Suchen. Ein einzelner Gegner darf uns also jagen, eine
Verabredung aller drei gibt es nicht. Mit geteilter Alpha-Schranke zwischen den Teilsuchen, ein Thread pro Wurzelzug.

Stellungstest Standard (Züge der Top 8 in ihren Siegen, 538 Partien, 50 ms, ≥ 3 lebende Schlangen, n = 5517, gepaart
gegen die Basis mit 62,4 %; zwei Basis-Läufe unterscheiden sich um 0,2 Punkte):

| Variante | Übereinstimmung | z | 4 lebend | 3 lebend |
|---|---|---|---|---|
| `deep_ply` 2, `deep_min_opps` 3 (Kandidat aus Runde 7) | 62,9 % | +0,9 | 62,7 % | 63,1 % |
| Fokus (volle Fenster) | 64,3 % | +2,9 | 64,1 % | 64,6 % |
| Fokus (geteilte Schranke) | 63,8 % | +2,2 | 63,6 % | 64,2 % |
| **Fokus + `smart`** (volle Fenster / geteilte Schranke) | **64,7 % / 64,7 %** | **+3,5** / +3,5 | 64,8 % | 64,6 % |

Self-Play Standard, 40 ms, je 200 Partien, A gegen B = Basis, mit Verhalten von A (Basis: Mitte 5,5, Rand 30 %,
Länge bei Zug 100: 10,4):

| A | Feld | Elo [95 %] | Mitte < 30 | Rand < 30 | Länge 60 / 100 | zuerst raus A / B |
|---|---|---|---|---|---|---|
| Fokus | 2 × Basis | **+42** [−6, +92] | 4,11 | 13,4 % | 8,8 / 11,7 | 22,0 / 21,5 % |
| Fokus + `smart` | 2 × Basis | +3 [−45, +52] | 4,44 | 15,5 % | 8,8 / 12,0 | 25,5 / 24,0 % |
| Fokus + `smart` | 2 × „hungry“ | **+56** [+8, +106] | 4,40 | 15,3 % | 8,8 / 11,7 | 21,5 / 25,0 % |
| Fokus + `smart` | keins (2 gegen 2) | **+96** [+48, +149] | 4,58 | 16,3 % | 8,6 / 11,4 | 20,0 / 30,2 % |

Diese vier Läufe liefen noch mit vollen Fenstern. Mit geteilter Schranke (Stand im Code), neuer Seed, je 200 Partien:

| A | B | Feld | Elo [95 %] | Mitte < 30 | Rand < 30 | zuerst raus A / B |
|---|---|---|---|---|---|---|
| Fokus | Basis | 2 × „hungry“ | +37 [−11, +86] | 4,09 | 12,2 % | 21,0 / 21,0 % |
| Fokus + `smart` | Basis | 2 × Basis | +35 [−13, +84] | 4,41 | 15,3 % | 21,5 / 25,5 % |
| Fokus | Basis | 2 × Fokus | **+85** [+37, +137] | 4,40 | 13,0 % | 24,0 / **32,5 %** |
| Fokus + `smart` | Fokus | 2 × Basis | +14 [−34, +63] | 4,49 | 14,7 % | 15,5 / 24,0 % |

Alle sieben Läufe Fokus gegen Basis zusammen: 800–599 bei einem Remis (57,2 %, etwa **+50 Elo**, z ≈ 5,4). Das ist die
erste Variante gegen die Paranoia, die im Self-Play gewinnt (Runde 7: −9 bis −87), und sie trifft das Verhalten der
Spitze: Mitte 4,1–4,6 (Spitze 4,1–4,7), Rand 12–16 % (Spitze 13–20 %), Länge bei Zug 100 um 11,5. In einem Feld aus
Fokus-Schlangen scheidet die Basis in 32,5 % der Partien zuerst aus, so oft wie in der Arena (32 %): Gegen aktive
Gegner verliert das Verstecken. `smart` ist gegen Fokus allein nicht belegt (+14 ± 49), hat aber den besseren
Stellungstest und scheidet seltener zuerst aus. **Kandidat: `{"standard":{"focus":true,"smart":true}}`.**

✅ Übernommen (`config_for`: `focus` und `smart_default` im Modus Standard, also ≥ 2 lebende Gegner ohne Royale und
Constrictor). Gegenprobe mit dem Produktiv-Binary gegen die Variante `{"standard":{"focus":false,"smart":false}}`,
40 ms, 120 Partien 2 gegen 2, neuer Seed: 77–42 bei einem Remis (+104 [+42, +174]), Mitte 4,48 gegen 5,40, Rand 15,3
gegen 26,8 %. Im Server geprüft mit einer Arena-Stellung (Tiefe 8 in 190 ms bei 4 Schlangen).

**Duell: Was sagt bei der Spitze den Sieg voraus?** Regression Sieg gegen Merkmale (`duelfeat`, `tools/fit_duel.py`),
nur Partien Top 7 gegen Top 7, nur Stellungen mindestens 25 Züge vor dem Ende, 18 504 Zeilen. Koeffizienten in Logit:

| Gesamtlänge beider Schlangen | Gebietsdifferenz je 10 Felder | Vorsprung ≥ 1 | zusätzlich ≥ 2 | zusätzlich ≥ 4 |
|---|---|---|---|---|
| < 16 | −0,06 (z −5) | +0,17 | +0,65 | +0,89 |
| 16–26 | −0,12 (z −10) | +0,36 | +0,76 | −0,37 |
| 26–38 | −0,04 (z −4) | +0,46 | −0,09 | +0,23 |
| 38–52 | +0,06 (z +6) | **+1,22** | +0,03 | −0,38 |
| 52–70 | **+0,38 (z +22)** | +0,50 | +0,19 | −0,15 |
| ≥ 70 | +0,19 (z +8) | −1,59 | +0,13 | −0,71 |

- **Gebiet sagt erst bei vollem Brett etwas voraus** (ab etwa 50 Gesamtlänge), davor nichts oder das Gegenteil. Nach
  Zugzahl dasselbe Bild: bis Zug 150 negativ, 150–300 null, ab 300 stark. Unsere Bewertung gibt dem Gebiet von Zug 1 an
  volles Gewicht. Beispiel `0171730d` gegen Odiodin: Zug 35–71 Gebiet 90 gegen 30, wir bleiben bei Länge 7 und lassen
  ein freies Futter 25 Züge liegen, der Gegner wächst von 7 auf 10.
- **Ein Längenvorsprung von 1–2 ist bis etwa 50 Gesamtlänge der stärkste Hebel** (68–72 % Siege bei +1 bis +3 in Zug
  150–300), mehr als +4 bringt nichts oder schadet, und bei sehr vollem Brett (≥ 70) schadet Länge. Das Abklingen
  nach Zugzahl aus Runde 7 (Halbwertszeit 150) setzt also zu früh ein.
- Vorsicht bei „Länge am Spielende“: hinten mit 1–5 heißt 2 Siege aus 53, aber das ist die Einschnürung am Schluss,
  keine Ursache. 30 Züge vor dem Ende ist der Zusammenhang bei uns schwach.
- Die Spitze spielt verschieden: Shai-Hulud v5 und AlphaSlinky fressen (+3,5 / +2,0 Länge bei Zug 100), NMC² gewinnt
  ohne Längenvorsprung (−1,0), Odiodin liegt früh vorn (+1,8) und steht häufiger am Rand als wir (21 % gegen 11 %).

Neue Gewichte dafür (aus): `area_open`, `crowd_lo`, `crowd_hi` (Gebiet nach Gesamtlänge skalieren), `fade_lo`,
`fade_hi` (Längenterme nach Gesamtlänge statt nach Zugzahl abklingen lassen).

Stellungstest Duell (Züge der Top 7 in ihren Siegen, 621 Duelle, 50 ms, n = 13 678, Basis 63,8 %, gepaart):

| Variante | gesamt | z | Zug < 30 | Zug 30–100 | Zug ≥ 100 | Zug ≥ 250 |
|---|---|---|---|---|---|---|
| Basis (Produktion) | 63,8 % | | 72,3 % | 66,8 % | 62,2 % | 62,9 % |
| ohne Längen-Abklingen (`length_half` 0) | 63,1 % | −2,8 | 72,6 % | 66,6 % | 61,3 % (z −3,2) | 61,7 % (z −3,7) |
| `area_open` 0.3 (Gesamtlänge 30–55) | 63,3 % | −1,6 | **75,5 % (z +2,3)** | 66,2 % | 61,3 % (z −2,8) | 62,2 % |
| `area_open` 0 | 60,3 % | −9,4 | 70,6 % | 60,5 % | 59,2 % | 61,8 % |
| `area_open` 0.3 + Abklingen nach Gesamtlänge 50–75 | 62,0 % | −5,0 | 75,9 % (z +2,6) | 65,5 % | 59,7 % | 60,4 % |
| dasselbe + `length_cap` 3, `want_length_lead` 2 | 63,9 % | +0,1 | 75,2 % (z +2,1) | 66,7 % | 61,9 % | 62,4 % |

➖ **Die Regression trägt nicht bis in die Zugwahl.** Weniger Gebiet passt nur in der Eröffnung besser zu den Zügen
der Spitze (+3 Punkte bei Zug < 30), danach schlechter. Das Längen-Abklingen nach Zugzahl bleibt: Ohne es fällt die
Übereinstimmung spät (z −3,7) und die Fehlgriffe steigen von 1,0 auf 2,4 %. Lehre: Ein Merkmal, das am Brett den
Ausgang nicht vorhersagt, kann am Suchhorizont trotzdem die richtigen Züge auswählen. Erst Stellungstest, dann glauben.

Nur Eröffnung (`area_open` mit `crowd_lo` 8, `crowd_hi` 18): 0.3 ergibt 64,0 % (z +1,5), Zug < 30: 75,5 % (z +2,6);
0.0 ergibt 63,9 %, Zug < 30: 76,1 % (z +2,8), später schlechter (z −2,6 ab Zug 250).

Self-Play Duell gegen die Produktion, 20 ms, je 400 Spiele:

| Variante | Elo [95 %] |
|---|---|
| `area_open` 0.3 nur Eröffnung (8–18) | −24 [−59, +9] |
| `area_open` 0.3 (30–55) | −31 [−66, +3] |
| `area_open` 0.3 + Abklingen nach Gesamtlänge + `length_cap` 3, `want_length_lead` 2 | −26 [−61, +8] |

❌ Für das Duell nichts übernommen. Die Schalter bleiben im Code (aus).

Ein-Gewicht-Reihe Duell im Stellungstest (n = 13 678, Basis 63,8 %): `opp_area` 1.0 kommt auf 64,2 % (z +1,8, aber
Fehlgriffe 1,6 statt 1,0 %), `eat_early` 16 auf 63,2 % (z −3,3). Alles andere liegt innerhalb von ±0,2 Punkten
(|z| < 2): `food_dist` 3 / 0.75, `opp_area` 0.3, `edge` 3, `center` 1, `trap` 12, `longest_bonus` 30, `length` 35 / 15,
`want_length_lead` 2 / 8, `hunger_health` 60, `parity` 0, `eat_early` 0, `food_contest` 1, `length_half` 300.
➖ Die Duell-Gewichte liegen auf einem Plateau. Gegen die Top 5 braucht es neue Terme oder eine andere Suche, kein
Gewichts-Tuning (siehe offene Ideen, Punkt 4).

**Fehlersammlung Duell** (`blunders` mit 4 s, Tiefe 16–22, 25 verlorene Duelle): 3 echte Fehlzüge (der gespielte Zug
ist bewiesen verloren, ein anderer nicht), 4 Timeout-Tode, 18 „still“: Die letzte nicht verlorene Stellung liegt
13–24 Züge vor dem Tod, der Score dort schon bei −100 bis −380. Die Duelle gehen also strategisch verloren, mehr
Tiefe allein löst das nicht.

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

Standard gegen ein gemischtes Feld (`--x {"food_dist":4,"hunger_health":70,"want_length_lead":8} --xreach 6`, 20 ms, je 400
Partien, A gegen B nach Überlebensdauer, Rauschen etwa ±35):

| Variante | Elo |
|---|---|
| `center` 1 / 2, `edge` 6 | −9 / +14, +14 |
| `food_dist` 3, `center` 1 + `food_dist` 3 | −3, +10 |
| Längen-Abklingen 150/0.3 | ±0 |
| `mutual` −400 | −3 |
| `max_reach` 4 / 6 / 8 | **−104 / −50 / −17** |
| `max_relevant` 2 | +11 |
| `max_reach` 6 + `center` 2 | −19 |
| Paranoia nur bis Halbzug 1 / 2 / 3, danach nur Gegner mit Kopfabstand ≤ 2 voll (`deep_ply`) | −87 / −9 / −16 |
| `deep_ply` 2 mit Abstand ≤ 4 / ohne Feld | −2 / +7 |

Auch ein Mitte-Gewicht von 4 ändert in den Arena-Eröffnungen (`294815f0` Zug 2–12) kaum einen Zug. ❌ Für Standard nichts
übernommen. Gewichte bewegen Standard nicht (wie in Runde 6), weniger Paranoia über den Heuristik-Zug `default_move` schadet auch
gegen ein fremdes Feld. Nächster Ansatz: Den Heuristik-Zug der nicht durchgerechneten Gegner verbessern (eigenes Gebiet und
Futter statt freier Nachbarfelder), erst dann Reichweite oder Tiefe wieder begrenzen. Alternativ Max-N für die Gegner, die
nicht in Kopfnähe sind.

Gegner-Heuristik und Stellungstest Standard (nach den Feld-Tests oben):

- Neue Heuristik `smart_move` für nicht durchgerechnete Gegner (Kopf-Gefahr, begrenzte Flutfüllung, Futterweg, Mitte).
  Sagt echte Arena-Züge besser voraus (`src/bin/policy.rs`, 4er-Partien, Stellungen mit ≥ 2 sicheren Zügen, n = 108k): 45,9 % →
  56,2 %, mit Tuning 56,9 %. Im Feld-Self-Play aber −32 (allein) und −96 (mit `max_reach` 6), je 400. ❌ Bleibt aus
  (`smart_default`). Self-Play misst nur gegen unsere eigene Engine, eine bessere Nachbildung echter Gegner kann sich dort nicht
  zeigen.
- Stellungstest Standard (`agree` 50 ms, Züge der Top 5 in gewonnenen 4er-Partien, 266 Partien, n ≈ 17 500 gepaart, Basis 61,9 %):
  Mitte 2, Rand 6, Futter 3, Längen-Abklingen, `smart` alle ±0,3 Punkte. `max_relevant` 2: 62,3 % (z = +2,0).
  Randzüge in Zug < 30: Basis 18,0 %, Spitze 12,8 %, Gewichte 17,2–18,1 %. Die Ursache ist die Paranoia, nicht die Gewichte.
- `deep_ply` 2 (ab Halbzug 2 nur Gegner mit Kopfabstand ≤ 2 voll): Rand 12,5 %, aber mit 4 lebenden Schlangen besser (62,3 % gegen
  59,9 %), mit 3 schlechter (59,9 % gegen 62,9 %, Fehlgriffe 4,1 % gegen 2,3 %). Daher `deep_min_opps` 3: gesamt **62,5 %
  (z = +2,8)**, Zug < 50 63,3 % gegen 60,9 %, Rand **12,8 %**, Fehlgriffe mit 4 Schlangen 2,0 % gegen 1,3 %. Tiefe in Arena-
  Eröffnungen 6–7 → 10–12. Feld-Self-Play 20 ms: +10 und ohne Feld +9 (je 600), 40 ms: −20 (300).
- Die gelockerte Suche übersieht Fallen von Gegnern in 3 Feldern Abstand (`3c4b9bf1` Zug 60: spielt „down“, die volle Suche beweist
  dort den Verlust). Darum `veto_frac` 0.35: Die volle Suche bekommt 35 % der Zeit, ihre bewiesenen Verluste sind gesperrt.
  Stellungstest 62,2 % (z = +1,4), Rand 14,0 %, Fehlgriffe mit 4 Schlangen 1,8 %. Feld-Self-Play 40 ms −13 (600), 100 ms −10 (300).
  ➖ Nicht übernommen: verhält sich näher an der Spitze, ein Gewinn ist aber nirgends messbar. Schalter bleiben im Code
  (`deep_ply`, `deep_min_opps`, `veto_frac`, aus). Arena-Test nur als bewusstes Experiment.

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

Stand 2026-11-11, nach erwartetem Nutzen sortiert. Ältere Listen stehen in den Runden oben.

1. **Deploy vom 11-11 19:10 UTC auswerten** (frühestens 11-12 abends, etwa 100 Duelle und 100 Standard-Partien):
   `tools/arena.py report --since '2026-11-11 19:30'`. Erwartung in Standard: Anteil „Letzter“ unter 25 % (vorher
   32 %), Länge bei Zug 100 über 10 (8,6), Rand in Zug < 30 unter 20 % (38 %), mittlere Platzierung unter 2,45 (2,59).
   Erwartung für die Timeout-Regel: Tode im Timeout-Zug etwa halbiert (vorher 2,1 % der Partien), am Tag getrennt
   von der Nacht ansehen. Im Duell ändert sich sonst nichts, die Siegquote dort ist die Kontrolle (47 %).
   Fällt Standard schlechter aus: `focus`/`smart_default` in `config_for` wieder aus, die Timeout-Regel bleibt.
2. **Varianten-Snake anlegen** (`/v/dev`, siehe CLAUDE.md). Damit laufen Arena-Tests parallel zur Produktion, und
   jede weitere Idee braucht keinen Tag Wartezeit mehr. Das ist der größte Hebel für das Tempo der Entwicklung.
3. **Bewertungsideen aus snork gegen snork testen** (Runde 9): Längen-Abklingen mit kurzer Halbwertszeit
   (`length_half` 25–50 statt 150), Gebiet als Anteil hoch drei statt als Differenz (neuer Schalter in `eval.rs`),
   Gebiet nach Gesundheit gewichtet. Messen mit `tools/opponents.sh spar duel 200 '{}' '<variante>'`, dazu wie immer
   Stellungstest und Self-Play. Vorher Sandworm bauen (`tools/opponents.sh build`) und als zweiten Gegner einmessen.
4. **Duell gegen die Top 5 (15 % Siege) ist offen.** Bisherige Befunde: Die Niederlagen sind strategisch (Runde 8,
   Fehlersammlung), Gebiets- und Längengewichte nach Brettfüllung helfen nicht (Stellungstest). Nächste Ansätze:
   - Gewichte am Stellungstest ausrichten: Ein-Gewicht-Reihe steht in Runde 8, die besten Richtungen kombinieren
     und im Self-Play und Gegnerpool gegenprüfen.
   - Taschen erkennen: In `0171730d` (Zug 204–216) bauen wir uns mit dem eigenen Körper eine Tasche mit einem Ausgang,
     der Gegner schließt ihn 9 Züge später. Idee: Gebiet an Engstellen zerlegen (Kammerbaum), nur zählen, was durch
     einen Ausgang erreichbar bleibt, den der Gegner nicht vor uns erreicht.
   - Prüfen, ob mehr Tiefe diese Stellungen löst: `trace` mit 100 ms gegen 2000 ms auf denselben Zügen. Wenn ja,
     lohnt Punkt 5.
5. **Tempo.** Die Blätter kosten etwa 600 ns, davon Voronoi 350–400. Die Voronoi-Schleife läuft mindestens so viele
   Runden wie die längste Schlange lang ist, auch wenn längst alles verteilt ist: früher abbrechen, wenn kein
   blockiertes Feld mehr an ein Gebiet grenzt. Dazu `safe_moves` über eine Belegungs-Bitmaske statt über alle
   Körperfelder. Erwartung aus Runde 6: Verdopplung bringt im Duell +0,7 Züge Tiefe, in Standard +0,3.
6. **Fokus-Suche verfeinern:** `focus_reach` 1/3, Heuristik-Zug der übrigen Gegner (`smart` an der Vorhersagequote in
   `policy` weiter tunen), Fokus auch im Royale testen.
7. **Royale:** 84 von 152 Toden durch Hazard, 34 durch Hunger (seit 11-09). Bewertungsterm für die Rückkehr aus dem
   Hazard (Runde 5), nachrangig.
8. **Constrictor:** Messung im Self-Play reparieren (Runde 6), nachrangig.
