use rcode::agent::file_tool::FileTool;

fn main() -> std::io::Result<()> {
    FileTool::list_files(".", false)?;
    FileTool::search_file("src/agent.rs", "file_tool")?;
    Ok(())
}
