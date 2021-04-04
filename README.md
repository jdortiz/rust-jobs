# rust-jobs

This repository contains my work on the code challenge to be
considered for a DevRel position in Teleport.

I will be writing all my notes about this project in an org-mode file
(plain text) that you can read [here](./Design.md "Design.md").

## Usage

The code is organized in a workspace (in the main directory of this
repo) with three packages: `worker`, `worker-api` and `worker-cli`.

### Worker ###

The worker library can be built and tested from the main directory of
this repo using the following command:
```
% cargo test -p worker
```

It also contains an example that can be executed using:
```
% cargo run --example run_jobs -p worker
```

### Worker-api ###

The web service can be started with the following command:
```
% cargo run -p worker-api
    Finished dev [unoptimized + debuginfo] target(s) in 0.36s
     Running `target/debug/worker-api`
🔧 Configured for debug.
    => address: 127.0.0.1
    => port: 8000
== Snip --
```

This will start the server without TLS. TLS is disabled due to a bug
in `reqwest` with self-signed certificates. In order to enable TLS, you
can use these commands:
```
% cd worker-api
% mkdir private
% cd private
% openssl req -x509 -nodes -days 3650 -newkey rsa:4096 -keyout rsakey.pem -out rsacert.pem
% cd ..
% mv Rocket.toml.off Rocket.toml
% cargo run
```

You can then use `curl` to test the API.

### worker-cli ###

The client set up for non-TLS communications, but the code to work
with TLS is commented out, so can easily be enabled. In order to use
`worker-client` run `cargo build` from the main directory of this
project, then you can use it as in the following example:
```
% target/debug/worker-cli login jorge sakdfjeqwoir 210405000351
Copy, paste and execute:
export TOKEN="eyJ0eXAiOiJKV1QiLC...JyUNVJtm4"
% export TOKEN="eyJ0eXAiOiJKV1QiLC...JyUNVJtm4"
% target/debug/worker-cli start -t $TOKEN "ls -l"  210405000641
Starting a job
New job started with id: '5ab65a18-7755-4c16-bcac-dfe08e23055f'
% target/debug/worker-cli status -t $TOKEN 5ab65a18-7755-4c16-bcac-dfe08e23055f
Querying the status of a job
Job '5ab65a18-7755-4c16-bcac-dfe08e23055f' status is DONE(exit code: 0) (0).
% target/debug/worker-cli output -t $TOKEN 5ab65a18-7755-4c16-bcac-dfe08e23055f
Querying the output of a job
--- BEGIN OUPUT of job 5ab65a18-7755-4c16-bcac-dfe08e23055f ---
total 24
-rw-r--r--  1 jorge  staff    0  5 abr 00:07 5ab65a18-7755-4c16-bcac-dfe08e23055f.txt
-rw-r--r--  1 jorge  staff  676  4 abr 16:43 Cargo.toml
-rw-r--r--  1 jorge  staff   71  4 abr 17:30 Rocket.toml.off
drwx------  4 jorge  staff  128  4 abr 17:26 private
-rw-r--r--  1 jorge  staff   15 30 mar 19:44 rustfmt.toml
drwxr-xr-x  9 jorge  staff  288  4 abr 13:49 src

--- END OUPUT ---
```

## Documentation ##

The code is documented and the generated documentation can be read
using:
```
% cargo doc --no-deps --open
```


## Personal Data

Jorge D. Ortiz-Fuentes and my email is jdortiz@gmail.com.

March 20th, 2021 Madrid, Spain
