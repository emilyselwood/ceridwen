# Ceridwen

A personal search engine. Locally hosted and designed to run entirely in your own home.

## Design requirements

* Must run inside a users network
* Must be multiplatform, at least windows and linux, but don't break mac support unless required
* Must be fast enough
* Must respect target server privacy, robots.txt and search flags
* Must not require run time dependencies (The web ui must not load things from other domains)
* Must not require external services apart from the database

This means that:

* The web front end doesn't need to scale. We absolutely do not support 100s of users. Never mind thousands. It is a personal search engine.
* We don't need to massively optimise the web front end because it will be hosted the local network. Prefer readable over small.
* We do not want to design for scale out. A home server is not going to be a massive cluster. We do not expect this to run on more than one machine.

## Prerequisites

To compile some of the dependencies we need a system implementation of libssl.

On debian and similar:
```
sudo apt install libssl-dev
```

## Installing

TODO: once release builds are defined document how to install here.

## Running

The first time you run ceridwen it will set up a default configuration. You can change this later.

`cargo run`

## Building

The project consists of a single application that is designed to run as a service type application. It runs the web server and runs the crawler in the background.

```
cargo build
```

Test using cargo too, also here in the root of the project

```
cargo test
```

To run the server in development use

```
cargo run
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
