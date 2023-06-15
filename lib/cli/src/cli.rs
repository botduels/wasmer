//! The logic for the Wasmer CLI tool.

#[cfg(target_os = "linux")]
use crate::commands::Binfmt;
#[cfg(feature = "compiler")]
use crate::commands::Compile;
#[cfg(any(feature = "static-artifact-create", feature = "wasmer-artifact-create"))]
use crate::commands::CreateExe;
#[cfg(feature = "wast")]
use crate::commands::Wast;
use crate::commands::{
    Add, Cache, Config, Init, Inspect, Login, Publish, Run, SelfUpdate, Validate, Whoami,
};
#[cfg(feature = "static-artifact-create")]
use crate::commands::{CreateObj, GenCHeader};
use crate::error::PrettyError;
use clap::{CommandFactory, Parser};

/// The main function for the Wasmer CLI tool.
pub fn wasmer_main() {
    // We allow windows to print properly colors
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).unwrap();

    PrettyError::report(wasmer_main_inner())
}

fn wasmer_main_inner() -> Result<(), anyhow::Error> {
    if is_binfmt_interpreter() {
        Run::from_binfmt_args().execute();
    }

    match Args::try_parse() {
        Ok(args) => {
            args.output.initialize_logging();
            args.execute()
        }
        Err(e) if e.kind() == clap::error::ErrorKind::InvalidSubcommand => {
            // Try to parse it as `wasmer some/package`
            Run::parse().execute()
        }
        Err(e) => {
            e.exit();
        }
    }
}

/// Command-line arguments for the Wasmer CLI.
#[derive(Parser, Debug)]
#[clap(about, author)]
#[cfg_attr(feature = "headless", clap(name = "wasmer-headless"))]
#[cfg_attr(not(feature = "headless"), clap(name = "wasmer-headless"))]
pub struct Args {
    /// Print version info and exit.
    #[clap(short = 'V', long)]
    version: bool,
    #[clap(flatten)]
    output: crate::logging::Output,
    #[clap(subcommand)]
    cmd: Option<Cmd>,
}

impl Args {
    fn execute(self) -> Result<(), anyhow::Error> {
        let Args {
            cmd,
            version,
            output,
        } = self;

        if version {
            return print_version(output.is_verbose());
        }

        if let Some(cmd) = cmd {
            cmd.execute()
        } else {
            Args::command().print_long_help()?;
            // Note: clap uses an exit code of 2 when CLI parsing fails
            std::process::exit(2);
        }
    }
}

#[derive(Parser, Debug)]
/// The options for the wasmer Command Line Interface
enum Cmd {
    /// Login into a wasmer.io-like registry
    Login(Login),

    /// Login into a wasmer.io-like registry
    #[clap(name = "publish")]
    Publish(Publish),

    /// Wasmer cache
    #[clap(subcommand)]
    Cache(Cache),

    /// Validate a WebAssembly binary
    Validate(Validate),

    /// Compile a WebAssembly binary
    #[cfg(feature = "compiler")]
    Compile(Compile),

    /// Compile a WebAssembly binary into a native executable
    ///
    /// To use, you need to set the `WASMER_DIR` environment variable
    /// to the location of your Wasmer installation. This will probably be `~/.wasmer`. It
    /// should include a `lib`, `include` and `bin` subdirectories. To create an executable
    /// you will need `libwasmer`, so by setting `WASMER_DIR` the CLI knows where to look for
    /// header files and libraries.
    ///
    /// Example usage:
    ///
    /// ```text
    /// $ # in two lines:
    /// $ export WASMER_DIR=/home/user/.wasmer/
    /// $ wasmer create-exe qjs.wasm -o qjs.exe # or in one line:
    /// $ WASMER_DIR=/home/user/.wasmer/ wasmer create-exe qjs.wasm -o qjs.exe
    /// $ file qjs.exe
    /// qjs.exe: ELF 64-bit LSB pie executable, x86-64 ...
    /// ```
    ///
    /// ## Cross-compilation
    ///
    /// Accepted target triple values must follow the
    /// ['target_lexicon'](https://crates.io/crates/target-lexicon) crate format.
    ///
    /// The recommended targets we try to support are:
    ///
    /// - "x86_64-linux-gnu"
    /// - "aarch64-linux-gnu"
    /// - "x86_64-apple-darwin"
    /// - "arm64-apple-darwin"
    #[cfg(any(feature = "static-artifact-create", feature = "wasmer-artifact-create"))]
    #[clap(name = "create-exe", verbatim_doc_comment)]
    CreateExe(CreateExe),

    /// Compile a WebAssembly binary into an object file
    ///
    /// To use, you need to set the `WASMER_DIR` environment variable to the location of your
    /// Wasmer installation. This will probably be `~/.wasmer`. It should include a `lib`,
    /// `include` and `bin` subdirectories. To create an object you will need `libwasmer`, so by
    /// setting `WASMER_DIR` the CLI knows where to look for header files and libraries.
    ///
    /// Example usage:
    ///
    /// ```text
    /// $ # in two lines:
    /// $ export WASMER_DIR=/home/user/.wasmer/
    /// $ wasmer create-obj qjs.wasm --object-format symbols -o qjs.obj # or in one line:
    /// $ WASMER_DIR=/home/user/.wasmer/ wasmer create-exe qjs.wasm --object-format symbols -o qjs.obj
    /// $ file qjs.obj
    /// qjs.obj: ELF 64-bit LSB relocatable, x86-64 ...
    /// ```
    ///
    /// ## Cross-compilation
    ///
    /// Accepted target triple values must follow the
    /// ['target_lexicon'](https://crates.io/crates/target-lexicon) crate format.
    ///
    /// The recommended targets we try to support are:
    ///
    /// - "x86_64-linux-gnu"
    /// - "aarch64-linux-gnu"
    /// - "x86_64-apple-darwin"
    /// - "arm64-apple-darwin"
    #[cfg(feature = "static-artifact-create")]
    #[structopt(name = "create-obj", verbatim_doc_comment)]
    CreateObj(CreateObj),

    /// Generate the C static_defs.h header file for the input .wasm module
    #[cfg(feature = "static-artifact-create")]
    GenCHeader(GenCHeader),

    /// Get various configuration information needed
    /// to compile programs which use Wasmer
    Config(Config),

    /// Update wasmer to the latest version
    #[clap(name = "self-update")]
    SelfUpdate(SelfUpdate),

    /// Inspect a WebAssembly file
    Inspect(Inspect),

    /// Initializes a new wasmer.toml file
    #[clap(name = "init")]
    Init(Init),

    /// Run spec testsuite
    #[cfg(feature = "wast")]
    Wast(Wast),

    /// Unregister and/or register wasmer as binfmt interpreter
    #[cfg(target_os = "linux")]
    Binfmt(Binfmt),

    /// Shows the current logged in user for the current active registry
    Whoami(Whoami),

    /// Add a Wasmer package's bindings to your application.
    Add(Add),

    /// Run a WebAssembly file or Wasmer container.
    #[clap(alias = "run-unstable")]
    Run(Run),

    // DEPLOY commands
    /// Deploy apps to the Wasmer Edge.
    Deploy(wasmer_deploy_cli::cmd::deploy::CmdDeploy),

    /// Manage deployed apps.
    #[clap(subcommand, alias = "apps")]
    App(wasmer_deploy_cli::cmd::app::CmdApp),

    /// Create a dynamic on the Deploy Edge, and connect to it through SSH.
    Ssh(wasmer_deploy_cli::cmd::ssh::CmdSsh),

    /// Manage Wasmer namespaces.
    #[clap(subcommand, alias = "namespaces")]
    Namespace(wasmer_deploy_cli::cmd::namespace::CmdNamespace),
}

impl Cmd {
    fn execute(self) -> Result<(), anyhow::Error> {
        use wasmer_deploy_cli::cmd::CliCommand;

        match self {
            Self::Run(options) => options.execute(),
            Self::SelfUpdate(options) => options.execute(),
            Self::Cache(cache) => cache.execute(),
            Self::Validate(validate) => validate.execute(),
            #[cfg(feature = "compiler")]
            Self::Compile(compile) => compile.execute(),
            #[cfg(any(feature = "static-artifact-create", feature = "wasmer-artifact-create"))]
            Self::CreateExe(create_exe) => create_exe.execute(),
            #[cfg(feature = "static-artifact-create")]
            Self::CreateObj(create_obj) => create_obj.execute(),
            Self::Config(config) => config.execute(),
            Self::Inspect(inspect) => inspect.execute(),
            Self::Init(init) => init.execute(),
            Self::Login(login) => login.execute(),
            Self::Publish(publish) => publish.execute(),
            #[cfg(feature = "static-artifact-create")]
            Self::GenCHeader(gen_heder) => gen_heder.execute(),
            #[cfg(feature = "wast")]
            Self::Wast(wast) => wast.execute(),
            #[cfg(target_os = "linux")]
            Self::Binfmt(binfmt) => binfmt.execute(),
            Self::Whoami(whoami) => whoami.execute(),
            Self::Add(install) => install.execute(),

            // Deploy commands.
            Self::Deploy(c) => c.run(),
            Self::App(apps) => apps.run(),
            Self::Ssh(ssh) => ssh.run(),
            Self::Namespace(namespace) => namespace.run(),
        }
    }
}

fn is_binfmt_interpreter() -> bool {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "linux")] {
            use std::{ffi::OsStr, path::PathBuf};

            // Note: we'll be invoked as
            let binary_path = match std::env::args_os().next() {
                Some(path) => PathBuf::from(path),
                None => return false,
            };
            binary_path.file_name().and_then(|f| f.to_str()) == Some(Binfmt::FILENAME)
        } else {
            false
        }
    }
}

fn print_version(verbose: bool) -> Result<(), anyhow::Error> {
    if !verbose {
        println!("wasmer {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    println!(
        "wasmer {} ({} {})",
        env!("CARGO_PKG_VERSION"),
        env!("WASMER_BUILD_GIT_HASH_SHORT"),
        env!("WASMER_BUILD_DATE")
    );
    println!("binary: {}", env!("CARGO_PKG_NAME"));
    println!("commit-hash: {}", env!("WASMER_BUILD_GIT_HASH"));
    println!("commit-date: {}", env!("WASMER_BUILD_DATE"));
    println!("host: {}", target_lexicon::HOST);

    let mut compilers = Vec::<&'static str>::new();
    if cfg!(feature = "singlepass") {
        compilers.push("singlepass");
    }
    if cfg!(feature = "cranelift") {
        compilers.push("cranelift");
    }
    if cfg!(feature = "llvm") {
        compilers.push("llvm");
    }
    println!("compiler: {}", compilers.join(","));

    Ok(())
}
