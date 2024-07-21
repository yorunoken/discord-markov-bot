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

    pub fn train(&mut self, sentences: Vec<String>) {
        for sentence in sentences {
            let words: Vec<&str> = sentence.split_whitespace().collect();
            for window in words.windows(2) {
                if window.len() == 2 {
                    let key = window[0].to_string();
                    let value = window[1].to_string();
                    self.chains.entry(key).or_insert_with(Vec::new).push(value);
                }
            }
        }
    }

    pub fn generate(&self, word_limit: usize) -> String {
        let mut rng = rand::thread_rng();
        let mut sentence = Vec::new();
        let mut current_word = match self.chains.keys().choose(&mut rng) {
            Some(word) => word.clone(),
            None => return String::new(),
        };

        for _ in 0..word_limit {
            sentence.push(current_word.clone());
            let next_words = self.chains.get(&current_word);
            match next_words {
                Some(words) if !words.is_empty() => {
                    current_word = match words.choose(&mut rng) {
                        Some(word) => word.clone(),
                        None => break,
                    };
                }
                _ => break,
            }
        }

        sentence.join(" ")
    }
}
