mod client;

use clap::{crate_authors, crate_name, crate_version, App, Arg, ArgMatches, SubCommand};
use client::WorkerClient;
use uuid::Uuid;

fn main() {
    const SUBC_LOGIN: &str = "login";
    const SUBC_START: &str = "start";
    const SUBC_STATUS: &str = "status";
    const SUBC_STOP: &str = "stop";

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
        .subcommand(
            SubCommand::with_name(SUBC_STATUS).about("get the status of a job")
		.arg(Arg::with_name("token")
                     .short("t")
                     .long("token")
                     .help("Authorized JWT token")
                     .takes_value(true)
                     .value_name("TOKEN_VALUE"))
		.arg(Arg::with_name("id")
                     .help("Id of the job to be queried.")
                     .required(true)
                     .value_name("UUID_V4")))
        .subcommand(
            SubCommand::with_name(SUBC_STOP).about("stop a job")
		.arg(Arg::with_name("token")
                     .short("t")
                     .long("token")
                     .help("Authorized JWT token")
                     .takes_value(true)
                     .value_name("TOKEN_VALUE"))
		.arg(Arg::with_name("id")
                     .help("Id of the job to be stopped.")
                     .required(true)
                     .value_name("UUID_V4")))
        .get_matches();

    let debug = matches.is_present("debug");
    let worker_client = WorkerClient::new();
    match matches.subcommand() {
        (SUBC_LOGIN, Some(subc_matches)) => {
            exec_login(&subc_matches, &worker_client, debug);
        }
        (SUBC_START, Some(subc_matches)) => {
            exec_start(&subc_matches, &worker_client, debug);
        }
        (SUBC_STATUS, Some(subc_matches)) => {
            exec_status(&subc_matches, &worker_client, debug);
        }
        (SUBC_STOP, Some(subc_matches)) => {
            exec_stop(&subc_matches, &worker_client, debug);
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

fn exec_start(matches: &ArgMatches, worker_client: &WorkerClient, debug: bool) {
    let token = matches.value_of("token").unwrap_or("");
    let id = matches
        .value_of("id")
        .map(|id| Uuid::parse_str(id).unwrap_or_else(|_| Uuid::new_v4()))
        .unwrap_or_else(Uuid::new_v4);
    let command_line = matches.value_of("command_line").unwrap_or_default();

    if !command_line.trim().is_empty() {
        println!("Starting a job");
        if debug {
            println!("Using token: '{}'", token);
            println!("New job id: '{}'", id.to_string());
            println!("Command line: '{}'", command_line);
        }

        match worker_client.start(token, id, command_line) {
            Ok(()) => {
                println!("New job started with id: '{}'", id.to_string());
            }
            Err(err) => {
                eprintln!("ERR: Start command error: {}", err);
            }
        }
    } else {
        eprintln!("ERR: empty command line.");
    }
}

fn exec_status(matches: &ArgMatches, worker_client: &WorkerClient, debug: bool) {
    let token = matches.value_of("token").unwrap_or("");
    if let Some(id) = matches
        .value_of("id")
        .map(|id| Uuid::parse_str(id).ok())
        .flatten()
    {
        println!("Querying the status of a job");
        if debug {
            println!("Using token: '{}'", token);
            println!("Job id: '{}'", id.to_string());
        }

        match worker_client.status(token, id) {
            Ok(status) => {
                println!("Job '{}' status is {}.", id.to_string(), status);
            }
            Err(err) => {
                eprintln!("ERR: Status command error: {}", err);
            }
        }
    } else {
        eprintln!("ERR: Invalid Id.");
    }
}

fn exec_stop(matches: &ArgMatches, worker_client: &WorkerClient, debug: bool) {
    let token = matches.value_of("token").unwrap_or("");
    if let Some(id) = matches
        .value_of("id")
        .map(|id| Uuid::parse_str(id).ok())
        .flatten()
    {
        println!("Stopping a job");
        if debug {
            println!("Using token: '{}'", token);
            println!("Job id: '{}'", id.to_string());
        }

        match worker_client.stop(token, id) {
            Ok(()) => {
                println!("Job with id '{}' has been stopped.", id.to_string());
            }
            Err(err) => {
                eprintln!("ERR: Stop command error: {}", err);
            }
        }
    } else {
        eprintln!("ERR: Invalid Id.");
    }
}
