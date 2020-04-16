use structopt::StructOpt;
pub mod utils;

/// Commands implement the Command trait, which lets us run() them
/// and get Output.
pub trait Command {
    /// Execute the command. TODO: should this return a Result?
    fn run(&self) {}
}

//#region    apollo <command>
#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
/// The [Experimental] Apollo CLI, for supporting all your graphql needs :)
pub enum Apollo {
    ///  🖨   parse and pretty print schemas to stdout
    Print(Print),
    ///  🔓  log in to apollo
    Login(Login),
    /// Create an object
    Create(Create)
}
//#endregion

//#region    ... print [-h] <files...>
pub mod print;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct Print {
    #[structopt(short = "h", long)]
    /// suppress headers when printing multiple files
    pub no_headers: bool,

    #[structopt(parse(from_os_str))]
    /// schemas to print
    pub files: std::vec::Vec<std::path::PathBuf>,
}
//#endregion

//#region    ... create
pub mod create;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum Create {
    /// Create a graph with Apollo Graph Manager
    /// This is interactive by default
    Graph(CreateGraph)
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct CreateGraph {}
//#endregion

//#region    ... login
pub mod login;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct Login {}
//#endregion

impl Command for Apollo {
    fn run(&self) {
        match self {
            Apollo::Print(cmd) => cmd.run(),
            Apollo::Login(cmd) => cmd.run(),
            Apollo::Create(cmd) => match cmd {
                Create::Graph(subcmd) => subcmd.run(),
            }
        }
    }
}

impl Apollo {
    pub fn main() {
        Apollo::from_args().run();
    }
}
