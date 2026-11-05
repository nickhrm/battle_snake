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
