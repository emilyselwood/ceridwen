use std::collections::HashMap;

pub fn tokenise(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(str::to_lowercase)
        .map(|w| {
            w.trim()
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .replace(
                    &[
                        '(', ')', ',', '\"', '.', ';', ':', '\'', '?', '<', '>', '\\', '/', '*',
                        '{', '}', '|', '#', '=', 'ʿ', '!',
                    ][..],
                    "",
                )
        })
        .collect()
}

pub fn filter(words: Vec<String>) -> Vec<String> {
    words
        .into_iter()
        .filter(|s| !s.is_empty() && !STOP_WORDS.contains(&s.as_str()))
        .collect()
}

pub fn count_words(words: Vec<String>) -> Vec<(String, u64)> {
    let mut result: HashMap<&String, u64> = HashMap::new();

    for word in words.iter() {
        *result.entry(word).or_insert(0) += 1;
    }

    result.iter().map(|(k, v)| ((*k).clone(), *v)).collect()
}

const STOP_WORDS: &[&str] = &[
    "what", "which", "who", "whom", "this", "that", "these", "those", "am", "is", "are", "was",
    "were", "be", "been", "being", "have", "has", "had", "having", "do", "does", "did", "doing",
    "a", "an", "the", "and", "but", "if", "or", "because", "as", "until", "while", "of", "at",
    "by", "for", "with", "about", "against", "between", "into", "through", "during", "before",
    "after", "above", "below", "to", "from", "up", "down", "in", "out", "on", "off", "over",
    "under", "again", "further", "then", "once", "here", "there", "when", "where", "why", "how",
    "all", "any", "both", "each", "few", "more", "most", "other", "some", "such", "only", "own",
    "same", "so", "than", "too", "very", "s", "t", "can", "will", "just", "should",
];
