use dioxus::prelude::*;
use ged_io::types::GedcomData;

use crate::components::person::Individu;

#[component]
// Composant pour afficher une famille
pub fn Famille(xref: String) -> Element {
    let data = use_context::<Store<GedcomData>>();
    let data = data.read();
    let mut husband_xref = String::new();
    let mut wife_xref = String::new();
    let moved_xref = xref.clone();
    let family = data
        .find_family(&moved_xref)
        .context(format!("Xref {} d'une famille non trouvé", moved_xref))?;

    // Recherche du mari et de la femme
    if let Some(xref) = &family.individual1 {
        husband_xref = xref.clone();
    }
    if let Some(xref) = &family.individual2 {
        wife_xref = xref.clone();
    }

    rsx! {
        div { id: xref, class: "grid grid-cols-3 gap-2",
            Individu { xref: husband_xref }
            div { "Union et enfant(s)" }
            Individu { xref: wife_xref }
        }
    }
}
