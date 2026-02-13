# LKR Planner Backlog

## Ziel
Umsetzbarer, priorisierter Backlog für die gemeinsame Umsetzung mit einem Coding Agent.
Der Fokus liegt auf kleinen, testbaren Inkrementen (Red-Green-Refactor) und klaren Abnahmekriterien.

## Leitplanken
- TDD in jedem Task: zuerst failing Test, dann minimale Implementierung, dann Refactor.
- UI-Texte immer auf Deutsch.
- API-Calls über `@tauri-apps/plugin-http`.
- Neue Dependencies nur nach expliziter Entscheidung (Optionen mit Pros/Cons vorbereiten).
- OpenAPI-Dateien in `docs/` lokal nutzen, aber nicht committen.

## Aktueller Stand (aus Codebasis)
- Wochenansicht mit Dummy-Daten ist vorhanden (`src/app/*`, `src/data/dummy-data.ts`).
- Datum-Helfer sind teilweise getestet (`src/app/util.spec.ts`).
- Daylite- und Planradar-Integration ist noch nicht implementiert.
- Employee-Management ist noch nicht implementiert.

## Priorisierte Epics
1. Projekt-Hygiene und Architektur-Basis
2. Domänenmodell und lokale Speicherung
3. Daylite-Integration
4. Planradar-Integration
5. Mitarbeiterverwaltung
6. Planungslogik und Kalender-Sync
7. Stabilität, Observability, Release

## Backlog-Items

## EPIC 1: Projekt-Hygiene und Architektur-Basis

### BL-001: OpenAPI-Dateien vor Commit schützen ✅
**Status:** Abgeschlossen (2026-02-13)  
Priorität: P0  
Aufwand: S

Scope:
- `.gitignore` so erweitern, dass lokale OpenAPI-Artefakte in `docs/` nicht versehentlich committed werden (z. B. `docs/*openapi*.json`).

Abnahmekriterien:
- ✅ `git status` zeigt OpenAPI-Dateien nicht mehr als neue Dateien.
- ✅ `docs/BACKLOG.md` bleibt versioniert.

Tests (zuerst schreiben):
- Kein Code-Test nötig; Verifikation über Git-Status-Check im Workflow.

**Umsetzung:**
- Pattern `docs/*openapi*.json` zu `.gitignore` hinzugefügt
- Verifiziert: `daylite-openapi.json` und `planradar-openapi.json` werden ignoriert
- Verifiziert: `docs/BACKLOG.md` bleibt versioniert

### BL-002: Test-Workflow vereinheitlichen ✅
**Status:** Abgeschlossen (2026-02-13)  
Priorität: P0  
Aufwand: S

Scope:
- `package.json` um klare Test-Skripte ergänzen (`test`, optional `test:watch`).
- README um standardisierten lokalen Qualitäts-Flow ergänzen (`bun test`, `bun lint`, `bun format:check`).

Abnahmekriterien:
- ✅ `bun test` läuft als offizieller Standardbefehl.
- ✅ Workflow ist für Agent und Entwickler identisch dokumentiert.

Tests (zuerst schreiben):
- Mindestens ein vorhandener Testlauf muss im CI/Lokal mit `bun test` laufen.

**Umsetzung:**
- `test` und `test:watch` Skripte zu `package.json` hinzugefügt
- README um Development-Sektion erweitert mit vollständigem lokalen Qualitäts-Workflow
- Verifiziert: `bun test` und `bun run test` funktionieren identisch
- Verifiziert: Alle Quality-Checks (`test`, `lint`, `format:check`) sind dokumentiert

### BL-003: Integrationsarchitektur festziehen (Frontend <-> Tauri Commands)
Priorität: P0  
Aufwand: M

Scope:
- Klaren Schnitt definieren:
  - React-UI konsumiert nur Service-Funktionen.
  - Netzwerk und Secrets laufen in Tauri/Rust Commands.
- Ordnerstruktur für Integrationen anlegen (z. B. `src/services`, `src-tauri/src/integrations`).

Abnahmekriterien:
- Mindestens ein exemplarischer Flow nutzt bereits den definierten Schnitt.
- Dokumentierte Architektur-Notiz im Repo (`docs/`).

Tests (zuerst schreiben):
- Unit-Test für Service-Fassade im Frontend (Mock auf Command-Aufruf).

## EPIC 2: Domänenmodell und lokale Speicherung

### BL-004: Domänen-Typen für Planung v1 definieren
Priorität: P0  
Aufwand: M

Scope:
- Typen ergänzen für:
  - `Project` (Daylite-Referenz, Name, Status)
  - `Employee` (Skills, Standort, iCal-URL, Aktiv-Flag)
  - `Assignment` (Mitarbeiter, Projekt, Zeitraum, Quelle, Sync-Status)
  - `SyncIssue` (Quelle, Code, Nachricht, Zeitstempel)

Abnahmekriterien:
- Dummy-Daten auf neue Typen migriert.
- Keine `any`-basierten Workarounds.

Tests (zuerst schreiben):
- Type-/Unit-Tests für zentrale Mapper/Guards.

### BL-005: Lokalen Konfigurations-Store für Mitarbeiter aufbauen
Priorität: P1  
Aufwand: M

Scope:
- Persistenz für lokale App-Konfiguration (z. B. Tauri Store oder Datei-Backend) für:
  - API-Endpoints
  - Tokens/Referenzen
  - Mitarbeiter-spezifische Einstellungen

Abnahmekriterien:
- Neustart-sicheres Laden/Speichern.
- Fehlerfälle geben deutsche User-Meldung und technische Debug-Details.

Tests (zuerst schreiben):
- Unit-Tests für Load/Save + Fehlerfall (defekte Datei, fehlende Felder).

## EPIC 3: Daylite-Integration

### BL-006: Daylite API Client (Basis)
Priorität: P0  
Aufwand: M

Scope:
- Minimalen Client für benötigte Endpunkte bauen:
  - Projekte lesen/suchen
  - Kontakte lesen/suchen (für Mitarbeiter-Mapping)
- Einheitliches Fehlerobjekt inkl. HTTP-Status.

Abnahmekriterien:
- Client liefert typisierte Responses.
- Fehler werden zentral normalisiert.

Tests (zuerst schreiben):
- Unit-Tests mit gemockten HTTP-Responses (200/401/429/500).

### BL-007: Daylite Projekt-Synchronisierung (Read)
Priorität: P0  
Aufwand: M

Scope:
- Daylite-Projekte als Source of Truth laden.
- In internes `Project`-Modell mappen.

Abnahmekriterien:
- UI kann Projektliste aus Daylite laden (ohne Dummy-Projekte).
- Zeitstempel „Zuletzt synchronisiert“ sichtbar.

Tests (zuerst schreiben):
- Mapper-Tests für Datums-/Status-Felder.
- Service-Test für erfolgreichen Sync + API-Fehler.

### BL-008: Daylite Kontakte für Mitarbeiter-Konfiguration nutzen
Priorität: P1  
Aufwand: M

Scope:
- Kontakte aus Daylite laden und als mögliche Mitarbeiterquelle anzeigen.
- Zuordnung Kontakt <-> lokaler Mitarbeiter ermöglichen.

Abnahmekriterien:
- Benutzer kann Kontakt als Mitarbeiter übernehmen/zuordnen.
- Persistierte Zuordnung bleibt nach Neustart erhalten.

Tests (zuerst schreiben):
- Test für Kontakt-zu-Mitarbeiter-Mapping.
- Test für Persistenz der Zuordnung.

## EPIC 4: Planradar-Integration

### BL-009: Planradar API Client (Basis)
Priorität: P0  
Aufwand: M

Scope:
- Minimalclient für:
  - Projekte suchen/listen
  - Projekt anlegen (Template-basiert, falls nötig)
  - Projektstatus prüfen (aktiv/reopen)

Abnahmekriterien:
- Typisierte Responses und standardisierte Fehler.
- Konfigurierbare Tenant/Account-Parameter.

Tests (zuerst schreiben):
- Unit-Tests analog Daylite-Client inkl. Auth- und Rate-Limit-Fälle.

### BL-010: Daylite -> Planradar Projektabgleich
Priorität: P0  
Aufwand: L

Scope:
- Abgleichlogik:
  - Existiert entsprechendes Planradar-Projekt?
  - Falls nein: anlegen (Template-Regel)
  - Falls vorhanden aber geschlossen: reopen/aktivieren
- Idempotenz sicherstellen.

Abnahmekriterien:
- Mehrfacher Lauf erzeugt keine Duplikate.
- Jede Aktion wird als Sync-Ereignis protokolliert.

Tests (zuerst schreiben):
- Service-Tests für Fälle: neu, bereits vorhanden, geschlossen, API-Fehler.

### BL-011: Mapping-Regeln Daylite-Projekt -> Planradar-Template
Priorität: P1  
Aufwand: M

Scope:
- Konfigurierbare Regelmatrix (z. B. nach Projektkategorie/Typ).
- Fallback-Regel für nicht gemappte Projekte.

Abnahmekriterien:
- Regelwerk ist in UI editierbar (mindestens Basisform).
- Fehlendes Mapping erzeugt klaren SyncIssue statt Hard-Fail.

Tests (zuerst schreiben):
- Regel-Engine Tests (Treffer, Fallback, ungültige Regel).

## EPIC 5: Mitarbeiterverwaltung

### BL-012: Mitarbeiterliste mit CRUD
Priorität: P0  
Aufwand: M

Scope:
- Screen für Mitarbeiterverwaltung:
  - Anlegen
  - Bearbeiten
  - Deaktivieren
  - Löschen (mit Schutz bei aktiven Zuweisungen)

Abnahmekriterien:
- Vollständiger CRUD-Flow ohne Reload.
- Alle Texte und Fehlermeldungen auf Deutsch.

Tests (zuerst schreiben):
- Unit-Tests für Validierung (Pflichtfelder, iCal-URL-Format).
- UI-Tests für Create/Edit/Delete-Flows.

### BL-013: Skills, Verfügbarkeit und Standort modellieren
Priorität: P1  
Aufwand: M

Scope:
- Mitarbeiter um strukturierte Skills, Wochenverfügbarkeit und Home-Location erweitern.

Abnahmekriterien:
- Daten werden im Formular gepflegt und persistiert.
- Planungsansicht zeigt Verfügbarkeitskontext (z. B. Hinweis bei Abwesenheit).

Tests (zuerst schreiben):
- Tests für Verfügbarkeitsberechnung je Wochentag.

### BL-014: iCal-Quelle pro Mitarbeiter hinterlegen und validieren
Priorität: P0  
Aufwand: M

Scope:
- iCal-URL pro Mitarbeiter speichern.
- Grundvalidierung + Verbindungstest (manuell auslösbar).

Abnahmekriterien:
- Ungültige URLs werden sauber abgefangen.
- Verbindungstest liefert klare Erfolg-/Fehlermeldung.

Tests (zuerst schreiben):
- Parser-/Validierungstests.
- Fehlerfalltests für nicht erreichbare Kalenderquellen.

## EPIC 6: Planungslogik und Kalender-Sync

### BL-015: Planungstabelle von Dummy-Daten auf echte Datenquelle umstellen
Priorität: P0  
Aufwand: M

Scope:
- `dummy-data` entkoppeln, stattdessen Service-Layer anbinden.
- Lade-, Leer- und Fehlerzustände in der Wochenansicht ergänzen.

Abnahmekriterien:
- Wochenansicht funktioniert mit persistenten Daten.
- Fehlerzustände sind für Nutzer verständlich (Deutsch).

Tests (zuerst schreiben):
- UI-Tests für Loading/Empty/Error.

### BL-016: Zuweisungen erstellen/bearbeiten/löschen in Wochenansicht
Priorität: P0  
Aufwand: L

Scope:
- Klick auf Zelle öffnet Editor für Assignment:
  - Projekt auswählen
  - Zeitraum setzen
  - Konflikte anzeigen
- Änderungen persistent speichern.

Abnahmekriterien:
- End-to-end Flow für Assignment-CRUD vorhanden.
- Konflikte werden vor dem Speichern sichtbar gemacht.

Tests (zuerst schreiben):
- Service-Tests für Overlap-Erkennung.
- UI-Tests für Create/Edit/Delete.

### BL-017: iCal-Synchronisierung für Mitarbeiterzuweisungen
Priorität: P0  
Aufwand: L

Scope:
- Änderungen an Assignments in Mitarbeiter-iCal spiegeln.
- Idempotente Synchronisierung (keine doppelten Termine).

Abnahmekriterien:
- Neu/Update/Delete in Planung erzeugt korrekte iCal-Aktion.
- Sync-Status pro Assignment einsehbar.

Tests (zuerst schreiben):
- Sync-Service Tests inkl. Retry-Szenarien.

### BL-018: Wochenbasierte Planradar-Aktionen aus Planung auslösen
Priorität: P1  
Aufwand: M

Scope:
- Für aktuelle Woche zugewiesene Projekte in Planradar anlegen/reaktivieren.
- Auslösung manuell und optional automatisch beim Wochenwechsel.

Abnahmekriterien:
- Aktion ist nachvollziehbar geloggt.
- Fehlgeschlagene Einträge sind einzeln erneut ausführbar.

Tests (zuerst schreiben):
- Tests für Trigger-Logik (nur aktuelle Woche).

## EPIC 7: Stabilität, Observability, Release

### BL-019: Zentrales Fehler- und Sync-Issue-Panel
Priorität: P1  
Aufwand: M

Scope:
- UI-Bereich mit letzten Fehlern, Warnungen und Sync-Issues.
- Filterbar nach Quelle (Daylite, Planradar, iCal).

Abnahmekriterien:
- Nutzer kann Fehlerfälle nachvollziehen und gezielt neu anstoßen.

Tests (zuerst schreiben):
- Reducer/State-Tests für Event-Sammlung und Filter.

### BL-020: Hintergrund-Sync und manuelle Synchronisierung
Priorität: P1  
Aufwand: M

Scope:
- Manueller „Jetzt synchronisieren“-Button.
- Optionaler Intervall-Sync mit Sperre gegen parallele Läufe.

Abnahmekriterien:
- Keine konkurrierenden Sync-Läufe.
- Sichtbares Feedback über laufenden Sync.

Tests (zuerst schreiben):
- Tests für Run-Lock und erneute Ausführung nach Fehler.

### BL-021: Release-Härtung für macOS
Priorität: P2  
Aufwand: M

Scope:
- Build-Checkliste (`bun build:macos`, Smoke-Test, Signierung/Notarisierung als separater Prozess, falls benötigt).
- Basis-Telemetrie/Logging für Supportfälle (lokal).

Abnahmekriterien:
- Reproduzierbarer Release-Ablauf dokumentiert.
- Kritische Fehler sind aus Logs rekonstruierbar.

Tests (zuerst schreiben):
- Smoke-Test-Checklist als ausführbarer Ablauf (manuell + Skript wo möglich).

## Empfohlene Umsetzungsreihenfolge (erste 3 Sprints)

### Sprint 1 (Fundament)
- BL-001
- BL-002
- BL-003
- BL-004
- BL-006
- BL-009

### Sprint 2 (erste echte End-to-End Synchronisierung)
- BL-007
- BL-010
- BL-012
- BL-015
- BL-016

### Sprint 3 (Mitarbeiter + Kalender + Stabilität)
- BL-013
- BL-014
- BL-017
- BL-018
- BL-019
- BL-020

## Offene Produktfragen
1. Was ist die fachliche Regel, um ein Daylite-Projekt eindeutig einem Planradar-Projekt zuzuordnen (ID-Feld, externer Schlüssel, Namensregel)?
2. Sollen Mitarbeiter primär aus Daylite-Kontakten kommen oder auch vollständig lokal gepflegt werden dürfen?
3. Wie soll Konfliktlogik priorisiert werden: harte Blockade beim Überbuchen oder nur Warnung?
4. Soll Planradar-Sync automatisch im Hintergrund laufen oder nur manuell ausgelöst werden (v1)?
5. Gibt es Anforderungen an Offline-Fähigkeit oder reicht „online-first“ für v1?

