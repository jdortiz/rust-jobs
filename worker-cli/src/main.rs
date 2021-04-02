mod client;

use clap::{crate_authors, crate_name, crate_version, App, Arg, ArgMatches, SubCommand};
use client::WorkerClient;
use uuid::Uuid;

fn main() {
    const SUBC_LOGIN: &str = "login";
    const SUBC_START: &str = "start";

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about("Creating more jobs than Monster!")
        .author(crate_authors!(""))
	.arg(Arg::with_name("debug")
             .short("d")
             .long("debug")
             .help("Show debug information"))
        .subcommand(
            SubCommand::with_name(SUBC_LOGIN)
                .about("login and get a token")
                .arg(Arg::with_name("user")
                        .help("Worker-api user name")
                        .required(true))
                .arg(Arg::with_name("password")
                        .help("Worker-api password")
                        .required(true)))
        .subcommand(
            SubCommand::with_name(SUBC_START).about("start a job")
		.arg(Arg::with_name("token")
                     .short("t")
                     .long("token")
                     .help("Authorized JWT token")
                     .takes_value(true)
                     .value_name("TOKEN_VALUE"))
		.arg(Arg::with_name("id")
                     .short("i")
                     .long("id")
                     .help("Id of the new job to be created (It must be unique). If ommitted, one is generated")
                     .takes_value(true)
                     .value_name("UUID_V4"))
                .arg(Arg::with_name("command_line")
                        .help("Command line to be executed in the job")
                        .required(true)))
        .get_matches();

    let debug = matches.is_present("debug");
    let worker_client = WorkerClient::new();
    match matches.subcommand() {
        (SUBC_LOGIN, Some(subc_matches)) => {
            exec_login(&subc_matches, &worker_client, debug);
        }
        (SUBC_START, Some(subc_matches)) => {
            exec_start(&&subc_matches, debug);
        }
        _ => {
            eprintln!("ERR: Unexpected subcommand")
        }
    }
}

fn exec_login(matches: &ArgMatches, worker_client: &WorkerClient, debug: bool) {
    let user = matches
        .value_of("user")
        .expect("ERR: Required argument 'user' is unexpectedly missing");
    let password = matches
        .value_of("password")
        .expect("ERR: Required argument 'password' is unexpectedly missing");
    if debug {
        println!("Login to worker-api as user '{}'", user);
    }
    match worker_client.login(user, password) {
        Ok(token) => {
            println!("Copy, paste and execute:");
            println!("export TOKEN={:?}", token);
        }
        Err(err) => {
            eprintln!("ERR: Login error: {}", err);
        }
    }
}

fn exec_start(matches: &ArgMatches, debug: bool) {
    let token = matches.value_of("token").unwrap_or("");
    let id = matches
        .value_of("id")
        .map(|id| Uuid::parse_str(id).unwrap_or_else(|_| Uuid::new_v4()))
        .unwrap_or_else(Uuid::new_v4);

    println!("Starting a job");
    if debug {
        println!("Using token: '{}'", token);
        println!("New job id: '{}'", id.to_string());
    }
}
