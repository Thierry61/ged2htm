use crate::ERRORS;
use anyhow::anyhow;

// Dans Geneanet, quand le prénom usuel d'un individu n'est pas le premier prénom
// de la liste de ses prénoms officiels il est recommandé de saisir le prénom usuel
// dans le champ "Prénoms" et la liste des prénoms officiels dans le champ "Prénom alias".
// Ca marche dans l'export GEDCOM (le nom complet (champ NAME) contient la liste de
// tous les prénoms avec le prénom usuel mis en entre guillemets) sauf quand le prénom
// usuel ne fait pas partie de la liste des prénoms officiels (l'export ne contient
// alors que le prénom usuel et a perdu les prénoms officiels).
// Il y a 2 moyens de contourner ce problème. Dans les 2 cas il faut saisir la liste des
// prénoms officiels dans le champ "Prénoms" et ensuite saisir uniquement le prénom usuel
// (sans le nom de famille) :
// - soit dans le champ "Sobriquet" (conçu par défaut pour un surnom ou prénom d'usage isolé,
//   sans le patronyme). Ce sobriquet se retrouve alors proprement sous le tag 2 NICK.
//   L'inconvénient est uniquement esthétique : le prénom usuel apparaît dans Geneanet en
//   plus des prénoms officiels et après le nom de surcroît.
// - soit dans le champ "Nom public" (sans y mettre le nom, sinon il est dédoublé dans
//   Geneanet). Visuellement la représentation de l'individu est celle attendue et le prénom
//   usuel apparaît dans l'export sous le tag 2 GIVN. L'inconvénient est que le nom public
//   est normalement censé remplacer l'intégralité de l'affichage "Prénom Nom" (comme Molière
//   ou Sœur Emmanuelle) et cette solution présente des risques concernant les recherches
//   Geneanet et la compatibilité standard du fichier GEDCOM.
// TODO: Réfléchir si un de ces contournement vaut la peine sachant que de mémoire je n'ai
// que 2 cas (Raymond Favier et Augustine Favier)

// Décompose le nom complet en isolant le prénom d'usage
// Le but est de pouvoir le mettre en évidence en le surlignant
#[derive(Default, Clone, PartialEq)]
pub struct NameParts {
    // Indique si une erreur a été rencontrée et donc si le nom doit être affiché en rouge
    pub error: bool,
    // Partie avant le prénom usuel ou nom complet quand il n'y en a pas
    pub before_or_full: String,
    // Prénom usuel et reste du nom,
    // None quand il n'y a pas de prénom usuel
    pub preferred_and_rest: Option<(String, String)>,
}

pub fn nom_complet(full_name: &str) -> NameParts {
    let parts: Vec<&str> = full_name.split('"').collect();
    match parts.len() {
        1 => NameParts {
            // Il n'y a pas de prénom usuel => on met le nom complet dans le champ before_or_full
            before_or_full: full_name.to_string(),
            ..Default::default()
        },
        3 => NameParts {
            // Il y a un prénom usuel => on décompose en 3 parties : avant, prénom usuel et reste du nom
            error: false,
            before_or_full: parts[0].to_string(),
            preferred_and_rest: Some((parts[1].to_string(), parts[2].to_string())),
        },
        _ => {
            // Le nombre de " est différent de 0 et 2 => on ne peut pas isoler le prénom d'usage
            // On empile l'erreur pour qu'elle soit affichée. A noter que ceci ne marcherait pas dans le
            // composant NomComplet, d'où l'extraction de cette fonction hors du composant.
            write_errors!(errors, {
                errors.insert(
                    full_name.to_string(),
                    anyhow!("Impossible de déterminer le prénom d'usage : {full_name}"),
                );
            });
            NameParts {
                // On met le nom complet dans le champ before_or_full et on positionne le flag d'erreur pour qu'il soit affiché en rouge
                error: true,
                before_or_full: full_name.to_string(),
                ..Default::default()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Les fonctions ok et err étant similaires entre les modules testés
    // on utilise des macros définies dans test_helpers.rs pour les générer
    crate::generer_verif_err!(
        fonction_test: nom_complet,
        chaine_assert: "le nom",
    );

    crate::generer_verif_ok!(
        fonction_test: nom_complet,
        chaine_assert: "le nom",
        types_retour: (String, Option<(String, String)>),
        extracteur: |p: NameParts| (
            p.before_or_full,
            p.preferred_and_rest
        )
    );

    #[test]
    fn test_ok() {
        // Pas de prénom usuel
        assert_eq!(ok("Aa Bb Cc"), ("Aa Bb Cc".to_string(), None));
        // Le prénom usuel est au début
        assert_eq!(
            ok(r#""Aa" Bb Cc"#),
            (
                "".to_string(),
                Some(("Aa".to_string(), " Bb Cc".to_string()))
            )
        );
        // Le prénom usuel est au milieu
        assert_eq!(
            ok(r#"Aa "Bb" Cc"#),
            (
                "Aa ".to_string(),
                Some(("Bb".to_string(), " Cc".to_string()))
            )
        );
        // Le prénom usuel est à la fin (cas hypothétique)
        assert_eq!(
            ok(r#"Aa Bb "Cc""#),
            (
                "Aa Bb ".to_string(),
                Some(("Cc".to_string(), "".to_string()))
            )
        );
    }

    #[test]
    fn test_err() {
        // 1 caractère "
        assert_eq!(
            err(r#"Aa "Bb Cc"#),
            [r#"Impossible de déterminer le prénom d'usage : Aa "Bb Cc"#]
        );
        // 3 caractères "
        assert_eq!(
            err(r#"Aa "Bb" "Cc"#),
            [r#"Impossible de déterminer le prénom d'usage : Aa "Bb" "Cc"#]
        );
    }
}
