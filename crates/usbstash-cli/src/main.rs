//! USB Stash CLI — encrypted vault management tool.
//!
//! Provides create, add, list, and extract commands over `usbstash-core`.

use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};

mod commands;
mod error;
mod password;

use error::CliError;

/// USB Stash — encrypted file vault for USB drives.
#[derive(Parser)]
#[command(name = "usbstash", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a new encrypted stash at the given path.
    Create {
        /// Directory path for the new stash.
        dir: PathBuf,

        /// Password (insecure — may appear in shell history).
        #[arg(long)]
        password: Option<String>,
    },
    /// Add a file to an existing stash.
    Add {
        /// Path to the stash directory.
        dir: PathBuf,

        /// Path to the file to add.
        file: PathBuf,

        /// Store the entry with this path instead of the filename.
        #[arg(long)]
        r#as: Option<String>,

        /// Password (insecure — may appear in shell history).
        #[arg(long)]
        password: Option<String>,
    },
    /// List all entries in a stash.
    List {
        /// Path to the stash directory.
        dir: PathBuf,

        /// Password (insecure — may appear in shell history).
        #[arg(long)]
        password: Option<String>,
    },
    /// Extract an entry from a stash to disk.
    Extract {
        /// Path to the stash directory.
        dir: PathBuf,

        /// Path of the entry to extract.
        path: String,

        /// Output file path (defaults to entry filename in current directory).
        #[arg(long)]
        output: Option<PathBuf>,

        /// Password (insecure — may appear in shell history).
        #[arg(long)]
        password: Option<String>,
    },
}

fn run() -> Result<(), CliError> {
    let cli = Cli::parse();

    match cli.command {
        Command::Create { dir, password } => {
            let pass = password::resolve_password(password.as_deref())?;
            commands::create(&dir, &pass)
        }
        Command::Add {
            dir,
            file,
            r#as,
            password,
        } => {
            let pass = password::resolve_password(password.as_deref())?;
            commands::add(&dir, &file, r#as.as_deref(), &pass)
        }
        Command::List { dir, password } => {
            let pass = password::resolve_password(password.as_deref())?;
            let entries = commands::list(&dir, &pass)?;
            if entries.is_empty() {
                println!("No entries.");
            } else {
                // Print header
                println!("{:<40} {:>10}  MIME TYPE", "PATH", "SIZE");
                println!("{:-<40} {:->10}  {:-<20}", "", "", "");
                for entry in &entries {
                    println!(
                        "{:<40} {:>10}  {}",
                        entry.path,
                        commands::list::format_bytes(entry.size),
                        entry.mime_type
                    );
                }
            }
            Ok(())
        }
        Command::Extract {
            dir,
            path,
            output,
            password,
        } => {
            let pass = password::resolve_password(password.as_deref())?;
            commands::extract(&dir, &path, output.as_deref(), &pass)?;
            Ok(())
        }
    }
}

fn main() {
    match run() {
        Ok(()) => process::exit(0),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
