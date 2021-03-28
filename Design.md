# Introduction #

This file contains my design approach to the coding challenge.

I have decided to use Rust for my code.  So, I will try to justify my
decisions and the possible trade-offs within the context of the Rust
ecosystem.

According to the requirements, there are three pieces that I need to
build: the library that will run the jobs, the HTTPS API interface to
the library and the client that will submit commands to be run.  I
will start explaining the things that are common to the three of them
and the top level design.  Then I will get into the specifics of
each one of the pieces.  Finally, I will end up with with some
trade-offs and to-dos.


# Code #

## Initial Code Organization ##

I would like all the pieces of this coding challenge to be available
in the same repository.  Cargo (Rust's building tool) allows to have
one library and many binaries in the same package.  However, if I put
the three pieces in a single package I will be restricted to using the
same configuration options for all of them and, what is more
important, I wouldn't be able to push the library crate, for example,
to crates.io.

There is another way to organize them, though.  Cargo has a feature
called _workspaces_, which allows us to manage multiple (related)
packages while keeping consistency among them and having them in the
same directory.  I will use a workspace with three packages.  I could
put the library and the `worker-api` in the same package, but in a
future it might make sense that the library is imported as a crate in
other Rust projects.

## Code Style ##

I will be using `rustfmt` for making sure that my formatting is
consistent.  I have configured **Emacs**  with the `rustic` package
and `rustfmt` is used every time I save a file.

`rustfmt` allows to specify [different
settings](https://github.com/rust-lang/rustfmt/blob/master/Configurations.md
"rustfmt configuration") to make the code format suit your needs.  In
this case, I will only use the `edition` setting.  I will be using the
"2018" edition which is the stable one and has (IMO) a nicer way to
work with crates than "2015".  "2021" edition will be released [later this
year](https://blog.rust-lang.org/inside-rust/2021/03/04/planning-rust-2021.html
"Planning the Rust 2021 Edition") and should be considered if I were to
evolve this package.

Finally, regarding the structure of the code itself, I am a strong
proponent of Clean Code so I prefer readability over conciseness.  That
means that I will use longer descriptive names for variables and
private functions when needed to make the code clear, avoiding longer
functions/methods.


# Top Level Design #

There are three pieces that need to be designed for this challenge:
the client (`worker-cli`), the API-server (`worker-api`), and the
library (`worker`) that implements the execution of the jobs and the
related features.

## Domain Logic in the library ##

In my opinion, the library is the right place to put all the domain
logic of this application.  There are several reasons for that:
- Maximize code reuse.
- Encapsulation.
- Improve testability.

Let me explain each of them briefly.

### Maximize Code Reusability ###

If we separate, the logic from its user interface, which in the case
of the `worker-api` is the HTTPS API, we could easily implement other
interfaces, like the gRPC one requested for level 3.  It would be as
simple as writing the new interface and invoking the right library
methods.  Martin Fowler has a [very nice
article](https://martinfowler.com/eaaDev/uiArchs.html) explaining this
same concept for graphical user interface architectures.

For this approach to be successful, I would need to follow some kind
of layering organization in which the inner layers don't depend on
outer layers using the [Dependency Inversion
Principle](https://en.wikipedia.org/wiki/Dependency_inversion_principle
"DIP") as suggested in architectures like the [Clean
Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html),
[Hexagonal/Ports and Adapters
Architecture](https://en.wikipedia.org/wiki/Hexagonal_architecture_(software))
or [Onion
Architecture](https://jeffreypalermo.com/2008/07/the-onion-architecture-part-1/).
The layer for the domain logic and the entities should be the most
inner one.  Moving all the domain logic into a separate library, and
the lack of support for cyclic dependencies in Cargo (same as in Go
modules) will help me to avoid any dependency on the outer layer
(`worker-api`) from the library (`worker`).

### Encapsulation ###

In the `worker` library I will be defining the data types used for the
job and the related operations for those types.  While _encapsulation_
is commonly understood in the context of Object Oriented Programming,
it is also relevant at the level of modules/libraries.  Having all
that logic implemented in a different library would simplify
refactoring that logic without affecting its interaction with
`worker-api`, because the implementation details would remain hidden
to it.

### Improve Testability ###

Let me make this clear: I should be able to have the same unit tests
no matter whether the jobs are defined in its own library or in the
same package as the API.  However, I would be obtaining some
additional benefits in terms of testability:
- I would get integration tests for this package covering all the
  parts of the domain logic, without having to worry about the HTTP
  parts and not affected by changes to them.
- Test coverage will be separated for the domain and the interface.
  Many companies like to put most of their effort in the unit test
  coverage of their domain logic, while they aren't that worried about
  the unit tests for the interface, because they tend to be harder to
  write, more fragile, and better covered by end-to-end tests.
- I can still create integration tests for the `worker-api` and its
  `worker` dependency that will focus on the end to end behavior of my
  code.  Those test would belong in the `/test` directory of the
  `worker-api` package.

## REST API ##

Although there is a explicit requirement to use an HTTPS API, the
challenge doesn't state how that API should be.  I have decided to
implement a REST API, because of their simplicity and ability to
fulfill the requirements for the task at hand.

There are other alternatives like the gRPC (mentioned for level 3), or
graphQL.  While the former will in fact be a very good alternative,
particularly for non customer facing APIs, the later would be a better
alternative for a more complex model schema with more relations
allowing more flexibility in the client to make custom queries.

Coming back to the implementation of the REST API, I will stay at
level 2 in the [Richardson's
model](https://martinfowler.com/articles/richardsonMaturityModel.html),
because I don't think that including the links here is going to be
very useful for a first iteration of this application.

I provide more details below about the API itself and its
authentication and authorization mechanisms.

## Concurrency ##

Rust has finally stabilized its mechanism for asynchronous
computations: async/await + futures.  However, they require run-time
support and the two big contenders here are
[`async_std`](https://docs.rs/async-std/1.9.0/async_std/), the async
version of the standard library, and [`tokio`](https://tokio.rs/) .  I
will stay with Tokio for this project because it is more mature and
because the other crates are using it.

I will store the shared data in a hash map protected by a
`std::sync::Mutex` and an `std::sync::Arc` pointer.  This two
components will allow me to share the data among Tokio tasks while
protecting them from race conditions.

I will hold the child process in the data to be able to keep track of
it it while running and query its status.  According to [its Tokio
implementation](https://docs.rs/tokio/1.4.0/tokio/process/struct.Child.html)
in implements `Send` and `Sync` so I should be ok having them in the
shared data of the API.

# Design of the Packages #

## Worker ##

The library will define the `Job` type as a struct.  This struct will
contain fields for: the id, the command string, the status, the owner
of the job (that I explain below) and the child process while in
progress (Option).

The `JobStatus` type is an enum with associated values that has four
variants: `InProgress`, `Stopped` (on user request), and `Done`. The
last variant will have the exit status value of the process.

The job will be created using the `new` static function of the `Job`
type as it is customary in Rust. The command will be launched when the
`Job` is created with the `Job::new()` function.  Having done so in a
separate method would to allow to create jobs that can be executed
later, in order to enable some kind of scheduling mechanism or even
queues.  We have decided to take the simpler approach here.

I have considered the option to make `Job` implement the `Executable`
trait that would abstract the execution capabilities of a type.  It
would make a lot of sense if would considering other other things that
could be executed by the server, e.g. database jobs, but it seems
unnecessary at this moment.

In order to spawn a command, I will be using the Tokio version of
`std::process::Command`, because I want it to be asynchronous.  The
instance of `tokio::process::Child` will be stored in the `Job`, so it
can be killed on user request.

Finally, I will redirect the stdout and the stderr of the command to a
file called `<job_uuid>.text`. This will be preserved as the output of
that `Job`.

## Worker-api ##

This package will implement the REST API that I mentioned previously.

### Security ###

Being a publicly exposed API, security is a very important part of the
design of this package.  In this section I would like to cover the
confidentiality in the communications, the authentication, the
authorization and some additional considerations.

#### Communications ####

There are well known best practices to configure HTTPS connections,
like the recommendations covered in [this post for
Go](https://blog.cloudflare.com/exposing-go-on-the-internet/).  Sadly
not all this options are available in Rust, and some depend on the web
framework that you decide to use.
[Rustls](https://github.com/ctz/rustls) is the most used native
implementation of TLS in Rust and can be integrated with other crates
(web frameworks).  However, most of those web frameworks don't allow
to configure the TLS communication in any way, so the desired settings
are either configured by default or not available.

There are some things that can be done, like using proper certificates
with at least 2048 bits keys.  But others, like using mTLS, are not
available to most web frameworks.

Rustls only supports TLS1.2 and TLS1.3, so there is no need to
configure it to avoid support for any version prior to 1.2.


#### Authentication ####

Regarding authentication, I will limit the scope to the one described
in the introduction to the coding challenge.  I will assume that
requests with a valid token are authenticated and authorized as if it
were an API key.

#### Authorization ####

I will be using a bearer token as specified in
[RFC-6750](https://tools.ietf.org/html/rfc6750).  That means that
instead of keeping record of each user session in the persistence, as
we have traditionally done, every request will contain a token in the
header that will identify the user.  That token should be generated
after some authentication that we will skip here, and it should be a
signed and encrypted string with the relevant user information, but we
will be using a random string here.

That authentication scheme doesn't require session cookies and can be
strengthened by limiting the validity of the token to a smaller time
length.

Basically, any client with a token is allowed to create jobs. In
future versions we could limit the number of jobs per user or restrict
the kind of jobs a user can launch. A user owns the `Job`s it creates.

Access to the data and output of a job as well as the ability to stop
it, are restricted to the owner of the job.  All of the `Job` methods
include an argument (`as_user`) that is filled with the user id
contained (and signed) in the JWT token.  In each operation, the value
is compared with the owner attribute to grant access.
#### Other Considerations ####

Using UUIDs for the jobs provides an extra layer of confidentiality.
While it doesn't help to protect from un-authorized access, it does
make it harder to guess the URI for a job.  Trying to find other
user's jobs becomes a brute force task rather than a trivial one if
the ids are incremental integer numbers.

### API ###

There are four operations that will be provided by the API:
- Start a new job.
- Stop an existing job.
- Get the status of an existing job.
- Get the output of an existing job.

#### Start a New Job ####

This is the creation of a resource and should be done with the HTTP
POST method.

```
HTTP method: POST
URI: /v1/jobs
Parameters: None
Header: token
Body: { "id": "<job_uuid>", "command": "<some linux command with
arguments>" }
Responses:
- 201 -> Successful creation
- 400 -> Bad request (most likely bad JSON)
- 401 -> Unauthorized (No token)
- 409 -> Conflict (There is a job with that uuid)
```

On success, a new job will be created and start executing.

#### Stop an Existing Job ####

This corresponds to updating an existing resource.  It will work set
the status of the `Job` to `Stopped` (instead of `InProgress` or
`Done`).  This command cannot be reversed and it is idempotent.
Access to the data of the job is still allowed.

```
HTTP method: DELETE
URI: /v1/jobs/<job_uuid>
Header: token
Parameters: None
Body: Empty
Responses:
- 200 -> Job successfull cancelled.
- 400 -> Bad request
- 401 -> Unauthorized (No token)
- 403 -> Forbiden (job created by another user)
- 404 -> Job not found
```

#### Get the Status of an Existing Job ####

This corresponds to accessing an existing resource.

```
HTTP method: GET
URI: /v1/jobs/<job_uuid>
Parameters: None
Header: token
Body: Empty
Responses:
- 200 -> Job successfull queried. Body contains the job data. '{
"status": "done", "exit_status": 0 }'
- 400 -> Bad request (Wrong uuid format)
- 401 -> Unauthorized (No token)
- 403 -> Forbiden (job created by another user)
- 404 -> Job not found
```

#### Get the Output of an Existing Job ####

This also corresponds to accessing an existing resource.

```
HTTP method: GET
URI: /v1/jobs/<job_uuid>/output
Parameters: None
Header: token
Body: Empty
Responses:
- 200 -> Job successfull queried. Response contains the file as
text/plain with the job output.
- 400 -> Bad request (Wrong uuid format)
- 401 -> Unauthorized (No token)
- 403 -> Forbiden (job created by another user)
- 404 -> Job not found
```

### Implementation ###

Although it would be possible to write everything from scratch in Rust
using, for example, `std::net::TcpListener`, that would be unpractical
and time consuming.  Instead, I will use some crate that simplifies
the process of writing an API in Rust, i.e. a web framework (similar
to [Gorilla/mux](https://github.com/gorilla/mux) or
[Gin](https://github.com/gin-gonic/gin) for Go).

It is worth mentioning that the Rust ecosystem is not as mature as the
Golang one, so there are several options, but all of them with
limitations and trade-offs. Let me introduce the ones that I have
considered and justify my final choice.

#### Hyper ####

[Hyper](https://github.com/hyperium/hyper) is a very mature library
with several customization capabilities and uses `tokio` for async
computations.  However, it is a relatively low level library, so things
like routing and parameter extraction aren't provided by it.  (This
would be a much simpler version of what is provided by `net/http` in
Go).

Integration with `Rustls` is possible, but requires another crate:
[`hyper-rustls`](https://github.com/ctz/hyper-rustls).

#### Warp ####

[`warp`](https://github.com/seanmonstar/warp) is a very promising web
server framework with nice syntax and very flexible and based on
hyper. It uses _filters_ to route requests and supports
async/await.  It supports TLS with server certificates, but client
certificates are not supported to the best of my knowledge.  Also the
documentation is still a little bit weak.

#### Actix ####

[`actix`](https://github.com/actix/actix) is with no doubt the most
mature and configurable web framework for Rust. However, it is based
on the Actor Model and I am not familiar with it.  It would be a nice
alternative to explore, should I have some extra time.  Anyhow, I
might consider switching to it if I get into any troubles with my
choice.

#### Rocket ####

[`rocket`](https://rocket.rs/) is probably the most awaited web
framework for Rust.  It has been under active development for the last
4 years and it shows.  Until very recently it could only be used with
the nightly version of the Rust tool-chain. Version 4.7 is finally
usable with stable Rust and version 0.5, still, in pre-release
includes support for async/await. I will be using 0.5 for my project
(fingers crossed).

Although it has some nice macros that simplify development, it is
still lacking in some aspects.  TLS configuration is one of them. It
allows to use TLS with a server certificate and a key, but that is
it. Another thing that is not ready yet, is the
authentication/authorization part. It does support cookies and API
keys, but no JWT and oauth2 is only available as a third party crate
that hasn't been updated in the last 6 months (before the last two
versions of Rocket).

I will start with this one as my best candidate, but I might change
and use some other, most likely Warp or Actix.

## Worker-cli ##

This will be a command line utility and I will use the [`clap`
crate](https://clap.rs/).  I will most likely use `reqwest` to
implement the communication with the server and the command line will
be simple and self documented.

The token will be provided as a command line parameter and the command
will be an argument using quotes if required.

### CLI UX ###

Each of the interactions with the API will be implemented as a
subcommand of `worker-cli`.

#### Start a Job ####

It is implemented with the start command, that takes an argument for
the command line to be executed and a parameter with the token. There
is an optional parameter for the UUID.  If the id is omitted, it will
generate a new one and it will be printed with the result of the
execution. The output will inform the user if it was successful or the
error otherwise.

```
% worker-cli start -t <TOKEN> -i <UUID> "ls -l"
Success: Job created.
```

#### Stop a Job ####

It is implemented with the stop command, that takes an argument for
the UUID of the job to be stopped and a parameter for the token.  The
output will inform the user if it was successful or the error
otherwise.

```
% worker-cli stop -t <TOKEN> <UUID>
Failed: Job not running.
```

#### Get Job Status ####

It is implemented with the status command, that takes an argument for
the UUID of the job to be queried and a parameter for the token.  The
output will inform the user of the status (`InProgress`, `Stopped`, or
`Done`). If the status is done if it will inform when it has received
a signal using `std::process::ExitStatus`.

> In Unix terms the return value is the exit status: the value passed
> to exit, if the process finished by calling exit. Note that on Unix
> the exit status is truncated to 8 bits, and that values that didn’t
> come from a program’s call to exit may be invented the runtime
> system (often, for example, 255, 254, 127 or 126).
>
> On Unix, this will return None if the process was terminated by a
> signal. ExitStatusExt is an extension trait for extracting any such
> signal, and other details, from the ExitStatus.

```
% worker-cli status -t <TOKEN> <UUID>
Job was stopped.
```

#### Get Job Output ####

It is implemented with the output command, that takes an argument for
the UUID of the job to be queried and a parameter for the token.  The
output of that job will be printed to the stdout or an error to stderr
if there is any problem.

```
% worker-cli output -t <TOKEN> <UUID>
--- BEGIN OUPUT of the command ---
...
--- END OUPUT of the command ---
```

# Trade-offs and To Dos for Evolving this Code #

A design document wouldn't be complete without talking about things
that could have been done in a different way.  I have decided to split
this section into two parts.  On the one hand, the trade-offs are the
result of a conscious decision in which, in my opinion, the cons were
outweighed by the pros.  They depend on the actual context for the
project and are in many cases harder to change later.  On the other
hand, the to-dos are simply things that I didn't implement to keep the
scope of this challenge simple, as requested.

## Trade-offs ##

### Persistence in an Outter Layer ###

When I talked about placing the domain logic in the `worker` library,
I didn't mention persistence at all. In most applications, persistence
is a(n outter) layer located above the domain layer and the domain
uses the dependency inversion principle to talk to it using some kind
of entity gateway.

I didn't need that part for this first iteration
of this project.  It was easier to return Job structs, when they are
created in the domain, have a hash map in `worker-api` that holds all
of those instance to keep control of them.

I believe that this can be easily changed at this stage of the project,
should I need to. I could implement use cases, like `GetStatusUseCase`
that:
- would take the UUID of the job,
- talk to an abstraction of the persistence to retrieve the data and
  put it into a `Job` instance,
- execute the `get_status()` method,
- and return its output to the presentation layer.

### Level 2 REST API ###

I have decided to stay at the level 2 of the maturity level.  I could
implement level 3 later, but it would be a breaking change that would
require bumping up the API version.  The reason for that breaking
change is that the responses should be organized in a different way,
like the one specified in the [json-api](https://jsonapi.org/), so
clients implementing the old version would be unable to talk to the
new API.

In a project meant for actual usage, I would spend some extra time and
implement a Level 3 REST API.

### Running Jobs at Creation ###

As I mentioned in the design of the `worker` library, I decided that
`Job`s should not be started at creation time.  However, if the job is
started when created I wouldn't need the `Pending` variant of the
`Status` enum.

This decision is easy to revert at this time.


## To-Dos ##

## Implementing Environment Variables for Jobs ##

Both the implementations of the `Command` type (the one in the
standard library and the one from Tokio) allow to set the environment
of the process before it is run.

That would be a very useful feature, as shown by many CI engines,
e.g. Travis.

## Proper authentication ##

I could use an OpenID Connect provider (GitHub, Google, Apple,
Facebook...), authenticate via Oauth2 and use the `id_token` in the
owner field of the `Job`s.
