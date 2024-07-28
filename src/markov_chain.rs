use rand::prelude::IteratorRandom;
use rand::seq::SliceRandom;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Chain {
    chains: HashMap<String, Vec<String>>,
}

impl Chain {
    pub fn new() -> Self {
        Chain {
            chains: HashMap::new(),
        }
    }

    /// Trains the chain using a vector of strings
    pub fn train(&mut self, sentences: Vec<String>) {
        // Loop over the sentences
        for sentence in sentences {
            // Split the sentence into its words
            let words: Vec<&str> = sentence.split_whitespace().collect();
            // Loop over the words with `windows`, so ["word1", "word2", "word3"]
            // will return ["word1", "word2"], and ["word2", "word3"]
            for window in words.windows(2) {
                // Make sure window has two elements
                if let [first, second] = window {
                    self.chains
                        .entry(first.to_string())
                        .or_insert_with(Vec::new)
                        .push(second.to_string());
                }
            }
        }
    }

    pub fn generate(&self, word_limit: usize, custom_word: Option<&str>) -> String {
        // Initiate the random number generator
        let mut rng = rand::thread_rng();
        let mut sentence: Vec<&str> = Vec::new();
        // Pick a random word from the chains
        let words: Vec<&str> = match custom_word {
            Some(word) => word.split_whitespace().collect(),
            None => match self.chains.keys().choose(&mut rng) {
                Some(word) => vec![word],
                None => return String::new(),
            },
        };

        for word in &words {
            sentence.push(word);
        }

        let mut current_word = &words[words.len() - 1].to_string();

        // Loop over the word_limit
        for _ in 0..word_limit {
            let next_words = self.chains.get(current_word);
            match next_words {
                Some(words) if !words.is_empty() => {
                    current_word = match words.choose(&mut rng) {
                        Some(word) => word,
                        None => break,
                    };
                }
                _ => break,
            }
        }

        sentence.join(" ")
    }
}
