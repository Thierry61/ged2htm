use dioxus::{prelude::*, CapturedError};

use super::person::{Individu, IndividuProps};

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
                li { class: "my-5 py-3 px-2 w-90 border-2 rounded-lg shadow-lg border-blue-200 break-inside-avoid",
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
