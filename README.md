[![Build Status](https://travis-ci.org/massita99/iuliia-rust.svg?branch=master)](https://travis-ci.org/massita99/iuliia-rust)
# iuliia-rust
Transliterate Cyrillic → Latin in every possible way https://dangry.ru/iuliia/


Transliteration means representing Cyrillic data (mainly names and geographic locations) with Latin letters. It is used for international passports, visas, green cards, driving licenses, mail and goods delivery etc.

`Iuliia` makes transliteration as easy as:

```rust
iuliia_rust::parse_by_schema_name("Юлия", "wikipedia") -> Yuliya
```

## Why use `Iuliia`

- [20 transliteration schemas](https://github.com/nalgeon/iuliia) (rule sets), including all main international and Russian standards.
- Correctly implements not only the base mapping, but all the special rules for letter combinations and word endings (AFAIK, Iuliia is the only library which does so).
- Simple API.

## Installation

Cargo.toml:
```toml
[dependencies]
iuliia-rust = "0.1.0"
```

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Make sure to add or update tests as appropriate.

## License

[MIT](https://choosealicense.com/licenses/mit/)