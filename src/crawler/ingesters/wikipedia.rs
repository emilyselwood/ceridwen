use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use crate::config::Config;
use crate::config::Ingester;
use crate::index_sled::Index;
use crate::data::Page;
use crate::utils::temp_dir;

use bytes::Buf;
use bzip2::read::MultiBzDecoder;
use flume::Receiver;
use log::debug;
use log::warn;
use quick_xml::events::Event;
use tokio::task::JoinHandle;
use url::Url;
use crate::error::Error;
use log::info;
use reqwest::Client;
use rss::Channel;
use time::format_description;
use time::macros::format_description;

use crate::crawler::web_client;

const WIKIPEDIA_DUMP_URL: &str =
    "https://dumps.wikimedia.org/enwiki/latest/enwiki-latest-pages-articles-multistream.xml.bz2";

const WIKIPEDIA_DUMP_RSS: &str = 
    "https://dumps.wikimedia.org/enwiki/latest/enwiki-latest-pages-articles-multistream.xml.bz2-rss.xml";


pub(crate) async fn process_wikipedia(
    ingester_config: Ingester,
    config: Config,
    index: Index,
) -> Result<(), Error> {
    let mut client = web_client::get_client(&config)?;
    let num_tasks = config.crawler.workers;
    // Load up the rss file and see if there is a new file.
    let last_update = get_rss_date(&mut client).await?;
    info!("got {} as the last update time!", last_update);
    if last_update < ingester_config.last_update {
        info!("wikipedia dump last updated {}, which is before {} when we last read it.", last_update, ingester_config.last_update);
        return Ok(());
    }

    // download the big archive to a temp folder somewhere. This must be chunked it will not fit in memory.
    info!("Downloading wikipedia archive");
    let archive_file = download_archive(&client, &last_update).await?;
    info!("Download completed. {:?}", archive_file);

    // Now to process the archive file...

    let file = File::open(archive_file)?;
    let bzip2_decoder = MultiBzDecoder::new(file);
    let buf_reader = BufReader::new(bzip2_decoder);
    let mut xml_reader = quick_xml::Reader::from_reader(buf_reader);
    
    // spin out index creation into other threads.
    // Flume, for channel and create a bunch of threads to do work.

    let (tx, rx) = flume::bounded(num_tasks);
    let mut workers: Vec<JoinHandle<()>> = Vec::new();
    info!("Starting {} page workers", config.crawler.workers);
    for _worker in 0..config.crawler.workers {
        workers.push(tokio::spawn(page_processor(config.clone(), index.clone(), rx.clone())))
    }
    info!("Starting page feed");
    // Now load up the queue
    while let Some(page) = read_page(&mut xml_reader)? {
        let result = tx.send_async(page).await;
        if let Err(error) = result {
            warn!("Error sending page into channel: {:?}", error);
            panic!("Could not send to channel!");
        }
    }
    info!("Done reading pages... waiting for processors to complete");
    // drop the sender so that the rx get closed eventually and everything slowly stops.
    drop(tx);

    // Wait for the workers to return
    for fut in workers {
        fut.await?;
    }

    info!("wikipedia page processors complete");

    Ok(())
}

async fn get_rss_date(client: &mut Client) -> Result<time::OffsetDateTime, Error> {
    let rss_bytes = web_client::get(client, WIKIPEDIA_DUMP_RSS).await?;
    let channel = Channel::read_from(rss_bytes.reader())?;

    let date_string = channel
        .items
        .first()
        .map(|i| i.pub_date()).unwrap_or(None);

    if date_string.is_none() {
        return Err(Error::WikipediaMissingDate)
    }

    // expected format: Fri, 02 Feb 2024 09:03:51 GMT
    Ok(time::OffsetDateTime::parse(
        date_string.unwrap(),
        &format_description::well_known::Rfc2822,
    )?)
}


async fn download_archive(client: &Client, update_date : &time::OffsetDateTime) -> Result<PathBuf, Error> {

    let file_name = format!("enwiki-latest-{}.xml.bz2", update_date.format(format_description!("[year]_[month]_[day]_[hour]_[minute]_[second]"))?);

    let target_path = temp_dir().join("wikipedia").join(file_name);
    if target_path.exists() {
        warn!("Intend download target already exists. Reusing it. If this is incomplete or corrupt please delete it. {:?}", target_path)
    } else {
        web_client::get_to_file(client, WIKIPEDIA_DUMP_URL, target_path.as_path()).await?;
    }

    Ok(target_path)
}

async fn page_processor(config: Config, index:Index, rx: Receiver<Page>) {
    while let Ok(page) = rx.clone().into_recv_async().await {
        let start_time = time::Instant::now();
        let title = page.title.clone();
        info!("processing page: {title}");
        
        let result = process_page_inner(&config, page, &index).await;
        if let Err(error) = result {
            warn!("Error Processing page {}: {}", title, error);
            panic!("errored processing wikipedia page");
        }
        info!("done processing page {}! took {}", title, start_time.elapsed());
    }
}

// async fn process_page(config: Config, page: Page, index: Index) {
//     let title = page.title.clone();
//     info!("processing page: {title}");
//     let start_time = time::Instant::now();
//     let result = process_page_inner(config, page, index).await;
//     if let Err(error) = result {
//         warn!("Error Processing page {}: {}", title, error);
//         panic!("errored processing wikipedia page");
//     }
//     info!("done processing page {}! took {}", title, start_time.elapsed());
// }

async fn process_page_inner(config: &Config, page: Page, index: &Index) -> Result<(), Error> {

    // filter out pages that are just redirects and 
    if page.content.starts_with("#REDIRECT") {
        debug!("skipping {} as its a redirect page", page.url);
        Ok(())
    } else {

        // Strip formatting characters and things we don't need
        let stripped_page = strip_page(&page);

        // info!("{}", &page.content);
        match index.add_page(&stripped_page, config.crawler.min_update_interval).await {
            Ok(()) => Ok(()),
            Err(e) => {
                warn!("Could not index page {}: {:?}", stripped_page.title, e);
                warn!("{}", stripped_page.content);
                Err(e)
            }
        }
    }
}

fn read_page(xml: &mut quick_xml::Reader<BufReader<MultiBzDecoder<File>>>) -> Result<Option<Page>, Error> {
    #[derive(Debug)]
    enum State {
        Limbo1,
        TitleStarted,
        Title { title: String },
        Limbo2 { title: String },
        TextStarted { title: String },
        Text { title: String, text: String },
        Limbo4 { title: String, text: String },
    }

    let mut buffer = Vec::new();
    let mut state = State::Limbo1;
    let mut done = false;

    while !done {
        let event =  xml.read_event_into(&mut buffer)?;
        
        state = match (state, event) {
            (State::Limbo1, Event::Eof) => {
                return Ok(None);
            }
            (State::Limbo1, Event::Start(data)) if data.name().into_inner() == b"title" => {
                State::TitleStarted
            }
            (limbo1 @ State::Limbo1, _) => limbo1,
            (State::TitleStarted, Event::Text(data)) => {
                let title = data.unescape()?.into_owned();
                State::Title { title }
            }
            (State::Title { title }, Event::End(data)) 
                if data.name().into_inner() == b"title" => {
                State::Limbo2 { title }
            }
            (State::Limbo2{title}, Event::Start(data)) if data.name().into_inner() == b"text" => {
                State::TextStarted{title}
            }
            (limbo2 @ State::Limbo2 { .. }, _) => limbo2,
            (State::TextStarted { title}, Event::Text(data)) => {
                let text = data.unescape()?.into_owned();
                State::Text { title, text }
            },
            (State::Text { title, text }, Event::End(data))
                if data.name().into_inner() == b"text" =>
            {
                State::Limbo4 { title, text }
            },
            (limbo4 @ State::Limbo4 { .. }, Event::End(data)) 
                if data.name().into_inner() == b"page" => {
                done = true; 
                limbo4
            },
            (limbo4 @ State::Limbo4 { .. }, _) => limbo4,
            (state, event) => {
                return Err(Error::InvalidState(format!("got into state: {state:?} processing event: {event:?}")))
            },
        };

        buffer.clear();
    }

    if let State::Limbo4 { title, text } = state {
        return Ok(Some(Page { url: create_url(&title)?, title, content: text }));
    }
    Ok(None)
}


fn create_url(title: &str) -> Result<Url, Error> {
    // NOTE: this is not a true slugify operation. it only seems to worry about spaces. Brackets and capitalisation remain 
    let slug = title.replace(' ', "_");
    Ok(Url::parse(&format!("https://en.wikipedia.org/wiki/{slug}"))?)
}


fn strip_page(page: &Page) -> Page {
    // filter special sections here
    let mut new_content = filter_between(&page.content, '{', '{', '}', '}');
    new_content = filter_between(&new_content, '{', '|', '|', '}');
    new_content = filter_square_brackets(&new_content);

    new_content = new_content.replace("==", "");

    Page {
        url: page.url.clone(),
        title: page.title.clone(),
        content: new_content,
    }
}


// Filter a string, removing everything between two instances of start and end
fn filter_between(content : &str, start1 : char, start2: char, end1: char, end2: char) -> String {
    // stack or level based scanning?
    let mut result : String = String::new();

    enum State {
        Outside,
        Starting(u32),
        Inside(u32),
        Ending(u32),
    }

    let mut state = State::Outside; 

    for c in content.chars() {
        match state {
            State::Outside => {
                if c == start1 {
                    state = State::Starting(0);
                } else {
                    result.push(c);
                }
            },
            State::Starting(i) => {
                if c == start2 {
                    state = State::Inside(i+1);
                } else {
                    if i <= 1 { 
                        state = State::Outside;
                    } else {
                        state = State::Inside(i);
                    }
                    // we must have had a single opening bracket before this so add it now.
                    result.push(start1);
                    result.push(c);
                }
            },
            State::Inside(i) => {
                if c == end1 {
                    state = State::Ending(i);
                } else if c == start1 {
                    state = State::Starting(i);
                }
            },
            State::Ending(i) => {
                if c == end2 {
                    if i <= 1 {
                        state = State::Outside;
                    } else {
                        state = State::Inside(i-1);
                    }
                }
            },
        }
        
    }

    result
}

fn filter_square_brackets(content: &str) -> String {
    let mut result : String = String::new();

    enum State {
        Outside,
        Starting(usize),
        Inside(usize),
        Ending(usize),
    }

    let mut state = State::Outside; 

    let mut buffer = Vec::new(); 

    for c in content.chars() {
        match state {
            State::Outside => {
                if c == '[' {
                    state = State::Starting(0);
                } else {
                    result.push(c);
                }
            },
            State::Starting(i) => {
                if c == '[' {
                    if i+1 >= buffer.len() {
                        buffer.push(String::new());
                    } else {
                        buffer[i+1] = String::new();
                    }
                    state = State::Inside(i+1);
                } else if i <= 1 {
                    state = State::Outside;
                    // we must have had a single opening square bracket before this so add it now
                    result.push('[');
                    result.push(c);
                } else {
                    state = State::Inside(i);
                    buffer[i-1].push('[');
                    buffer[i-1].push(c);
                }
                   
                
            },
            State::Inside(i) => {
                
                match c {
                    ']' => {
                        state = State::Ending(i);
                    }
                    '|' => {
                        buffer[i-1] = String::new();
                    }
                    '[' => {
                        state = State::Starting(i);
                    }
                    _ => {
                        buffer[i-1].push(c);    
                    }
                }

            },
            State::Ending(i) => {
                if c == ']' {
                    if i <= 1 {
                        result.push_str(&buffer[i-1]);
                        state = State::Outside;
                    } else {
                        let buffer_value = buffer[i-1].clone();
                        buffer[i-2].push_str(&buffer_value);
                        state = State::Inside(i-1);
                    }
                }
            },
        }        
    }

    result
}

#[cfg(test)]
mod tests {
    use crate::crawler::ingesters::wikipedia::filter_between;
    use crate::crawler::ingesters::wikipedia::filter_square_brackets;


    #[test]
    fn test_filter_between() {

        let cases = [
            ("here {{is a thing}} opens", "here  opens"),
            ("here {{is a thing {{with nesting stuff}}}} opens", "here  opens"),
        ];

        for (input, expected) in cases.into_iter() {
            let result = filter_between(input, '{', '{', '}', '}');
            assert_eq!(result, expected);
        }

    }


    #[test]
    fn test_square_brackets() {
        let cases = [
            ("some text [[an article name|article]] foo bar", "some text article foo bar"),
            ("some text [[an article name]] foo bar", "some text an article name foo bar"),
            ("some text [[an article name|something [[nested]]]] foo bar", "some text something nested foo bar"),
            ("some text [[an article name|something [[with sub bits|nested]]]] foo bar", "some text something nested foo bar"),
        ];

        for (input, expected) in cases.into_iter() {
            let result = filter_square_brackets(input);
            assert_eq!(result, expected)
        }
    }

}