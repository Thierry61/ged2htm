use dioxus::{prelude::*, CapturedError};

use super::family::{Famille, FamilleProps};
use super::person::{Individu, IndividuProps};
use crate::adopt::Adoption;

// Liste d'individus
#[component]
pub fn ListeIndividus(xrefs: Vec<String>) -> Element {
    rsx! {
        ul {
            // J'aurais voulu utiliser le composant ErrorBoundary pour intercepter les éventuelles erreurs émises
            // par le composant Individu avec quelque chode du genre :
            // for xref in xrefs {
            //     ErrorBoundary {
            //         handle_error: |errors: ErrorContext| {
            //             rsx! {
            //                 div { class: "error", ... }
            //             }
            //         },
            //         Individu { xref }
            //     }
            // }
            // Mais ErrorBoundary est défaillant en SSR : Les éléments Individu erronés sont certes effacés,
            // mais le rsx de la fonction handle_error n'est pas affiché.
            // Le problème est contourné en appelant le composant directement en Rust
            // et en effectuant un match sur la valeur de retour.
            for xref in xrefs {
                li { class: "mt-5 break-inside-avoid",
                    match Individu(IndividuProps {
                        xref: xref.clone(),
                    }) {
                        Ok(node) => rsx! {
                            {node}
                        },
                        Err(RenderError::Error(CapturedError(error))) => rsx! {
                            div { class: "error",
                                div { class: "underline", "Erreur:" }
                                for cause in error.chain() {
                                    div { class: "ml-5", "{cause}" }
                                }
                            }
                        },
                        Err(RenderError::Suspended(_)) => {
                            rsx! {}
                        }
                    }
                }
            }
        }
    }
}

// Liste de familles
#[component]
pub fn ListeFamilles(xrefs: Vec<String>) -> Element {
    rsx! {
        ul {
            for xref in xrefs.iter().filter(|xref| !Adoption::is_family_virtual(xref)) {
                li { class: "mt-5 break-inside-avoid",
                    match Famille(FamilleProps { xref: xref.clone() }) {
                        Ok(node) => rsx! {
                            {node}
                        },
                        Err(RenderError::Error(CapturedError(error))) => rsx! {
                            div { class: "error",
                                div { class: "underline", "Erreur:" }
                                for cause in error.chain() {
                                    div { class: "ml-5", "{cause}" }
                                }
                            }
                        },
                        Err(RenderError::Suspended(_)) => {
                            rsx! {}
                        }
                    }
                }
            }
        }
    }
}
