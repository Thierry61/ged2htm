use crate::ERRORS;
use anyhow::anyhow;

// Dans Geneanet, lorsque le prénom usuel d'un individu est dans la liste
// de ses prénoms officiels, une solution pour le désigner consiste à saisir
// le prénom usuel dans le champ "Prénoms" et les prénoms officiels dans
// le champ "Prénom alias".
//
// Cette représentation est correctement restituée dans l'export GEDCOM lorsque
// le prénom usuel fait partie des prénoms officiels : le champ NAME contient
// alors tous les prénoms, avec le prénom usuel entre guillemets.
//
// En revanche, lorsque le prénom usuel ne fait pas partie des prénoms officiels,
// l'export observé ne conserve dans NAME que le prénom usuel et perd donc les
// prénoms officiels.
//
// Pour éviter cette perte, il faut conserver la liste des prénoms officiels dans
// le champ "Prénoms" et enregistrer séparément le prénom usuel.
//
// Deux contournements sont possibles :
//
// - utiliser le champ "Sobriquet" pour le seul prénom usuel. Celui-ci est exporté
//   sous le tag NICK, ce qui permet de distinguer proprement les prénoms officiels
//   du prénom d'usage. En contrepartie, l'affichage dans Geneanet est moins naturel
//   et, d'après mes tests, le prénom usuel n'apparaît pas dans l'index général
//   des individus.
//
// - utiliser le champ "Nom public" pour le seul prénom usuel. L'affichage dans
//   Geneanet correspond alors davantage au résultat recherché et le prénom usuel
//   est exporté sous GIVN. Cependant, "Nom public" est normalement destiné à une
//   appellation publique remplaçant l'ensemble "Prénom Nom" (Bourvil, Molière,
//   Sœur Emmanuelle, etc.). Son utilisation pour un simple prénom d'usage détourne
//   donc la fonction du champ et rend le GEDCOM plus dépendant du comportement
//   particulier de Geneanet.
//
// => Choix du champ "Sobriquet" : malgré un affichage moins satisfaisant dans
//    Geneanet, il offre le meilleur compromis entre conservation de l'information,
//    distinction entre prénoms officiels et prénom usuel, et portabilité GEDCOM.

// Décompose le nom complet en isolant un prénom d'usage inclus dans la liste des
// prénoms offciels de l'état civil.
// Lorsque celui-ci n'y est pas inclus il est alors stocké dans le champ nick.
#[derive(Default, Clone, PartialEq)]
pub struct NameParts {
    // Indique si une erreur a été rencontrée et donc si le nom doit être affiché en rouge
    pub error: bool,
    // Partie avant le prénom usuel ou nom complet quand il n'y en a pas
    pub before_or_full: String,
    // Prénom usuel (affiché en souligné) et reste du nom,
    // None quand il n'y a pas de prénom usuel
    pub preferred_and_rest: Option<(String, String)>,
    // Prénom usuel quand il ne fait pas partie de la liste des prénoms officiels
    // Il est affiché après le nom en italique-souligné
    pub nick: Option<String>,
}

pub fn nom_complet(full_name_and_nick_name: (&str, Option<&str>)) -> NameParts {
    let full_name = full_name_and_nick_name.0;
    let nick = full_name_and_nick_name.1.map(|n| n.to_string());
    let parts: Vec<&str> = full_name.split('"').collect();
    match parts.len() {
        1 => NameParts {
            // Il n'y a pas de prénom usuel => on met le nom complet dans le champ before_or_full
            before_or_full: full_name.to_string(),
            nick,
            ..Default::default()
        },
        3 => NameParts {
            // Il y a un prénom usuel => on décompose en 3 parties : avant, prénom usuel et reste du nom
            error: false,
            before_or_full: parts[0].to_string(),
            preferred_and_rest: Some((parts[1].to_string(), parts[2].to_string())),
            nick,
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
                nick,
                ..Default::default()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Fonction intermédiaire avec un seul argument d'entrée permettant les tests ok et err
    // (on ne teste que le cas prénom usuel inclus dans la liste des prénoms officiels)
    fn nom_complet_sans_surnom(full_name: &str) -> NameParts {
        nom_complet((full_name, None))
    }

    // Les fonctions ok et err étant similaires entre les modules testés
    // on utilise des macros définies dans test_helpers.rs pour les générer
    crate::generer_verif_err!(
        err<&str>
        fonction_test: nom_complet_sans_surnom,
        chaine_assert: "le nom",
    );

    crate::generer_verif_ok!(
        ok<&str, (String, Option<(String, String)>)>
        fonction_test: nom_complet_sans_surnom,
        chaine_assert: "le nom",
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
