use std::collections::HashMap;
use std::io::BufRead;

use crate::error::Error;
use bytes::Bytes;
use reqwest::Client;
use url::Url;

use crate::crawler::web_client;

/// Returns a true result if the robots.txt file for the url provided allows us to process it.
pub async fn check_robots_file(client: &Client, url: &Url) -> Result<bool, Error> {
    let target_url = robots_url(url)?;

    // TODO: cache the robots.txt file somewhere

    let robots_result = web_client::get_url(client, &target_url).await;
    // if the robots.txt file isn't found we can return that we are allowed to load the page
    if robots_result.is_err()
        && robots_result.as_ref().unwrap_err() == &Error::PageNotFound(target_url.to_string())
    {
        return Ok(true);
    }

    // Parse the file
    let robots = Robots::parse_file(robots_result.unwrap())?;

    // look for our user agent and url and see if it matches
    let rule = robots.check_url(web_client::USER_AGENT, url);
    match rule {
        Some(RobotRule::Deny(_)) => Ok(false),
        Some(RobotRule::Allow(_)) => Ok(true),
        None => Ok(true),
    }
}

fn robots_url(url: &Url) -> Result<Url, Error> {
    match url.host_str() {
        Some(host) => Ok(Url::parse(&format!(
            "{}://{}/robots.txt",
            url.scheme(),
            host
        ))?),
        None => Err(Error::MissingHost(url.to_string())),
    }
}

#[derive(Debug, Clone)]
struct Robots {
    // TODO: support for site map urls
    entries: HashMap<String, Vec<RobotRule>>,
}

#[derive(Debug, Clone, PartialEq)]
enum RobotRule {
    Allow(String),
    Deny(String),
}

impl Robots {
    fn parse_file(body: Bytes) -> Result<Self, Error> {
        #[derive(Debug, PartialEq, Eq)]
        enum ParseState {
            UserAgents,
            Rules,
        }

        let mut result = Robots {
            entries: HashMap::new(),
        };

        let mut user_agent_buffer: Vec<String> = Vec::new();
        let mut rules_buffer: Vec<RobotRule> = Vec::new();
        let mut parse_state = ParseState::UserAgents;

        for line_result in body.lines() {
            if let Err(error) = line_result {
                return Err(Error::IOError(error));
            }

            let line = line_result.unwrap();
            let trimmed = line.trim();
            // skip blank lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.len() > 12 && trimmed[0..12].to_lowercase() == "user-agent: " {
                // this is the start of a new block of useragents so make sure the rules from the previous block
                // are added to the result
                if parse_state == ParseState::Rules {
                    for ua in user_agent_buffer.iter() {
                        result
                            .entries
                            .entry(ua.to_string())
                            .or_default()
                            .append(&mut rules_buffer.clone())
                    }
                    user_agent_buffer.clear();
                    rules_buffer.clear();
                    parse_state = ParseState::UserAgents;
                }

                // Now process the user agent
                user_agent_buffer.push(trimmed[12..].to_string());
            } else if trimmed.len() > 10 && trimmed[..10].to_lowercase() == "disallow: " {
                if parse_state == ParseState::UserAgents {
                    parse_state = ParseState::Rules;
                }
                rules_buffer.push(RobotRule::Deny(trimmed[10..].to_string()))
            } else if trimmed.len() > 7 && trimmed[..7].to_lowercase() == "allow: " {
                if parse_state == ParseState::UserAgents {
                    parse_state = ParseState::Rules;
                }
                rules_buffer.push(RobotRule::Allow(trimmed[7..].to_string()))
            }
        }

        // Add the last group to the results
        for ua in user_agent_buffer.iter() {
            result
                .entries
                .entry(ua.to_string())
                .or_default()
                .append(&mut rules_buffer.clone())
        }

        // if there is more than one rule for a url match, the last one should win, so reverse the list.
        for (_, v) in result.entries.iter_mut() {
            v.reverse();
        }

        Ok(result)
    }

    pub fn get_rules(&self, name: &str) -> Vec<RobotRule> {
        // get rules mentioning this entry by name
        // if there are no rules matching by name check for wild card rules
        match self.entries.get(name) {
            Some(v) => v.to_vec(),
            None => match self.entries.get("*") {
                Some(v) => v.to_vec(),
                None => Vec::new(),
            },
        }
    }

    pub fn check_url(&self, user_agent: &str, url: &Url) -> Option<RobotRule> {
        let rules = self.get_rules(user_agent);
        if rules.is_empty() {
            return None;
        }

        let file_path = url.path();

        // TODO: longest path matching?
        for rule in rules.iter() {
            // TODO: allow wild card matching rules in here. Regex?
            let matches = match rule {
                RobotRule::Allow(prefix) => file_path.starts_with(prefix),
                RobotRule::Deny(prefix) => file_path.starts_with(prefix),
            };

            if matches {
                return Some(rule.clone());
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::crawler::robots_text::RobotRule;
    use crate::crawler::robots_text::Robots;

    #[test]
    fn test_robots_parsing_basic() {
        // based on https://www.slate.com/robots.txt
        let input = "User-agent: feedjira
Disallow: /

User-agent: magpie-crawler
Disallow: /

User-agent: *
Disallow: /bullpen/
        ";

        let input_bytes = Bytes::from(input);

        let result = Robots::parse_file(input_bytes);
        assert!(!result.is_err(), "Parsing input should not have failed");

        let robots_result = result.unwrap();
        println!("Result: {:?}", robots_result);

        let feedjira_result = robots_result.get_rules("feedjira");
        assert_eq!(
            feedjira_result.len(),
            1,
            "Wrong number of results for feedjira"
        );

        assert_eq!(
            feedjira_result[0],
            RobotRule::Deny("/".to_string()),
            "feedjira rule not as expected"
        );

        let wildcard_result = robots_result.get_rules("Non Existant User Agent");
        assert_eq!(
            wildcard_result.len(),
            1,
            "Wrong number of results for wildcard"
        );

        assert_eq!(
            wildcard_result[0],
            RobotRule::Deny("/bullpen/".to_string()),
            "wildcard rule not as expected"
        )
    }
}
