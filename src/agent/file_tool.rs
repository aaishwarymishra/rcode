use ignore::WalkBuilder;
use rig::completion::request::ToolDefinition;
use rig::tool::Tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct ListFilesArgs {
    pub path: String,
    pub include_hidden: bool,
    pub include_gitignore: bool,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct SearchFileArgs {
    pub path: String,
    pub regex_pattern: String,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct GetLinesArgs {
    pub path: String,
    pub start: usize,
    pub end: usize,
}

pub struct ListFiles;
impl Tool for ListFiles {
    const NAME: &'static str = "list_files";
    type Args = ListFilesArgs;
    type Output = String;
    type Error = std::io::Error;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Lists all files in a directory tree, respecting gitignore.".to_string(),
            parameters: json!(
                {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The root directory to list files from."
                        },
                        "include_hidden": {
                            "type": "boolean",
                            "description": "Whether to include hidden files (those starting with a dot like .venv .git etc). Only show hidden files if explicitly requested.",
                            "default": false
                        },
                        "include_gitignore": {
                            "type": "boolean",
                            "description": "Whether to respect .gitignore rules when listing files.",
                            "default": true
                        }
                    },
                    "required": ["path"]
                }
            ),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let mut results = Vec::new();
        let mut builder = WalkBuilder::new(args.path);
        builder.hidden(!args.include_hidden);
        builder.git_ignore(args.include_gitignore);

        let walker = builder.build();
        for result in walker {
            let entry = result.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                results.push(entry.path().display().to_string());
            }
        }
        Ok(results.join("\n"))
    }
}

pub struct SearchFile;
impl Tool for SearchFile {
    const NAME: &'static str = "search_file";
    type Args = SearchFileArgs;
    type Output = String;
    type Error = std::io::Error;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Search for a regex pattern in a file and return matching lines."
                .to_string(),
            parameters: serde_json::to_value(schemars::schema_for!(Self::Args)).unwrap(),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let mut results = Vec::new();
        let regex = regex::Regex::new(&args.regex_pattern)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let file = File::open(args.path)?;
        let reader = BufReader::new(file);

        for (idx, line_res) in reader.lines().enumerate() {
            let line = line_res?;
            if regex.is_match(&line) {
                results.push(format!("{}: {}", idx + 1, line));
            }
        }
        Ok(results.join("\n"))
    }
}

pub struct GetLines;
impl Tool for GetLines {
    const NAME: &'static str = "get_lines";
    type Args = GetLinesArgs;
    type Output = String;
    type Error = std::io::Error;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Read a specific range of lines from a file.".to_string(),
            parameters: serde_json::to_value(schemars::schema_for!(Self::Args)).unwrap(),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let mut results = Vec::new();
        let file = File::open(args.path)?;
        let reader = BufReader::new(file);

        for (idx, line_res) in reader.lines().enumerate() {
            let line_num = idx + 1;
            if line_num > args.end {
                break;
            }

            if line_num >= args.start {
                results.push(line_res?);
            }
        }
        Ok(results.join("\n"))
    }
}
