use ged_io::types::note::Note;

pub fn concat_notes(notes: &Vec<Note>) -> Vec<String> {
    let mut ret = Vec::<String>::new();
    for note in notes {
        // Le type mime émis par Geneanet est toujours text/plain et jamais text/html,
        // même si la note contient des éléments comme <br> ou <pre>
        // => on considère le text comme du html
        if let Some(ref value) = note.value {
            // Convertit les retour-chariots
            let value = value.replace("\n", "<br>");
            // Remplace les doubles <br> introduits par la conversion des \n
            let value = value.replace("<br><br>", "<p style='margin-bottom: 4px'></p>");
            ret.push(value);
        }
    }
    ret
}
