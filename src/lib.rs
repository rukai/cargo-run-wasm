pub mod target_dir;

use pico_args::Arguments;
use std::env;
use std::ffi::OsStr;
use std::process::Command;
use target_dir::CargoDirectories;

const HELP: &str = "\
cargo run-wasm

Hosts a binary or example of the local package as wasm in a local web server.

USAGE:
  cargo run-wasm [OPTIONS]

OPTIONS:
  cargo run-wasm custom options:
    --build-only                 Only build the WASM artifacts, do not run the dev server
    --host <HOST>                Makes the dev server listen on host (default 'localhost')
    --port <PORT>                Makes the dev server listen on port (default '8000')

  cargo run default options:
    -q, --quiet                     Do not print cargo log messages
        --bin [<NAME>]              Name of the bin target to run
        --example [<NAME>]          Name of the example target to run
    -p, --package [<SPEC>...]       Package with the target to run
    -v, --verbose                   Use verbose output (-vv very verbose/build.rs output)
    -j, --jobs <N>                  Number of parallel jobs, defaults to # of CPUs
        --color <WHEN>              Coloring: auto, always, never
        --keep-going                Do not abort the build as soon as there is an error (unstable)
        --frozen                    Require Cargo.lock and cache are up to date
    -r, --release                   Build artifacts in release mode, with optimizations
        --locked                    Require Cargo.lock is up to date
        --profile <PROFILE-NAME>    Build artifacts with the specified profile
    -F, --features <FEATURES>       Space or comma separated list of features to activate
        --offline                   Run without accessing the network
        --all-features              Activate all available features
        --config <KEY=VALUE>        Override a configuration value
        --no-default-features       Do not activate the `default` feature
    -Z <FLAG>                       Unstable (nightly-only) flags to Cargo, see 'cargo -Z help' for
                                    details
        --manifest-path <PATH>      Path to Cargo.toml
        --message-format <FMT>      Error format
        --unit-graph                Output build graph in JSON (unstable)
        --ignore-rust-version       Ignore `rust-version` specification in packages
        --timings[=<FMTS>...]       Timing output formats (unstable) (comma separated): html, json
    -h, --help                      Print help information

At least one of `--package`, `--bin` or `--example` must be used.

Normally you can run just `cargo run` to run the main binary of the current package.
The equivalent of that is `cargo run-wasm --package name_of_current_package`
";

struct Args {
    help: bool,
    release: bool,
    build_only: bool,
    host: Option<String>,
    port: Option<String>,
    build_args: Vec<String>,
    package: Option<String>,
    example: Option<String>,
    bin: Option<String>,
    binary_name: String,
}

impl Args {
    pub fn from_env() -> Result<Self, String> {
        let mut args = Arguments::from_env();
        let release = args.contains("--release");
        let build_only = args.contains("--build-only");
        let help = args.contains("--help") || args.contains("-h");

        let host: Option<String> = args.opt_value_from_str("--host").unwrap();
        let port: Option<String> = args.opt_value_from_str("--port").unwrap();

        let package: Option<String> = args.opt_value_from_str("--package").unwrap();
        let example: Option<String> = args.opt_value_from_str("--example").unwrap();
        let bin: Option<String> = args.opt_value_from_str("--bin").unwrap();

        let banned_options = ["--target", "--target-dir"];
        for option in banned_options {
            if args
                .opt_value_from_str::<_, String>(option)
                .unwrap()
                .is_some()
            {
                return Err(format!(
                    "cargo-run-wasm does not support the {option} option"
                ));
            }
        }

        let binary_name = match example.as_ref().or(bin.as_ref()).or(package.as_ref()) {
            Some(name) => name.clone(),
            None => {
                return Err("Need to use at least one of `--package NAME`, `--example NAME` `--bin NAME`.\nRun cargo run-wasm --help for more info.".to_owned());
            }
        };

        let build_args = args
            .finish()
            .into_iter()
            .map(|x| x.into_string().unwrap())
            .collect();

        Ok(Args {
            help,
            release,
            build_only,
            host,
            port,
            build_args,
            package,
            example,
            bin,
            binary_name,
        })
    }
}

/// Call this in your run-wasm application.
///
/// It will:
/// 1. Get CLI args from env
/// 2. Compile the rust project to wasm
/// 3. Run wasm-bindgen
/// 4. Generate an index.html that runs the wasm
/// 5. Launch a tiny webserver to serve index.html + your wasm
///
/// It will block forever to keep the webserver running until killed with ctrl-c or similar
///
/// The css argument will be included directly into a `<style type="text/css"></style>` element in the generated page.
/// By default the body element will include some margin, so for full page apps you will want to remove that by calling like:
/// ```no_run
///     cargo_run_wasm::run_wasm_with_css("body { margin: 0px; }");
/// ```
pub fn run_wasm_with_css(css: &str) {
    // validate css
    //
    // Someone could easily get around this with some extra spaces
    // but im not about to import regex or do a complicated implementation by hand.
    if css.contains("</style>") {
        panic!(
            "`</style>` detected in the css. This is disallowed to prevent injecting elements into the DOM."
        )
    }

    let args = match Args::from_env() {
        Ok(args) => args,
        Err(err) => {
            println!("{}\n\n{}", err, HELP);
            return;
        }
    };
    if args.help {
        println!("{}", HELP);
        return;
    }

    let profile = if args.release { "release" } else { "debug" };

    // build wasm example via cargo
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());

    let CargoDirectories {
        workspace_root,
        target_directory,
    } = CargoDirectories::new(&cargo);
    let target_target = target_directory.join("wasm-examples-target");
    let mut cargo_args = vec![
        "build".as_ref(),
        "--target".as_ref(),
        "wasm32-unknown-unknown".as_ref(),
        // It is common to setup a faster linker such as mold or lld to run for just your native target.
        // It cant be set for wasm as wasm doesnt support building with these linkers.
        // This results in a separate rustflags value for native and wasm builds.
        // Currently rust triggers a full rebuild every time the rustflags value changes.
        //
        // Therefore we have this hack where we use a different target dir for wasm builds to avoid constantly triggering full rebuilds.
        // When this issue is resolved we might be able to remove this hack: https://github.com/rust-lang/cargo/issues/8716
        "--target-dir".as_ref(),
        target_target.as_os_str(),
    ];

    if let Some(package) = args.package.as_ref() {
        cargo_args.extend([OsStr::new("--package"), package.as_ref()]);
    }
    if let Some(example) = args.example.as_ref() {
        cargo_args.extend([OsStr::new("--example"), example.as_ref()]);
    }
    if let Some(bin) = args.bin.as_ref() {
        cargo_args.extend([OsStr::new("--bin"), bin.as_ref()]);
    }
    if args.release {
        cargo_args.push("--release".as_ref());
    }

    cargo_args.extend(args.build_args.iter().map(OsStr::new));
    let status = Command::new(&cargo)
        .current_dir(&workspace_root)
        .args(&cargo_args)
        .status()
        .unwrap();
    if !status.success() {
        // We can return without printing anything because cargo will have already displayed an appropriate error.
        return;
    }

    // run wasm-bindgen on wasm file output by cargo, write to the destination folder
    let target_profile = target_target.join("wasm32-unknown-unknown").join(profile);
    let wasm_source = if args.example.is_some() {
        target_profile.join("examples")
    } else {
        target_profile
    }
    .join(format!("{}.wasm", args.binary_name));

    if !wasm_source.exists() {
        println!("There is no binary at {wasm_source:?}, maybe you used `--package NAME` on a package that has no binary?");
        return;
    }

    let example_dest = target_directory
        .join("wasm-examples")
        .join(&args.binary_name);
    std::fs::create_dir_all(&example_dest).unwrap();
    let mut bindgen = wasm_bindgen_cli_support::Bindgen::new();
    bindgen
        .web(true)
        .unwrap()
        .omit_default_module_path(false)
        .input_path(&wasm_source)
        .generate(&example_dest)
        .unwrap();

    // process template index.html and write to the destination folder
    let index_template = include_str!("index.template.html");
    let index_processed = index_template
        .replace("{{name}}", &args.binary_name)
        // This is fine because a replaced {{name}} cant contain `{{css}} ` due to `{` not being valid in a crate name
        .replace("{{css}}", css);
    std::fs::write(example_dest.join("index.html"), index_processed).unwrap();

    if !args.build_only {
        let host = args.host.unwrap_or_else(|| "localhost".into());
        let port = args
            .port
            .unwrap_or_else(|| "8000".into())
            .parse()
            .expect("Port should be an integer");

        // run webserver on destination folder
        println!(
            "\nServing `{}` on http://{}:{}",
            args.binary_name, host, port
        );
        devserver_lib::run(
            &host,
            port,
            example_dest.as_os_str().to_str().unwrap(),
            false,
            "",
        );
    }
}
