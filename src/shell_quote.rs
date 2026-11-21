// Reconstitution de la ligne de commande, y compris les quotings

use std::ffi::OsString;

// Cible native Unix
#[cfg(unix)]
fn shell_quote(s: &str) -> String {
    if s.is_empty() {
        return "''".to_string();
    }

    if s.chars()
        .all(|c| c.is_ascii_alphanumeric() || "-_./".contains(c))
    {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

// Cible native Windows (conventions argv Windows, adaptée à CMD mais pas au quoting PowerShell)
#[cfg(windows)]
fn shell_quote(s: &str) -> String {
    if s.is_empty() {
        return "\"\"".to_string();
    }

    if !s.chars().any(|c| c.is_whitespace() || c == '"') {
        return s.to_string();
    }

    let mut result = String::from("\"");
    let mut backslashes = 0;

    for c in s.chars() {
        match c {
            '\\' => {
                backslashes += 1;
            }

            '"' => {
                // Les antislashs précédant " doivent être doublés,
                // plus un antislash pour échapper le guillemet.
                result.push_str(&"\\".repeat(backslashes * 2 + 1));
                result.push('"');
                backslashes = 0;
            }

            _ => {
                result.push_str(&"\\".repeat(backslashes));
                backslashes = 0;
                result.push(c);
            }
        }
    }

    // Les antislashs précédant le " final doivent être doublés.
    result.push_str(&"\\".repeat(backslashes * 2));
    result.push('"');

    result
}

// Autre cible, notamment celle utilisée par `dx build`
#[cfg(not(any(unix, windows)))]
fn shell_quote(s: &str) -> String {
    s.to_string()
}

// Fonction interne avec un itérateur pour pouvoir simuler ArgOs dans les tests
fn inner_command_line<I: IntoIterator<Item = OsString>>(args: I) -> String {
    args.into_iter()
        .map(|arg| shell_quote(&arg.to_string_lossy()))
        .collect::<Vec<_>>()
        .join(" ")
}

// Fonction exposée reconstituant la ligne de commande originale
pub fn command_line() -> String {
    inner_command_line(std::env::args_os())
}

// TODO: Tester la ligne de commande reconstituée sur Ubuntu

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn test_shell_quote() {
        // L'argument reçu par le programme est :
        //      "Marie" Eugénie Briançon
        // On vérifie que shell_quote reconstitue le texte saisi dans le shell :
        //      "\"Marie\" Eugénie Briançon"

        #[cfg(windows)]
        assert_eq!(
            shell_quote(r#""Marie" Eugénie Briançon"#),
            r#""\"Marie\" Eugénie Briançon""#
        );
    }

    #[test]
    fn test_command_line() {
        // Simule les arguments reçus par le programme :
        //      ...
        //      -s
        //      "Marie" Eugénie Briançon
        // et vérifie la reconstitution de la ligne de commande :
        //      ... -s "\"Marie\" Eugénie Briançon"

        let args = [
            r#"target\debug\ged2htm.exe"#,
            "-i",
            "input.ged",
            "book",
            "-a",
            "-s",
            r#""Marie" Eugénie Briançon"#,
        ]
        .into_iter()
        .map(OsString::from);

        #[cfg(windows)]
        assert_eq!(
            inner_command_line(args),
            r#"target\debug\ged2htm.exe -i input.ged book -a -s "\"Marie\" Eugénie Briançon""#
        );
    }
}
