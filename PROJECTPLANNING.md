# Task List and Major themes

* Release builds and packages
* Find and set up example sites
    * Something to spider
* Refactor errors in crawler and lib

## Init process

* Setup empty index
* Prevent overwriting unless flag is passed

## Index

* Work out query list
* Locks on directories to prevent overwriting

## Crawler

* cache robots.txt files
* Ingesters
    * RSS feed
    * Wikipedia
    * rust docs
    * python docs
    * crates.io? 
    * pipy?
    * Full spider of url?
* Clean html page - remove all the cruft and end up with just the text of it. News articles body etc.

## Server

* Only allow connections from local subnet
* Build search results page
* Query api
* favicon

# DONE!!!!!

* Set up different workspaces - make sure it works with actix web and tokio (https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
    * Init (bin)
    * Crawler (bin)
    * Server (bin)
    * Ceridwen (lib)

* Find and set up example sites
    * Something with rss and robots.txt

## init

* Create folders and places to put stuff. ($HOME equivalent)
* Config file loading
* Config file saving

## Server

* Start up
* Static files copied to target dir
* Serve home page
* Build home page and make it hit the query end point

## Index

* Define data structure
* Create api around creating, appending, deleting, and looking up

## Crawler

* look at config
* Engine
* Define useragent
* Read robots.txt files