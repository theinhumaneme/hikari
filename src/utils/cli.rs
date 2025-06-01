use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct HikariCli {
    #[command(subcommand)]
    pub command: HikariCommands,
}

#[derive(Subcommand)]
pub enum HikariCommands {
    /// encrypt the configuration file
    Encrypt {
        #[arg(
            short = 'i',
            long,
            value_name = "input",
            help = "Path to the input configuration file to encrypt"
        )]
        input_file: String,
        #[arg(
            short = 'o',
            long,
            value_name = "output",
            help = "Path to the output file for the encrypted configuration"
        )]
        output_file: String,
    },
    /// decrypt the configuration file
    Decrypt {
        #[arg(
            short = 'i',
            long,
            value_name = "input",
            help = "Path to the input configuration file to encrypt"
        )]
        input_file: String,
        #[arg(
            short = 'o',
            long,
            value_name = "output",
            help = "Path to the output file for the encrypted configuration"
        )]
        output_file: String,
    },
    /// Generate all the COMPOSE YML files locally
    DryRun {
        #[arg(
            short = 'i',
            long,
            value_name = "input",
            help = "Path to the input configuration file to encrypt"
        )]
        input_file: String,
    },
    /// Run hikari in Daemon Mode (Standalone Mode)
    Daemon,
    /// Run hikari in Server Mode
    Server,
}
