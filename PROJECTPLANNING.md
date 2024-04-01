# Task List and Major themes

* Release builds and packages
* Find and set up example sites
    * Something to spider

## Init process

* Prevent overwriting unless flag is passed

## Admin tool

* Refactor init to be a sub tool of admin
* Vacuum index
* check index (look for records that look partially deleted)
* CRUD ingester configuration

## Index

* Consider using an sqlite3 database for index
* Work out query list
* Locks on directories/files to prevent overwriting

## Crawler

* lock file to prevent multiple crawlers running at once
* cache robots.txt files
* Ingesters
    * Wikipedia
    * rust docs
    * python docs
    * crates.io? 
    * pipy?
    * Stack overflow: https://archive.org/details/stackexchange
    * Full spider of url?
* Clean html page - remove all the cruft and end up with just the text of it. News articles body etc.
* deep mode on rss feeds, so it will load up the linked page and index that instead of the summery in the feed

## Server

* allow connections from local subnet rather than just localhost
* Make the index page a template
* Add sub page includes for components like the header 
* Make logo lines thicker
* Admin interface
    * Stats
    * configuration editing


# !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
# !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!! DONE !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
# !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

* Set up different workspaces - make sure it works with actix web and tokio (https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
    * Init (bin)
    * Crawler (bin)
    * Server (bin)
    * Ceridwen (lib)

* Find and set up example sites
    * Something with rss and robots.txt

* Refactor errors in crawler and lib

* Logo

## init

* Create folders and places to put stuff. ($HOME equivalent)
* Config file loading
* Config file saving

## Server

* Start up
* Static files copied to target dir
* Serve home page
* Build home page and make it hit the query end point
* Build search results page
* Query api
* build.rs download fonts into resources/static/fonts if they don't exist.
* favicon

## Index

* Define data structure
* Create api around creating, appending, deleting, and looking up

## Crawler

* look at config
* Engine
* Define useragent
* Read robots.txt files
* Ingesters
    * RSS feed