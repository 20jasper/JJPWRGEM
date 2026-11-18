pub fn markdown_table(header: &[String], separator: &[&str], rows: &[Vec<String>]) -> String {
    let line = |cells: &[String]| {
        cells.iter().fold(String::from("|"), |mut line, cell| {
            if !cell.is_empty() {
                line.push(' ');
                line.push_str(cell);
            }
            line.push_str(" |");
            line
        })
    };
    let mut out = format!("{}\n|{}|\n", line(header), separator.join("|"));
    for row in rows {
        out.push_str(&line(row));
        out.push('\n');
    }
    out
}
