use crate::ERRORS;
use anyhow::{bail, Context, Result};
use jiff::{civil::Date, Unit};
use phf::phf_map;

// Conversion des dates GEDCOM
// - Remplacement des tokens (mois, ABT, BET, ...)
// - Prise en compte du jour et éventuellement du mois manquant
// - Ajout de préfixe: le, en, vers, entre, ...

// Table de hashage statique (du module phf) pour la traduction des mois
// Les casses possibles sont : tout en majuscule, tout en minuscule ou
// la première lettre en majuscule et le reste en minuscule.
static MOIS_TRADUCTION: phf::Map<&'static str, (&'static str, i8)> = phf_map! {
    "JAN" | "Jan" | "jan" => ("janv.", 1),
    "FEB" | "Feb" | "feb" => ("févr.", 2),
    "MAR" | "Mar" | "mar" => ("mars", 3),
    "APR" | "Apr" | "apr" => ("avr.", 4),
    "MAY" | "May" | "may" => ("mai", 5),
    "JUN" | "Jun" | "jun" => ("juin", 6),
    "JUL" | "Jul" | "jul" => ("juil.", 7),
    "AUG" | "Aug" | "aug" => ("août", 8),
    "SEP" | "Sep" | "sep" => ("sept.", 9),
    "OCT" | "Oct" | "oct" => ("oct.", 10),
    "NOV" | "Nov" | "nov" => ("nov.", 11),
    "DEC" | "Dec" | "dec" => ("déc.", 12),
};

/// Formate le jour (01-31) selon les règles de typographie françaises :
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
fn traduire_mois(mois_str: &str) -> Result<(&'static str, i8)> {
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
fn traduire_date_simple(input: &str, elements: &[&str]) -> Result<(String, Date), anyhow::Error> {
    match elements {
        // Cas 1 : Jour (1 ou 2 chiffres), Mois (3 lettres), Année (4 chiffres)
        [jour, mois, annee] => {
            let j = formater_jour(jour)
                .with_context(|| format!("Échec du traitement du jour dans la date '{input}'"))?;
            let m = traduire_mois(mois).with_context(|| {
                format!("Échec de la traduction du mois dans la date '{input}'")
            })?;
            let a = valider_annee(annee).with_context(|| {
                format!("Échec de la validation de l'année dans la date '{input}'")
            })?;
            // Parse la date pour pouvoir faire des calculs d'âge ou d'ordonnancement d'événements
            let date = Date::new(annee.parse()?, m.1, jour.parse()?)
                .with_context(|| format!("Échec de la conversion de la date '{input}'"))?;
            Ok((format!("{} {} {}", j, m.0, a), date))
        }

        // Cas 2 : Mois (3 lettres), Année (4 chiffres) (Le jour est absent)
        [mois, annee] => {
            let m = traduire_mois(mois).with_context(|| {
                format!("Échec de la traduction du mois dans la date '{input}'")
            })?;
            let a = valider_annee(annee).with_context(|| {
                format!("Échec de la validation de l'année dans la date '{input}'")
            })?;
            let date = Date::new(annee.parse()?, m.1, 1)
                .with_context(|| format!("Échec de la conversion de la date '{input}'"))?;
            Ok((format!("{} {}", m.0, a), date))
        }

        // Cas 3 : Année seule (4 chiffres) (Le jour et le mois sont absents)
        [annee] => {
            let a = valider_annee(annee).with_context(|| {
                format!("Échec de la validation de l'année dans la date '{input}'")
            })?;
            let date = Date::new(annee.parse()?, 1, 1)
                .with_context(|| format!("Échec de la conversion de la date '{input}'"))?;
            Ok((a.to_string(), date))
        }

        // Format invalide (chaîne vide ou trop d'éléments)
        _ => bail!(
            "Format de date invalide. Attendu: 'JJ MMM AAAA', 'MMM AAAA' ou 'AAAA'. Reçu: '{input}'"
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
    // Date parsée
    pub date: Date,
    // Indique si la date est approximative (mot clé ABT, BET, ... ou bien jour et éventuellement mois manquant)
    approx: bool,
    // Date supplémentaire pour les cas BET et FROM
    pub opt_date2: Option<ElementaryDateParts>,
}

impl std::fmt::Display for DateParts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.date1.date)
    }
}

// Construction d'une date simple ou double
impl DateParts {
    fn new_simple(prefix: &'static str, date: (String, Date), approx: bool) -> Self {
        Self {
            date1: ElementaryDateParts {
                prefix,
                date: date.0,
            },
            date: date.1,
            approx,
            ..Default::default()
        }
    }
    // Date à 2 élèments (pour BET et FROM)
    fn new_double(
        prefix1: &'static str,
        date1: (String, Date),
        prefix2: &'static str,
        date2: (String, Date),
    ) -> Self {
        Self {
            date1: ElementaryDateParts {
                prefix: prefix1,
                date: date1.0,
            },
            date: date1.1,
            opt_date2: Some(ElementaryDateParts {
                prefix: prefix2,
                date: date2.0,
            }),
            // La date est approximative dans ces 2 cas
            approx: true,
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
fn traduire_date_avec_err(input: &str) -> Result<DateParts> {
    // Découpage par espace en ignorant les espaces multiples potentiels
    let elements: Vec<&str> = input.split_whitespace().collect();
    let (&first, rest) = elements.split_first().context("Date vide !")?;
    match first {
        // Vers
        "ABT" => {
            let prefix = if rest.len() == 3 { "vers le" } else { "vers" };
            Ok(DateParts::new_simple(
                prefix,
                traduire_date_simple(input, rest)?,
                true, // La date est approximative
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
                traduire_date_simple(input, rest)?,
                true, // La date est approximative
            ))
        }
        // Avant
        "BEF" => {
            let prefix = if rest.len() == 3 { "avant le" } else { "avant" };
            Ok(DateParts::new_simple(
                prefix,
                traduire_date_simple(input, rest)?,
                true, // La date est approximative
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
                traduire_date_simple(input, rest)?,
                true, // La date est approximative
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
                .with_context(|| format!("Mot clé AND manquant dans la date '{input}"))?;
            if rest1.is_empty() {
                bail!("Date vide entre BET et AND dans la date '{input}")
            }
            let prefix1 = if rest1.len() == 3 {
                "entre le"
            } else {
                "entre"
            };
            let rest2 = iter
                .next()
                // rest2 est en fait vide quand AND est le dernier élément, mais il faut bien gérer ce cas d'erreur
                .with_context(|| format!("Date vide après AND dans la date '{input}"))?;
            if rest2.is_empty() {
                bail!("Date vide après AND dans la date '{input}")
            }
            let prefix2 = if rest2.len() == 3 { "et le" } else { "et" };
            Ok(DateParts::new_double(
                prefix1,
                traduire_date_simple(input, rest1)?,
                prefix2,
                traduire_date_simple(input, rest2)?,
            ))
        }
        // Ce format désigne un événement continu ou répétitif qui s'est déroulé sur toute
        // la durée de l'intervalle. L'événement a commencé à la première date et s'est terminé
        // à la seconde.
        "FROM" => {
            let mut iter = rest.split(|&k| k == "TO");
            let rest1 = iter
                .next()
                .with_context(|| format!("Mot clé TO manquant dans la date '{input}"))?;
            if rest1.is_empty() {
                bail!("Date vide entre FROM et TO dans la date '{input}")
            }
            let prefix1 = if rest1.len() == 3 { "du" } else { "de" };
            let rest2 = iter
                .next()
                // rest2 est en fait vide quand TO est le dernier élément, mais il faut bien gérer ce cas d'erreur
                .with_context(|| format!("Date vide après TO dans la date '{input}"))?;
            if rest2.is_empty() {
                bail!("Date vide après TO dans la date '{input}")
            }
            let prefix2 = if rest2.len() == 3 { "au" } else { "à" };
            Ok(DateParts::new_double(
                prefix1,
                traduire_date_simple(input, rest1)?,
                prefix2,
                traduire_date_simple(input, rest2)?,
            ))
        }
        _ => {
            let prefix = if elements.len() == 3 { "le" } else { "en" };
            Ok(DateParts::new_simple(
                prefix,
                traduire_date_simple(input, &elements)?,
                // La date est approximative s'il manque un élément (jour et éventuellement mois)
                elements.len() != 3,
            ))
        }
    }
}

pub fn traduire_date(date: &str) -> DateParts {
    match traduire_date_avec_err(date) {
        Ok(parts) => parts,
        Err(err) => {
            // On enregistre l'erreur
            write_errors!(errors, {
                errors.insert(date.to_string(), err);
            });
            // On retourne la date brute d'origine sans préfixe
            let mut ret = DateParts::new_simple("", (date.to_string(), Date::default()), true);
            // On positionne le flag d'erreur pour qu'elle soit affichée en rouge.
            ret.error = true;
            ret
        }
    }
}

// Calcul l'age d'un individu lors d'un évènement (mariage et décès)
// L'age retourné est l'age révolu avec un préfixe indiquant s'il est approximatif ou pas.
pub fn age_avec_err(birth: &Date, event: &Date, approx: bool) -> Result<String> {
    let span = birth.until((Unit::Year, *event))?;
    if span.is_negative() {
        // Cas spécial car get_years retourne 0 au lieu de -1
        // quand la différence est négative mais de moins d'un an
        bail!("Age négatif ({event} < {birth})");
    }
    if span.is_zero() {
        if !approx {
            // Un enfant mort le jour de sa naissance n'est pas forcément mort-né
            // mais on simplifie
            return Ok("mort-né".to_string());
        } else {
            // En revanche si les ages sont approximatifs, on indique "en bas âge"
            return Ok("en bas âge".to_string());
        }
    }
    match span.get_years() {
        // Age négatif => ne doit pas se produire car ce cas est déjà traité plus haut
        age if age < 0 => bail!("Age négatif (BUG)"),
        // avant un an / en bas âge
        0 => Ok((if approx { "en bas âge" } else { "avant un an" }).to_string()),
        // à un an / vers un an
        1 => Ok(format!("{} un an", if approx { "vers" } else { "à" })),
        // à {age} ans / vers {age} ans
        age => Ok(format!("{} {age} ans", if approx { "vers" } else { "à" })),
    }
}

// Age d'un individu lors d'un événement de type mariage ou décès.
// Il est éventuellement approximatif et a un préfixe (à, vers, ..)
// Un flag d'erreur indique si le calcul a généré une erreur.
#[derive(Default, Clone, PartialEq)]
pub struct Age {
    // Age préfixé et éventuellement approximatif
    pub age: String,
    // Flag d'erreur
    pub error: bool,
}

pub fn age(birth: &DateParts, event: &DateParts) -> Age {
    if birth.error || event.error {
        // On retourne un age vide si l'une des 2 dates est erronée, sans déclencher d'erreur supplémentaire
        // (car une erreur sur birth ou event à déjà été générée)
        return Age::default();
    }
    if birth.date == Date::default() || event.date == Date::default() {
        // Idem si une des 2 dates n'a pas été renseignée
        return Age::default();
    }
    match age_avec_err(&birth.date, &event.date, birth.approx || event.approx) {
        Ok(age) => Age { age, error: false },
        Err(err) => {
            // On construit une clé basée sur la 2ème date (plus simple pour les tests)
            let key = event.date1.date.clone();
            // On enregistre l'erreur
            write_errors!(errors, {
                errors.insert(key.clone(), err);
            });
            // On retourne la clé (choix possibles: ne pas l'afficher ou l'afficher en rouge)
            Age {
                age: key,
                error: true,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date;

    // Les fonctions ok et err étant similaires entre les modules testés
    // on utilise des macros définies dans test_helpers.rs pour les générer
    crate::generer_verif_err!(
        err<&str>
        fonction_test: traduire_date,
        chaine_assert: "la date",
    );

    crate::generer_verif_ok!(
        ok<&str, (&'static str, String, Option<(&'static str, String)>)>
        fonction_test: traduire_date,
        chaine_assert: "la date",
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
    fn test_erreurs_format_general() {
        // Trop de segments
        assert_eq!(err("01 JAN AAAA 2026"), ["Format de date invalide. Attendu: 'JJ MMM AAAA', 'MMM AAAA' ou 'AAAA'. Reçu: '01 JAN AAAA 2026'"]);

        // Vide
        assert_eq!(err(""), ["Date vide !"]);
    }

    // Test de dates syntaxiquement correctes mais sémantiquement invalides
    #[test]
    fn test_dates_invalides() {
        // 31 d'un mois sur 30 jours
        assert_eq!(
            err("31 APR 2026"),
            [
                "Échec de la conversion de la date '31 APR 2026'",
                "parameter 'day' for `2026-04` is invalid, must be in range `1..=30`"
            ]
        );

        // 29 février d'une année non bissextile
        assert_eq!(
            err("29 FEB 2026"),
            [
                "Échec de la conversion de la date '29 FEB 2026'",
                "parameter 'day' for `2026-02` is invalid, must be in range `1..=28`"
            ]
        );
    }

    // Construction d'un DateParts pour les tests
    fn date_parts(date: Date, date_str: &str, approx: bool) -> DateParts {
        let date1 = ElementaryDateParts {
            prefix: "",
            date: date_str.to_string(),
        };
        DateParts {
            date1,
            date,
            approx,
            ..Default::default()
        }
    }

    // Fonction fixant la date d'origine pour les calculs d'age pour
    // pouvoir utiliser les fonctions ok et err avec un seul argument
    fn gen_age(event: &DateParts) -> Age {
        let date = date(1940, 10, 15);
        let birth = DateParts {
            date,
            ..Default::default()
        };
        age(&birth, event)
    }

    crate::generer_verif_err!(
        err2<&DateParts>
        fonction_test: gen_age,
        chaine_assert: "l'age",
    );

    crate::generer_verif_ok!(
        ok2<&DateParts, String>
        fonction_test: gen_age,
        chaine_assert: "l'age",
        extracteur: |p: Age| p.age
    );

    #[test]
    fn test_ages_exacts() {
        // Test d'un age négatif
        assert_eq!(
            err2(&date_parts(date(1940, 10, 14), "14 OCT 1940", false)),
            ["Age négatif (1940-10-14 < 1940-10-15)"]
        );

        // Test mort-né
        assert_eq!(ok2(&date_parts(date(1940, 10, 15), "", false)), "mort-né");

        // Test moins d'un an
        assert_eq!(
            ok2(&date_parts(date(1940, 10, 16), "", false)),
            "avant un an"
        );

        // Test un an
        assert_eq!(ok2(&date_parts(date(1941, 10, 15), "", false)), "à un an");

        // Test > un an
        assert_eq!(ok2(&date_parts(date(1942, 10, 15), "", false)), "à 2 ans");

        // Test de la transition de l'age révolu
        assert_eq!(ok2(&date_parts(date(1950, 10, 14), "", false)), "à 9 ans");
        assert_eq!(ok2(&date_parts(date(1950, 10, 15), "", false)), "à 10 ans");
    }

    #[test]
    fn test_ages_approximatifs() {
        // Test d'un age négatif
        assert_eq!(
            err2(&date_parts(date(1940, 10, 14), "14 OCT 1940", true)),
            ["Age négatif (1940-10-14 < 1940-10-15)"]
        );

        // Test mort-né
        assert_eq!(ok2(&date_parts(date(1940, 10, 15), "", true)), "en bas âge");

        // Test moins d'un an
        assert_eq!(ok2(&date_parts(date(1940, 10, 16), "", true)), "en bas âge");

        // Test un an
        assert_eq!(ok2(&date_parts(date(1941, 10, 15), "", true)), "vers un an");

        // Test > un an
        assert_eq!(ok2(&date_parts(date(1942, 10, 15), "", true)), "vers 2 ans");

        // Test de la transition de l'age révolu
        assert_eq!(ok2(&date_parts(date(1950, 10, 14), "", true)), "vers 9 ans");
        assert_eq!(
            ok2(&date_parts(date(1950, 10, 15), "", true)),
            "vers 10 ans"
        );
    }
}
