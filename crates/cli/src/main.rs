use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use rapidagent_core::{compile_source, optimize, parse_rag};
use rapidagent_renderer::{render_xml, render_json, render_markdown, render_folder, FolderFormat};
use rapidagent_security::{scan_for_injection, hash_prompt};

#[derive(Parser, Debug)]
#[command(name = "syspro")]
#[command(about = "RapidWebs System Prompt Compiler")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Compile a .spc or .rag file to output
    Compile {
        /// Input file (.spc or .rag)
        input: String,
        /// Output format (xml, markdown, json, folder)
        #[arg(short, long, default_value = "xml")]
        format: String,
        /// Output file or directory (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Show the compiled prompt structure
    Inspect {
        /// Input .spc file
        input: String,
    },
    /// Scan for security issues
    Scan {
        /// Input .spc file
        input: String,
    },
    /// Version operations
    #[command(subcommand)]
    Version(VersionCommand),
}

#[derive(Subcommand, Debug)]
enum VersionCommand {
    /// Show current version
    Show,
    /// Initialize versioning
    Init {
        /// Directory to initialize
        #[arg(default_value = ".versions")]
        dir: String,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Compile { input, format, output } => {
            let source = std::fs::read_to_string(&input)?;
            
            if input.ends_with(".rag") {
                let artifact = parse_rag(&source)?;
                eprintln!("Parsed .rag: {} nodes, {} edges",
                    artifact.graph.as_ref().map(|g| g.nodes.len()).unwrap_or(0),
                    artifact.graph.as_ref().map(|g| g.edges.len()).unwrap_or(0));
                
                match format.as_str() {
                    "folder" => {
                        let path = output.map(PathBuf::from).unwrap_or_else(|| PathBuf::from("compiled-agent"));
                        render_folder(&artifact.to_compiled_artifact(), &path, FolderFormat::Standard)?;
                        eprintln!("Written to {}", path.display());
                    }
                    "binary" => {
                        let path = output.map(PathBuf::from).unwrap_or_else(|| PathBuf::from("agent.rapidagent"));
                        use rapidagent_renderer::{to_binary, CompilerIdentity};
                        let compiled = artifact.to_compiled_artifact();
                        let identity = CompilerIdentity::default();
                        let binary = to_binary(&compiled, &identity)?;
                        let binary_len = binary.len();
                        std::fs::write(&path, binary)?;
                        eprintln!("Written {} bytes to {}", binary_len, path.display());
                        eprintln!("UUID: {}", compiled.metadata.uuid);
                        eprintln!("Hash: {}", compiled.metadata.hash);
                    }
                    _ => {
                        let ir = artifact.to_prompt_ir();
                        let rendered = match format.as_str() {
                            "xml" => render_xml(&ir),
                            "markdown" | "md" => render_markdown(&ir),
                            "json" => render_json(&ir)?,
                            _ => return Err(anyhow::anyhow!("Unsupported format: {}. Use 'xml', 'markdown', 'json', or 'folder'.", format)),
                        };
                        if let Some(path) = output {
                            std::fs::write(&path, rendered)?;
                            eprintln!("Written to {}", path);
                        } else {
                            println!("{}", rendered);
                        }
                    }
                }
            } else {
                let mut ir = compile_source(&source)?;
                optimize(&mut ir);
                
                let rendered = match format.as_str() {
                    "xml" => render_xml(&ir),
                    "markdown" | "md" => render_markdown(&ir),
                    "json" => render_json(&ir)?,
                    _ => return Err(anyhow::anyhow!("Unsupported format: {}. Use 'xml', 'markdown', or 'json'.", format)),
                };
                
                if let Some(path) = output {
                    std::fs::write(&path, rendered)?;
                    eprintln!("Written to {}", path);
                } else {
                    println!("{}", rendered);
                }
                
                let hash = hash_prompt(&ir);
                eprintln!("Hash: {}", hash);
            }
        }
        Command::Inspect { input } => {
            let source = std::fs::read_to_string(&input)?;
            let ir = if input.ends_with(".rag") {
                let artifact = parse_rag(&source)?;
                eprintln!("Parsed .rag: {} nodes, {} edges", 
                    artifact.graph.as_ref().map(|g| g.nodes.len()).unwrap_or(0),
                    artifact.graph.as_ref().map(|g| g.edges.len()).unwrap_or(0));
                artifact.to_prompt_ir()
            } else {
                compile_source(&source)?
            };
            
            println!("Name: {}", ir.name);
            println!("Version: {}", ir.version);
            println!("Sections: {}", ir.sections.len());
            println!();
            for section in &ir.sections {
                println!("[{}] {} - weight: {}", section.priority, section.name, section.priority.weight());
            }
        }
        Command::Scan { input } => {
            let source = std::fs::read_to_string(&input)?;
            let ir = if input.ends_with(".rag") {
                let artifact = parse_rag(&source)?;
                artifact.to_prompt_ir()
            } else {
                compile_source(&source)?
            };
            let warnings = scan_for_injection(&ir);
            
            if warnings.is_empty() {
                println!("No security issues found.");
            } else {
                println!("Found {} security issue(s):", warnings.len());
                for w in warnings {
                    println!("  [{}] {}: {}", w.severity, w.section, w.message);
                }
            }
        }
        Command::Version(v) => match v {
            VersionCommand::Show => {
                println!("syspro {}", env!("CARGO_PKG_VERSION"));
            }
            VersionCommand::Init { dir } => {
                std::fs::create_dir_all(&dir)?;
                println!("Initialized version store in {}", dir);
            }
        },
    }

    Ok(())
}