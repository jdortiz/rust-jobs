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
% cargo run--example run_jobs -p worker
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
