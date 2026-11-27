use crate::ERRORS;
use anyhow::anyhow;
use ged_io::types::event::Event;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::LazyLock;

use ged_io::types::individual::family_link::adopted::AdoptedByWhichParent;
use ged_io::types::individual::family_link::FamilyLinkType;
use ged_io::types::GedcomData;

// Module recherchant les enfants adoptés.
// Dans l'export GED effectué depuis Geneanet, ceux-ci sont rattachés à un adoptant via
// une famille virtuelle référencée depuis l'enfant. Le lien est défini par les attributs
// suivants :
// - 1 ADOP
// - 2 FAMC @Fnnn@
// - 3 ADOP HUSB ou 3 ADOP WIFE selon que l'adoptant est un homme ou une femme
// Il est à noter que si 2 époux adoptent l'enfant alors 2 familles sont synthétisées
// alors que le format GED permettrait la valeur BOTH.
// Les liens sont donc individuels (1 enfant <-> 1 parent) mais passent par des familles
// virtuelles synthétisées dans ce seul but.
// Avec le crate ged_io il faut :
// - parcourir les évènements de l'enfant
// - garder ceux de type adoption
// - récupérer leurs liens familiaux
// - garder ceux de type enfant
// - récupérer le champ adopted_by (Husband, Wife ou Both)
// - la famille (virtuelle) ainsi référencée à 1 ou 2 époux adoptants

// Structures de données à précalculer avant de lister les individus ou les familles.
#[derive(Default)]
pub struct Adoption {
    // Map mettant en correspondance le xref de l'enfant avec les xrefs des adoptants
    // pour permettre l'indication du ou des adoptants sur l'individu adopté.
    adopted: BTreeMap<String, Vec<String>>,
    // Mashmap inverse pour permettre l'indication des enfants adoptés. Dans le HTML
    // cela sera indiqué dans la famille dès que qu'un des époux est adoptant.
    // Certes, l'adoption est individuelle mais je préfère cette présentation.
    // En outre dans les faits l'adoptant officiel n'est pas connu, on sait juste
    // que l'enfant a été adopté dans la famille.
    _adopter: BTreeMap<String, Vec<String>>,
    // Set des familles virtuelles pour ne pas les afficher dans la liste des familles.
    virtual_families: BTreeSet<String>,
}

pub static ADOPTIONS: LazyLock<std::sync::RwLock<Adoption>> =
    LazyLock::new(|| Adoption::default().into());

impl Adoption {
    // Précalcul des 3 structures de données
    pub fn init(data: &GedcomData) {
        match ADOPTIONS.write() {
            Err(err) => {
                write_errors!(errors, {
                    errors.insert(
                        "Adoption write locking".to_string(),
                        anyhow!(err.to_string()),
                    );
                });
            }
            Ok(mut adoptions) => {
                for individual in &data.individuals {
                    if let Some(ref individual_xref) = individual.xref {
                        for link in individual
                            .events
                            .iter()
                            .filter(|e| e.event == Event::Adoption)
                            .filter_map(|e| e.family_link.as_ref())
                        {
                            if link.family_link_type == FamilyLinkType::Child {
                                if let Some(ref by_which_parent) = link.adopted_by {
                                    let family_xref = &link.xref;
                                    if let Some(family) = data.find_family(family_xref) {
                                        let mut parents_adoptifs = vec![];
                                        if *by_which_parent == AdoptedByWhichParent::Both
                                            || *by_which_parent == AdoptedByWhichParent::Husband
                                        {
                                            if let Some(ref xref) = family.individual1 {
                                                parents_adoptifs.push(xref.clone());
                                            }
                                        }
                                        if *by_which_parent == AdoptedByWhichParent::Both
                                            || *by_which_parent == AdoptedByWhichParent::Wife
                                        {
                                            if let Some(ref xref) = family.individual2 {
                                                parents_adoptifs.push(xref.clone());
                                            }
                                        }
                                        adoptions.virtual_families.insert(family_xref.clone());
                                        adoptions
                                            .adopted
                                            .entry(individual_xref.to_string())
                                            .and_modify(|e| e.append(&mut parents_adoptifs))
                                            .or_insert(parents_adoptifs);
                                    } else {
                                        write_errors!(errors, {
                                            errors.insert(
                                                family_xref.clone(),
                                                anyhow!(
                                                    "Famille adoptante non trouvée (xref={})",
                                                    family_xref.clone()
                                                ),
                                            );
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                // TODO: remplir le champ adopter
            }
        }
    }
    // Test si c'est un famille virtuelle (pour la filtrer de la liste des familles)
    pub fn is_family_virtual(xref: &str) -> bool {
        return match ADOPTIONS.read() {
            Err(err) => {
                write_errors!(errors, {
                    errors.insert(
                        "Adoption read locking".to_string(),
                        anyhow!(err.to_string()),
                    );
                });
                false
            }
            Ok(adoptions) => adoptions.virtual_families.contains(xref),
        };
    }
    // Retourne les parents adoptifs d'un enfant adopté
    pub fn adopted(xref: &str) -> Vec<String> {
        return match ADOPTIONS.read() {
            Err(err) => {
                write_errors!(errors, {
                    errors.insert(
                        "Adoption read locking".to_string(),
                        anyhow!(err.to_string()),
                    );
                });
                vec![]
            }
            // More idiomatic than: .map(|v| v.clone()).unwrap_or(vec![])
            Ok(adoptions) => adoptions.adopted.get(xref).cloned().unwrap_or_default(),
        };
    }
}
