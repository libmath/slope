use argh::FromArgs;

#[derive(FromArgs, Debug)]
/// Generate a file that imports everything below it.
#[argh(subcommand, name = "generate")]
pub struct Generate {
    #[argh(positional)]
    pub target: String,
}

#[derive(FromArgs, Debug)]
/// Check the formatting of `*.lean` files.
#[argh(subcommand, name = "check")]
pub struct Check {}

#[derive(FromArgs, Debug)]
/// Build all *.lean files below the current directory.
#[argh(subcommand, name = "build")]
pub struct Build {
    /// a simple prefix matcher to filter *.lean files.
    #[argh(positional)]
    pub targets: Vec<String>,
    //
    //
    // /// whether or not in include the *.lean files in the current directory.
    // #[argh(switch)]
    // pub include_cwd: bool,
}

#[derive(FromArgs, Debug)]
/// List all the indexed theorems in a ripgrep-like search.
#[argh(subcommand, name = "rg")]
pub struct Ripgrep {}

#[derive(FromArgs, Debug)]
/// Graphs the dependencies of Mathlib.
#[argh(subcommand, name = "graph")]
pub struct Graph {}

#[derive(FromArgs, Debug)]
/// Experimental: Search tool
#[argh(subcommand, name = "search")]
pub struct Search {}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
pub enum Subcommand {
    Generate(Generate),
    Check(Check),
    Build(Build),
    Ripgrep(Ripgrep),
    Graph(Graph),
    Search(Search),
}

#[derive(FromArgs, Debug)]
/// Top-level CLI.
pub struct Cli {
    #[argh(subcommand)]
    pub subcommand: Subcommand,
}

pub fn parse() -> Cli {
    argh::from_env()
}
