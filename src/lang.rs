//! The words of the window in the eight languages of the repository (English, Spanish, German, French, Italian, Japanese, Chinese and Portuguese of Portugal).

/// The languages of the window, in the order the lists of words below follow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Es,
    De,
    Fr,
    It,
    Ja,
    Zh,
    Pt,
}

impl Lang {
    pub const ALL: [Lang; 8] = [
        Lang::En,
        Lang::Es,
        Lang::De,
        Lang::Fr,
        Lang::It,
        Lang::Ja,
        Lang::Zh,
        Lang::Pt,
    ];

    /// The two letters of the language (`en`, `es`, `de`, `fr`, `it`, `ja`, `zh`, `pt`).
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Es => "es",
            Lang::De => "de",
            Lang::Fr => "fr",
            Lang::It => "it",
            Lang::Ja => "ja",
            Lang::Zh => "zh",
            Lang::Pt => "pt",
        }
    }

    /// The name of the language in the language itself.
    pub fn name(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Es => "Español",
            Lang::De => "Deutsch",
            Lang::Fr => "Français",
            Lang::It => "Italiano",
            Lang::Ja => "日本語",
            Lang::Zh => "中文",
            Lang::Pt => "Português",
        }
    }

    /// The language of a code such as `es`, `es_ES.UTF-8` or `pt-PT`.
    pub fn from_code(code: &str) -> Option<Lang> {
        let two: String = code
            .chars()
            .take(2)
            .collect::<String>()
            .to_ascii_lowercase();
        Lang::ALL.into_iter().find(|l| l.code() == two)
    }

    /// The language of the person from the variables of the system; English when it is none of the eight.
    pub fn from_env() -> Lang {
        for var in ["LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Ok(v) = std::env::var(var) {
                if !v.is_empty() {
                    return Lang::from_code(&v).unwrap_or(Lang::En);
                }
            }
        }
        Lang::En
    }
}

/// Puts the values in the places `{0}`, `{1}`... of a text.
pub fn fill(template: &str, values: &[&str]) -> String {
    let mut out = template.to_string();
    for (k, v) in values.iter().enumerate() {
        out = out.replace(&format!("{{{k}}}"), v);
    }
    out
}

/// Every word and sentence of the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Msg {
    NewProject,
    OpenProject,
    Refresh,
    ShowAll,
    Language,
    RecentProjects,
    NoRecent,
    CurrentProject,
    NoProjectHint,
    Layers,
    NoLayers,
    AddLayer,
    Remove,
    VerifyFiles,
    Programs,
    NoPrograms,
    Launch,
    WithProject,
    Version,
    Maturity,
    State,
    Built,
    NotBuilt,
    Workspace,
    Skipped,
    Details,
    Up,
    AllFiles,
    Cancel,
    EmptyFolder,
    FileName,
    ProjectName,
    Create,
    Open,
    Add,
    NewTitle,
    OpenTitle,
    AddTitle,
    Ready,
    Opened,
    Created,
    LayerAdded,
    LayerRemoved,
    Verified,
    VerifiedPartly,
    VerifyFailed,
    Running,
    Ended,
    Signalled,
    GaveUp,
    Failed,
    NoProjectOpen,
    Busy,
    Refreshed,
    BadSettings,
    NeedFileName,
}

impl Msg {
    pub const ALL: [Msg; 56] = [
        Msg::NewProject,
        Msg::OpenProject,
        Msg::Refresh,
        Msg::ShowAll,
        Msg::Language,
        Msg::RecentProjects,
        Msg::NoRecent,
        Msg::CurrentProject,
        Msg::NoProjectHint,
        Msg::Layers,
        Msg::NoLayers,
        Msg::AddLayer,
        Msg::Remove,
        Msg::VerifyFiles,
        Msg::Programs,
        Msg::NoPrograms,
        Msg::Launch,
        Msg::WithProject,
        Msg::Version,
        Msg::Maturity,
        Msg::State,
        Msg::Built,
        Msg::NotBuilt,
        Msg::Workspace,
        Msg::Skipped,
        Msg::Details,
        Msg::Up,
        Msg::AllFiles,
        Msg::Cancel,
        Msg::EmptyFolder,
        Msg::FileName,
        Msg::ProjectName,
        Msg::Create,
        Msg::Open,
        Msg::Add,
        Msg::NewTitle,
        Msg::OpenTitle,
        Msg::AddTitle,
        Msg::Ready,
        Msg::Opened,
        Msg::Created,
        Msg::LayerAdded,
        Msg::LayerRemoved,
        Msg::Verified,
        Msg::VerifiedPartly,
        Msg::VerifyFailed,
        Msg::Running,
        Msg::Ended,
        Msg::Signalled,
        Msg::GaveUp,
        Msg::Failed,
        Msg::NoProjectOpen,
        Msg::Busy,
        Msg::Refreshed,
        Msg::BadSettings,
        Msg::NeedFileName,
    ];

    /// The words in the order English, Spanish, German, French, Italian, Japanese, Chinese, Portuguese.
    fn words(self) -> [&'static str; 8] {
        match self {
            Msg::NewProject => [
                "New project...",
                "Nuevo proyecto...",
                "Neues Projekt...",
                "Nouveau projet...",
                "Nuovo progetto...",
                "新規プロジェクト...",
                "新建项目...",
                "Novo projeto...",
            ],
            Msg::OpenProject => [
                "Open project...",
                "Abrir proyecto...",
                "Projekt öffnen...",
                "Ouvrir un projet...",
                "Apri progetto...",
                "プロジェクトを開く...",
                "打开项目...",
                "Abrir projeto...",
            ],
            Msg::Refresh => [
                "Refresh",
                "Actualizar",
                "Aktualisieren",
                "Actualiser",
                "Aggiorna",
                "更新",
                "刷新",
                "Atualizar",
            ],
            Msg::ShowAll => [
                "Also show firmware, services and apps",
                "Mostrar también firmware, servicios y apps",
                "Auch Firmware, Dienste und Apps anzeigen",
                "Afficher aussi les firmwares, services et applications",
                "Mostra anche firmware, servizi e app",
                "ファームウェア、サービス、アプリも表示",
                "同时显示固件、服务和应用",
                "Mostrar também firmware, serviços e aplicações",
            ],
            Msg::Language => [
                "Language",
                "Idioma",
                "Sprache",
                "Langue",
                "Lingua",
                "言語",
                "语言",
                "Idioma",
            ],
            Msg::RecentProjects => [
                "Recent projects",
                "Proyectos recientes",
                "Letzte Projekte",
                "Projets récents",
                "Progetti recenti",
                "最近のプロジェクト",
                "最近的项目",
                "Projetos recentes",
            ],
            Msg::NoRecent => [
                "No recent projects yet",
                "Aún no hay proyectos recientes",
                "Noch keine letzten Projekte",
                "Aucun projet récent pour l'instant",
                "Ancora nessun progetto recente",
                "最近のプロジェクトはまだありません",
                "还没有最近的项目",
                "Ainda não há projetos recentes",
            ],
            Msg::CurrentProject => [
                "Project",
                "Proyecto",
                "Projekt",
                "Projet",
                "Progetto",
                "プロジェクト",
                "项目",
                "Projeto",
            ],
            Msg::NoProjectHint => [
                "No project is open: open one from the list or create a new one",
                "No hay ningún proyecto abierto: abra uno de la lista o cree uno nuevo",
                "Kein Projekt geöffnet: Öffnen Sie eines aus der Liste oder legen Sie ein neues an",
                "Aucun projet ouvert : ouvrez-en un de la liste ou créez-en un nouveau",
                "Nessun progetto aperto: aprine uno dall'elenco o creane uno nuovo",
                "プロジェクトが開かれていません。一覧から開くか、新規作成してください",
                "没有打开的项目：请从列表中打开或新建一个",
                "Nenhum projeto aberto: abra um da lista ou crie um novo",
            ],
            Msg::Layers => [
                "Files of the project",
                "Ficheros del proyecto",
                "Dateien des Projekts",
                "Fichiers du projet",
                "File del progetto",
                "プロジェクトのファイル",
                "项目文件",
                "Ficheiros do projeto",
            ],
            Msg::NoLayers => [
                "The project has no files yet",
                "El proyecto aún no tiene ficheros",
                "Das Projekt hat noch keine Dateien",
                "Le projet n'a pas encore de fichiers",
                "Il progetto non ha ancora file",
                "プロジェクトにはまだファイルがありません",
                "项目还没有文件",
                "O projeto ainda não tem ficheiros",
            ],
            Msg::AddLayer => [
                "Add file...",
                "Añadir fichero...",
                "Datei hinzufügen...",
                "Ajouter un fichier...",
                "Aggiungi file...",
                "ファイルを追加...",
                "添加文件...",
                "Adicionar ficheiro...",
            ],
            Msg::Remove => [
                "Remove",
                "Quitar",
                "Entfernen",
                "Retirer",
                "Rimuovi",
                "削除",
                "移除",
                "Remover",
            ],
            Msg::VerifyFiles => [
                "Verify files",
                "Verificar ficheros",
                "Dateien prüfen",
                "Vérifier les fichiers",
                "Verifica i file",
                "ファイルを検証",
                "校验文件",
                "Verificar ficheiros",
            ],
            Msg::Programs => [
                "Programs of the suite",
                "Programas de la suite",
                "Programme der Suite",
                "Programmes de la suite",
                "Programmi della suite",
                "スイートのプログラム",
                "套件中的程序",
                "Programas da suite",
            ],
            Msg::NoPrograms => [
                "No programs found",
                "No se ha encontrado ningún programa",
                "Keine Programme gefunden",
                "Aucun programme trouvé",
                "Nessun programma trovato",
                "プログラムが見つかりません",
                "没有找到程序",
                "Nenhum programa encontrado",
            ],
            Msg::Launch => [
                "Launch",
                "Iniciar",
                "Starten",
                "Lancer",
                "Avvia",
                "起動",
                "启动",
                "Iniciar",
            ],
            Msg::WithProject => [
                "Start the programs with the project",
                "Iniciar los programas con el proyecto",
                "Programme mit dem Projekt starten",
                "Lancer les programmes avec le projet",
                "Avvia i programmi con il progetto",
                "プロジェクトを指定してプログラムを起動",
                "使用项目启动程序",
                "Iniciar os programas com o projeto",
            ],
            Msg::Version => [
                "Version",
                "Versión",
                "Version",
                "Version",
                "Versione",
                "バージョン",
                "版本",
                "Versão",
            ],
            Msg::Maturity => [
                "Maturity",
                "Madurez",
                "Reifegrad",
                "Maturité",
                "Maturità",
                "成熟度",
                "成熟度",
                "Maturidade",
            ],
            Msg::State => [
                "State",
                "Estado",
                "Status",
                "État",
                "Stato",
                "状態",
                "状态",
                "Estado",
            ],
            Msg::Built => [
                "built",
                "compilado",
                "gebaut",
                "compilé",
                "compilato",
                "ビルド済み",
                "已编译",
                "compilado",
            ],
            Msg::NotBuilt => [
                "not built",
                "sin compilar",
                "nicht gebaut",
                "non compilé",
                "non compilato",
                "未ビルド",
                "未编译",
                "por compilar",
            ],
            Msg::Workspace => [
                "Folder of the programs: {0}",
                "Carpeta de los programas: {0}",
                "Ordner der Programme: {0}",
                "Dossier des programmes : {0}",
                "Cartella dei programmi: {0}",
                "プログラムのフォルダ：{0}",
                "程序所在文件夹：{0}",
                "Pasta dos programas: {0}",
            ],
            Msg::Skipped => [
                "Folders skipped",
                "Carpetas omitidas",
                "Übersprungene Ordner",
                "Dossiers ignorés",
                "Cartelle saltate",
                "スキップしたフォルダ",
                "已跳过的文件夹",
                "Pastas ignoradas",
            ],
            Msg::Details => [
                "Details",
                "Detalles",
                "Einzelheiten",
                "Détails",
                "Dettagli",
                "詳細",
                "详情",
                "Detalhes",
            ],
            Msg::Up => [
                "Up",
                "Subir",
                "Hoch",
                "Haut",
                "Su",
                "上へ",
                "上一级",
                "Subir",
            ],
            Msg::AllFiles => [
                "All files",
                "Todos los ficheros",
                "Alle Dateien",
                "Tous les fichiers",
                "Tutti i file",
                "すべてのファイル",
                "所有文件",
                "Todos os ficheiros",
            ],
            Msg::Cancel => [
                "Cancel",
                "Cancelar",
                "Abbrechen",
                "Annuler",
                "Annulla",
                "キャンセル",
                "取消",
                "Cancelar",
            ],
            Msg::EmptyFolder => [
                "Nothing here",
                "No hay nada aquí",
                "Hier ist nichts",
                "Rien ici",
                "Qui non c'è nulla",
                "ここには何もありません",
                "这里没有内容",
                "Não há nada aqui",
            ],
            Msg::FileName => [
                "File name",
                "Nombre del fichero",
                "Dateiname",
                "Nom du fichier",
                "Nome del file",
                "ファイル名",
                "文件名",
                "Nome do ficheiro",
            ],
            Msg::ProjectName => [
                "Name of the project",
                "Nombre del proyecto",
                "Name des Projekts",
                "Nom du projet",
                "Nome del progetto",
                "プロジェクト名",
                "项目名称",
                "Nome do projeto",
            ],
            Msg::Create => [
                "Create",
                "Crear",
                "Anlegen",
                "Créer",
                "Crea",
                "作成",
                "创建",
                "Criar",
            ],
            Msg::Open => [
                "Open",
                "Abrir",
                "Öffnen",
                "Ouvrir",
                "Apri",
                "開く",
                "打开",
                "Abrir",
            ],
            Msg::Add => [
                "Add",
                "Añadir",
                "Hinzufügen",
                "Ajouter",
                "Aggiungi",
                "追加",
                "添加",
                "Adicionar",
            ],
            Msg::NewTitle => [
                "New project",
                "Nuevo proyecto",
                "Neues Projekt",
                "Nouveau projet",
                "Nuovo progetto",
                "新規プロジェクト",
                "新建项目",
                "Novo projeto",
            ],
            Msg::OpenTitle => [
                "Open a project",
                "Abrir un proyecto",
                "Projekt öffnen",
                "Ouvrir un projet",
                "Apri un progetto",
                "プロジェクトを開く",
                "打开项目",
                "Abrir um projeto",
            ],
            Msg::AddTitle => [
                "Add a file to the project",
                "Añadir un fichero al proyecto",
                "Datei zum Projekt hinzufügen",
                "Ajouter un fichier au projet",
                "Aggiungi un file al progetto",
                "プロジェクトにファイルを追加",
                "向项目添加文件",
                "Adicionar um ficheiro ao projeto",
            ],
            Msg::Ready => [
                "Ready",
                "Listo",
                "Bereit",
                "Prêt",
                "Pronto",
                "準備完了",
                "就绪",
                "Pronto",
            ],
            Msg::Opened => [
                "Project {0}: {1} files",
                "Proyecto {0}: {1} ficheros",
                "Projekt {0}: {1} Dateien",
                "Projet {0} : {1} fichiers",
                "Progetto {0}: {1} file",
                "プロジェクト {0}：{1} ファイル",
                "项目 {0}：{1} 个文件",
                "Projeto {0}: {1} ficheiros",
            ],
            Msg::Created => [
                "Written {0}",
                "Escrito {0}",
                "{0} geschrieben",
                "{0} écrit",
                "Scritto {0}",
                "{0} を書き込みました",
                "已写入 {0}",
                "Escrito {0}",
            ],
            Msg::LayerAdded => [
                "Added {0} ({1} files in the project)",
                "Añadido {0} ({1} ficheros en el proyecto)",
                "{0} hinzugefügt ({1} Dateien im Projekt)",
                "{0} ajouté ({1} fichiers dans le projet)",
                "Aggiunto {0} ({1} file nel progetto)",
                "{0} を追加しました（プロジェクト内 {1} ファイル）",
                "已添加 {0}（项目中共 {1} 个文件）",
                "Adicionado {0} ({1} ficheiros no projeto)",
            ],
            Msg::LayerRemoved => [
                "Removed {0} ({1} files left)",
                "Quitado {0} (quedan {1} ficheros)",
                "{0} entfernt ({1} Dateien übrig)",
                "{0} retiré ({1} fichiers restants)",
                "Rimosso {0} ({1} file rimasti)",
                "{0} を削除しました（残り {1} ファイル）",
                "已移除 {0}（剩余 {1} 个文件）",
                "Removido {0} (restam {1} ficheiros)",
            ],
            Msg::Verified => [
                "{0}: {1} files, all there and unchanged",
                "{0}: {1} ficheros, todos presentes y sin cambios",
                "{0}: {1} Dateien, alle vorhanden und unverändert",
                "{0} : {1} fichiers, tous présents et inchangés",
                "{0}: {1} file, tutti presenti e invariati",
                "{0}：{1} ファイル、すべて存在し変更なし",
                "{0}：{1} 个文件，全部存在且未更改",
                "{0}: {1} ficheiros, todos presentes e inalterados",
            ],
            Msg::VerifiedPartly => [
                "{0}: {1} files, all there; {2} without a checksum, not verified: {3}",
                "{0}: {1} ficheros, todos presentes; {2} sin suma de comprobación, sin verificar: {3}",
                "{0}: {1} Dateien, alle vorhanden; {2} ohne Prüfsumme, nicht geprüft: {3}",
                "{0} : {1} fichiers, tous présents ; {2} sans somme de contrôle, non vérifiés : {3}",
                "{0}: {1} file, tutti presenti; {2} senza checksum, non verificati: {3}",
                "{0}：{1} ファイル、すべて存在。チェックサムなしの {2} 件は未検証：{3}",
                "{0}：{1} 个文件，全部存在；{2} 个没有校验和，未校验：{3}",
                "{0}: {1} ficheiros, todos presentes; {2} sem soma de verificação, não verificados: {3}",
            ],
            Msg::VerifyFailed => [
                "{0} problems found in the files of the project (see the details)",
                "Se han encontrado {0} problemas en los ficheros del proyecto (vea los detalles)",
                "{0} Probleme in den Dateien des Projekts gefunden (siehe Einzelheiten)",
                "{0} problèmes trouvés dans les fichiers du projet (voir les détails)",
                "Trovati {0} problemi nei file del progetto (vedi i dettagli)",
                "プロジェクトのファイルに {0} 件の問題が見つかりました（詳細を参照）",
                "在项目文件中发现 {0} 个问题（见详情）",
                "Encontrados {0} problemas nos ficheiros do projeto (veja os detalhes)",
            ],
            Msg::Running => [
                "{0} is running...",
                "{0} se está ejecutando...",
                "{0} läuft...",
                "{0} est en cours d'exécution...",
                "{0} è in esecuzione...",
                "{0} を実行中...",
                "{0} 正在运行...",
                "{0} está a executar-se...",
            ],
            Msg::Ended => [
                "{0} ended well",
                "{0} terminó bien",
                "{0} wurde ordnungsgemäß beendet",
                "{0} s'est terminé correctement",
                "{0} è terminato correttamente",
                "{0} は正常に終了しました",
                "{0} 正常结束",
                "{0} terminou bem",
            ],
            Msg::Signalled => [
                "{0}: it was ended by a signal",
                "{0}: lo terminó una señal",
                "{0}: durch ein Signal beendet",
                "{0} : arrêté par un signal",
                "{0}: terminato da un segnale",
                "{0}：シグナルによって終了されました",
                "{0}：被信号终止",
                "{0}: foi terminado por um sinal",
            ],
            Msg::GaveUp => [
                "{0}: it failed {1} times in a row (last code {2})",
                "{0}: falló {1} veces seguidas (último código {2})",
                "{0}: {1} Fehlschläge hintereinander (letzter Code {2})",
                "{0} : échec {1} fois de suite (dernier code {2})",
                "{0}: non è riuscito {1} volte di seguito (ultimo codice {2})",
                "{0}：{1} 回連続で失敗しました（最後のコード {2}）",
                "{0}：连续失败 {1} 次（最后的代码 {2}）",
                "{0}: falhou {1} vezes seguidas (último código {2})",
            ],
            Msg::Failed => [
                "Error: {0}",
                "Error: {0}",
                "Fehler: {0}",
                "Erreur : {0}",
                "Errore: {0}",
                "エラー：{0}",
                "错误：{0}",
                "Erro: {0}",
            ],
            Msg::NoProjectOpen => [
                "Open or create a project first",
                "Primero abra o cree un proyecto",
                "Öffnen oder legen Sie zuerst ein Projekt an",
                "Ouvrez ou créez d'abord un projet",
                "Prima apri o crea un progetto",
                "先にプロジェクトを開くか作成してください",
                "请先打开或新建项目",
                "Primeiro abra ou crie um projeto",
            ],
            Msg::Busy => [
                "{0} is already running",
                "{0} ya se está ejecutando",
                "{0} läuft bereits",
                "{0} est déjà en cours d'exécution",
                "{0} è già in esecuzione",
                "{0} はすでに実行中です",
                "{0} 已在运行",
                "{0} já está a executar-se",
            ],
            Msg::Refreshed => [
                "{0} programs found",
                "{0} programas encontrados",
                "{0} Programme gefunden",
                "{0} programmes trouvés",
                "{0} programmi trovati",
                "{0} 個のプログラムが見つかりました",
                "找到 {0} 个程序",
                "{0} programas encontrados",
            ],
            Msg::BadSettings => [
                "The settings file is not valid, the defaults are used: {0}",
                "El fichero de ajustes no es válido, se usan los valores por defecto: {0}",
                "Die Einstellungsdatei ist ungültig, die Standardwerte werden verwendet: {0}",
                "Le fichier de réglages n'est pas valide, les valeurs par défaut sont utilisées : {0}",
                "Il file delle impostazioni non è valido, si usano i valori predefiniti: {0}",
                "設定ファイルが無効なため、既定値を使用します：{0}",
                "设置文件无效，使用默认值：{0}",
                "O ficheiro de definições não é válido, usam-se os valores predefinidos: {0}",
            ],
            Msg::NeedFileName => [
                "Write the name of the file of the project",
                "Escriba el nombre del fichero del proyecto",
                "Geben Sie den Dateinamen des Projekts ein",
                "Écrivez le nom du fichier du projet",
                "Scrivi il nome del file del progetto",
                "プロジェクトのファイル名を入力してください",
                "请输入项目文件名",
                "Escreva o nome do ficheiro do projeto",
            ],
        }
    }

    /// The text in a language.
    pub fn tr(self, lang: Lang) -> &'static str {
        self.words()[lang as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sentence_has_a_text_in_each_of_the_eight_languages() {
        for m in Msg::ALL {
            for l in Lang::ALL {
                assert!(!m.tr(l).trim().is_empty(), "{m:?} in {}", l.code());
            }
        }
    }

    #[test]
    fn the_list_of_sentences_has_no_repeats() {
        for (i, a) in Msg::ALL.iter().enumerate() {
            assert!(!Msg::ALL[i + 1..].contains(a), "{a:?} is listed twice");
        }
    }

    #[test]
    fn a_template_has_the_same_places_in_every_language() {
        for m in Msg::ALL {
            let places = |l: Lang| {
                (0..5)
                    .filter(|k| m.tr(l).contains(&format!("{{{k}}}")))
                    .count()
            };
            for l in Lang::ALL {
                assert_eq!(places(l), places(Lang::En), "{m:?} in {}", l.code());
            }
        }
    }

    #[test]
    fn the_order_of_the_languages_matches_the_lists() {
        for (k, l) in Lang::ALL.into_iter().enumerate() {
            assert_eq!(l as usize, k);
        }
        assert_eq!(Msg::Cancel.tr(Lang::Es), "Cancelar");
        assert_eq!(Msg::Cancel.tr(Lang::De), "Abbrechen");
        assert_eq!(Msg::Launch.tr(Lang::Pt), "Iniciar");
    }

    #[test]
    fn a_language_is_found_from_the_code_of_the_system() {
        assert_eq!(Lang::from_code("es_ES.UTF-8"), Some(Lang::Es));
        assert_eq!(Lang::from_code("PT-pt"), Some(Lang::Pt));
        assert_eq!(Lang::from_code("ja"), Some(Lang::Ja));
        assert_eq!(Lang::from_code("xx"), None);
        assert_eq!(Lang::from_code(""), None);
    }

    #[test]
    fn the_places_of_a_template_are_filled() {
        assert_eq!(fill("{1} and {0}", &["a", "b"]), "b and a");
        assert_eq!(fill("none", &["a"]), "none");
    }
}
