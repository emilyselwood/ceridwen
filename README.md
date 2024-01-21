# Ceridwen

A personal search engine. Locally hosted and designed to run entirely in your own home.

## Design requirements

* Must run inside a users network
* Must be multiplatform, at least windows and linux, but don't break mac support unless required
* Must be fast enough
* Must respect target server privacy, robots.txt and search flags
* Must not require run time dependencies (The web ui must not load things from other domains)
* Must not require external services (No database to set up and host, docker not required)

This means that:

* The web front end doesn't need to scale. We absolutely do not support 100s of users. Never mind thousands. It is a personal search engine.
* We don't need to massively optimise the web front end because it will be hosted the local network. Prefer readable over small.
* This means we do not want to design for scale out. A home server is not going to be a massive cluster. We do not expect this to run on more than one machine.

## Installing

TODO: once release builds are defined document how to install here.

## Running

TODO: document setup process and how to run.

## Building

This project uses workspaces to allow for multiple mains that don't interact with each other. This is needed because of the actix_web based server application which has it's own main function generation that doesn't play well with others.

The project is split into four parts. Three commands and one library

* ceridwen - the library with shared code.
* init - a tool to set up the config and index files as needed.
* crawler - the indexing tool, should be run periodically
* server - the web server that allows you to search

Build using cargo in the root of the project (IE: where this read me file is.) This will automatically build all the workspaces

```
cargo build
```

Test using cargo too, also here in the root of the project

```
cargo test
```

To run the individual parts of the project we also use cargo and need to specify the workspace with the `-p` argument

```
cargo run -p ceridwen-init
cargo run -p ceridwen-crawler
cargo run -p ceridwen-server
```

TODO: document how to build a release package

## For web site owners

If you've arrived here because you've found Ceridwen in your logs, hello! If you want to block any Ceridwen instance from indexing your web site you can using the robots.txt functionality that most search engines support. We use the user agent `ceridwen-crawler`

Currently we don't support wild card path matching (its on the todo list) so we will work with path prefix matches only.

Here for your convenience is an example:

```
User-agent: ceridwen-crawler
Disallow: /something/you/do/not/want/indexed/
```

`/` is supported as a Disallow rule so you can block all instances of ceridwen from indexing your site if you wish. We also support the wild card (`*`) user agent and will respect that, if `ceridwen-crawler` is not specified separately. 

Please also note that unlike other search engines there is no central host of Ceridwen, if an instance of Ceridwen is misbehaving please contact the instance owner (or just block them at your firewall)