<p align="center">
  <img src="images/EH3D_BANNER.svg" alt="EH3D-SUITE-LAUNCHER banner" width="100%">
</p>

# 📖 EH3D-SUITE-LAUNCHER

<p align="center"><a href="README.md">🇺🇸 English</a> | <a href="README_spa.md">🇪🇸 Español</a> | <a href="README_fra.md">🇫🇷 Français</a> | 🇮🇹 <b>Italiano</b> | <a href="README_deu.md">🇩🇪 Deutsch</a> | <a href="README_zho.md">🇨🇳 中文</a> | <a href="README_jpn.md">🇯🇵 日本語</a> | <a href="README_por.md">🇵🇹 Português</a></p>

### Il centro della EH3D Fabrication Suite: trova i programmi, li avvia e conserva il file di progetto

<p align="center">
  <img src="https://img.shields.io/badge/License-GPL%203.0--or--later-blue.svg" alt="GPL 3.0 or later">
  <img src="https://img.shields.io/badge/Stack-Rust-DEA584.svg" alt="Rust">
  <img src="https://img.shields.io/badge/Languages-8-00E5FF.svg" alt="Languages">
  <img src="https://img.shields.io/badge/Version-0.0.5-0f6cbd.svg" alt="Version">
  <img src="https://img.shields.io/badge/Maturity-scaffolding-FFB020.svg" alt="Maturity">
</p>

---

**Verifica di onestà - cosa funziona oggi:** una libreria Rust e un programma a riga di comando (`eh3d-launcher`) con 100 test eseguiti su un computer: trova i programmi della suite dai loro manifesti, li avvia e li riavvia se falliscono, conserva i progetti recenti e scrive e verifica il file di progetto con lo SHA-256 di ogni file. Una finestra (`eh3d-launcher-gui`, compilata con la funzione `gui`) è scritta e la sua logica è testata, ma non è stata compilata né guardata su uno schermo; nulla è stato provato su una macchina e nulla è stato verificato su una raccolta di file reali.

---

> **Stato di sicurezza:** questo programma non controlla alcuna macchina e non è un sistema di sicurezza. Ciò che produce deve essere controllato da una persona prima di eseguire qualsiasi cosa su una macchina; nulla di questo ha girato su hardware reale.


## 📋 In breve

| | |
|---|---|
| Versione | `0.0.5` |
| Maturità | Impalcatura |
| Test | 100 |
| Stack tecnologico | Rust |
| Su hardware reale | Non provato: nulla di questo ha girato su una macchina |

**Vai a:** [Panoramica](#-panoramica) · [Architettura](#-architettura) · [Avvio rapido](#-avvio-rapido) · [Test](#-test) · [Struttura](#-struttura-del-repository) · [Correlati](#-progetti-correlati)

## 🚦 Stato del progetto

Ogni riga è una parte di questo repository e il semaforo dice a che punto è oggi. È scritta in base a ciò che il repository può mostrare (test, compilazioni, esecuzioni), non a ciò che è previsto.

🟢 Fatto e verificato · 🟡 Funziona, con limiti noti · 🟠 Fatto a metà o non ancora verificato · 🔴 Non fatto o non funzionante

| Area | Stato | Cosa significa qui |
|---|---|---|
| Nucleo | 🟡 | trova i programmi della suite dai loro manifesti, li avvia e li riavvia se falliscono, conserva i progetti recenti e scrive e verifica il file di progetto con lo SHA-256 di ogni file; provato su un computer con input normali e danneggiati, non con una raccolta di file reali. |
| Riga di comando | 🟡 | `eh3d-launcher` esegue le stesse funzioni della libreria e stampa il risultato; ogni errore torna come messaggio e codice di uscita diverso da zero. |
| Test | 🟡 | 100 test passano (`cargo test`), con casi normali, casi limite e input danneggiati. |
| Sicurezza | 🟡 | Rust sicuro e semplice (`forbid(unsafe_code)`): nulla va in panico con i file ricevuti e un valore errato è un errore con il suo motivo; nessuna revisione indipendente. |
| Finestra | 🟠 | Scritta (egui, `--features gui`, programma `eh3d-launcher-gui`): i progetti recenti, aprire o creare un progetto, aggiungere e rimuovere i suoi file con i controlli della riga di comando e i programmi della suite con un pulsante per avviare ciascuno; la sua logica è testata, ma non è ancora stata compilata né guardata su uno schermo. |
| Hardware reale | 🔴 | Non provato: nulla di questo è stato eseguito su una macchina. |
| Documentazione | 🟡 | README in otto lingue, il changelog e le due immagini disegnate da `docs/`. |

## 🎯 Panoramica

* **Il registro** (`src/registry.rs`): ogni repository della suite dice cos'è nel suo `eh3d.project.json`; il launcher li legge, salta quelli danneggiati indicando il motivo e dice quali programmi sono compilati.
* **Il lanciatore** (`src/launch.rs`): avvia un programma della suite e, se termina con errore, lo riavvia fino a due volte con un'attesa che raddoppia; un programma che termina bene viene lasciato in pace.
* **Il file di progetto** (`src/project.rs`, `.eh3dlist`): il nome, i file del lavoro con tipo e SHA-256 e lo zero di lavoro; in lettura rifiuta i percorsi che escono dal progetto e migra il primo formato. **Progetti recenti** (`src/recent.rs`) conserva gli ultimi dieci.
* **Ricerca e riavvio:** il binario di un programma si trova dal nome `[[bin]]` del suo `Cargo.toml` (`eh3d-camdebug`, `eh3d-pnp`...), e un programma che l'operatore ha chiuso di proposito (un segnale) o che ha rifiutato i suoi argomenti (codice di uscita 1 o 2) non viene riavviato: solo un arresto anomalo.
* **La finestra** (`src/window.rs`, `src/gui.rs`; `--features gui`, programma `eh3d-launcher-gui`): i progetti recenti, aprire o creare un progetto, aggiungere e rimuovere i suoi file con gli stessi controlli di `add-layer` e i programmi della suite con un pulsante che avvia ciascuno con il progetto. Stato, azioni, impostazioni e parole sono codice semplice con test; il disegno è sottile. Scritta, non ancora vista su uno schermo.

## 🧩 Ruolo in EH3D

EH3D-SUITE-LAUNCHER è un programma della EH3D Fabrication Suite. Funziona come libreria, come riga di comando e, compilato con la funzione `gui`, come finestra (`eh3d-launcher-gui`) con l'aspetto comune agli altri programmi della suite. Legge e scrive file semplici, quindi può essere usato da solo o tra gli altri programmi.

## 🏛️ Architettura

<p align="center">
  <img src="images/architecture.svg" alt="Architettura di EH3D-SUITE-LAUNCHER: chi lo usa, che cosa contiene e da che cosa dipende" width="100%">
</p>


Come si incastrano le parti di questo repository: chi lo usa, cosa contiene e da cosa dipende. L'immagine è disegnata da `docs/architecture.json` e verificata con il codice.

## ✨ Funzionalità

* **Libreria e riga di comando:** ogni comando di `eh3d-launcher` è una funzione della libreria, così un altro programma della suite può chiamarla senza avviare un processo.
* **Test con input danneggiati:** i test forniscono anche file vuoti, spazzatura, numeri enormi e file tagliati, e nessuno può andare in panico.
* **Nessuna ipotesi:** un valore privo di senso è un errore che dice quale e perché, mai un risultato che sembra solo giusto.
* **Nessun codice unsafe:** `#![forbid(unsafe_code)]` nella libreria e nel programma.
* **Elenchi e progetti più sicuri:** il percorso di un livello non può essere `.`, un nome che Windows rifiuta (`CON`, `AUX`, `COM1`...) né finire con un punto o uno spazio, due livelli che differiscono solo per le maiuscole sono uno solo, l'elenco dei recenti conserva il percorso completo di un file esistente e due esecuzioni contemporanee (`recent`, `add-layer`) non perdono la modifica dell'altra perché il file è bloccato mentre cambia; un livello viene calcolato a blocchi di 64 KiB e un programma terminato da un segnale viene indicato come tale e non come un errore.

## 🔒 Sicurezza e protezione

| Minaccia | Difesa |
|---|---|
| Un file danneggiato, tagliato o vuoto | nessun panico: i test tagliano un input valido in ogni posizione e forniscono spazzatura |
| Un numero privo di senso (negativo, non finito, assurdamente grande) | rifiutato con un errore che nomina il valore; nulla viene arrotondato in qualcosa di plausibile |
| Uso non sicuro della memoria | Rust sicuro e semplice con `forbid(unsafe_code)` |

**Cosa non è difeso:** un risultato prodotto senza errori non è un risultato verificato. Nulla qui conosce i limiti di una macchina e il programma non ha avuto revisione indipendente. Segnalate una vulnerabilità in privato (vedi [SECURITY.md](SECURITY.md)).

## 🔌 Interfacce

La libreria è `eh3d_suite_launcher` (`use eh3d_suite_launcher::...`); la riga di comando è `eh3d-launcher`. I comandi sono questi; ciascuno stampa il risultato sull'output standard e segnala un errore sull'output di errore con un codice di uscita diverso da zero. Non c'è accesso alla rete.

```text
EH3D-SUITE-LAUNCHER/
├── src/
│   ├── browse.rs                                l'elenco dei file della finestra che apre un file
│   ├── gui.rs                                   la finestra (egui, con la funzione gui): disegna lo stato e inoltra ciò che fa la persona
│   ├── gui_main.rs                              la finestra come programma a sé, senza console su Windows
│   ├── lang.rs                                  le parole della finestra nelle otto lingue
│   ├── launch.rs                                una parte della logica
│   ├── lib.rs                                   la libreria: ciò che usa ogni comando
│   ├── main.rs                                  la riga di comando
│   ├── project.rs                               una parte della logica
│   ├── recent.rs                                una parte della logica
│   ├── registry.rs                              una parte della logica
│   ├── settings.rs                              le impostazioni di launcher.cfg: lingua, riavvii e dimensione della finestra
│   └── window.rs                                lo stato della finestra senza schermo: programmi, progetti, file e avvio
├── tests/
│   └── launcher.rs                              test della libreria e della riga di comando
├── docs/
│   ├── architecture.json                        dati dell'immagine di Architettura
│   └── structure.json                           dati dell'immagine di Struttura
├── tools/                                       controllo della CI e del README e strumenti per le immagini
├── images/                                      banner e le immagini di Architettura e Struttura del repository
├── .github/                                     workflow di CI e di rilascio, modelli di issue e pull request
├── eh3d.project.json                            manifesto: versione, ruolo, maturità e comando di compilazione
├── Cargo.toml Cargo.lock                        il workspace e le sue dipendenze bloccate
├── rust-toolchain.toml                          la toolchain Rust fissata
├── CHANGELOG.md                                 cosa è cambiato in ogni versione
├── SECURITY.md SUPPORT.md                       come segnalare una vulnerabilità e dove chiedere
├── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   come contribuire, codice di condotta e licenza
└── launcher.cfg.example                         tutte le impostazioni di launcher.cfg con il valore predefinito
```

## ⚙️ Configurazione

Il file di configurazione è `launcher.cfg` nella cartella del launcher, letto da `launch` e dalla finestra: la sua riga `workspace = PERCORSO_COMPLETO` è la cartella da cui si avviano i programmi (solo quelli noti della suite), e le righe facoltative `language`, `max_restarts`, `backoff_ms`, `window_width` e `window_height` fissano la lingua della finestra, quante volte si riavvia un programma andato in crash, il tempo di attesa e la dimensione della finestra. `launcher.cfg.example` elenca ogni chiave con il valore predefinito e i limiti. A parte questo, ciò che la persona sceglie si passa come argomenti della riga di comando (vedi Interfacce) e ogni valore predefinito è scritto nella guida del comando (`eh3d-launcher help`). I limiti di un valore sono costanti con nome nel codice, coperte da test.

## 🚀 Avvio rapido

Serve una toolchain Rust stabile (1.95 o più recente).

```bash
cargo build --release
cargo test
cargo run --release -- list --workspace ..
# la finestra (richiede la funzione gui)
cargo build --release --features gui
cargo run --release --features gui --bin eh3d-launcher-gui
```

## 🧪 Test

La suite ha 100 test (`cargo test`, eseguiti su Windows e Linux nella CI). Verificano i casi normali, i casi limite e gli input danneggiati: file vuoti, spazzatura e file tagliati non causano mai un panico. La CI esegue anche `cargo fmt --check`, `cargo clippy` (anche con `--features gui`) con gli avvisi come errori e `python -I tools/ci_validate.py`.

## 🏗️ Compilazione e rilascio

La versione sta in `Cargo.toml` e in `eh3d.project.json`; ogni versione ha la sua voce in [CHANGELOG.md](CHANGELOG.md). `build.bat` / `build.sh` compilano il programma, `test.bat` / `test.sh` eseguono i test e `run.bat` / `run.sh` lo eseguono con un esempio. Le versioni sono pubblicate su GitHub.

### Download

Ogni versione contiene il programma a riga di comando e la finestra per Windows (64 e 32 bit), Linux (64 bit) e macOS (Apple silicon e Intel), come `<nome>-<versione>-<sistema>.zip` (Windows) o `.tar.gz` con i due programmi, la licenza, il readme, l'icona e `launcher.cfg.example`. I file sono costruiti da GitHub Actions ed elencati in `SHA256SUMS`. Per verificare un download, confronta il suo hash con il file `.sha256` accanto (`sha256sum -c`).

## 📂 Struttura del repository

<p align="center">
  <img src="images/repository-structure.svg" alt="Struttura del repository EH3D-SUITE-LAUNCHER" width="100%">
</p>

<details>
<summary>Versione testuale dell'albero</summary>

```text
EH3D-SUITE-LAUNCHER/
├── src/
│   ├── browse.rs                                l'elenco dei file della finestra che apre un file
│   ├── gui.rs                                   la finestra (egui, con la funzione gui): disegna lo stato e inoltra ciò che fa la persona
│   ├── gui_main.rs                              la finestra come programma a sé, senza console su Windows
│   ├── lang.rs                                  le parole della finestra nelle otto lingue
│   ├── launch.rs                                una parte della logica
│   ├── lib.rs                                   la libreria: ciò che usa ogni comando
│   ├── main.rs                                  la riga di comando
│   ├── project.rs                               una parte della logica
│   ├── recent.rs                                una parte della logica
│   ├── registry.rs                              una parte della logica
│   ├── settings.rs                              le impostazioni di launcher.cfg: lingua, riavvii e dimensione della finestra
│   └── window.rs                                lo stato della finestra senza schermo: programmi, progetti, file e avvio
├── tests/
│   └── launcher.rs                              test della libreria e della riga di comando
├── docs/
│   ├── architecture.json                        dati dell'immagine di Architettura
│   └── structure.json                           dati dell'immagine di Struttura
├── tools/                                       controllo della CI e del README e strumenti per le immagini
├── images/                                      banner e le immagini di Architettura e Struttura del repository
├── .github/                                     workflow di CI e di rilascio, modelli di issue e pull request
├── eh3d.project.json                            manifesto: versione, ruolo, maturità e comando di compilazione
├── Cargo.toml Cargo.lock                        il workspace e le sue dipendenze bloccate
├── rust-toolchain.toml                          la toolchain Rust fissata
├── CHANGELOG.md                                 cosa è cambiato in ogni versione
├── SECURITY.md SUPPORT.md                       come segnalare una vulnerabilità e dove chiedere
├── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   come contribuire, codice di condotta e licenza
└── launcher.cfg.example                         tutte le impostazioni di launcher.cfg con il valore predefinito
```

</details>

## 🗺️ Roadmap e limiti noti

**Limiti noti oggi**

* La finestra è scritta e la sua logica è testata, ma non è ancora stata compilata né guardata su uno schermo.
* Non è stato provato su una macchina né con una raccolta di file reali e non ha avuto revisione indipendente.
* Non scopre ancora le macchine in rete, non mantiene aggiornata la suite (lo fa EH3D-UPDATER) e il `.eh3dlist` è un file JSON che punta ai file, non un archivio che li contiene.

**Prossimi passi**

* Scoprire le macchine dell'officina in rete e chiamare EH3D-UPDATER dalla finestra.
* Guardare la finestra su uno schermo di ogni sistema e correggere ciò che si trova.
* Provarlo con file reali e con gli altri programmi della suite.

## 🌍 Lingue

Il README e i testi esistono in inglese, spagnolo, francese, italiano, tedesco, cinese, giapponese e portoghese europeo.

## 🔗 Progetti correlati

* [EH3D-COMMON](https://github.com/JuanenRac/EH3D-COMMON): il contenitore di progetto e i tipi condivisi della suite
* [EH3D-UPDATER](https://github.com/JuanenRac/EH3D-UPDATER): l'aggiornatore della suite
* [EH3D-DRILLPRO](https://github.com/JuanenRac/EH3D-DRILLPRO): il programma di foratura
* [EH3D-ROUTEPRO](https://github.com/JuanenRac/EH3D-ROUTEPRO): il programma di fresatura del contorno
* [EH3D-BUNGARD-GUI](https://github.com/JuanenRac/EH3D-BUNGARD-GUI): la finestra delle macchine Bungard
* [EH3D-PHOENIX-CNC](https://github.com/JuanenRac/EH3D-PHOENIX-CNC): il programma di controllo della macchina

## 📚 Documentazione

* [Changelog](CHANGELOG.md)
* [Dati di architettura e struttura](docs/architecture.json): i dati da cui si disegnano le due immagini.

## 🤝 Contribuire e comunità

* Prima di una pull request eseguite `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` e `python -I tools/ci_validate.py`: tutto deve passare. Vedi [CONTRIBUTING.md](CONTRIBUTING.md) e il [codice di condotta](CODE_OF_CONDUCT.md).
* Il README esiste in otto lingue: una modifica al testo si fa in tutti e otto i file, e le due immagini sono disegnate da `docs/architecture.json` e `docs/structure.json` con `tools/make_architecture.py` e `tools/make_structure.py`.
* Segnalate una vulnerabilità in privato: [SECURITY.md](SECURITY.md). Domande, idee e segnalazioni: electrohobby3d@gmail.com ([SUPPORT.md](SUPPORT.md)).

## 👤 AUTORE

**JuanenRac (Electro Hobby 3D)** · electrohobby3d@gmail.com

## 📜 LICENZA

Software: GPL-3.0-or-later. Vedi [LICENSE](LICENSE). I progetti hardware della suite usano la CERN OHL-S.
