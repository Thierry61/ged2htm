use anyhow::anyhow;
use dioxus::prelude::*;
use ged_io::types::individual::family_link::FamilyLinkType;
use ged_io::types::individual::gender::{Gender, GenderType};
use ged_io::types::GedcomData;

use crate::date::{age, traduire_date, Age, DateParts};
use crate::full_name::{nom_complet, NameParts};
use crate::notes::concat_notes;
use crate::place::{interpreter_lieu, PlaceParts};
use crate::Adoption;
use crate::ERRORS;
use crate::{FEMALE_PNG, MALE_PNG};

// Composant pour afficher des notes
#[component]
fn Notes(notes: Vec<String>) -> Element {
    rsx! {
        div {
            for note in notes {
                div {
                    class: "mb-1 border-l-2 border-blue-300 pl-1 text-gray-600",
                    dangerous_inner_html: note,
                }
            }
        }
    }
}

// Composant pour afficher la date et le lieu d'un évènement
// - event: "Né" ou "Décédé"
// - feminine: "" ou "e"
// - date: les éléments de la date avec leurs préfixes (le, en, vers, ...)
// - place: le lieu de l'événement avec son préfixe
// - notes: les notes associés à cet événement
#[component]
fn DateLieu(
    event: &'static str,
    feminine: &'static str,
    date: DateParts,
    age: Age,
    place_parts: PlaceParts,
    notes: Vec<String>,
) -> Element {
    let date1 = date.date1;
    rsx! {
        if !date1.date.is_empty() || !place_parts.place.is_empty() {
            div { class: "mb-1",
                span { class: "italic text-gray-400", "{event}{feminine} " }
                if !date1.date.is_empty() {
                    // Préfixe de la date
                    span { class: "italic text-gray-400", "{date1.prefix}" }
                    // Date (en rouge s'il y a eu une erreur)
                    span { class: if date.error { "error" }, " {date1.date}" }
                    // Deuxième date (avec son propre préfixe) pour une date composée (BET ou FROM)
                    if let Some(date2) = date.opt_date2 {
                        span { class: "italic text-gray-400", " {date2.prefix}" }
                        " {date2.date}"
                    }
                    span { class: "italic text-gray-600",
                        // Indication éventuelle de l'age
                        if !age.error && !age.age.is_empty() {
                            " {&age.age}"
                        }
                        // Virgule et espace si un lieu suit
                        if !place_parts.place.is_empty() {
                            ", "
                        }
                    }
                }
                if !place_parts.place.is_empty() {
                    // Préfixe du lieu
                    span { class: "italic text-gray-400", "{place_parts.prefix} " }
                    // Lieu (en rouge s'il y a eu une erreur)
                    span { class: if place_parts.error { "error" }, {place_parts.place} }
                }
            }
        }
        Notes { notes }
    }
}

// Composant pour afficher le nom complet d'un indivdu en surligant le prénom d'usage
// Le tuple commence par un booléen indiquant si la recherche du prénom usuel s'est mal passée
#[component]
fn NomComplet(parts: NameParts) -> Element {
    if parts.error {
        // Affiche le nom en rouge si le prénom d'usage n'a pu être isolé
        // A noter que générer une erreur dans ce cas ne marche pas en SSR car l'erreur n'est pas interceptée dans App
        // et le composant est juste effacé.
        return rsx! {
            span { class: "error", {parts.before_or_full} }
        };
    }
    rsx! {
        span { {parts.before_or_full} }
        if let Some((preferred, rest)) = parts.preferred_and_rest {
            // Il y a un prénom d'usage
            span { class: "underline", {preferred} }
            span { {rest} }
        }
        if let Some(nick) = parts.nick {
            // Surnom
            span {
                " ("
                span { class: "italic underline", {nick} }
                ")"
            }
        }
    }
}

// Affichage du nom complet du père ou de la mère
#[component]
fn Parent(parts: NameParts, parent: &'static str, xref: String) -> Element {
    rsx! {
        if !parts.before_or_full.is_empty() || parts.preferred_and_rest.is_some() {
            div { class: "mb-1",
                span { class: "italic text-gray-400", "{parent} : " }
                a {
                    class: "hover:underline hover:decoration-dotted rounded-lg",
                    href: "#{xref}",
                    NomComplet { parts }
                }
            }
        }
    }
}

#[component]
fn AdoptePar(feminine: &'static str, adopting_parents: Vec<(String, NameParts)>) -> Element {
    rsx! {
        if !adopting_parents.is_empty() {
            div { class: "mb-1",
                span { class: "italic text-gray-400", "Adopté{feminine} par : " }
                for (i , (xref , parts)) in adopting_parents.into_iter().enumerate() {
                    if i > 0 {
                        ", "
                    }
                    a {
                        class: "hover:underline hover:decoration-dotted rounded-lg",
                        href: "#{xref}",
                        NomComplet { parts }
                    }
                }
            }
        }
    }
}

#[component]
// Composant pour afficher un individu
pub fn Individu(xref: String) -> Element {
    // Occupe une case vide pour le cas d'un conjoint manquant dans la liste des familles.
    if xref.is_empty() {
        return rsx! {
            div { "???" }
        };
    }
    let data = use_context::<Store<GedcomData>>();
    let data = data.read();
    let mut father = NameParts::default();
    let mut father_xref = String::new();
    let mut mother = NameParts::default();
    let mut mother_xref = String::new();
    let mut birth = DateParts::default();
    let mut birth_place = PlaceParts::default();
    let mut birth_notes = Vec::new();
    let mut death = DateParts::default();
    let mut death_place = PlaceParts::default();
    let mut death_notes = Vec::new();
    let moved_xref = xref.clone();
    let individual = if let Some(individual) = data.find_individual(&moved_xref) {
        individual
    } else {
        write_errors!(errors, {
            errors.insert(
                format!("Individual xref {xref}"),
                anyhow!("Individu de la liste non trouvée"),
            );
        });
        return rsx! {};
    };
    let notes = concat_notes(&individual.notes);
    let full_name = nom_complet(&xref, &data);
    let (feminine, gender_icon) = match individual.sex.as_ref() {
        Some(Gender {
            value: GenderType::Male,
            ..
        }) => ("", Some(&*MALE_PNG)),
        Some(Gender {
            value: GenderType::Female,
            ..
        }) => ("e", Some(&*FEMALE_PNG)),
        _ => ("", None),
    };
    // Naissance de l'individu
    if let Some(details) = individual.birth() {
        if let Some(date) = details.date.as_ref() {
            if let Some(value) = date.value.as_ref() {
                if !value.is_empty() {
                    birth = traduire_date(value);
                    birth_notes = concat_notes(&details.notes);
                }
            }
        }
        if let Some(place) = details.place.as_ref() {
            if let Some(value) = place.value.as_ref() {
                if !value.is_empty() {
                    birth_place = interpreter_lieu(value);
                }
            }
        }
    }
    // Décès de l'individu
    if let Some(details) = individual.death() {
        if let Some(date) = details.date.as_ref() {
            if let Some(value) = date.value.as_ref() {
                if !value.is_empty() {
                    death = traduire_date(value);
                    death_notes = concat_notes(&details.notes);
                }
            }
        }
        if let Some(place) = details.place.as_ref() {
            if let Some(value) = place.value.as_ref() {
                if !value.is_empty() {
                    death_place = interpreter_lieu(value);
                }
            }
        }
    }
    // Parents de l'individu
    let families = individual.families();
    for link in families {
        let family_xref = &link.xref;
        if let Some(family) = data.find_family(family_xref) {
            if link.family_link_type == FamilyLinkType::Child {
                // Recherche du père
                if let Some(xref) = &family.individual1 {
                    father = nom_complet(xref, &data);
                    father_xref = xref.clone();
                }
                // Recherche de la mère
                if let Some(xref) = &family.individual2 {
                    mother = nom_complet(xref, &data);
                    mother_xref = xref.clone();
                }
            }
        } else {
            write_errors!(errors, {
                errors.insert(
                    format!("Family xref {family_xref}"),
                    anyhow!("Parents de {} non trouvés", full_name.before_or_full),
                );
            });
        }
    }
    // Age lors du décès
    let age = age(&birth, &death);

    let mut adopting_parents = Vec::new();
    for xref in Adoption::adopted(&xref) {
        let parent = nom_complet(&xref, &data);
        adopting_parents.push((xref.clone(), parent));
    }

    rsx! {
        div { id: xref,
            div { class: "mb-2 py-1.5 px-2 border-2 rounded-lg shadow-lg border-blue-200",
                if let Some(gender_icon) = gender_icon {
                    img { class: "inline mr-2 align-baseline", src: gender_icon }
                }
                span { class: "font-bold text-lg text-green-700",
                    NomComplet { parts: full_name }
                }
            }
            Notes { notes }
            DateLieu {
                event: "Né",
                feminine,
                date: birth,
                age: Age::default(),
                place_parts: birth_place,
                notes: birth_notes,
            }
            DateLieu {
                event: "Décédé",
                feminine,
                date: death,
                age,
                place_parts: death_place,
                notes: death_notes,
            }
            Parent { parts: father, parent: "Père", xref: father_xref }
            Parent { parts: mother, parent: "Mère", xref: mother_xref }
            AdoptePar { feminine, adopting_parents }
        }
    }
}
