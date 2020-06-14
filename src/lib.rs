#[macro_use]
extern crate include_dir;
extern crate regex;

use include_dir::Dir;
use regex::Regex;
use lazy_static::lazy_static;

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

const SCHEMA_DIR: Dir = include_dir!("./iuliia");
const DUMMY_SYMBOL: &str = "$";

/// Describe struct of transliterate schema
#[derive(Serialize, Deserialize, Debug)]
pub struct Schema {
    name: String,
    description: String,
    url: String,
    mapping: Option<HashMap<String, String>>,
    prev_mapping: Option<HashMap<String, String>>,
    next_mapping: Option<HashMap<String, String>>,
    ending_mapping: Option<HashMap<String, String>>,
    samples: Option<Vec<Vec<String>>>,
}

impl Schema {
    /// Return Schema object by schema name
    pub fn for_name(s: &str) -> Schema {
        let schema_file = SCHEMA_DIR.get_file(format!("{}{}", s, ".json"))
            .expect(&format!("There are no schema with name {}", s));
        serde_json::from_str(schema_file.contents_utf8().unwrap()).unwrap()
    }

    pub fn get_pref(&self, s: &str) -> Option<String> {
        if self.prev_mapping.is_none() {
            return None;
        }
        match self.prev_mapping.as_ref().unwrap().get(&s.replace(DUMMY_SYMBOL.clone(), "").to_lowercase()) {
            Some(result) => Some(result.clone()),
            None => None
        }
    }

    pub fn get_next(&self, s: &str) -> Option<String> {
        if self.next_mapping.is_none() {
            return None;
        }
        match self.next_mapping.as_ref().unwrap().get(&s.replace(DUMMY_SYMBOL.clone(), "").to_lowercase()) {
            Some(result) => Some(result.clone()),
            None => None
        }
    }

    pub fn get_letter(&self, s: &str) -> Option<String> {
        if self.mapping.is_none() {
            return None;
        }
        match self.mapping.as_ref().unwrap().get(&s.to_lowercase()) {
            Some(result) => Some(result.clone()),
            None => None
        }
    }

    pub fn get_ending(&self, s: &str) -> Option<String> {
        if self.ending_mapping.is_none() {
            return None;
        }
        match self.ending_mapping.as_ref().unwrap().get(&s.to_lowercase()) {
            Some(result) => Some(result.clone()),
            None => None
        }
    }
}

/// Transliterate a slice of str using name of schema to `String`
///
/// ```
/// assert_eq!(iuliia_rust::parse_by_schema_name("Юлия", "wikipedia"), "Yuliya")
/// ```
///
pub fn parse_by_schema_name(s: &str, schema_name: &str) -> String {
    let schema = Schema::for_name(schema_name);
    parse_by_schema(&s, &schema)

}

/// Transliterate a slice of str using `Schema` to `String`
///
/// ```
///
/// let input = "Юлия, съешь ещё этих мягких французских булок из Йошкар-Олы, да выпей алтайского чаю";
/// let expected = "Yuliya, syesh yeshchyo etikh myagkikh frantsuzskikh bulok iz Yoshkar-Oly, da vypey altayskogo chayu";
/// let schema = iuliia_rust::Schema::for_name("wikipedia");
/// 
/// let transliterated_word = iuliia_rust::parse_by_schema(&input, &schema);
///
/// assert_eq!(transliterated_word, expected)
/// ```
///
pub fn parse_by_schema(s: &str, schema: &Schema) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\b").unwrap();
    }
    RE.split(s)
        .map(|word| parse_word_by_schema(word, schema))
        .collect()
}

fn parse_word_by_schema(s: &str, schema: &Schema) -> String {
    let word_by_letters: Vec<String> = s.chars()
        .map(|char| char.to_string())
        .collect::<Vec<_>>();

    //Parse ending
    let ending = parse_ending(&word_by_letters, schema);
    let mut parsed_end = String::new();
    let word_without_ending = match ending {
        Some(matched) => {
            parsed_end = matched.translate;
            word_by_letters[..matched.ending_start].to_vec()
        }
        None => word_by_letters
    };

    //Add dummy symbols for window function
    let mut word_for_parse: Vec<String> = Vec::with_capacity(word_without_ending.len() + 2);
    let dummy_string: Vec<String> = vec![String::from(DUMMY_SYMBOL.clone())];
    word_for_parse.extend(dummy_string.clone());
    word_for_parse.extend(word_without_ending);
    word_for_parse.extend(dummy_string);

    //Parse each letter
    let parsed_word: String = word_for_parse
        .windows(3)
        .map(|letter_with_neighbors| parse_letter(letter_with_neighbors, schema))
        .collect();

    //Concat with ending
    format!("{}{}", parsed_word, parsed_end)
}

fn parse_ending(s: &Vec<String>, schema: &Schema) -> Option<Ending> {
    let length = s.len();
    if length < 3 {
        return None;
    }

    match schema.get_ending(&s[length - 1..].concat()) {
        Some(matched) => return Some(Ending {
            translate: propagate_case_from_source(matched, &s[length - 1..].concat(), false),
            ending_start: length - 1,
        }),
        None => ()
    };
    return match schema.get_ending(&s[length - 2..].concat()) {
        Some(matched) => return Some(Ending {
            translate: propagate_case_from_source(matched, &s[length - 2..].concat(), false),
            ending_start: length - 2,
        }),
        None => None
    };
}

struct Ending {
    translate: String,
    ending_start: usize,
}

/// Find letter transliteration with steps priority(apply higher):
/// 1. prefix parse
/// 2. postfix parse
/// 3. letter parse
/// 4. use input letter
fn parse_letter(letter_with_neighbors: &[String], schema: &Schema) -> String {
    let prefix: String = letter_with_neighbors[..2].concat();
    let postfix: String = letter_with_neighbors[1..].concat();
    let letter: String = letter_with_neighbors[1..2].concat();
    let mut result = letter.clone();
    match schema.get_letter(&letter) {
        Some(matched) => result = matched,
        None => ()
    };
    match schema.get_next(&postfix) {
        Some(matched) => result = matched,
        None => ()
    };
    match schema.get_pref(&prefix) {
        Some(matched) => result = matched,
        None => ()
    };
    propagate_case_from_source(result, &letter, true)
}

fn propagate_case_from_source(result: String, source_letter: &str, only_first_symbol: bool) -> String {
    // Determinate case of letter
    let letter_upper = source_letter.chars().any(|letter| letter.is_uppercase());

    if !letter_upper {
        return result.to_owned();
    }

    if only_first_symbol {
        let mut c = result.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    } else {
        result.to_uppercase()
    }
}


#[cfg(test)]
mod tests {
    use crate::{Schema, parse_by_schema};

    #[test]
    fn schema_test() {
        let schema = Schema::for_name("ala_lc");
        assert_eq!(schema.name, "ala_lc")
    }

    #[test]
    fn simple_word_test() {
        //Given
        let test_words = vec!["б", "пол"];
        let expected_words = vec!["b", "pol"];
        let schema = Schema::for_name("wikipedia");

        //When
        let transliterated_words: Vec<String> = test_words.iter()
            .map(|word| parse_by_schema(&word, &schema))
            .collect();

        //Then
        assert_eq!(transliterated_words, expected_words)
    }

    #[test]
    fn prefix_word_test() {
        //Given
        let test_words = vec!["ель"];
        let expected_words = vec!["yel"];
        let schema = Schema::for_name("wikipedia");

        //When
        let transliterated_words: Vec<String> = test_words.iter()
            .map(|word| parse_by_schema(&word, &schema))
            .collect();

        //Then
        assert_eq!(transliterated_words, expected_words)
    }

    #[test]
    fn postfix_word_test() {
        //Given
        let test_words = vec!["бульон"];
        let expected_words = vec!["bulyon"];
        let schema = Schema::for_name("wikipedia");

        //When
        let transliterated_words: Vec<String> = test_words.iter()
            .map(|word| parse_by_schema(&word, &schema))
            .collect();

        //Then
        assert_eq!(transliterated_words, expected_words)
    }

    #[test]
    fn test_letter_case() {
        //Given
        let test_words = vec!["ноГа", "Рука"];
        let expected_words = vec!["noGa", "Ruka"];
        let schema = Schema::for_name("wikipedia");

        //When
        let transliterated_words: Vec<String> = test_words.iter()
            .map(|word| parse_by_schema(&word, &schema))
            .collect();

        //Then
        assert_eq!(transliterated_words, expected_words)
    }

    #[test]
    fn test_ending() {
        //Given
        let test_words = vec!["хороший"];
        let expected_words = vec!["khoroshy"];
        let schema = Schema::for_name("wikipedia");

        //When
        let transliterated_words: Vec<String> = test_words.iter()
            .map(|word| parse_by_schema(&word, &schema))
            .collect();

        //Then
        assert_eq!(transliterated_words, expected_words)
    }

    #[test]
    fn test_sentence() {
        //Given
        let test_words = vec!["Юлия, съешь ещё этих мягких французских булок из Йошкар-Олы, да выпей алтайского чаю", "ВЕЛИКИЙ"];
        let expected_words = vec!["Yuliya, syesh yeshchyo etikh myagkikh frantsuzskikh bulok iz Yoshkar-Oly, da vypey altayskogo chayu", "VELIKY"];
        let schema = Schema::for_name("wikipedia");

        //When
        let transliterated_words: Vec<String> = test_words.iter()
            .map(|word| parse_by_schema(&word, &schema))
            .collect();

        //Then
        assert_eq!(transliterated_words, expected_words)
    }
}
