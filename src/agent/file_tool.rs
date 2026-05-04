use ignore::WalkBuilder;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

pub struct FileTool;

impl FileTool {
    pub fn list_files(path: &str, include_hidden: bool) -> std::io::Result<()> {
        let mut builder = WalkBuilder::new(path);
        builder.hidden(!include_hidden); // show/hide dotfiles
        builder.git_ignore(true); // respect .gitignore
        builder.git_exclude(true); // respect .git/info/exclude
        let walker = builder.build();

        for result in walker {
            let entry = result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                println!("Found file: {:?}", entry.path());
            }
        }
        Ok(())
    }

    pub fn search_file(path: &str, regex_pattern: &str) -> std::io::Result<()> {
        let regex = regex::Regex::new(regex_pattern)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        for (idx, line_res) in reader.lines().enumerate() {
            let line = line_res?;
            if regex.is_match(&line) {
                println!("Match found in {}:{}", path, idx + 1);
                println!("> {}", line);
            }
        }
        Ok(())
    }
}
