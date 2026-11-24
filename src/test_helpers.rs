// Module contenat des items partagées entre les fonctions de tests unitaires

// Macro générant un fonction de test err
// Par exemple elle permet de générer l'équivalent de la fonction suivante:
// fn err(place: &str) -> Vec<String> {
//     let parts = interpreter_lieu(place);
//     assert!(parts.error, "Un échec était attendu pour le lieu {}", place);
//     let errors = ERRORS.read().unwrap();
//     let error = errors.get(place).unwrap();
//     error.chain().map(|err| err.to_string()).collect()
// }
// L'appel pour cet exemple est :
// generer_verif_err!(
//     err<&str>
//     fonction_test: interpreter_lieu,
//     chaine_assert: "le lieu",
// );
#[macro_export]
macro_rules! generer_verif_err {
    (
        $err_name:ident < $type_arg:ty >
        fonction_test: $f_test:expr,
        chaine_assert: $msg_assert:expr,
    ) => {
        fn $err_name(input: $type_arg) -> Vec<String> {
            // Appel de la fonction testée
            let parts = $f_test(input);
            // Elle doit générer une erreur
            assert!(
                parts.error,
                concat!("Un échec était attendu pour ", $msg_assert, " {}"),
                input
            );
            let errors = ERRORS.read().unwrap();
            // Retrouve l'erreur correspondant au test courant
            // (les tests étant lancés en parallèle d'autres threads ont pu ajouter d'autres erreurs)
            if let Some(error) = errors.get(&input.to_string()) {
                error.chain().map(|err| err.to_string()).collect()
            } else {
                vec![format!("Missing error key: {input}")]
            }
        }
    };
}

// Macro générant un fonction de test ok
// Par exemple elle permet de générer l'équivalent de la fonction suivante:
//    fn ok(place: &str) -> (&'static str, String) {
//        let parts = interpreter_lieu(place);
//        assert!(!parts.error, "Un succès était attendu pour le lieu {}", place);
//        (parts.prefix, parts.place)
//    }
// Cette macro est plus complexe que la précédente car le type et l'expression retournés sont
// paramétrés. La closure permet de résoudre un problème d'hygiène de macro sur la variable parts.
// L'appel pour cet exemple est :
// generer_verif_ok!(
//     ok<&str, (&'static str, String)>
//     fonction_test: interpreter_lieu,
//     chaine_assert: "le lieu",
//     extracteur: |p: PlaceParts| (p.prefix, p.place)
// );
#[macro_export]
macro_rules! generer_verif_ok {
    (
        $ok_name:ident < $type_arg:ty , $type_ret:ty >
        fonction_test: $f_test:expr,
        chaine_assert: $msg_assert:expr,
        extracteur: $extracteur:expr // <--- Reçoit une closure |parts| -> tuple
    ) => {
        fn $ok_name(input: $type_arg) -> $type_ret {
            // Appel de la fonction testée
            let parts = $f_test(input);
            // Elle ne doit pas générer d'erreur
            assert!(
                !parts.error,
                concat!("Un succès était attendu pour ", $msg_assert, " {}"),
                input
            );
            // On appelle la closure en lui passant parts
            $extracteur(parts)
        }
    };
}
