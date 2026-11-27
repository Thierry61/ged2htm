# Génération d'un fichier HTML à partir d'un fichier GEDCOM.

C'est un programme permettant de générer le livre d'ascendance d'un
individu dont le nom complet est passé en paramètre. Le résultat
s'inspire du livre d'ascendance généré par Généanet à la différence
que les ascendants sont regroupés par couple. Le but est de supprimer
les redondances des lignes de vie des époux.

De manière plus générale le programme offre les fonctions suivantes :
- livre d'ascendance (option `book -a -s <start>`)
- livre de descendance (option `book -d -s <start>`)
- liste de tous les individus (option `all -i`)
- liste de toutes les familles (option `all -f`)

ATTENTION, POUR LE MOMENT SEULE LA LISTE DE TOUS LES INDIVIDUS A ETE IMPLEMENTEE

## Mode d'emploi

Il faut d'abord exporter le fichier GED depuis [Geneanet](https://www.geneanet.org/) en suivant
le menu : "Ma généalogie" puis "Mise à jour GEDCOM" puis "Exports et sauvegardes". Il faut ensuite
cocher "Mon arbre en ligne" et sélectionner le format GEDCOM (*.ged) et enfin cliquer sur le bouton
"Télécharger".

Le fichier exporté doit être ensuite passé en paramètre au programme selon la syntaxe :

```
Usage: ged2htm.exe --input <INPUT> book [OPTIONS] --start <START> <--ascending|--descending>

Options:
  -s, --start <START>              Nom de départ exact (ex: "\"Marie\" Eugénie Briançon")
  -g, --generations <GENERATIONS>  Nombre de générations à extraire (1 = individu seul, 2 = avec parents ou enfants, etc.) [default: 1]
  -a, --ascending                  Génère le livre d'ascendance
  -d, --descending                 Génère le livre de descendance
  -h, --help                       Print help
```

## Exemple d'utilisation en développement

`dx build && cargo run -- -i input.ged book -a -s "\"Marie\" Eugénie Briançon"`

## Génération du binaire en mode release

`dx build --release && cargo build --release`

Le binaire généré est ensuite invoquable avec `target\release\ged2htm.exe -i input.ged book -a -s "\"Marie\" Eugénie Briançon"`

## Technologie

C'est une application Rust basée sur le composant Dioxus et créée
en sélectionnant la plateforme Web. Mais elle utilise le module SSR
pour générer le fichier HTML, le serveur Web n'est donc pas utilisé,
ni en développement, ni au runtime.

Ceci apporte quelques contraintes :
- le hot reloading n'est pas fonctionnel
- les assets ne sont pas gérés par le pipeline Dioxus
- le composant ErrorBoundary est défaillant
- il n'y a aucune interactivité

Le parseur des fichiers GED utilise le crate ged_io.
TODO: celui-ci a été archivé il faudra donc le migrer vers oxidged
