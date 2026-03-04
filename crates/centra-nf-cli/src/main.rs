//! centra-nf — Command-line interface for CENTRA-NF compiler
//!
//! Usage:
//!   centra-nf compile <input.cnf> [--output <output>]
//!   centra-nf check <input.cnf>
//!   centra-nf help

use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

use cnf_compiler::compile;

#[derive(Parser)]
#[command(name = "centra-nf")]
#[command(about = "CENTRA-NF Compiler — Deterministic, fail-fast compilation", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a .cnf source file to intermediate representation
    Compile {
        /// Input .cnf file
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output file for IR (default: stdout)
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Verbose output (show IR instructions)
        #[arg(short, long)]
        verbose: bool,
    },

    /// Check syntax of a .cnf file without compiling
    Check {
        /// Input .cnf file
        #[arg(value_name = "FILE")]
        input: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile {
            input,
            output,
            verbose,
        } => {
            compile_file(&input, output.as_ref(), verbose);
        }
        Commands::Check { input } => {
            check_file(&input);
        }
    }
}

/// Compile a .cnf file and output IR
fn compile_file(input_path: &PathBuf, output_path: Option<&PathBuf>, verbose: bool) {
    // Read source file
    let source = match fs::read_to_string(input_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file '{}': {}", input_path.display(), e);
            std::process::exit(1);
        }
    };

    // Compile
    match compile(&source) {
        Ok(instructions) => {
            if verbose {
                eprintln!(
                    "ℹ️  Compiled successfully. Generated {} instruction(s)",
                    instructions.len()
                );
            }

            // Format output
            let output_text = if instructions.is_empty() {
                "(empty program)".to_string()
            } else {
                instructions
                    .iter()
                    .map(|instr| instr.to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            };

            // Write output
            if let Some(out_path) = output_path {
                match fs::write(out_path, &output_text) {
                    Ok(_) => {
                        if verbose {
                            eprintln!("✓ Output written to '{}'", out_path.display());
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Error writing file '{}': {}", out_path.display(), e);
                        std::process::exit(1);
                    }
                }
            } else {
                println!("{}", output_text);
            }
        }
        Err(e) => {
            eprintln!("❌ Compilation error:\n{}", e);
            std::process::exit(1);
        }
    }
}

/// Check syntax of a .cnf file
fn check_file(input_path: &PathBuf) {
    // Read source file
    let source = match fs::read_to_string(input_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file '{}': {}", input_path.display(), e);
            std::process::exit(1);
        }
    };

    // Compile (check only)
    match compile(&source) {
        Ok(_) => {
            eprintln!("✓ Syntax OK: '{}'", input_path.display());
        }
        Err(e) => {
            eprintln!("❌ Syntax error in '{}':\n{}", input_path.display(), e);
            std::process::exit(1);
        }
    }
}
