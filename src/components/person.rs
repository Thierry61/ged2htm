use dioxus::prelude::*;
use ged_io::types::individual::family_link::FamilyLinkType;
use ged_io::types::individual::gender::{Gender, GenderType};
use ged_io::types::GedcomData;

use crate::date::{traduire_date, DateParts};
use crate::full_name::{nom_complet, NameParts};
use crate::place::{interpreter_lieu, PlaceParts};
use crate::{FEMALE_PNG, MALE_PNG};

// Composant pour afficher la date et le lieu d'un évènement dans un grille 2x2
// - event: "Né" ou "Décédé"
// - feminine: "" ou "e"
// - date: les éléments de la date avec leurs préfixes (le, en, vers, ...)
// - place: le lieu de l'événement avec son préfixe
// Nota: Les tuples date et place commencent par un booléen indiquant s'ils sont entachés d'une erreur
// auquel cas ils sont affiché en rouge
// La grille générée est la suivante :
// ------------------------------------------
// | event + feminine | date(s) préfixée(s) |
// -------------------| --------------------|
// |                  | lieu préfixé        |
// ------------------------------------------
#[component]
fn DateLieu(
    event: &'static str,
    feminine: &'static str,
    date: DateParts,
    place_parts: PlaceParts,
) -> Element {
    let date1 = date.date1;
    rsx! {
        if !date1.date.is_empty() || !place_parts.place.is_empty() {
            div { class: "grid grid-flow-col grid-rows-[auto_auto] justify-start gap-x-1",
                div { class: "row-span-2 italic text-gray-400", "{event}{feminine}" }
                div {
                    if !date1.date.is_empty() {
                        // Préfixe de la date
                        span { class: "italic text-gray-400", "{date1.prefix} " }
                        // Date (en rouge s'il y a eu une erreur)
                        span { class: if date.error { "error" }, {date1.date} }
                        // Deuxième date (avec son propre préfixe) pour une date composée (BET ou FROM)
                        if let Some(date2) = date.opt_date2 {
                            span { class: "italic text-gray-400", " {date2.prefix} " }
                            {date2.date}
                        }
                    }
                }
                div {
                    if !place_parts.place.is_empty() {
                        // Préfixe du lieu
                        span { class: "italic text-gray-400", "{place_parts.prefix} " }
                        // Lieu (en rouge s'il y a eu une erreur)
                        span { class: if place_parts.error { "error" }, {place_parts.place} }
                    }
                }
            }
        }
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
    }
}

#[component]
// Composant pour afficher un individu
pub fn Individu(xref: String) -> Element {
    let data = use_context::<Store<GedcomData>>();
    let data = data.read();
    let mut father = String::new();
    let mut father_xref = String::new();
    let mut mother = String::new();
    let mut mother_xref = String::new();
    let mut birth = DateParts::default();
    let mut birth_place = PlaceParts::default();
    let mut death = DateParts::default();
    let mut death_place = PlaceParts::default();
    let moved_xref = xref.clone();
    let individual = data
        .find_individual(&moved_xref)
        .context(format!("Xref {} d'un individu non trouvé", moved_xref))?;
    let full_name = individual.full_name().context(format!(
        "Nom complet manquant pour l'individu : {:?}",
        individual
    ))?;
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
        let xref = &link.xref;
        let family = data
            .find_family(xref)
            .context(format!("Xref {} d'une famille non trouvé", xref))?;
        if link.family_link_type == FamilyLinkType::Child {
            // Recherche du père
            if let Some(xref) = &family.individual1 {
                let individual = data
                    .find_individual(xref)
                    .context(format!("Xref {} d'un individu non trouvé", xref))?;
                father = individual.full_name().context(format!(
                    "Nom complet manquant pour l'individu : {:?}",
                    individual
                ))?;
                father_xref = xref.clone();
            }
            // Recherche de la mère
            if let Some(xref) = &family.individual2 {
                let individual = data
                    .find_individual(xref)
                    .context(format!("Xref {} d'un individu non trouvé", xref))?;
                mother = individual.full_name().context(format!(
                    "Nom complet manquant pour l'individu : {:?}",
                    individual
                ))?;
                mother_xref = xref.clone();
            }
        }
    }
    // Décomposition des noms complets en séparant le prénom usuel
    let full_name = nom_complet(&full_name);
    let father = nom_complet(&father);
    let mother = nom_complet(&mother);
    rsx! {
        div { id: xref,
            div { class: "mb-1",
                if let Some(gender_icon) = gender_icon {
                    img { class: "inline mr-2 align-baseline", src: gender_icon }
                }
                span { class: "font-bold text-lg text-green-700",
                    NomComplet { parts: full_name }
                }
            }
            DateLieu {
                event: "Né",
                feminine,
                date: birth,
                place_parts: birth_place,
            }
            DateLieu {
                event: "Décédé",
                feminine,
                date: death,
                place_parts: death_place,
            }
            div {
                span { class: "italic text-gray-400", "Père : " }
                a {
                    class: "hover:underline hover:decoration-dotted rounded-lg",
                    href: "#{father_xref}",
                    NomComplet { parts: father }
                }
            }
            div {
                span { class: "italic text-gray-400", "Mère : " }
                a {
                    class: "hover:underline hover:decoration-dotted rounded-lg",
                    href: "#{mother_xref}",
                    NomComplet { parts: mother }
                }
            }
        }
    }
}
