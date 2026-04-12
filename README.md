# country-emoji

[![crates.io](https://img.shields.io/crates/v/country-emoji)](https://crates.io/crates/country-emoji)
[![docs.rs](https://img.shields.io/docsrs/country-emoji)](https://docs.rs/country-emoji)

country-emoji provides conversions between country names, ISO 3166-1 alpha-2 codes, and Unicode flag emojis.

The crate supports exact lookups by code or flag emoji, plus alias-aware and normalized matching for country names.

## Features

- Exact lookups backed by precomputed tables for codes, flags, and normalized names
- Country-name normalization for case, whitespace, diacritics, and common abbreviations
- Alias, formal-name, and fuzzy matching for text input
- Bidirectional conversions between names, ISO codes, and flag emojis
- Borrowed string returns for code and name APIs where possible

## Quick Start

```rust
use country_emoji::{code, flag, name};

// Convert a country code to a flag emoji.
assert_eq!(flag("US"), Some("🇺🇸".to_string()));

// Convert a flag emoji to a country code.
assert_eq!(code("🇨🇦"), Some("CA"));

// Convert a country code to a display name.
assert_eq!(name("DE"), Some("Germany"));

// Convert a country name to a code.
assert_eq!(code("United Kingdom"), Some("GB"));
assert_eq!(code("UAE"), Some("AE"));
```

## Name Matching

Text lookups support several normalized and alias-based variations:

```rust
use country_emoji::code;

// Aliases and abbreviations.
assert_eq!(code("UK"), Some("GB"));
assert_eq!(code("UAE"), Some("AE"));
assert_eq!(code("Russia"), Some("RU"));

// Formal names and government titles.
assert_eq!(code("Republic of Moldova"), Some("MD"));
assert_eq!(code("Democratic People's Republic of Korea"), Some("KP"));
assert_eq!(code("United States of America"), Some("US"));

// Comma-reversed names.
assert_eq!(code("Virgin Islands, British"), Some("VG"));
assert_eq!(code("Korea, Republic of"), Some("KR"));

// Saint/St. normalization.
assert_eq!(code("Saint Lucia"), Some("LC"));
assert_eq!(code("St. Lucia"), Some("LC"));
assert_eq!(code("St Lucia"), Some("LC"));

// And/ampersand equivalence.
assert_eq!(code("Bosnia and Herzegovina"), Some("BA"));
assert_eq!(code("Bosnia & Herzegovina"), Some("BA"));

// Diacritic handling.
assert_eq!(code("Cote d'Ivoire"), Some("CI"));
assert_eq!(code("Côte d'Ivoire"), Some("CI"));

// Partial matching for unique names.
assert_eq!(code("Vatican"), Some("VA"));
```

## Explicit Conversion APIs

When the input type is already known, use the direct conversion functions:

```rust
use country_emoji::{code_to_flag, code_to_name, flag_to_code, is_code, is_country_flag, name_to_code};

assert_eq!(code_to_flag("FR"), Some("🇫🇷".to_string()));
assert_eq!(flag_to_code("🇮🇹"), Some("IT"));
assert_eq!(name_to_code("Spain"), Some("ES"));
assert_eq!(code_to_name("BR"), Some("Brazil"));

assert!(is_code(Some("CA")));
assert!(is_country_flag("🇯🇵"));
```

## Invalid And Ambiguous Input

The library returns `None` for invalid or ambiguous inputs:

```rust
use country_emoji::code;

// Invalid inputs.
assert_eq!(code("ZZ"), None);
assert_eq!(code("Atlantis"), None);

// Ambiguous inputs.
assert_eq!(code("Korea"), None);
assert_eq!(code("United"), None);
```

## Performance

The crate is designed for fast exact lookups and a narrowed fuzzy-matching fallback.

- Precomputed lookup tables for exact and normalized matches
- Cached normalized country data
- Candidate narrowing before fuzzy scoring
- Benchmarks under `benches/`

To measure performance locally, run:

```bash
cargo bench --bench country_lookup -- --quick
```

## Country Data

The dataset includes:

- ISO 3166-1 alpha-2 country codes and names
- Common aliases, abbreviations, and formal names
- Territories and dependencies used in practice
- Selected legacy and compatibility entries

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
country-emoji = "0.3"
```

## Related Libraries

Related projects:

- JavaScript: [country-emoji](https://github.com/meeDamian/country-emoji)
- Swift: [SwiftFlags](https://github.com/BubiDevs/SwiftFlags)

## Contributing & Feedback

- Bug reports: [GitHub Issues](https://github.com/leodutra/country-emoji/issues/new)
- Support development: [Patreon](https://patreon.com/leodutra)
- Contact: leodutra.br+foss@gmail.com or [@leodutra](http://twitter.com/leodutra)

## Credits

This crate builds on prior work including:

- [country-emoji](https://github.com/meeDamian/country-emoji) for the original concept and API design
- [flag-emoji-from-country-code](https://github.com/bendodson/flag-emoji-from-country-code) for the flag generation approach

## License

MIT @ [Leo Dutra](https://github.com/leodutra)
