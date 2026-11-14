<p align="center">
  <img src="images/EH3D_BANNER.svg" alt="EH3D-SUITE-LAUNCHER banner" width="100%">
</p>

# 📖 EH3D-SUITE-LAUNCHER

<p align="center"><a href="README.md">🇺🇸 English</a> | 🇪🇸 <b>Español</b> | <a href="README_fra.md">🇫🇷 Français</a> | <a href="README_ita.md">🇮🇹 Italiano</a> | <a href="README_deu.md">🇩🇪 Deutsch</a> | <a href="README_zho.md">🇨🇳 中文</a> | <a href="README_jpn.md">🇯🇵 日本語</a> | <a href="README_por.md">🇵🇹 Português</a></p>

### El centro de la EH3D Fabrication Suite: encuentra los programas, los arranca y guarda el fichero de proyecto

<p align="center">
  <img src="https://img.shields.io/badge/License-GPL%203.0--or--later-blue.svg" alt="GPL 3.0 or later">
  <img src="https://img.shields.io/badge/Stack-Rust-DEA584.svg" alt="Rust">
  <img src="https://img.shields.io/badge/Languages-8-00E5FF.svg" alt="Languages">
  <img src="https://img.shields.io/badge/Version-0.0.6-0f6cbd.svg" alt="Version">
  <img src="https://img.shields.io/badge/Maturity-scaffolding-FFB020.svg" alt="Maturity">
</p>

---

**Comprobación de honestidad - lo que funciona hoy:** una biblioteca Rust y un programa de línea de comandos (`eh3d-launcher`) con 119 pruebas que se ejecutan en un ordenador: encuentra los programas de la suite por sus manifiestos, los arranca y los vuelve a arrancar si fallan, guarda los proyectos recientes y escribe y comprueba el fichero de proyecto con el SHA-256 de cada fichero. Una ventana (`eh3d-launcher-gui`, compilada con la característica `gui`) está escrita y su lógica está probada, pero no se ha compilado ni mirado en una pantalla; nada se ha probado en una máquina y nada se ha comprobado con una colección de ficheros reales.

---

> **Estado de seguridad:** este programa no controla ninguna máquina y no es un sistema de seguridad. Lo que produce debe revisarlo una persona antes de ejecutar nada en una máquina; nada de esto se ha ejecutado en hardware real.


## 📋 De un vistazo

| | |
|---|---|
| Versión | `0.0.6` |
| Madurez | Andamiaje |
| Pruebas | 119 |
| Pila tecnológica | Rust |
| En hardware real | No probado: nada de esto se ha ejecutado en una máquina |

**Ir a:** [Resumen](#-resumen) · [Arquitectura](#-arquitectura) · [Inicio rápido](#-inicio-rápido) · [Pruebas](#-pruebas) · [Estructura](#-estructura-del-repositorio) · [Relacionados](#-proyectos-relacionados)

## 🚦 Estado del proyecto

Cada línea es una parte de este repositorio y la luz dice hasta dónde llega hoy. Se escribe con lo que el repositorio puede demostrar (pruebas, compilaciones, ejecuciones), no con lo previsto.

🟢 Hecho y verificado · 🟡 Funciona, con límites conocidos · 🟠 Hecho a medias o sin verificar · 🔴 Sin hacer o sin funcionar

| Área | Estado | Qué significa aquí |
|---|---|---|
| Núcleo | 🟡 | encuentra los programas de la suite por sus manifiestos, los arranca y los vuelve a arrancar si fallan, guarda los proyectos recientes y escribe y comprueba el fichero de proyecto con el SHA-256 de cada fichero; probado en un ordenador con entradas normales y dañadas, no con una colección de ficheros reales. |
| Línea de comandos | 🟡 | `eh3d-launcher` ejecuta las mismas funciones que la biblioteca e imprime el resultado; cada error vuelve como mensaje y un código de salida distinto de cero. |
| Pruebas | 🟡 | 119 pruebas pasan (`cargo test`), con casos normales, casos límite y entradas dañadas. |
| Seguridad | 🟡 | Rust seguro y simple (`forbid(unsafe_code)`): nada entra en pánico con los ficheros recibidos y un valor malo es un error con su motivo; sin revisión independiente. |
| Ventana | 🟠 | Escrita (egui, `--features gui`, programa `eh3d-launcher-gui`): los proyectos recientes, abrir o crear un proyecto, añadir y quitar sus ficheros con las comprobaciones de la línea de comandos y los programas de la suite con un botón para iniciar cada uno; su lógica está probada, pero aún no se ha compilado ni mirado en una pantalla. |
| Hardware real | 🔴 | No probado: nada de esto se ha ejecutado en una máquina. |
| Documentación | 🟡 | README en ocho idiomas, el registro de cambios y las dos imágenes dibujadas desde `docs/`. |

## 🎯 Resumen

* **El registro** (`src/registry.rs`): cada repositorio de la suite dice lo que es en su `eh3d.project.json`; el launcher los lee, salta los dañados indicando el motivo y dice qué programas están compilados.
* **El lanzador** (`src/launch.rs`): arranca un programa de la suite y, si termina con error, lo vuelve a arrancar hasta dos veces con una espera que se duplica; un programa que termina bien se deja en paz.
* **El fichero de proyecto** (`src/project.rs`, `.eh3dlist`): el nombre, los ficheros del trabajo con su tipo y SHA-256 y el cero de trabajo; al leer rechaza rutas que salen del proyecto y migra el primer formato. **Proyectos recientes** (`src/recent.rs`) guarda los diez últimos.
* **Búsqueda y reinicio:** el binario de un programa se encuentra por el nombre `[[bin]]` de su `Cargo.toml` (`eh3d-camdebug`, `eh3d-pnp`...), y un programa que el operador cerró a propósito (una señal) o que rechazó sus argumentos (código de salida 1 o 2) no se vuelve a lanzar: solo un fallo grave sí.
* **La ventana** (`src/window.rs`, `src/gui.rs`; `--features gui`, programa `eh3d-launcher-gui`): los proyectos recientes, abrir o crear un proyecto, añadir y quitar sus ficheros con las mismas comprobaciones que `add-layer` y los programas de la suite con un botón para iniciar cada uno con el proyecto. El estado, las acciones, los ajustes y las palabras son código simple con pruebas; el dibujo es fino. Escrita, aún no vista en una pantalla.

## 🧩 Papel en EH3D

EH3D-SUITE-LAUNCHER es un programa de la EH3D Fabrication Suite. Funciona como biblioteca, como línea de comandos y, compilado con la característica `gui`, como ventana (`eh3d-launcher-gui`) con el aspecto que comparten los demás programas de la suite. Lee y escribe ficheros simples, así que puede usarse solo o entre los demás programas.

## 🏛️ Arquitectura

<p align="center">
  <img src="images/architecture.svg" alt="Arquitectura de EH3D-SUITE-LAUNCHER: quién lo usa, qué contiene y de qué depende" width="100%">
</p>


Cómo encajan las partes de este repositorio: quién lo usa, qué contiene y de qué depende. La imagen se dibuja desde `docs/architecture.json` y se comprueba con el código.

## ✨ Funciones

* **Biblioteca y línea de comandos:** cada orden de `eh3d-launcher` es una función de la biblioteca, así que otro programa de la suite puede llamarla sin lanzar un proceso.
* **Pruebas con entradas dañadas:** las pruebas también dan ficheros vacíos, basura, números enormes y ficheros cortados, y ninguna puede provocar un pánico.
* **Sin adivinar:** un valor que no tiene sentido es un error que dice cuál y por qué, nunca un resultado que solo lo parece.
* **Límite de tamaño de los ficheros que lee:** los manifiestos de los repositorios y la lista de proyectos recientes se leen solo si no pesan más que `max_file_mib` de `launcher.cfg` (16 MiB por defecto, de 1 a 1024), en la línea de comandos y en la ventana, y el propio `launcher.cfg` hasta 1 MiB; un manifiesto demasiado grande se omite con el código `file_too_large` y su nombre como cualquier otro manifiesto dañado, `eh3d-launcher recent` termina con ese código si la lista es demasiado grande y la ventana arranca con la lista vacía. Los otros códigos son `file_unreadable` y `file_not_text`.
* **Sin código unsafe:** `#![forbid(unsafe_code)]` en la biblioteca y el programa.
* **Listas y proyectos más seguros:** la ruta de una capa no puede ser `.`, un nombre que Windows rechaza (`CON`, `AUX`, `COM1`...) ni acabar en punto o espacio, dos capas que solo se diferencian en mayúsculas son una, la lista de recientes guarda la ruta completa de un fichero que existe y dos ejecuciones a la vez (`recent`, `add-layer`) no pierden el cambio de la otra porque el fichero se bloquea mientras se modifica, una capa se resume en bloques de 64 KiB y un programa terminado por una señal se dice así y no como un fallo.

## 🔒 Seguridad y protección

| Amenaza | Defensa |
|---|---|
| Un fichero dañado, cortado o vacío | sin pánico: las pruebas cortan una entrada válida en cada posición y dan basura |
| Un número sin sentido (negativo, no finito, absurdamente grande) | se rechaza con un error que nombra el valor; nada se redondea hasta parecer plausible |
| Uso inseguro de memoria | Rust seguro y simple con `forbid(unsafe_code)` |

**Lo que no se defiende:** un resultado producido sin errores no es un resultado verificado. Nada de esto conoce los límites de una máquina y el programa no ha tenido revisión independiente. Informe de una vulnerabilidad en privado (véase [SECURITY.md](SECURITY.md)).

## 🔌 Interfaces

La biblioteca es `eh3d_suite_launcher` (`use eh3d_suite_launcher::...`); la línea de comandos es `eh3d-launcher`. Las órdenes son estas; cada una imprime su resultado por la salida estándar e informa de un error por la salida de errores con un código de salida distinto de cero. No hay acceso a la red.

```text
EH3D-SUITE-LAUNCHER/
├── src/
│   ├── browse.rs                                la lista de ficheros de la ventana que abre un fichero
│   ├── gui.rs                                   la ventana (egui, con la característica gui): dibuja el estado y transmite lo que hace la persona
│   ├── gui_main.rs                              la ventana como programa propio, sin consola en Windows
│   ├── lang.rs                                  las palabras de la ventana en los ocho idiomas
│   ├── launch.rs                                una parte de la lógica
│   ├── limit.rs                                 ficheros leídos con límite de tamaño y un código por rechazo
│   ├── lib.rs                                   la biblioteca: lo que usa cada orden
│   ├── main.rs                                  la línea de comandos
│   ├── project.rs                               una parte de la lógica
│   ├── recent.rs                                una parte de la lógica
│   ├── registry.rs                              una parte de la lógica
│   ├── settings.rs                              los ajustes de launcher.cfg: idioma, reinicios y tamaño de la ventana
│   ├── shot.rs                                  el modo de captura de la ventana (--shot): sus argumentos, leídos y comprobados sin pantalla
│   └── window.rs                                el estado de la ventana sin pantalla: programas, proyectos, ficheros e inicio de programas
├── tests/
│   ├── launcher.rs                              pruebas de la biblioteca y la línea de comandos
│   ├── read_limit.rs                            pruebas del límite de tamaño de los ficheros que se leen
│   └── shot_sample.rs                           pruebas de las muestras en la ventana de una captura
├── examples/
│   ├── workspace/                               manifiestos de muestra de programas de la suite, para mostrar en la ventana de una captura
│   └── demo/                                    una lista de proyecto de muestra con tres archivos y sus sumas de comprobación
├── docs/
│   ├── architecture.json                        datos de la imagen de Arquitectura
│   └── structure.json                           datos de la imagen de Estructura
├── tools/                                       comprobación del CI y del README y herramientas de imágenes
├── images/                                      banner y las imágenes de Arquitectura y Estructura del repositorio
├── .github/                                     flujos de CI y de publicación, plantillas de issues y pull requests
├── eh3d.project.json                            manifiesto: versión, rol, madurez y comando de compilación
├── Cargo.toml Cargo.lock                        el workspace y sus dependencias fijadas
├── rust-toolchain.toml                          la toolchain de Rust fijada
├── CHANGELOG.md                                 qué cambió en cada versión
├── SECURITY.md SUPPORT.md                       cómo informar de una vulnerabilidad y dónde preguntar
├── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   cómo contribuir, el código de conducta y la licencia
└── launcher.cfg.example                         todos los ajustes de launcher.cfg con su valor por defecto
```

## ⚙️ Configuración

El fichero de configuración es `launcher.cfg` en la carpeta del lanzador, leído por `launch` y por la ventana: su línea `workspace = RUTA_COMPLETA` es la carpeta desde la que se inician los programas (solo los programas conocidos de la suite), y las líneas opcionales `language`, `max_restarts`, `backoff_ms`, `window_width` y `window_height` fijan el idioma de la ventana, cuántas veces se vuelve a iniciar un programa que se ha caído, el tiempo de espera y el tamaño de la ventana. `launcher.cfg.example` enumera cada clave con su valor por defecto y sus límites. Aparte de eso, lo que elige la persona se da como argumentos de la línea de comandos (véase Interfaces) y cada valor por defecto está escrito en la ayuda de la orden (`eh3d-launcher help`). Los límites de un valor son constantes con nombre del código y las pruebas los cubren.

## 🚀 Inicio rápido

Necesita una cadena de herramientas Rust estable (1.95 o más reciente).

```bash
cargo build --release
cargo test
cargo run --release -- list --workspace ..
# la ventana (necesita la característica gui)
cargo build --release --features gui
cargo run --release --features gui --bin eh3d-launcher-gui
cargo run --release --features gui --bin eh3d-launcher-gui -- --shot screen.png --open examples/workspace --open examples/demo/demo.eh3dlist   # a picture of the window
```

Para sacar una captura de la ventana sin abrirla a mano, ejecútelo con `--shot FICHERO.png` (también `--size WxH`, `--theme forge|dark|light`, `--wait N`, `--lang CODE`, `--open FILE`, `--replace`): se abre con ese tamaño, guarda el PNG, no guarda ajustes y se cierra solo (código de salida 0 si se escribió el fichero, 1 si no, 2 si los argumentos están mal).

## 🧪 Pruebas

El conjunto tiene 119 pruebas (`cargo test`, ejecutadas en Windows y Linux en el CI). Comprueban los casos normales, los casos límite y las entradas dañadas: ficheros vacíos, basura y ficheros cortados nunca provocan un pánico. El CI ejecuta además `cargo fmt --check`, `cargo clippy` (también con `--features gui`) con las advertencias como errores y `python -I tools/ci_validate.py`.

## 🏗️ Compilación y publicación

La versión está en `Cargo.toml` y en `eh3d.project.json`; cada versión tiene su entrada en [CHANGELOG.md](CHANGELOG.md). `build.bat` / `build.sh` compilan el programa, `test.bat` / `test.sh` ejecutan las pruebas y `run.bat` / `run.sh` lo ejecutan con un ejemplo. Las versiones se publican en GitHub.

### Descargas

Cada versión trae el programa de línea de comandos y la ventana para Windows (64 y 32 bits), Linux (64 bits) y macOS (Apple silicon e Intel), como `<nombre>-<versión>-<sistema>.zip` (Windows) o `.tar.gz` con los dos programas, la licencia, el readme, el icono y `launcher.cfg.example`. Los ficheros los compila GitHub Actions y están en `SHA256SUMS`. Para comprobar una descarga, compare su hash con el fichero `.sha256` que la acompaña (`sha256sum -c`).

## 📂 Estructura del repositorio

<p align="center">
  <img src="images/repository-structure.svg" alt="Estructura del repositorio EH3D-SUITE-LAUNCHER" width="100%">
</p>

<details>
<summary>Versión en texto del árbol</summary>

```text
EH3D-SUITE-LAUNCHER/
├── src/
│   ├── browse.rs                                la lista de ficheros de la ventana que abre un fichero
│   ├── gui.rs                                   la ventana (egui, con la característica gui): dibuja el estado y transmite lo que hace la persona
│   ├── gui_main.rs                              la ventana como programa propio, sin consola en Windows
│   ├── lang.rs                                  las palabras de la ventana en los ocho idiomas
│   ├── launch.rs                                una parte de la lógica
│   ├── lib.rs                                   la biblioteca: lo que usa cada orden
│   ├── main.rs                                  la línea de comandos
│   ├── project.rs                               una parte de la lógica
│   ├── recent.rs                                una parte de la lógica
│   ├── registry.rs                              una parte de la lógica
│   ├── settings.rs                              los ajustes de launcher.cfg: idioma, reinicios y tamaño de la ventana
│   ├── shot.rs                                  el modo de captura de la ventana (--shot): sus argumentos, leídos y comprobados sin pantalla
│   └── window.rs                                el estado de la ventana sin pantalla: programas, proyectos, ficheros e inicio de programas
├── tests/
│   ├── launcher.rs                              pruebas de la biblioteca y la línea de comandos
│   └── shot_sample.rs                           pruebas de las muestras en la ventana de una captura
├── examples/
│   ├── workspace/                               manifiestos de muestra de programas de la suite, para mostrar en la ventana de una captura
│   └── demo/                                    una lista de proyecto de muestra con tres archivos y sus sumas de comprobación
├── docs/
│   ├── architecture.json                        datos de la imagen de Arquitectura
│   └── structure.json                           datos de la imagen de Estructura
├── tools/                                       comprobación del CI y del README y herramientas de imágenes
├── images/                                      banner y las imágenes de Arquitectura y Estructura del repositorio
├── .github/                                     flujos de CI y de publicación, plantillas de issues y pull requests
├── eh3d.project.json                            manifiesto: versión, rol, madurez y comando de compilación
├── Cargo.toml Cargo.lock                        el workspace y sus dependencias fijadas
├── rust-toolchain.toml                          la toolchain de Rust fijada
├── CHANGELOG.md                                 qué cambió en cada versión
├── SECURITY.md SUPPORT.md                       cómo informar de una vulnerabilidad y dónde preguntar
├── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   cómo contribuir, el código de conducta y la licencia
└── launcher.cfg.example                         todos los ajustes de launcher.cfg con su valor por defecto
```

</details>

## 🗺️ Hoja de ruta y límites conocidos

**Límites conocidos hoy**

* La ventana está escrita y su lógica está probada, pero aún no se ha compilado ni mirado en una pantalla.
* No se ha probado en una máquina ni con una colección de ficheros reales y no ha tenido revisión independiente.
* Todavía no descubre máquinas en la red, no mantiene la suite al día (lo hace EH3D-UPDATER) y el `.eh3dlist` es un fichero JSON que apunta a los ficheros, no un archivo que los contiene.

**Siguiente**

* Descubrir las máquinas del taller en la red y llamar a EH3D-UPDATER desde la ventana.
* Mirar la ventana en una pantalla de cada sistema y ajustar lo que se encuentre.
* Probarlo con ficheros reales y con los demás programas de la suite.

## 🌍 Idiomas

El README y los textos existen en inglés, español, francés, italiano, alemán, chino, japonés y portugués de Portugal.

## 🔗 Proyectos relacionados

* [EH3D-COMMON](https://github.com/JuanenRac/EH3D-COMMON): el contenedor de proyecto y los tipos compartidos de la suite
* [EH3D-UPDATER](https://github.com/JuanenRac/EH3D-UPDATER): el actualizador de la suite
* [EH3D-DRILLPRO](https://github.com/JuanenRac/EH3D-DRILLPRO): el programa de taladrado
* [EH3D-ROUTEPRO](https://github.com/JuanenRac/EH3D-ROUTEPRO): el programa de fresado de contorno
* [EH3D-BUNGARD-GUI](https://github.com/JuanenRac/EH3D-BUNGARD-GUI): la ventana de las máquinas Bungard
* [EH3D-PHOENIX-CNC](https://github.com/JuanenRac/EH3D-PHOENIX-CNC): el programa de control de máquina

## 📚 Documentación

* [Registro de cambios](CHANGELOG.md)
* [Datos de arquitectura y estructura](docs/architecture.json): los datos con los que se dibujan las dos imágenes.

## 🤝 Contribuir y comunidad

* Antes de una pull request ejecute `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` y `python -I tools/ci_validate.py`: todo debe pasar. Véanse [CONTRIBUTING.md](CONTRIBUTING.md) y el [código de conducta](CODE_OF_CONDUCT.md).
* El README existe en ocho idiomas: un cambio de texto se hace en los ocho ficheros, y las dos imágenes se dibujan desde `docs/architecture.json` y `docs/structure.json` con `tools/make_architecture.py` y `tools/make_structure.py`.
* Informe de una vulnerabilidad en privado: [SECURITY.md](SECURITY.md). Preguntas, ideas e informes: electrohobby3d@gmail.com ([SUPPORT.md](SUPPORT.md)).

## 👤 AUTOR

**JuanenRac (Electro Hobby 3D)** · electrohobby3d@gmail.com

## 📜 LICENCIA

Software: GPL-3.0-or-later. Véase [LICENSE](LICENSE). Los diseños de hardware de la suite usan CERN OHL-S.
