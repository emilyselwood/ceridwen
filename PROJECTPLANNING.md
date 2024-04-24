# Task List and Major themes

* Release builds and packages
* Find and set up example sites
    * Something to spider

## Index

## Crawler

* Schedule runs, on start, and once per day at 3am
* get config when starting run
* Stats recording
* cache robots.txt files
* Ingesters
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
        * Adding sites
* Admin API


# !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
# !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!! DONE !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
# !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

* Refactor everything into one process.
* Find and set up example sites
    * Something with rss and robots.txt

* Logo

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
* lock file to prevent multiple crawlers running at once
* Ingesters
    * RSS feed
* Wikipedia