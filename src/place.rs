use anyhow::anyhow;
use phf::{phf_map, phf_set};
use crate::ERRORS;

// Déclaration du Set statique et immuable des régions françaises
// selon l'ancienne nomenclature d'avant la réforme de 2016 telle qu'utilisée par Geneanet
static ANCIENNES_REGIONS_FRANCE: phf::Set<&'static str> = phf_set! {
    // 22 Anciennes régions métropolitaines
    "Alsace",
    "Aquitaine",
    "Auvergne",
    "Basse-Normandie",
    "Bourgogne",
    "Bretagne",
    "Centre", // Souvent appelé "Centre-Val de Loire" aujourd'hui, mais historiquement "Centre"
    "Champagne-Ardenne",
    "Corse",
    "Franche-Comté",
    "Haute-Normandie",
    "Île-de-France",
    "Languedoc-Roussillon",
    "Limousin",
    "Lorraine",
    "Midi-Pyrénées",
    "Nord-Pas-de-Calais",
    "Pays de la Loire",
    "Picardie",
    "Poitou-Charentes",
    "Provence-Alpes-Côte d'Azur",
    "Rhône-Alpes",

    // 5 Régions / Départements d'outre-mer
    "Guadeloupe",
    "Guyane",
    "La Réunion",
    "Martinique",
    "Mayotte",
};

// Map statique immuable : Nom Département -> Ancienne Région (Geneanet)
static DEPARTEMENT_TO_ANCIENNE_REGION: phf::Map<&'static str, &'static str> = phf_map! {
    // Métropole ordonnée par numéro de département
    "Ain" => "Rhône-Alpes", // 01
    "Aisne" => "Picardie", // 02
    "Allier" => "Auvergne", // 03
    "Alpes-de-Haute-Provence" => "Provence-Alpes-Côte d'Azur", // 04
    "Hautes-Alpes" => "Provence-Alpes-Côte d'Azur", // 05
    "Alpes-Maritimes" => "Provence-Alpes-Côte d'Azur", // 06
    "Ardèche" => "Rhône-Alpes", // 07
    "Ardennes" => "Champagne-Ardenne", // 08
    "Ariège" => "Midi-Pyrénées", // 09
    "Aube" => "Champagne-Ardenne", // 10
    "Aude" => "Languedoc-Roussillon", // 11
    "Aveyron" => "Midi-Pyrénées", // 12
    "Bouches-du-Rhône" => "Provence-Alpes-Côte d'Azur", // 13
    "Calvados" => "Basse-Normandie", // 14
    "Cantal" => "Auvergne", // 15
    "Charente" => "Poitou-Charentes", // 16
    "Charente-Maritime" => "Poitou-Charentes", // 17
    "Cher" => "Centre", // 18
    "Corrèze" => "Limousin", // 19
    "Côte-d'Or" => "Bourgogne", // 21
    "Côtes-d'Armor" => "Bretagne", // 22
    "Creuse" => "Limousin", // 23
    "Dordogne" => "Aquitaine", // 24
    "Doubs" => "Franche-Comté", // 25
    "Drôme" => "Rhône-Alpes", // 26
    "Eure" => "Haute-Normandie", // 27
    "Eure-et-Loir" => "Centre", // 28
    "Finistère" => "Bretagne", // 29
    "Corse-du-Sud" => "Corse", // 2A
    "Haute-Corse" => "Corse", // 2B
    "Gard" => "Languedoc-Roussillon", // 30
    "Haute-Garonne" => "Midi-Pyrénées", // 31
    "Gers" => "Midi-Pyrénées", // 32
    "Gironde" => "Aquitaine", // 33
    "Hérault" => "Languedoc-Roussillon", // 34
    "Ille-et-Vilaine" => "Bretagne", // 35
    "Indre" => "Centre", // 36
    "Indre-et-Loire" => "Centre", // 37
    "Isère" => "Rhône-Alpes", // 38
    "Jura" => "Franche-Comté", // 39
    "Landes" => "Aquitaine", // 40
    "Loir-et-Cher" => "Centre", // 41
    "Loire" => "Rhône-Alpes", // 42
    "Haute-Loire" => "Auvergne", // 43
    "Loire-Atlantique" => "Pays de la Loire", // 44
    "Loiret" => "Centre", // 45
    "Lot" => "Midi-Pyrénées", // 46
    "Lot-et-Garonne" => "Aquitaine", // 47
    "Lozère" => "Languedoc-Roussillon", // 48
    "Maine-et-Loire" => "Pays de la Loire", // 49
    "Manche" => "Basse-Normandie", // 50
    "Marne" => "Champagne-Ardenne", // 51
    "Haute-Marne" => "Champagne-Ardenne", // 52
    "Mayenne" => "Pays de la Loire", // 53
    "Meurthe-et-Moselle" => "Lorraine", // 54
    "Meuse" => "Lorraine", // 55
    "Morbihan" => "Bretagne", // 56
    "Moselle" => "Lorraine", // 57
    "Nièvre" => "Bourgogne", // 58
    "Nord" => "Nord-Pas-de-Calais", // 59
    "Oise" => "Picardie", // 60
    "Orne" => "Basse-Normandie", // 61
    "Pas-de-Calais" => "Nord-Pas-de-Calais", // 62
    "Puy-de-Dôme" => "Auvergne", // 63
    "Pyrénées-Atlantiques" => "Aquitaine", // 64
    "Hautes-Pyrénées" => "Midi-Pyrénées", // 65
    "Pyrénées-Orientales" => "Languedoc-Roussillon", // 66
    "Bas-Rhin" => "Alsace", // 67
    "Haut-Rhin" => "Alsace", // 68
    "Rhône" => "Rhône-Alpes", // 69 (Inclut la Métropole de Lyon)
    "Haute-Saône" => "Franche-Comté", // 70
    "Saône-et-Loire" => "Bourgogne", // 71
    "Sarthe" => "Pays de la Loire", // 72
    "Savoie" => "Rhône-Alpes", // 73
    "Haute-Savoie" => "Rhône-Alpes", // 74
    "Paris" => "Île-de-France", // 75
    "Seine-Maritime" => "Haute-Normandie", // 76
    "Seine-et-Marne" => "Île-de-France", // 77
    "Yvelines" => "Île-de-France", // 78
    "Deux-Sèvres" => "Poitou-Charentes", // 79
    "Somme" => "Picardie", // 80
    "Tarn" => "Midi-Pyrénées", // 81
    "Tarn-et-Garonne" => "Midi-Pyrénées", // 82
    "Var" => "Provence-Alpes-Côte d'Azur", // 83
    "Vaucluse" => "Provence-Alpes-Côte d'Azur", // 84
    "Vendée" => "Pays de la Loire", // 85
    "Vienne" => "Poitou-Charentes", // 86
    "Haute-Vienne" => "Limousin", // 87
    "Vosges" => "Lorraine", // 88
    "Yonne" => "Bourgogne", // 89
    "Territoire de Belfort" => "Franche-Comté", // 90
    "Essonne" => "Île-de-France", // 91
    "Hauts-de-Seine" => "Île-de-France", // 92
    "Seine-Saint-Denis" => "Île-de-France", // 93
    "Val-de-Marne" => "Île-de-France", // 94
    "Val-d'Oise" => "Île-de-France", // 95

    // Départements d'outre-mer
    "Guadeloupe" => "Guadeloupe", // 971
    "Martinique" => "Martinique", // 972
    "Guyane" => "Guyane", // 973
    "La Réunion" => "La Réunion", // 974
    "Mayotte" => "Mayotte", // 976
};

// Prépositions de lieu devant les noms de pays :
// - en : devant les pays féminins (se terminent par -e) ou masculins commençant par une voyelle
// - au : devant les pays masculins commençant par une consonne
// - aux : devant les pays au pluriel
// - à : devant les îles isolées qui n'ont pas d'article (comme Cuba, Madagascar ou Haïti)
// - à la : devant les îles féminines avec article (comme Barbade, Dominique, Grenade ou Jamaïque)
// Note pour Haïti, Chypre, Bahreïn et Djibouti : Comme ils commencent par une voyelle ou un H,
// l'usage moderne hésite parfois avec "en" (en Haïti, en Chypre), mais la recommandation officielle
// reste la préposition « à » (à Haïti, à Chypre).
static PREFIXE_PAYS: phf::Map<&'static str, &'static str> = phf_map! {
    "Afghanistan" => "en",
    "Afrique du Sud" => "en",
    "Albanie" => "en",
    "Algérie" => "en",
    "Allemagne" => "en",
    "Andorre" => "en",
    "Angola" => "en",
    "Antigua-et-Barbuda" => "à",
    "Arabie saoudite" => "en",
    "Argentine" => "en",
    "Arménie" => "en",
    "Australie" => "en",
    "Autriche" => "en",
    "Azerbaïdjan" => "en",
    "Bahamas" => "aux",
    "Bahreïn" => "à",
    "Bangladesh" => "au",
    "Barbade" => "à la",
    "Belgique" => "en",
    "Belize" => "au",
    "Bénin" => "au",
    "Bhoutan" => "au",
    "Biélorussie" => "en",
    "Birmanie" => "en",
    "Bolivie" => "en",
    "Bosnie-Herzégovine" => "en",
    "Botswana" => "au",
    "Brésil" => "au",
    "Brunei" => "au",
    "Bulgarie" => "en",
    "Burkina Faso" => "au",
    "Burundi" => "au",
    "Cabo Verde" => "au",
    "Cambodge" => "au",
    "Cameroun" => "au",
    "Canada" => "au",
    "Chili" => "au",
    "Chine" => "en",
    "Chypre" => "à",
    "Colombie" => "en",
    "Comores" => "aux",
    "Congo" => "au",
    "Corée du Nord" => "en",
    "Corée du Sud" => "en",
    "Costa Rica" => "au",
    "Côte d'Ivoire" => "en",
    "Croatie" => "en",
    "Cuba" => "à",
    "Danemark" => "au",
    "Djibouti" => "à",
    "Dominique" => "à la",
    "Égypte" => "en",
    "Émirats arabes unis" => "aux",
    "Équateur" => "en",
    "Érythrée" => "en",
    "Espagne" => "en",
    "Estonie" => "en",
    "Eswatini" => "en",
    "États-Unis" => "aux",
    "Éthiopie" => "en",
    "Fidji" => "aux",
    "Finlande" => "en",
    "France" => "en",
    "Gabon" => "au",
    "Gambie" => "en",
    "Géorgie" => "en",
    "Ghana" => "au",
    "Grèce" => "en",
    "Grenade" => "à la",
    "Guatemala" => "au",
    "Guinée" => "en",
    "Guinée équatoriale" => "en",
    "Guinée-Bissau" => "en",
    "Guyana" => "au",
    "Haïti" => "à",
    "Honduras" => "au",
    "Hongrie" => "en",
    "Inde" => "en",
    "Indonésie" => "en",
    "Irak" => "en",
    "Iran" => "en",
    "Irlande" => "en",
    "Islande" => "en",
    "Israël" => "en",
    "Italie" => "en",
    "Jamaïque" => "à la",
    "Japon" => "au",
    "Jordanie" => "en",
    "Kazakhstan" => "au",
    "Kenya" => "au",
    "Kirghizistan" => "au",
    "Kiribati" => "aux",
    "Koweït" => "au",
    "Laos" => "au",
    "Lesotho" => "au",
    "Lettonie" => "en",
    "Liban" => "au",
    "Libéria" => "au",
    "Libye" => "en",
    "Liechtenstein" => "au",
    "Lituanie" => "en",
    "Luxembourg" => "au",
    "Macédoine du Nord" => "en",
    "Madagascar" => "à",
    "Malaisie" => "en",
    "Malawi" => "au",
    "Maldives" => "aux",
    "Mali" => "au",
    "Malte" => "à",
    "Maroc" => "au",
    "Maurice" => "à",
    "Mauritanie" => "en",
    "Mexique" => "au",
    "Micronésie" => "en",
    "Moldavie" => "en",
    "Monaco" => "à",
    "Mongolie" => "en",
    "Monténégro" => "au",
    "Mozambique" => "au",
    "Namibie" => "en",
    "Nauru" => "à",
    "Népal" => "au",
    "Nicaragua" => "au",
    "Niger" => "au",
    "Nigeria" => "au",
    "Norvège" => "en",
    "Nouvelle-Zélande" => "en",
    "Oman" => "en",
    "Ouganda" => "en",
    "Ouzbékistan" => "en",
    "Pakistan" => "au",
    "Palaos" => "aux",
    "Palestine" => "en",
    "Panama" => "au",
    "Papouasie-Nouvelle-Guinée" => "en",
    "Paraguay" => "au",
    "Pays-Bas" => "aux",
    "Pérou" => "au",
    "Philippines" => "aux",
    "Pologne" => "en",
    "Portugal" => "au",
    "Qatar" => "au",
    "République centrafricaine" => "en",
    "République démocratique du Congo" => "en",
    "République dominicaine" => "en",
    "République tchèque" => "en",
    "Roumanie" => "en",
    "Royaume-Uni" => "au",
    "Russie" => "en",
    "Rwanda" => "au",
    "Saint-Christophe-et-Niévès" => "à",
    "Sainte-Lucie" => "à",
    "Saint-Marin" => "à",
    "Saint-Vincent-et-les-Grenadines" => "à",
    "Salomon" => "aux",
    "Salvador" => "au", // Le nom officiel est "Le Salvador", mais Généanet utilise "Salvador" ce qui simplifie la gestion du préfixe
    "Samoa" => "aux",
    "Sao Tomé-et-Principe" => "à",
    "Sénégal" => "au",
    "Serbie" => "en",
    "Seychelles" => "aux",
    "Sierra Leone" => "en",
    "Singapour" => "à",
    "Slovaquie" => "en",
    "Slovénie" => "en",
    "Somalie" => "en",
    "Soudan" => "au",
    "Soudan du Sud" => "au",
    "Sri Lanka" => "au",
    "Suède" => "en",
    "Suisse" => "en",
    "Suriname" => "au",
    "Syrie" => "en",
    "Tadjikistan" => "au",
    "Taïwan" => "à",
    "Tanzanie" => "en",
    "Tchad" => "au",
    "Thaïlande" => "en",
    "Timor oriental" => "au",
    "Togo" => "au",
    "Tonga" => "aux",
    "Trinité-et-Tobago" => "à",
    "Tunisie" => "en",
    "Turkménistan" => "au",
    "Turquie" => "en",
    "Tuvalu" => "aux",
    "Ukraine" => "en",
    "Uruguay" => "au",
    "Vanuatu" => "au",
    "Vatican" => "au",
    "Venezuela" => "au",
    "Vietnam" => "au",
    "Yémen" => "au",
    "Zambie" => "en",
    "Zimbabwe" => "au",
};

// Préfixe "à", "en", "dans le", "dans la", "dans les" ou "dans l'" pour les départements et régions
// La donnéee est un tuple répétant le lieu ce qui permet de ne pas le modifier le cas "dans l'" et
// donc de rester dans le domaine &str
// Nota : je ne respecte pas la norme académique pour Loir-et-Cher, Lot-et-Garonne et Tarn-et-Garonne
// car le préfixe "en" me semble bizarre à l'oreille et j'ai mis "dans le" qui sonne mieux
// (sans doute parce que le premier mot est un nom masculin commençant par une consomme, la tentation est donc
// forte de dire « dans le » exactement comme on dit « dans le Tarn » ou « dans le Lot »).
// Par contre pour les autres départements coordonnés par "-et-" j'ai gardé le "en" normatif car le premier mot
// est féminin et la construction avec en" est répandue pour eux et donc ne surprend pas.
// (cf. https://www.dictionnaire-academie.fr/article/QDL058)
static PREFIXE_DEPARTEMENTS_REGIONS: phf::Map<&'static str, (&'static str, &'static str)> = phf_map! {
    // Départements de métropole
    "Ain" => ("dans", "l'Ain"),                                     // 01
    "Aisne" => ("dans", "l'Aisne"),                                 // 02
    "Allier" => ("dans", "l'Allier"),                               // 03
    "Alpes-de-Haute-Provence" => ("dans les", "Alpes-de-Haute-Provence"), // 04
    "Hautes-Alpes" => ("dans les", "Hautes-Alpes"),                 // 05
    "Alpes-Maritimes" => ("dans les", "Alpes-Maritimes"),           // 06
    "Ardèche" => ("en", "Ardèche"),                                 // 07
    "Ardennes" => ("dans les", "Ardennes"),                         // 08
    "Ariège" => ("en", "Ariège"),                                   // 09
    "Aube" => ("dans", "l'Aube"),                                   // 10
    "Aude" => ("dans", "l'Aude"),                                   // 11
    "Aveyron" => ("en", "Aveyron"),                                 // 12
    "Bouches-du-Rhône" => ("dans les", "Bouches-du-Rhône"),         // 13
    "Calvados" => ("dans le", "Calvados"),                          // 14
    "Cantal" => ("dans le", "Cantal"),                              // 15
    "Charente" => ("en", "Charente"),                               // 16
    "Charente-Maritime" => ("en", "Charente-Maritime"),             // 17
    "Cher" => ("dans le", "Cher"),                                  // 18
    "Corrèze" => ("en", "Corrèze"),                                 // 19
    "Corse-du-Sud" => ("en", "Corse-du-Sud"),                       // 2A
    "Haute-Corse" => ("en", "Haute-Corse"),                         // 2B
    "Côte-d'Or" => ("en", "Côte-d'Or"),                             // 21
    "Côtes-d'Armor" => ("dans les", "Côtes-d'Armor"),               // 22
    "Creuse" => ("en", "Creuse"),                                   // 23
    "Dordogne" => ("en", "Dordogne"),                               // 24
    "Doubs" => ("dans le", "Doubs"),                                // 25
    "Drôme" => ("dans la", "Drôme"),                                // 26
    "Eure" => ("dans", "l'Eure"),                                   // 27
    "Eure-et-Loir" => ("en", "Eure-et-Loir"),                       // 28
    "Finistère" => ("dans le", "Finistère"),                        // 29
    "Gard" => ("dans le", "Gard"),                                  // 30
    "Haute-Garonne" => ("en", "Haute-Garonne"),                     // 31
    "Gers" => ("dans le", "Gers"),                                  // 32
    "Gironde" => ("en", "Gironde"),                                 // 33
    "Hérault" => ("dans", "l'Hérault"),                             // 34
    "Ille-et-Vilaine" => ("en", "Ille-et-Vilaine"),                 // 35
    "Indre" => ("en", "Indre"),                                     // 36
    "Indre-et-Loire" => ("en", "Indre-et-Loire"),                   // 37
    "Isère" => ("en", "Isère"),                                     // 38
    "Jura" => ("dans le", "Jura"),                                  // 39
    "Landes" => ("dans les", "Landes"),                             // 40
    "Loir-et-Cher" => ("dans le", "Loir-et-Cher"),                  // 41
    "Loire" => ("dans la", "Loire"),                                // 42
    "Haute-Loire" => ("en", "Haute-Loire"),                         // 43
    "Loire-Atlantique" => ("en", "Loire-Atlantique"),               // 44
    "Loiret" => ("dans le", "Loiret"),                              // 45
    "Lot" => ("dans le", "Lot"),                                    // 46
    "Lot-et-Garonne" => ("dans le", "Lot-et-Garonne"),              // 47
    "Lozère" => ("en", "Lozère"),                                   // 48
    "Maine-et-Loire" => ("en", "Maine-et-Loire"),                   // 49
    "Manche" => ("dans la", "Manche"),                              // 50
    "Marne" => ("dans la", "Marne"),                                // 51
    "Haute-Marne" => ("en", "Haute-Marne"),                         // 52
    "Mayenne" => ("en", "Mayenne"),                                 // 53
    "Meurthe-et-Moselle" => ("en", "Meurthe-et-Moselle"),           // 54
    "Meuse" => ("dans la", "Meuse"),                                // 55
    "Morbihan" => ("dans le", "Morbihan"),                          // 56
    "Moselle" => ("en", "Moselle"),                                 // 57
    "Nièvre" => ("dans la", "Nièvre"),                              // 58
    "Nord" => ("dans le", "Nord"),                                  // 59
    "Oise" => ("dans", "l'Oise"),                                   // 60
    "Orne" => ("dans", "l'Orne"),                                   // 61
    "Pas-de-Calais" => ("dans le", "Pas-de-Calais"),                // 62
    "Puy-de-Dôme" => ("dans le", "Puy-de-Dôme"),                    // 63
    "Pyrénées-Atlantiques" => ("dans les", "Pyrénées-Atlantiques"), // 64
    "Hautes-Pyrénées" => ("dans les", "Hautes-Pyrénées"),           // 65
    "Pyrénées-Orientales" => ("dans les", "Pyrénées-Orientales"),   // 66
    "Bas-Rhin" => ("dans le", "Bas-Rhin"),                          // 67
    "Haut-Rhin" => ("dans le", "Haut-Rhin"),                        // 68
    "Rhône" => ("dans le", "Rhône"),                                // 69
    "Haute-Saône" => ("en", "Haute-Saône"),                         // 70
    "Saône-et-Loire" => ("en", "Saône-et-Loire"),                   // 71
    "Sarthe" => ("dans la", "Sarthe"),                              // 72
    "Savoie" => ("en", "Savoie"),                                   // 73
    "Haute-Savoie" => ("en", "Haute-Savoie"),                       // 74
    "Paris" => ("à", "Paris"),                                      // 75
    "Seine-Maritime" => ("en", "Seine-Maritime"),                   // 76
    "Seine-et-Marne" => ("en", "Seine-et-Marne"),                   // 77
    "Yvelines" => ("dans les", "Yvelines"),                         // 78
    "Deux-Sèvres" => ("dans les", "Deux-Sèvres"),                   // 79
    "Somme" => ("dans la", "Somme"),                                // 80
    "Tarn" => ("dans le", "Tarn"),                                  // 81
    "Tarn-et-Garonne" => ("dans le", "Tarn-et-Garonne"),            // 82
    "Var" => ("dans le", "Var"),                                    // 83
    "Vaucluse" => ("dans le", "Vaucluse"),                          // 84
    "Vendée" => ("en", "Vendée"),                                   // 85
    "Vienne" => ("dans la", "Vienne"),                              // 86
    "Haute-Vienne" => ("en", "Haute-Vienne"),                       // 87
    "Vosges" => ("dans les", "Vosges"),                             // 88
    "Yonne" => ("dans", "l'Yonne"),                                 // 89
    "Territoire de Belfort" => ("dans le", "Territoire de Belfort"),// 90
    "Essonne" => ("dans", "l'Essonne"),                             // 91
    "Hauts-de-Seine" => ("dans les", "Hauts-de-Seine"),             // 92
    "Seine-Saint-Denis" => ("en", "Seine-Saint-Denis"),             // 93
    "Val-de-Marne" => ("dans le", "Val-de-Marne"),                  // 94
    "Val-d'Oise" => ("dans le", "Val-d'Oise"),                      // 95

    // Départements/Régions d'Outre-Mer
    "Guadeloupe" => ("en", "Guadeloupe"),
    "Guyane" => ("en", "Guyane"),
    "Martinique" => ("en", "Martinique"),
    "La Réunion" => ("à", "La Réunion"),
    "Mayotte" => ("à", "Mayotte"),

    // Régions de métropole
    "Alsace" => ("en", "Alsace"),
    "Aquitaine" => ("en", "Aquitaine"),
    "Auvergne" => ("en", "Auvergne"),
    "Basse-Normandie" => ("en", "Basse-Normandie"),
    "Bourgogne" => ("en", "Bourgogne"),
    "Bretagne" => ("en", "Bretagne"),
    "Centre" => ("dans le", "Centre"),
    "Champagne-Ardenne" => ("en", "Champagne-Ardenne"),
    "Corse" => ("en", "Corse"),
    "Franche-Comté" => ("en", "Franche-Comté"),
    "Haute-Normandie" => ("en", "Haute-Normandie"),
    "Île-de-France" => ("en", "Île-de-France"),
    "Languedoc-Roussillon" => ("en", "Languedoc-Roussillon"),
    "Limousin" => ("dans le", "Limousin"),
    "Lorraine" => ("en", "Lorraine"),
    "Midi-Pyrénées" => ("en", "Midi-Pyrénées"),
    "Nord-Pas-de-Calais" => ("dans le", "Nord-Pas-de-Calais"),
    "Pays de la Loire" => ("dans les", "Pays de la Loire"),
    "Picardie" => ("en", "Picardie"),
    "Poitou-Charentes" => ("en", "Poitou-Charentes"),
    "Provence-Alpes-Côte d'Azur" => ("en", "Provence-Alpes-Côte d'Azur"),
    "Rhône-Alpes" => ("en", "Rhône-Alpes"),
};

// TODO: A faire en fonction du début du lieu lorsqu'il est affiché sans le préfixe :
// - Pur une ville/lieu-dit: s'il commence par "L'" alors il ne faut rien faire (par exemple L'Aigle reste inchangé)
// - Pour un département : s'il commence par "l'" alors il faut supprimer le "l'" (par exemple l'Aube devient Aube)
// (le type de lieu se distingue par la casse de la lettre (L et l) ou le préfixe ("à" et "dans")

// TODO : Modifier dans geneanet Grau-du-Roi en Le Grau-du-Roi, idem pour Petit-Paris

// Décomposition d'un lieu et de son préfixe
#[derive(Default, Clone, PartialEq)]
pub struct PlaceParts {
    // Indique si une erreur a été rencontrée et donc si le lieu doit être affiché en rouge
    pub error: bool,
    // Préfixe du lieu (par défaut "à" mais peut être "en", "au", "aux", "à la", "dans le", "dans la", "dans les" ou "dans l'")
    pub prefix: &'static str,
    // Lieu
    pub place: String,
}

/// Interpretation et transformation d'un lieu:
/// - Gestion des arrondissements (par exemple "[2ème arrondissement] - Lyon" => "Lyon", "2ème arrondissement")
/// - Gestion des lieux-dits (par exemple "[Le Moulin] - Saint-Nazaire-le-Désert" => "Le Moulin", "Saint-Nazaire-le-Désert")
/// - Suppression des codes postaux
/// - Suppression de "France" si c'est le dernier élément
/// - Génération d'une erreur si un pays n'est pas en dernier position
/// - Vérification de la région quand le département est présent
/// - Supression des doublons (comme Paris à la fois ville et département, ou les outre-mer à la fois départements et régions)
/// - Ajout du prefixe sur le premier élément
pub fn interpreter_lieu(place_entree: &str) -> PlaceParts {
    let mut index_pays = 0usize;
    let mut last_index = 0usize;
    let mut bad_region = false;
    let mut previous_element = "";
    let mut prefix = "à";
    // Court-circuit rapide quand le lieu est réduit à un pays
    if !place_entree.contains(", ") {
        if let Some(prefix) = PREFIXE_PAYS.get(place_entree) {
            // On retourne le prefix correspondant ("en" ou "au")
            return PlaceParts { prefix, place: place_entree.to_string(), ..Default::default() };
        } else {
            // L'élément unique n'est pas un pays => on empile l'erreur et on retourne le lieu tel quel sans prefix
            // mais avec le flag d'erreur positionné pour qu'il soit affiché en rouge
            write_errors!(errors, {
                errors.insert(place_entree.to_string(), anyhow!("Le lieu est réduit à un seul élément qui n'est pas un pays : '{place_entree}'"))
            });
            return PlaceParts { error: true, place: place_entree.to_string(), ..Default::default() };
        }
    }
    let mut v: Vec<&str> = place_entree.split(", ").collect();
    // Scinde le 1er élément en deux s'il commence par un arrondissemnt ou un lieu-dit
    // Un tel lieu a pour structure: [arrondissement/lieu-dit] - ville
    if v[0].starts_with("[") {
        let v2: Vec<&str> = v[0].split("] - ").collect();
        if v2.len() == 2 {
            let (first, second) = if v2[0].ends_with("arrondissement") {
                // Ordre : ville, arrondissemment
                (v2[1], &v2[0][1..])
            } else {
                // Ordre : lieu-dit, ville
                (&v2[0][1..], v2[1])
            };
            // Remplace le premier élément par les 2 élements générés
            v[0] = first;
            v.insert(1, second);
        } else {
            write_errors!(errors, {
                errors.insert(place_entree.to_string(), anyhow!("Structure invalide pour un lieu commençant par '[' : '{place_entree}'"));
            });
            return PlaceParts { error: true, place: place_entree.to_string(), ..Default::default() };
        }
    }
    let v: Vec<&str> = v.into_iter().enumerate()
        // Supprime les codes postaux (ils sont toujours sur 5 chiffres,
        // par exemple Chalencon en Ardèche a pour code postal 07048)
        .filter(|&(_u, x)| x.len() != 5 || !x.chars().all(|c| c.is_ascii_digit()))
        // Suppression des doublons et de "France" en position finale
        // vérification de la région du département et qu'un pays est en fin
        .filter(|&(u, lieu)| {
            // Suppression des doublons (Paris et outre-mer).
            if lieu == previous_element {
                return false;
            }
            if u > 0 {
                // Positionne un flag d'erreur quand l'élément courant est une région et que le précédent est un département
                // mais qu'il n'est pas dans cette région
                if ANCIENNES_REGIONS_FRANCE.contains(lieu) {
                    if let Some(&expected_region) = DEPARTEMENT_TO_ANCIENNE_REGION.get(previous_element) {
                        if expected_region != lieu {
                            bad_region = true;
                        }
                    }
                }
            }
            // L'arrondissement est sauté pour la gestion des doublons
            if !lieu.ends_with ("arrondissement") {
                previous_element = lieu;
            }
            last_index = u;
            if PREFIXE_PAYS.contains_key(lieu) {
                // On mémorise l'index d'un pays pour vérifier que c'est bien le dernier élément
                index_pays = u;
                // On ne conserve pas l'élément "France" (sauf si c'est le seul élément, mais ce cas est géré
                // dans le court-circuit au début de la fonction)
                if lieu == "France" {
                    return false;
                }
            }
            // On conserve les autres éléments
            true
        })
        // Ajout du préfixe pour les villes/lieux-dits et départements/régions
        .map(|(u, lieu)| {
            if u == 0 {
                if let Some((prefix_dep, dep)) = PREFIXE_DEPARTEMENTS_REGIONS.get(lieu) {
                    // Ajout du préfixe des départements
                    prefix = *prefix_dep;
                    return *dep;
                } else if let Some(stripped) = lieu.strip_prefix("Le ") {
                    // On supprime "Le " et le préfixe devient "au", par exemple Le Mans => au Mans
                    prefix = "au";
                    return stripped;
                } else if let Some(stripped) = lieu.strip_prefix("Les ") {
                    // On supprime "Les " et le préfixe devient "aux", par exemple Les Sables-d’Olonne => aux Sables-d’Olonne
                    prefix = "aux";
                    return stripped;
                } else if let Some(stripped) = lieu.strip_prefix("La ") {
                    // On supprime "La " et le préfixe devient "à la", par exemple La Rochelle => à la Rochelle
                    prefix = "à la";
                    return stripped;
                } else if lieu.starts_with("L'") {
                    // On ne supprime rien et le préfixe reste "à", par exemple L'Aigle => à L'Aigle
                    // Pour être rigoureux il faudrait remplacer le L majuscule par l minuscule (comme dans "à l'Aigle")
                    // mais cela oblige à convertir tous les lieux en String et entraine des complications quand on veut
                    // afficher le lieu sans le préfixe (auquel cas il faudrait remettre le L majuscule).
                    // En outre le fait que le préfixe et le lieu sont stylés de manière différente rend la chose acceptable.
                    return lieu;
                }
            }
            lieu
        })
        .collect();
    if index_pays != 0 && index_pays != last_index {
        write_errors!(errors, {
            errors.insert(place_entree.to_string(), anyhow!("Le nom d'un pays doit être en derière position : '{place_entree}'"));
        });
        return PlaceParts { error: true, place: place_entree.to_string(), ..Default::default() };
    }
    if bad_region {
        write_errors!(errors, {
            errors.insert(place_entree.to_string(), anyhow!("Le départment n'appartient pas à la région spécifiée : '{place_entree}"));
        });
        return PlaceParts { error: true, place: place_entree.to_string(), ..Default::default() };
    }
    PlaceParts { prefix, place: v.join(", "), ..Default::default() }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Les fonctions ok et err étant similaires entre les modules testés
    // on utilise des macros définies dans test_helpers.rs pour les générer
    crate::generer_verif_err!(
        fonction_test: interpreter_lieu,
        chaine_assert: "le lieu",
    );

    crate::generer_verif_ok!(
        fonction_test: interpreter_lieu,
        chaine_assert: "le lieu",
        types_retour: (&'static str, String),
        extracteur: |p: PlaceParts| (p.prefix, p.place)
    );

    #[test]
    fn test_pays() {
        assert_eq!(ok("France"), ("en", "France".to_string()));
        assert_eq!(ok("Italie"), ("en", "Italie".to_string()));
        assert_eq!(ok("Japon"), ("au", "Japon".to_string()));
        assert_eq!(ok("Fidji"), ("aux", "Fidji".to_string()));
        assert_eq!(ok("Cuba"), ("à", "Cuba".to_string()));
        assert_eq!(ok("Jamaïque"), ("à la", "Jamaïque".to_string()));
    }

    #[test]
    fn test_villes() {
        assert_eq!(ok("Casalvieri, Italie"), ("à", "Casalvieri, Italie".to_string()));
        assert_eq!(ok("Valence, Drôme, Rhône-Alpes, France"), ("à", "Valence, Drôme, Rhône-Alpes".to_string()));
        assert_eq!(ok("Le Havre, Seine-Maritime, Haute-Normandie, France"), ("au", "Havre, Seine-Maritime, Haute-Normandie".to_string()));
        assert_eq!(ok("Les Sables-d’Olonne, Vendée, Pays de la Loire"), ("aux", "Sables-d’Olonne, Vendée, Pays de la Loire".to_string()));
        assert_eq!(ok("La Rochelle, Charente-Maritime, France"), ("à la", "Rochelle, Charente-Maritime".to_string()));

        // En toute rigueur on devrait générer "l'Aigle" mais cela apporte trop de complications
        assert_eq!(ok("L'Aigle, Orne, France"), ("à", "L'Aigle, Orne".to_string()));

        // Cas particulier à tester car le caractère UTF-8 É prend 2 octets. Ce test était utile quand les lieux étaient des String
        // et que ce cas utilisait l'instruction format!("l'{}", &lieu[2..])
        assert_eq!(ok("L'Épine, Vendée, France"), ("à", "L'Épine, Vendée".to_string()));

        // Test de Paris à la fois ville et département
        assert_eq!(ok("[7ème arrondissement] - Paris, 75056, Paris, Ile-de-France, France"), ("à", "Paris, 7ème arrondissement, Ile-de-France".to_string()));

        // Test d'un lieu-dit
        assert_eq!(ok("[Les Joinas] - Brette, Drôme, Rhône-Alpes, France"), ("aux", "Joinas, Brette, Drôme, Rhône-Alpes".to_string()));

    }

    #[test]
    fn test_departements_regions() {
        // Test de départements
        assert_eq!(ok("Allier, France"), ("dans", "l'Allier".to_string()));
        assert_eq!(ok("Drôme, France"), ("dans la", "Drôme".to_string()));
        assert_eq!(ok("Yvelines, France"), ("dans les", "Yvelines".to_string()));

        // Test de régions
        assert_eq!(ok("Alsace, France"), ("en", "Alsace".to_string()));
        assert_eq!(ok("Limousin, France"), ("dans le", "Limousin".to_string()));

        // Test de départements qui sont aussi des régions
        assert_eq!(ok("Guadeloupe, Guadeloupe, France"), ("en", "Guadeloupe".to_string()));
        assert_eq!(ok("Mayotte, France"), ("à", "Mayotte".to_string()));
        assert_eq!(ok("La Réunion, La Réunion, France"), ("à", "La Réunion".to_string()));
    }

    #[test]
    fn test_lieux_invalides() {
        assert_eq!(err("Vietnamx"), ["Le lieu est réduit à un seul élément qui n'est pas un pays : 'Vietnamx'"]);
        assert_eq!(err("Les Sables-d’Olonne, Vendée, Rhône-Alpes"), ["Le départment n'appartient pas à la région spécifiée : 'Les Sables-d’Olonne, Vendée, Rhône-Alpes"]);
        assert_eq!(err("Valence, Drôme, France, Rhône-Alpes"), ["Le nom d'un pays doit être en derière position : 'Valence, Drôme, France, Rhône-Alpes'"]);
    }
}
