use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "pangolin-user")]
#[command(multicall = true)]
pub struct UserCli {
    #[command(subcommand)]
    pub command: UserCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum UserCommand {
    /// Login to the Pangolin server
    Login {
        #[arg(short, long)]
        username: Option<String>,
        #[arg(short, long)]
        password: Option<String>,
    },
    /// List available catalogs
    ListCatalogs,
    /// Search for tables and views
    Search {
        /// Search query
        query: String,
    },
    /// Generate code snippets for connecting to a table
    GenerateCode {
        /// Type of code to generate
        #[arg(short, long, value_enum)]
        language: CodeLanguage,
        /// Full name of the table (catalog.namespace.table)
        #[arg(short, long)]
        table: String,
    },
    // --- Data Engineering: Branches ---
    ListBranches {
        #[arg(short, long)]
        catalog: String,
    },
    CreateBranch {
        #[arg(short, long)]
        catalog: String,
        name: String,
        #[arg(long)]
        from: Option<String>,
    },
    MergeBranch {
        #[arg(short, long)]
        catalog: String,
        source: String,
        target: String,
    },
    // --- Data Engineering: Tags ---
    ListTags {
        #[arg(short, long)]
        catalog: String,
    },
    CreateTag {
        #[arg(short, long)]
        catalog: String,
        name: String,
        #[arg(long)]
        commit_id: String,
    },
    // --- Access Requests ---
    ListRequests,
    RequestAccess {
        #[arg(short, long)]
        resource: String,
        #[arg(short, long)]
        role: String,
        #[arg(long)]
        reason: String,
    },
    /// Exit the REPL
    Exit,
    /// Clear the screen
    Clear,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum CodeLanguage {
    Pyiceberg,
    Pyspark,
    Dremio,
    Sql,
}
