# Experiment-Log: Weg auf Platz 1

Was probiert wurde, wie es ausging und was als Nächstes ansteht. **Vor jeder neuen Runde lesen,
nach jeder Runde ergänzen.** Neueste Einträge oben in jedem Abschnitt.

## Stand

| Datum      | Standard 11x11 | Duels 11x11 | Notiz |
|------------|----------------|-------------|-------|
| 2026-11-03 | 52.3 (47 % Siege, 93 Spiele) | 36.8 (79 % Siege, 92 Spiele) | vor Runde 1 |

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

## Beobachtungen aus Arena-Spielen

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
