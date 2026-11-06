<p align="center">
  <img src="images/EH3D_BANNER.svg" alt="EH3D-SUITE-LAUNCHER banner" width="100%">
</p>

# 📖 EH3D-SUITE-LAUNCHER

<p align="center"><a href="README.md">🇺🇸 English</a> | <a href="README_spa.md">🇪🇸 Español</a> | <a href="README_fra.md">🇫🇷 Français</a> | <a href="README_ita.md">🇮🇹 Italiano</a> | 🇩🇪 <b>Deutsch</b> | <a href="README_zho.md">🇨🇳 中文</a> | <a href="README_jpn.md">🇯🇵 日本語</a> | <a href="README_por.md">🇵🇹 Português</a></p>

### Die Zentrale der EH3D Fabrication Suite: findet die Programme, startet sie und führt die Projektdatei

<p align="center">
  <img src="https://img.shields.io/badge/License-GPL%203.0--or--later-blue.svg" alt="GPL 3.0 or later">
  <img src="https://img.shields.io/badge/Stack-Rust-DEA584.svg" alt="Rust">
  <img src="https://img.shields.io/badge/Languages-8-00E5FF.svg" alt="Languages">
  <img src="https://img.shields.io/badge/Version-0.0.5-0f6cbd.svg" alt="Version">
  <img src="https://img.shields.io/badge/Maturity-scaffolding-FFB020.svg" alt="Maturity">
</p>

---

**Ehrlichkeitscheck - was heute wirklich läuft:** eine Rust-Bibliothek und ein Kommandozeilenprogramm (`eh3d-launcher`) mit 113 Tests, die auf einem Computer laufen: findet die Programme der Suite über ihre Manifeste, startet sie und startet sie bei Fehlern neu, führt die zuletzt verwendeten Projekte und schreibt und prüft die Projektdatei mit dem SHA-256 jeder Datei. Ein Fenster (`eh3d-launcher-gui`, mit dem Feature `gui` gebaut) ist geschrieben und seine Logik getestet, aber es wurde weder gebaut noch auf einem Bildschirm angesehen; nichts wurde an einer Maschine ausprobiert und nichts an einer Sammlung echter Dateien geprüft.

---

> **Sicherheitsstatus:** Dieses Programm steuert keine Maschine und ist kein Sicherheitssystem. Was es erzeugt, muss ein Mensch prüfen, bevor etwas an einer Maschine läuft; nichts davon lief auf echter Hardware.


## 📋 Auf einen Blick

| | |
|---|---|
| Version | `0.0.5` |
| Reifegrad | Gerüst |
| Tests | 113 |
| Technologie-Stack | Rust |
| Auf echter Hardware | Nicht erprobt: nichts davon lief an einer Maschine |

**Springe zu:** [Überblick](#-überblick) · [Architektur](#-architektur) · [Schnellstart](#-schnellstart) · [Tests](#-tests) · [Struktur](#-struktur-des-repositorys) · [Verwandte](#-verwandte-projekte)

## 🚦 Projektstatus

Jede Zeile ist ein Teil dieses Repositorys, und die Ampel sagt, wie weit er heute ist. Sie beruht auf dem, was das Repository zeigen kann (Tests, Builds, Läufe), nicht auf dem, was geplant ist.

🟢 Fertig und überprüft · 🟡 Funktioniert, mit bekannten Grenzen · 🟠 Teilweise fertig oder noch nicht überprüft · 🔴 Nicht gemacht oder funktioniert nicht

| Bereich | Stand | Was das hier bedeutet |
|---|---|---|
| Kern | 🟡 | findet die Programme der Suite über ihre Manifeste, startet sie und startet sie bei Fehlern neu, führt die zuletzt verwendeten Projekte und schreibt und prüft die Projektdatei mit dem SHA-256 jeder Datei; am Computer mit normalen und beschädigten Eingaben getestet, nicht mit einer Sammlung echter Dateien. |
| Kommandozeile | 🟡 | `eh3d-launcher` führt dieselben Funktionen wie die Bibliothek aus und gibt das Ergebnis aus; jeder Fehler kommt als Meldung mit einem Exit-Code ungleich null zurück. |
| Tests | 🟡 | 113 Tests bestehen (`cargo test`), mit Normalfällen, Grenzfällen und beschädigten Eingaben. |
| Sicherheit | 🟡 | Reines sicheres Rust (`forbid(unsafe_code)`): nichts gerät bei den übergebenen Dateien in Panik und ein schlechter Wert ist ein Fehler mit Begründung; keine unabhängige Prüfung. |
| Fenster | 🟠 | Geschrieben (egui, `--features gui`, Programm `eh3d-launcher-gui`): die letzten Projekte, Projekt öffnen oder anlegen, seine Dateien mit den Prüfungen der Kommandozeile hinzufügen und entfernen, und die Programme der Suite mit einer Schaltfläche zum Starten jedes Programms; seine Logik ist getestet, aber es wurde noch nicht gebaut oder auf einem Bildschirm angesehen. |
| Echte Hardware | 🔴 | Nicht erprobt: nichts davon wurde an einer Maschine ausgeführt. |
| Dokumentation | 🟡 | README in acht Sprachen, das Changelog und die zwei aus `docs/` gezeichneten Bilder. |

## 🎯 Überblick

* **Das Register** (`src/registry.rs`): Jedes Repository der Suite sagt in seiner `eh3d.project.json`, was es ist; der Launcher liest sie, überspringt beschädigte mit Begründung und meldet, welche Programme gebaut sind.
* **Der Starter** (`src/launch.rs`): startet ein Programm der Suite und startet es bei einem Fehler bis zu zweimal neu, mit sich verdoppelnder Wartezeit; ein Programm, das gut endet, wird in Ruhe gelassen.
* **Die Projektdatei** (`src/project.rs`, `.eh3dlist`): Name, die Dateien des Auftrags mit Art und SHA-256 und der Arbeitsnullpunkt; beim Lesen werden Pfade, die das Projekt verlassen, abgelehnt und das erste Format migriert. **Zuletzt verwendet** (`src/recent.rs`) merkt sich die letzten zehn.
* **Finden und Neustart:** Die Programmdatei wird über den `[[bin]]`-Namen der `Cargo.toml` gefunden (`eh3d-camdebug`, `eh3d-pnp` ...); ein Programm, das der Bediener absichtlich beendet hat (Signal) oder das seine Argumente abgelehnt hat (Exit-Code 1 oder 2), wird nicht neu gestartet: nur ein Absturz.
* **Das Fenster** (`src/window.rs`, `src/gui.rs`; `--features gui`, Programm `eh3d-launcher-gui`): die letzten Projekte, Projekt öffnen oder anlegen, seine Dateien mit denselben Prüfungen wie `add-layer` hinzufügen und entfernen, und die Programme der Suite mit einer Schaltfläche, die jedes mit dem Projekt startet. Zustand, Aktionen, Einstellungen und Wörter sind einfacher Code mit Tests; das Zeichnen ist dünn. Geschrieben, noch nicht auf einem Bildschirm gesehen.

## 🧩 Rolle in EH3D

EH3D-SUITE-LAUNCHER ist ein Programm der EH3D Fabrication Suite. Es arbeitet als Bibliothek, als Kommandozeile und, mit dem Feature `gui` gebaut, als Fenster (`eh3d-launcher-gui`) im gemeinsamen Aussehen der Suite. Es liest und schreibt einfache Dateien und lässt sich allein oder zwischen den anderen Programmen nutzen.

## 🏛️ Architektur

<p align="center">
  <img src="images/architecture.svg" alt="Architektur von EH3D-SUITE-LAUNCHER: wer es nutzt, was es enthält und wovon es abhängt" width="100%">
</p>


Wie die Teile dieses Repositorys zusammenpassen: wer es nutzt, was es enthält und wovon es abhängt. Das Bild wird aus `docs/architecture.json` gezeichnet und gegen den Code geprüft.

## ✨ Funktionen

* **Bibliothek und Kommandozeile:** Jeder Befehl von `eh3d-launcher` ist eine Funktion der Bibliothek, sodass ein anderes Programm der Suite ihn ohne Prozessstart aufrufen kann.
* **Tests mit beschädigten Eingaben:** Die Tests füttern auch leere Dateien, Datenmüll, riesige Zahlen und abgeschnittene Dateien; keiner darf in Panik geraten.
* **Kein Raten:** Ein unsinniger Wert ist ein Fehler, der sagt welcher und warum, nie ein Ergebnis, das nur richtig aussieht.
* **Kein unsafe-Code:** `#![forbid(unsafe_code)]` in Bibliothek und Programm.
* **Sicherere Listen und Projekte:** ein Ebenenpfad darf nicht `.` sein, kein von Windows abgelehnter Name (`CON`, `AUX`, `COM1` ...) und nicht auf Punkt oder Leerzeichen enden, zwei Ebenen, die sich nur in der Groß-/Kleinschreibung unterscheiden, sind eine, die Liste der zuletzt verwendeten Projekte behält den vollständigen Pfad einer vorhandenen Datei, und zwei gleichzeitige Läufe (`recent`, `add-layer`) verlieren die Änderung des anderen nicht, weil die Datei während der Änderung gesperrt ist; eine Ebene wird in Blöcken zu 64 KiB gehasht, und ein durch ein Signal beendetes Programm wird als solches gemeldet und nicht als Fehler.

## 🔒 Sicherheit und Schutz

| Bedrohung | Schutz |
|---|---|
| Eine beschädigte, abgeschnittene oder leere Datei | keine Panik: die Tests schneiden gültige Eingaben an jeder Stelle ab und füttern Datenmüll |
| Eine unsinnige Zahl (negativ, nicht endlich, absurd groß) | wird mit einem Fehler abgelehnt, der den Wert nennt; nichts wird zu etwas Plausiblem gerundet |
| Unsichere Speichernutzung | reines sicheres Rust mit `forbid(unsafe_code)` |

**Was nicht abgesichert ist:** Ein fehlerfrei erzeugtes Ergebnis ist kein geprüftes Ergebnis. Nichts hier kennt die Grenzen einer Maschine, und das Programm hatte keine unabhängige Prüfung. Eine Schwachstelle bitte privat melden (siehe [SECURITY.md](SECURITY.md)).

## 🔌 Schnittstellen

Die Bibliothek ist `eh3d_suite_launcher` (`use eh3d_suite_launcher::...`); die Kommandozeile ist `eh3d-launcher`. Die Befehle sind diese; jeder gibt sein Ergebnis auf der Standardausgabe aus und meldet einen Fehler auf der Fehlerausgabe mit einem Exit-Code ungleich null. Es gibt keinen Netzwerkzugriff.

```text
EH3D-SUITE-LAUNCHER/
├── src/
│   ├── browse.rs                                die Dateiliste des Fensters zum Öffnen einer Datei
│   ├── gui.rs                                   das Fenster (egui, mit dem Feature gui): zeichnet den Zustand und gibt weiter, was die Person tut
│   ├── gui_main.rs                              das Fenster als eigenes Programm, unter Windows ohne Konsole
│   ├── lang.rs                                  die Wörter des Fensters in den acht Sprachen
│   ├── launch.rs                                ein Teil der Logik
│   ├── lib.rs                                   die Bibliothek: was jeder Befehl nutzt
│   ├── main.rs                                  die Kommandozeile
│   ├── project.rs                               ein Teil der Logik
│   ├── recent.rs                                ein Teil der Logik
│   ├── registry.rs                              ein Teil der Logik
│   ├── settings.rs                              die Einstellungen aus launcher.cfg: Sprache, Neustarts und Fenstergröße
│   ├── shot.rs                                  der Bildmodus des Fensters (--shot): seine Argumente, ohne Bildschirm gelesen und geprüft
│   └── window.rs                                der Zustand des Fensters ohne Bildschirm: Programme, Projekte, Dateien und Start
├── tests/
│   ├── launcher.rs                              Tests von Bibliothek und Kommandozeile
│   └── shot_sample.rs                           Tests der Beispiele im Fenster eines Bildes
├── examples/
│   ├── workspace/                               Beispiel-Manifeste von Programmen der Suite, für das Fenster eines Bildes
│   └── demo/                                    eine Beispiel-Projektliste mit drei Dateien und ihren Prüfsummen
├── docs/
│   ├── architecture.json                        Daten des Architektur-Bildes
│   └── structure.json                           Daten des Struktur-Bildes
├── tools/                                       CI-Prüfung, README-Prüfung und die Bildwerkzeuge
├── images/                                      Banner und die Bilder von Architektur und Repository-Struktur
├── .github/                                     CI- und Release-Workflows, Vorlagen für Issues und Pull Requests
├── eh3d.project.json                            Manifest: Version, Rolle, Reifegrad und Build-Befehl
├── Cargo.toml Cargo.lock                        der Workspace und seine festgelegten Abhängigkeiten
├── rust-toolchain.toml                          die festgelegte Rust-Toolchain
├── CHANGELOG.md                                 was sich in jeder Version geändert hat
├── SECURITY.md SUPPORT.md                       Schwachstellen melden und Fragen stellen
├── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   Mitwirken, Verhaltenskodex und Lizenz
└── launcher.cfg.example                         alle Einstellungen von launcher.cfg mit ihrem Standardwert
```

## ⚙️ Konfiguration

Die Konfigurationsdatei ist `launcher.cfg` im Ordner des Launchers, von `launch` und vom Fenster gelesen: Ihre Zeile `workspace = VOLLSTÄNDIGER_PFAD` ist der Ordner, aus dem Programme gestartet werden (nur bekannte Programme der Suite), und die optionalen Zeilen `language`, `max_restarts`, `backoff_ms`, `window_width` und `window_height` legen die Sprache des Fensters fest, wie oft ein abgestürztes Programm neu gestartet wird, die Wartezeit und die Größe des Fensters. `launcher.cfg.example` listet jeden Schlüssel mit Standardwert und Grenzen auf. Davon abgesehen: Was die Person wählt, wird als Argument der Kommandozeile angegeben (siehe Schnittstellen), und jeder Standardwert steht in der Hilfe des Befehls (`eh3d-launcher help`). Die Grenzen eines Werts sind benannte Konstanten im Code und durch Tests abgedeckt.

## 🚀 Schnellstart

Sie brauchen eine stabile Rust-Toolchain (1.95 oder neuer).

```bash
cargo build --release
cargo test
cargo run --release -- list --workspace ..
# das Fenster (braucht das Feature gui)
cargo build --release --features gui
cargo run --release --features gui --bin eh3d-launcher-gui
cargo run --release --features gui --bin eh3d-launcher-gui -- --shot screen.png --open examples/workspace --open examples/demo/demo.eh3dlist   # a picture of the window
```

Um ein Bild des Fensters aufzunehmen, ohne es von Hand zu öffnen, starten Sie es mit `--shot DATEI.png` (auch `--size WxH`, `--theme forge|dark|light`, `--wait N`, `--lang CODE`, `--open FILE`, `--replace`): Es öffnet sich in dieser Größe, speichert das PNG, behält keine Einstellungen und schließt sich selbst (Exit-Code 0, wenn die Datei geschrieben wurde, 1, wenn nicht, 2 bei falschen Argumenten).

## 🧪 Tests

Die Suite hat 113 Tests (`cargo test`, in der CI unter Windows und Linux). Sie prüfen Normalfälle, Grenzfälle und beschädigte Eingaben: leere Dateien, Datenmüll und abgeschnittene Dateien lösen nie eine Panik aus. Die CI führt außerdem `cargo fmt --check`, `cargo clippy` (auch mit `--features gui`) mit Warnungen als Fehlern und `python -I tools/ci_validate.py` aus.

## 🏗️ Build und Veröffentlichung

Die Version steht in `Cargo.toml` und in `eh3d.project.json`; jede Version hat ihren Eintrag in [CHANGELOG.md](CHANGELOG.md). `build.bat` / `build.sh` bauen das Programm, `test.bat` / `test.sh` führen die Tests aus und `run.bat` / `run.sh` starten es mit einem Beispiel. Releases erscheinen auf GitHub.

### Downloads

Jedes Release enthält das Kommandozeilenprogramm und das Fenster für Windows (64 und 32 Bit), Linux (64 Bit) und macOS (Apple Silicon und Intel) als `<Name>-<Version>-<System>.zip` (Windows) oder `.tar.gz` mit den beiden Programmen, der Lizenz, der Readme, dem Symbol und `launcher.cfg.example`. Die Dateien werden von GitHub Actions gebaut und stehen in `SHA256SUMS`. Um einen Download zu prüfen, vergleichen Sie seinen Hash mit der danebenliegenden `.sha256`-Datei (`sha256sum -c`).

## 📂 Struktur des Repositorys

<p align="center">
  <img src="images/repository-structure.svg" alt="Struktur des Repositorys EH3D-SUITE-LAUNCHER" width="100%">
</p>

<details>
<summary>Textversion des Baums</summary>

```text
EH3D-SUITE-LAUNCHER/
├── src/
│   ├── browse.rs                                die Dateiliste des Fensters zum Öffnen einer Datei
│   ├── gui.rs                                   das Fenster (egui, mit dem Feature gui): zeichnet den Zustand und gibt weiter, was die Person tut
│   ├── gui_main.rs                              das Fenster als eigenes Programm, unter Windows ohne Konsole
│   ├── lang.rs                                  die Wörter des Fensters in den acht Sprachen
│   ├── launch.rs                                ein Teil der Logik
│   ├── lib.rs                                   die Bibliothek: was jeder Befehl nutzt
│   ├── main.rs                                  die Kommandozeile
│   ├── project.rs                               ein Teil der Logik
│   ├── recent.rs                                ein Teil der Logik
│   ├── registry.rs                              ein Teil der Logik
│   ├── settings.rs                              die Einstellungen aus launcher.cfg: Sprache, Neustarts und Fenstergröße
│   ├── shot.rs                                  der Bildmodus des Fensters (--shot): seine Argumente, ohne Bildschirm gelesen und geprüft
│   └── window.rs                                der Zustand des Fensters ohne Bildschirm: Programme, Projekte, Dateien und Start
├── tests/
│   ├── launcher.rs                              Tests von Bibliothek und Kommandozeile
│   └── shot_sample.rs                           Tests der Beispiele im Fenster eines Bildes
├── examples/
│   ├── workspace/                               Beispiel-Manifeste von Programmen der Suite, für das Fenster eines Bildes
│   └── demo/                                    eine Beispiel-Projektliste mit drei Dateien und ihren Prüfsummen
├── docs/
│   ├── architecture.json                        Daten des Architektur-Bildes
│   └── structure.json                           Daten des Struktur-Bildes
├── tools/                                       CI-Prüfung, README-Prüfung und die Bildwerkzeuge
├── images/                                      Banner und die Bilder von Architektur und Repository-Struktur
├── .github/                                     CI- und Release-Workflows, Vorlagen für Issues und Pull Requests
├── eh3d.project.json                            Manifest: Version, Rolle, Reifegrad und Build-Befehl
├── Cargo.toml Cargo.lock                        der Workspace und seine festgelegten Abhängigkeiten
├── rust-toolchain.toml                          die festgelegte Rust-Toolchain
├── CHANGELOG.md                                 was sich in jeder Version geändert hat
├── SECURITY.md SUPPORT.md                       Schwachstellen melden und Fragen stellen
├── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   Mitwirken, Verhaltenskodex und Lizenz
└── launcher.cfg.example                         alle Einstellungen von launcher.cfg mit ihrem Standardwert
```

</details>

## 🗺️ Roadmap und bekannte Grenzen

**Bekannte Grenzen heute**

* Das Fenster ist geschrieben und seine Logik getestet, wurde aber noch nicht gebaut oder auf einem Bildschirm angesehen.
* Es wurde weder an einer Maschine noch mit einer Sammlung echter Dateien erprobt und hatte keine unabhängige Prüfung.
* Es findet noch keine Maschinen im Netz, hält die Suite nicht aktuell (das macht EH3D-UPDATER) und die `.eh3dlist` ist eine JSON-Datei, die auf die Dateien zeigt, kein Archiv, das sie enthält.

**Als Nächstes**

* Die Maschinen der Werkstatt im Netz finden und EH3D-UPDATER aus dem Fenster aufrufen.
* Das Fenster auf einem Bildschirm jedes Systems ansehen und das Gefundene anpassen.
* Mit echten Dateien und den anderen Programmen der Suite ausprobieren.

## 🌍 Sprachen

README und Texte gibt es auf Englisch, Spanisch, Französisch, Italienisch, Deutsch, Chinesisch, Japanisch und europäischem Portugiesisch.

## 🔗 Verwandte Projekte

* [EH3D-COMMON](https://github.com/JuanenRac/EH3D-COMMON): der Projektcontainer und die gemeinsamen Typen der Suite
* [EH3D-UPDATER](https://github.com/JuanenRac/EH3D-UPDATER): der Updater der Suite
* [EH3D-DRILLPRO](https://github.com/JuanenRac/EH3D-DRILLPRO): das Bohrprogramm
* [EH3D-ROUTEPRO](https://github.com/JuanenRac/EH3D-ROUTEPRO): das Konturfräsprogramm
* [EH3D-BUNGARD-GUI](https://github.com/JuanenRac/EH3D-BUNGARD-GUI): das Fenster der Bungard-Maschinen
* [EH3D-PHOENIX-CNC](https://github.com/JuanenRac/EH3D-PHOENIX-CNC): das Maschinensteuerungsprogramm

## 📚 Dokumentation

* [Changelog](CHANGELOG.md)
* [Architektur- und Strukturdaten](docs/architecture.json): die Daten, aus denen die beiden Bilder gezeichnet werden.

## 🤝 Mitwirken und Community

* Vor einem Pull Request `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` und `python -I tools/ci_validate.py` ausführen: alles muss bestehen. Siehe [CONTRIBUTING.md](CONTRIBUTING.md) und den [Verhaltenskodex](CODE_OF_CONDUCT.md).
* Die README gibt es in acht Sprachen: Eine Textänderung erfolgt in allen acht Dateien, und die zwei Bilder werden aus `docs/architecture.json` und `docs/structure.json` mit `tools/make_architecture.py` und `tools/make_structure.py` gezeichnet.
* Eine Schwachstelle privat melden: [SECURITY.md](SECURITY.md). Fragen, Ideen und Berichte: electrohobby3d@gmail.com ([SUPPORT.md](SUPPORT.md)).

## 👤 AUTOR

**JuanenRac (Electro Hobby 3D)** · electrohobby3d@gmail.com

## 📜 LIZENZ

Software: GPL-3.0-or-later. Siehe [LICENSE](LICENSE). Die Hardware-Entwürfe der Suite verwenden die CERN OHL-S.
