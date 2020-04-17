use structopt::StructOpt;
pub mod utils;

/// Commands implement the Command trait, which lets us run() them
/// and get Output.
pub trait Command {
    /// Execute the command. TODO: should this return a Result?
    fn run(&self) -> i32 {0}
}

//#region    apollo <command>
#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
/// The [Experimental] Apollo CLI, for supporting all your graphql needs :)
pub enum Apollo {
    ///  🖨   parse and pretty print schemas to stdout
    Print(Print),
    ///  🔓  log in to apollo
    Auth(Auth),
    ///  🆕  create an object
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
pub mod auth;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum Auth {
    /// login as a user with an interactive command
    Login(AuthLogin),
    /// configure authentication via pasting in an existing key
    Configure(AuthConfigure)
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct AuthLogin {}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct AuthConfigure {}

//#endregion

impl Command for Apollo {
    fn run(&self) -> i32 {
        match self {
            Apollo::Print(cmd) => cmd.run(),
            Apollo::Create(cmd) => match cmd {
                Create::Graph(sub) => sub.run(),
            },
            Apollo::Auth(cmd) => match cmd {
                Auth::Configure(sub) => sub.run(),
                Auth::Login(sub) => sub.run(),
            }
        }
    }
}

impl Apollo {
    pub fn main() {
        std::process::exit(Apollo::from_args().run());
    }
}
