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
        #[arg(long)]
        tenant_id: Option<String>,
    },
    /// List available catalogs
    ListCatalogs {
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        offset: Option<usize>,
    },
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
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        offset: Option<usize>,
    },
    CreateBranch {
        #[arg(short, long)]
        catalog: String,
        name: String,
        #[arg(long)]
        from: Option<String>,
        /// Branch type: main, feature, or experiment
        #[arg(long)]
        branch_type: Option<String>,
        /// Comma-separated list of assets for partial branching
        #[arg(long)]
        assets: Option<String>,
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
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        offset: Option<usize>,
    },
    CreateTag {
        #[arg(short, long)]
        catalog: String,
        name: String,
        #[arg(long)]
        commit_id: String,
    },
    // --- Access Requests ---
    ListRequests {
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        offset: Option<usize>,
    },
    RequestAccess {
        #[arg(short, long)]
        resource: String,
        #[arg(short, long)]
        role: String,
        #[arg(long)]
        reason: String,
    },
    // --- Token Generation ---
    /// Generate a JWT token for automation/scripts
    GetToken {
        /// Description of what this token will be used for
        #[arg(short, long)]
        description: String,
        /// Token expiration in days (default: 90)
        #[arg(long, default_value = "90")]
        expires_in: u32,
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
