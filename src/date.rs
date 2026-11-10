use crate::ERRORS;
use anyhow::{bail, Context, Result};
use phf::phf_map;

// Conversion des dates GEDCOM
// - Remplacement des tokens (mois, ABT, BET, ...)
// - Prise en compte du jour et éventuellement du mois manquant
// - Ajout de préfixe: le, en, vers, entre, ...

// Table de hashage statique (du module phf) pour la traduction des mois
// Les casses possibles sont : tout en majuscule, tout en minuscule ou
// la première lettre en majuscule et le reste en minuscule.
static MOIS_TRADUCTION: phf::Map<&'static str, &'static str> = phf_map! {
    "JAN" | "Jan" | "jan" => "janv.",
    "FEB" | "Feb" | "feb" => "févr.",
    "MAR" | "Mar" | "mar" => "mars",
    "APR" | "Apr" | "apr" => "avr.",
    "MAY" | "May" | "may" => "mai",
    "JUN" | "Jun" | "jun" => "juin",
    "JUL" | "Jul" | "jul" => "juil.",
    "AUG" | "Aug" | "aug" => "août",
    "SEP" | "Sep" | "sep" => "sept.",
    "OCT" | "Oct" | "oct" => "oct.",
    "NOV" | "Nov" | "nov" => "nov.",
    "DEC" | "Dec" | "dec" => "déc.",
};

/// Formate le jour anglais (01-31) selon les règles de typographie françaises :
/// "01" -> "1er", "02"-"09" -> "2"-"9", les autres restent identiques.
fn formater_jour(jour_str: &str) -> Result<String> {
    // Vérification que le jour est bien numérique
    if !jour_str.chars().all(|c| c.is_ascii_digit()) {
        bail!(
            "Le jour doit être composé uniquement de chiffres : '{}'",
            jour_str
        )
    }

    let jour_num: u32 = jour_str
        .parse()
        .context("Impossible de parser le jour en entier numérique")?;

    if jour_num == 0 || jour_num > 31 {
        bail!(
            "Le jour '{}' est invalide (doit être compris entre 01 et 31)",
            jour_str
        )
    }

    match jour_num {
        1 => Ok("1er".to_string()),
        n => Ok(n.to_string()),
    }
}

/// Valide le format de l'année (4 chiffres)
fn valider_annee(annee_str: &str) -> Result<&str> {
    if annee_str.len() != 4 || !annee_str.chars().all(|c| c.is_ascii_digit()) {
        bail!(
            "L'année doit être composée de 4 chiffres exactement : '{}'",
            annee_str
        )
    }
    Ok(annee_str)
}

/// Traduit le mois abrégé à l'aide de la table statique phf
fn traduire_mois(mois_str: &str) -> Result<&'static str> {
    let first_three_chars = mois_str
        .get(0..3)
        .with_context(|| format!("Mois abrégé anglais trop court : '{}'", mois_str))?;
    MOIS_TRADUCTION
        .get(first_three_chars)
        .copied()
        .with_context(|| format!("Mois abrégé anglais inconnu ou invalide : '{}'", mois_str))
}

/// Parse et traduit une date élémentaire de l'anglais vers le français.
/// Formats acceptés : "JJ MMM AAAA", "MMM AAAA" ou "AAAA"
fn traduire_date_simple(date_entree: &str, elements: &[&str]) -> Result<String, anyhow::Error> {
    match elements {
        // Cas 1 : Jour (1 ou 2 chiffres), Mois (3 lettres), Année (4 chiffres)
        [jour, mois, annee] => {
            let j = formater_jour(jour).with_context(|| format!("Échec du traitement du jour dans la date '{date_entree}'"))?;
            let m = traduire_mois(mois).with_context(|| format!("Échec de la traduction du mois dans la date '{date_entree}'"))?;
            let a = valider_annee(annee).with_context(|| format!("Échec de la validation de l'année dans la date '{date_entree}'"))?;
            // TODO: convertir la date pour vérifier qu'elle est valide (pour exclure des dates comme 29 févr. 2026, 31 avr. 2024, ...)
            // TODO: retourner cette date pour faire des calculs d'âge
            Ok(format!("{} {} {}", j, m, a))
        }

        // Cas 2 : Mois (3 lettres), Année (4 chiffres) (Le jour est absent)
        [mois, annee] => {
            let m = traduire_mois(mois).with_context(|| format!("Échec de la traduction du mois dans la date '{date_entree}'"))?;
            let a = valider_annee(annee).with_context(|| format!("Échec de la validation de l'année dans la date '{date_entree}'"))?;
            Ok(format!("{} {}", m, a))
        }

        // Cas 3 : Année seule (4 chiffres) (Le jour et le mois sont absents)
        [annee] => {
            let a = valider_annee(annee).with_context(|| format!("Échec de la validation de l'année dans la date '{date_entree}'"))?;
            Ok(a.to_string())
        }

        // Format invalide (chaîne vide ou trop d'éléments)
        _ => bail!(
            "Format de date invalide. Attendu: 'JJ MMM AAAA', 'MMM AAAA' ou 'AAAA'. Reçu: '{date_entree}'"
        ),
    }
}

// Date avec un préfixe (comme "le" dans "le 3 janv. 1923" ou bien "de" et "à" dans "de févr. 1924 à mars 1925")
#[derive(Default, Clone, PartialEq)]
pub struct ElementaryDateParts {
    pub prefix: &'static str,
    pub date: String,
}

// Une ou deux dates préfixées
#[derive(Default, Clone, PartialEq)]
pub struct DateParts {
    // Indique si une erreur a été rencontrée et donc si la date doit être affichée en rouge
    pub error: bool,
    // Date de base
    pub date1: ElementaryDateParts,
    // Date supplémentaire pour les cas BET et FROM
    pub opt_date2: Option<ElementaryDateParts>,
}

// Construction d'une date simple ou double
impl DateParts {
    fn new_simple(prefix: &'static str, date: String) -> Self {
        Self {
            date1: ElementaryDateParts { prefix, date },
            ..Default::default()
        }
    }
    fn new_double(
        prefix1: &'static str,
        date1: String,
        prefix2: &'static str,
        date2: String,
    ) -> Self {
        Self {
            date1: ElementaryDateParts {
                prefix: prefix1,
                date: date1,
            },
            opt_date2: Some(ElementaryDateParts {
                prefix: prefix2,
                date: date2,
            }),
            ..Default::default()
        }
    }
}

/// Parse et traduit une ou deux dates au format GEDCOM.
/// Formats acceptés :
/// - date : le préfixe retourné est alors "le" ou "en" en fonction de la présence ou pas du jour de l'année
/// - ABT date : le préfixe retourné est alors "vers le" ou "vers"
/// - EST date : le préfixe retourné est alors "peut-être le" ou "peut-être"
/// - BEF date : le préfixe retourné est alors "avant le" ou "avant"
/// - AFT date : le préfixe retourné est alors "après le" ou "après"
/// - BET date AND date : le préfixe de la 1ère date est alors "entre le" ou "entre" et celui de la 2ième date "et le" ou "et"
/// - FROM date TO date : le préfixe de la 1ère date est alors "du" ou "de" et celui de la 2ième date "au" ou "à"
fn traduire_date_avec_err(date_entree: &str) -> Result<DateParts> {
    // Découpage par espace en ignorant les espaces multiples potentiels
    let elements: Vec<&str> = date_entree.split_whitespace().collect();
    let (&first, rest) = elements.split_first().context("Date vide !")?;
    match first {
        // Vers
        "ABT" => {
            let prefix = if rest.len() == 3 { "vers le" } else { "vers" };
            Ok(DateParts::new_simple(
                prefix,
                traduire_date_simple(date_entree, rest)?,
            ))
        }
        // Peut-être
        "EST" => {
            let prefix = if rest.len() == 3 {
                "peut-être le"
            } else {
                "peut-être"
            };
            Ok(DateParts::new_simple(
                prefix,
                traduire_date_simple(date_entree, rest)?,
            ))
        }
        // Avant
        "BEF" => {
            let prefix = if rest.len() == 3 { "avant le" } else { "avant" };
            Ok(DateParts::new_simple(
                prefix,
                traduire_date_simple(date_entree, rest)?,
            ))
        }
        // Après
        "AFT" => {
            let prefix = if rest.len() == 3 {
                "après le"
            } else {
                "après"
            };
            Ok(DateParts::new_simple(
                prefix,
                traduire_date_simple(date_entree, rest)?,
            ))
        }
        // Ce format désigne un événement ponctuel unique qui a eu lieu à une date précise, mais
        // cette date exacte est inconnue. L'événement s'est produit à un moment donné situé
        // entre la date de début et la date de fin.
        // A noter que Geneanet propose un OU entre 2 dates qui se traduit aussi par ce code.
        "BET" => {
            let mut iter = rest.split(|&k| k == "AND");
            let rest1 = iter
                .next()
                .with_context(|| format!("Mot clé AND manquant dans la date '{date_entree}"))?;
            if rest1.is_empty() {
                bail!("Date vide entre BET et AND dans la date '{date_entree}")
            }
            let prefix1 = if rest1.len() == 3 {
                "entre le"
            } else {
                "entre"
            };
            let rest2 = iter
                .next()
                // rest2 est en fait vide quand AND est le dernier élément, mais il faut bien gérer ce cas d'erreur
                .with_context(|| format!("Date vide après AND dans la date '{date_entree}"))?;
            if rest2.is_empty() {
                bail!("Date vide après AND dans la date '{date_entree}")
            }
            let prefix2 = if rest2.len() == 3 { "et le" } else { "et" };
            Ok(DateParts::new_double(
                prefix1,
                traduire_date_simple(date_entree, rest1)?,
                prefix2,
                traduire_date_simple(date_entree, rest2)?,
            ))
        }
        // Ce format désigne un événement continu ou répétitif qui s'est déroulé sur toute
        // la durée de l'intervalle. L'événement a commencé à la première date et s'est terminé
        // à la seconde.
        "FROM" => {
            let mut iter = rest.split(|&k| k == "TO");
            let rest1 = iter
                .next()
                .with_context(|| format!("Mot clé TO manquant dans la date '{date_entree}"))?;
            if rest1.is_empty() {
                bail!("Date vide entre FROM et TO dans la date '{date_entree}")
            }
            let prefix1 = if rest1.len() == 3 { "du" } else { "de" };
            let rest2 = iter
                .next()
                // rest2 est en fait vide quand TO est le dernier élément, mais il faut bien gérer ce cas d'erreur
                .with_context(|| format!("Date vide après TO dans la date '{date_entree}"))?;
            if rest2.is_empty() {
                bail!("Date vide après TO dans la date '{date_entree}")
            }
            let prefix2 = if rest2.len() == 3 { "au" } else { "à" };
            Ok(DateParts::new_double(
                prefix1,
                traduire_date_simple(date_entree, rest1)?,
                prefix2,
                traduire_date_simple(date_entree, rest2)?,
            ))
        }
        _ => {
            let prefix = if elements.len() == 3 { "le" } else { "en" };
            Ok(DateParts::new_simple(
                prefix,
                traduire_date_simple(date_entree, &elements)?,
            ))
        }
    }
}

pub fn traduire_date(date_entree: &str) -> DateParts {
    match traduire_date_avec_err(date_entree) {
        Ok(parts) => parts,
        Err(err) => {
            // On enregistre l'erreur
            write_errors!(errors, {
                errors.insert(date_entree.to_string(), err);
            });
            // On retourne la date brute d'origine sans préfixe
            let mut ret = DateParts::new_simple("", date_entree.to_string());
            // On positionne le flag d'erreur pour qu'elle soit affichée en rouge.
            ret.error = true;
            ret
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Les fonctions ok et err étant similaires entre les modules testés
    // on utilise des macros définies dans test_helpers.rs pour les générer
    crate::generer_verif_err!(
        fonction_test: traduire_date,
        chaine_assert: "la date",
    );

    crate::generer_verif_ok!(
        fonction_test: traduire_date,
        chaine_assert: "la date",
        types_retour: (&'static str, String, Option<(&'static str, String)>),
        extracteur: |p: DateParts| (
            p.date1.prefix,
            p.date1.date,
            if let Some(date2) = p.opt_date2 {
                Some((date2.prefix, date2.date))
            } else {
                None
            }
        )
    );

    #[test]
    fn test_dates_completes() {
        assert_eq!(
            ok("01 JAN 2026"),
            ("le", "1er janv. 2026".to_string(), None)
        );
        assert_eq!(
            ok("ABT 01 JAN 2026"),
            ("vers le", "1er janv. 2026".to_string(), None)
        );
        assert_eq!(
            ok("BET 01 JANUARY 2026 AND 03 MARCH 2027"),
            (
                "entre le",
                "1er janv. 2026".to_string(),
                Some(("et le", "3 mars 2027".to_string()))
            )
        );
        assert_eq!(ok("5 Aug 2024"), ("le", "5 août 2024".to_string(), None));
        assert_eq!(ok("12 dec 2025"), ("le", "12 déc. 2025".to_string(), None));
    }

    #[test]
    fn test_dates_sans_jour() {
        assert_eq!(ok("FEB 2023"), ("en", "févr. 2023".to_string(), None));
        assert_eq!(ok("ABT FEB 2023"), ("vers", "févr. 2023".to_string(), None));
        assert_eq!(
            ok("BET JAN 2026 AND MAR 2027"),
            (
                "entre",
                "janv. 2026".to_string(),
                Some(("et", "mars 2027".to_string()))
            )
        );
        assert_eq!(ok("MAR 2020"), ("en", "mars 2020".to_string(), None));
    }

    #[test]
    fn test_dates_annee_seule() {
        assert_eq!(ok("2027"), ("en", "2027".to_string(), None));
        assert_eq!(ok("ABT 2027"), ("vers", "2027".to_string(), None));
        assert_eq!(
            ok("BET 2026 AND 2027"),
            (
                "entre",
                "2026".to_string(),
                Some(("et", "2027".to_string()))
            )
        );
    }

    #[test]
    fn test_erreurs_jours_invalides() {
        // Jour hors limites (00 ou >31)
        assert_eq!(
            err("00 JAN 2026"),
            [
                "Échec du traitement du jour dans la date '00 JAN 2026'",
                "Le jour '00' est invalide (doit être compris entre 01 et 31)"
            ]
        );
        assert_eq!(
            err("32 JAN 2026"),
            [
                "Échec du traitement du jour dans la date '32 JAN 2026'",
                "Le jour '32' est invalide (doit être compris entre 01 et 31)"
            ]
        );

        // Mauvais format de chiffres en entrée (attendu sur des chiffres uniquement)
        assert_eq!(
            err("a1 JAN 2026"),
            [
                "Échec du traitement du jour dans la date 'a1 JAN 2026'",
                "Le jour doit être composé uniquement de chiffres : 'a1'"
            ]
        );
    }

    #[test]
    fn test_erreurs_mois_invalides() {
        // Code mois qui n'existe pas en anglais
        assert_eq!(
            err("01 XYZ 2026"),
            [
                "Échec de la traduction du mois dans la date '01 XYZ 2026'",
                "Mois abrégé anglais inconnu ou invalide : 'XYZ'"
            ]
        );

        // Mois avec une casse non supportée
        assert_eq!(
            err("01 dEc 2026"),
            [
                "Échec de la traduction du mois dans la date '01 dEc 2026'",
                "Mois abrégé anglais inconnu ou invalide : 'dEc'"
            ]
        );

        // Mois avec moins de 3 caractères
        assert_eq!(
            err("01 SE 2026"),
            [
                "Échec de la traduction du mois dans la date '01 SE 2026'",
                "Mois abrégé anglais trop court : 'SE'"
            ]
        );
    }

    #[test]
    fn test_erreurs_annees_invalides() {
        // Année sur 2 chiffres au lieu de 4
        assert_eq!(
            err("01 JAN 26"),
            [
                "Échec de la validation de l'année dans la date '01 JAN 26'",
                "L'année doit être composée de 4 chiffres exactement : '26'"
            ]
        );

        // Année contenant des lettres
        assert_eq!(
            err("01 JAN 202a"),
            [
                "Échec de la validation de l'année dans la date '01 JAN 202a'",
                "L'année doit être composée de 4 chiffres exactement : '202a'"
            ]
        );
    }

    #[test]
    fn test_erreur_format_general() {
        // Trop de segments
        assert_eq!(err("01 JAN AAAA 2026"), ["Format de date invalide. Attendu: 'JJ MMM AAAA', 'MMM AAAA' ou 'AAAA'. Reçu: '01 JAN AAAA 2026'"]);

        // Vide
        assert_eq!(err(""), ["Date vide !"]);
    }
}
