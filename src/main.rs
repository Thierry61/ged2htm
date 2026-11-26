use anyhow::{Error, Result};
use base64::{prelude::BASE64_STANDARD, Engine};
use clap::{ArgGroup, Parser, Subcommand};
use dioxus::prelude::*;
use ged_io::types::GedcomData;
use ged_io::Gedcom;
use jiff::{Unit, ZonedDateTime};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::LazyLock;

// Gestion du BTreeMap contenant les erreurs détectées (BTreeMap pour garder un ordonnacement par clé)/
// L'encapsulation par use_context::<Store<...> ne marchant pas en mode test (car le runtime Dioxus n'est pas présent),
// on utilise la structure LazyLock<RwLock<...> à la place.
// Pourquoi cette approche :
// - LazyLock gère l'initialisation unique : Il permet de contourner le fait que BTreeMap::new() et RwLock::new()
//   ne peuvent pas être évalués directement à la compilation.
// - RwLock (Read-Write Lock) permet la concurrence : Contrairement à un Mutex classique qui bloque tout le monde
//   dès qu'on y accède, le RwLock offre un excellent compromis :
//   - Plusieurs threads peuvent lire (.read()) la BTreeMap en même temps sans se bloquer.
//   - Un seul thread peut écrire (.write()) à la fois, bloquant temporairement les lecteurs pour éviter les corruptions
//     de données (data races).
static ERRORS: LazyLock<std::sync::RwLock<BTreeMap<String, Error>>> =
    LazyLock::new(|| std::sync::RwLock::new(BTreeMap::new()));

// Pour réduire la durée des verrous en écriture il ne faut pas définir la variable mutable errors en début de fonction,
// mais il faut la définir au plus près de chaque écriture du BTreeMap. Cette macro aide en cela en définissant un scope
// limité pour le lock nécessaire à l'écriture.
macro_rules! write_errors {
    ($errors:ident, $body:expr) => {{
        let mut $errors = ERRORS.write().unwrap();
        $body
    }};
}

// Exports pour les composants
mod date;
mod full_name;
mod notes;
mod place;

// Import des composants
mod components;
use components::list::{ListeFamilles, ListeFamillesProps, ListeIndividus, ListeIndividusProps};

// Autres imports
mod shell_quote;
use shell_quote::command_line;
mod adopt;
use adopt::Adoption;

#[cfg(test)]
mod test_helpers;

// Bypass Dioxus asset pipeline because the program isn't a web server but just a CLI tool.
// Instead embed assets inside the binary at compile time.
// Using this strategy ged2htm.exe remains 100% portable. It will read the genealogy file,
// compile a single html file, and embed all styles, icons, and layouts directly inside it,
// so the user can easily share the file with anyone else.
const TAILWIND_CSS: &str = include_str!("../assets/tailwind.css");

// For binary assets we additionally need to encode them in base64
static FAVICON: LazyLock<String> = LazyLock::new(|| {
    let encoded = BASE64_STANDARD.encode(include_bytes!("../assets/smalllogo.png"));
    format!("data:image/x-icon;base64,{}", encoded)
});
static MALE_PNG: LazyLock<String> = LazyLock::new(|| {
    let encoded = BASE64_STANDARD.encode(include_bytes!("../assets/male.png"));
    format!("data:image/png;base64,{}", encoded)
});
static FEMALE_PNG: LazyLock<String> = LazyLock::new(|| {
    let encoded = BASE64_STANDARD.encode(include_bytes!("../assets/female.png"));
    format!("data:image/png;base64,{}", encoded)
});

#[derive(Parser, Debug, Clone, PartialEq)]
#[command(
    author,
    version,
    about = "Générateur HTML à partir d'un fichier GEDCOM"
)]
struct Args {
    /// Chemin vers le fichier GED d'entrée
    #[arg(short, long)]
    input: PathBuf,

    /// Chemin du fichier HTML de sortie
    #[arg(short, long, default_value = "output.html")]
    output: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
enum Command {
    /// Génère un livre généalogique
    Book(BookArgs),

    /// Liste tous les individus ou toutes les familles
    All(AllArgs),
}

#[derive(clap::Args, Debug, Clone, PartialEq)]
#[command(group(
    ArgGroup::new("direction")
        .required(true)
        .args(["ascending", "descending"])
))]
struct BookArgs {
    /// Nom de départ exact (ex: "\"Marie\" Eugénie Briançon")
    #[arg(short, long)]
    start: String,

    /// Nombre de générations à extraire (1 = individu seul, 2 = avec parents ou enfants, etc.)
    #[arg(short, long, default_value_t = 1)]
    generations: usize,

    /// Génère le livre d'ascendance
    #[arg(short, long)]
    ascending: bool,

    /// Génère le livre de descendance
    #[arg(short, long)]
    descending: bool,
}

#[derive(clap::Args, Debug, Clone, PartialEq)]
#[command(group(
    ArgGroup::new("kind")
        .required(true)
        .args(["individuals", "families"])
))]
struct AllArgs {
    /// Liste tous les individus
    #[arg(short, long)]
    individuals: bool,

    /// Liste toutes les familles
    #[arg(short, long)]
    families: bool,
}

// Composant racine de l'application
// TODO: pour l'instant l'application ne fait qu'afficher la liste de tous les individus
#[component]
fn App(args: Args, data: GedcomData) -> Element {
    // Date et heure du fichier GED
    let mut ged_date = "Unknown".to_string();
    let mut ged_time = "Unknown".to_string();
    if let Some(header) = &data.header {
        if let Some(ref date) = header.date {
            if let Some(ref date) = date.value {
                ged_date = date.clone();
            }
            if let Some(ref time) = date.time {
                ged_time = time.clone();
            }
        }
    }

    // Date et heure courante
    let now = ZonedDateTime::now().round(Unit::Second)?;
    let now = now.strftime("%d %b %Y %H:%M:%S").to_string().to_uppercase();

    // Nombre total d'individus et de familles
    let nb_individus = data.individuals.len();
    let nb_familles = data.families.len();

    // Définitions des adoptions
    Adoption::init(&data);

    // Context provider pour éviter de passer la structure GEDCOM en paramètre à chaque composant
    let data = use_store(|| data);
    use_context_provider(|| data);

    // Entités affichées (individus ou familles)
    let entity = match args.command {
        Command::All(ref all) if all.individuals => "d'individus affichés",
        _ => "de familles affichées",
    };

    // Précalcul du composant constituant le contenu de la page
    // (dépendant de la commande et de ses arguments)
    // Ceci permet d'afficher la liste des erreurs AVANT le contenu.
    let (title, nb, contents): (String, usize, Element) = match args.command {
        Command::Book(book) => {
            if book.ascending {
                (
                    format!("Livre d'ascendance de {}", book.start),
                    0,
                    rsx! {
                        div { class: "mt-4", "TODO: livre ascendant à développer" }
                    },
                )
            } else {
                (
                    format!("Livre de descendance de {}", book.start),
                    0,
                    rsx! {
                        div { class: "mt-4", "TODO: livre descendant à développer" }
                    },
                )
            }
        }
        Command::All(all) => {
            if all.individuals {
                let xrefs = data
                    .read()
                    .individuals
                    .iter()
                    .map(|individual| {
                        individual.xref.clone().with_context(|| {
                            format!("Xref manquant pour l'individu : {:?}", individual)
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                (
                    "Liste de tous les individus".to_string(),
                    xrefs.len(),
                    ListeIndividus(ListeIndividusProps { xrefs }),
                )
            } else {
                let xrefs = data
                    .read()
                    .families
                    .iter()
                    .map(|family| {
                        family.xref.clone().with_context(|| {
                            format!("Xref manquant pour la famille : {:?}", family)
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                (
                    "Liste de toutes les familles".to_string(),
                    xrefs.len(),
                    ListeFamilles(ListeFamillesProps { xrefs }),
                )
            }
        }
    };

    rsx! {
        link { rel: "icon shortcut", r#type: "image/x-icon", href: &*FAVICON }
        style { "{TAILWIND_CSS}" }
        body { class: "bg-stone-50 p-5 text-xs font-sans",
            div {
                style {
                    "li:has(.error) {{ border-color: var(--color-red-200); }}
                    .error {{ color: var(--color-red-800); }}
                    h1 {{ font-weight: bold; text-decoration: underline; font-size: var(--text-lg); }}
                    /* 00A0 = nbsp and inline block prevents underline expanding to after text */
                    h1::after {{ content: '\\00A0:'; display: inline-block; }}"
                }
                div { class: "max-w-420 bg-white p-5 rounded-r-lg shadow-md my-0 mx-auto",
                    h1 { class: "pb-2.5", "Rapport d'extraction GEDCOM" }
                    div { class: "border-l-2 border-l-solid border-blue-500 bg-blue-75 rounded-r-lg p-4 mb-5",
                        p {
                            strong { "Ligne de commande : " }
                            {command_line()}
                        }
                        br {}
                        p {
                            strong { "Fichier GED d'entrée : " }
                            "{args.input.display()}"
                        }
                        p {
                            strong { "Date et heure de génération du fichier GED : " }
                            "{ged_date} {ged_time}"
                        }
                        p {
                            strong { "Nombre total d'individus : " }
                            "{nb_individus}"
                        }
                        p {
                            strong { "Nombre total de familles : " }
                            "{nb_familles}"
                        }
                        br {}
                        p {
                            strong { "Fichier HTML de sortie : " }
                            "{args.output.display()}"
                        }
                        p {
                            strong { "Date et heure de génération du fichier HTML : " }
                            "{now}"
                        }
                        p {
                            strong { "Nombre {entity} : " }
                            "{nb}"
                        }
                    }
                    if let Ok(errors) = ERRORS.read() {
                        if !errors.is_empty() {
                            h1 { class: "pb-0.5", "Liste des erreurs rencontrées ({errors.len()})" }
                            div { class: "border-l-2 border-l-solid border-red-500 bg-red-100 rounded-r-lg p-3 my-2",
                                ul {
                                    for error in errors.iter() {
                                        li { class: "error ml-4 my-2 list-disc list-outside",
                                            " [ "
                                            span { class: "font-bold underline", "{error.0}" }
                                            " ] :"
                                            for cause in error.1.chain() {
                                                div { "{cause}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    h1 { class: "-mb-2.5 mt-4", {title} }
                    {contents}
                }
            }
        }
    }
}

fn main() -> Result<()> {
    // Récupération et validation des arguments CLI
    let args = Args::parse();

    // Lecture et extraction des données du fichier GED
    // Grâce à anyhow, on utilise `.context()` pour donner du contexte aux erreurs
    let source = fs::read_to_string(&args.input).context(format!(
        "Impossible d'ouvrir le fichier d'entrée : {:?}",
        args.input
    ))?;

    let mut gedcom = Gedcom::new(source.chars()).context(format!(
        "Impossible de charger le fichier d'entrée : {:?}",
        args.input
    ))?;

    let data = gedcom.parse_data().context(format!(
        "Impossible d'analyser le fichier d'entrée : {:?}",
        args.input
    ))?;

    println!("Génération du HTML",);

    // Rendu du composant Dioxus en chaîne HTML statique
    let html_string = dioxus_ssr::render_element(rsx! {
        App { args: args.clone(), data }
    });

    // Création du fichier HTML de sortie
    let mut file_out = File::create(&args.output).context(format!(
        "Impossible de créer le fichier de sortie : {:?}",
        args.output
    ))?;

    // Ecriture de la clause DOCTYPE pour que la page ne soit pas en mode quirks
    file_out
        .write_all(b"<!DOCTYPE html>")
        .context("Erreur lors de l'écriture de la déclaration DOCTYPE")?;

    // Ecriture du reste de la page html
    file_out
        .write_all(html_string.as_bytes())
        .context("Erreur lors de l'écriture des données HTML")?;

    println!("Fichier HTML généré dans : {:?}", args.output);
    Ok(())
}
