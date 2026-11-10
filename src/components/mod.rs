// Fichier nécessaire pour exporter les modules de ce répertoire

// Modules exportés localement pour les autres composants
// Ils sont utilisables avec le préfixe super::
mod person;

// Modules exportés pour la totalité du crate (et importés dans main.rs)
// Ils sont utilisables avec le préfixe crate::
pub mod person_list;
