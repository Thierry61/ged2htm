# Génération d'un fichier HTML à partir d'un fichier GEDCOM.

C'est un programme permettant de générer l'arbre d'ascendance d'un
individu dont le nom complet est passé en paramètre. Le résultat
s'inspire de l'arbre d'ascendance généré par Généanet à la différence
que les ascendants sont regroupés par couple. Le but est de supprimer
les redondances des lignes de vie des époux.

ATTENTION, POUR LE MOMENT LE PROGRAMME NE FAIT PAS CELA ET GENERE SIMPLEMENT LA LISTE DE TOUS LES INDIVIDUS

## Mode d'emploi

Il faut d'abord exporter le fichier GED depuis [Geneanet](https://www.geneanet.org/) en suivant
le menu : "Ma généalogie" puis "Mise à jour GEDCOM" puis "Exports et sauvegardes". Il faut ensuite
cocher "Mon arbre en ligne" et sélectionner le format GEDCOM (*.ged) et enfin cliquer sur le bouton
"Télécharger".

Le fichier exporté doit être ensuite passé en paramètre au progamme selon la syntaxe :

```
Usage: ged2htm.exe [OPTIONS] --input <INPUT> --start-name <START_NAME>

Options:
  -i, --input <INPUT>              Chemin vers le fichier GED d'entrée
  -o, --output <OUTPUT>            Chemin du fichier HTML de sortie [default: output.html]
  -s, --start-name <START_NAME>    Nom de départ exact (ex: "\"Marie\" Eugénie Briançon")
  -g, --generations <GENERATIONS>  Nombre de générations à extraire (1 = individu seul, 2 = parents, etc.) [default: 1]
  -h, --help                       Print help
  -V, --version                    Print version
  ```

## Exemple d'utilisation en développement

`dx build && cargo run -- -i input.ged -s "\"Marie\" Eugénie Briançon"`
(à noter que le hot reloading ne marche pas)

## Génération du binaire en mode release

`dx build --release && cargo build --release`

Le binaire généré est invoquable avec `target\release\ged2htm.exe -i input.ged -s "\"Marie\" Eugénie Briançon"`

## Technologie

C'est une application Rust basée sur le composant Dioxus et créée
en sélectionnant la plateforme Web. Mais elle utilise le module SSR
pour générer le fichier HTML, le serveur Web n'est donc pas utilisé,
ni en développement, ni au runtime.

Ceci apporte quelques contraintes :
- le hot reloading n'est pas fonctionnel
- les assets ne sont pas gérés par le pipeline Dioxus
- le composant ErrorBoundary est défaillant
