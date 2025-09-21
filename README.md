# country-emoji

[![crate.io](https://img.shields.io/crates/v/country-emoji)](https://crates.io/crates/country-emoji)
[![docs.rs](https://img.shields.io/docsrs/country-emoji)](https://docs.rs/country-emoji)

A lightweight, fast Rust library that converts between country names, ISO 3166-1 codes and flag emojis. Features intelligent fuzzy matching, normalization, and comprehensive country data.

## Features

- **Fast lookups** - Optimized for performance with pre-compiled regex patterns
- **Fuzzy matching** - Handles alternative names, government titles, and formatting variations
- **Comprehensive data** - All ISO 3166-1 countries including recent additions
- **Normalization** - Handles diacritics, case-insensitivity, whitespace, and abbreviations
- **Bidirectional conversion** - Convert between any combination of codes, names, and flag emojis
- **Zero-copy** - Returns string slices where possible for optimal memory usage

## Usage

### Basic Operations

```rust
use country_emoji::{flag, code, name};

// Generate flag emoji from country code
assert_eq!(flag("US"), Some("🇺🇸".to_string()));

// Extract country code from flag emoji
assert_eq!(code("🇨🇦"), Some("CA"));

// Get country name from code
assert_eq!(name("DE"), Some("Germany"));

// Convert country name to code
assert_eq!(code("Japan"), Some("JP"));
```

### Advanced Fuzzy Matching

The library handles many name variations and formats intelligently:

```rust
use country_emoji::code;

// Alternative names and abbreviations
assert_eq!(code("UK"), Some("GB"));
assert_eq!(code("UAE"), Some("AE"));
assert_eq!(code("Russia"), Some("RU"));

// Government titles and formal names
assert_eq!(code("Republic of Moldova"), Some("MD"));
assert_eq!(code("Democratic People's Republic of Korea"), Some("KP"));
assert_eq!(code("United States of America"), Some("US"));

// Comma-reversed formats
assert_eq!(code("Virgin Islands, British"), Some("VG"));
assert_eq!(code("Korea, Republic of"), Some("KR"));

// Saint/St. normalization
assert_eq!(code("Saint Lucia"), Some("LC"));
assert_eq!(code("St. Lucia"), Some("LC"));
assert_eq!(code("St Lucia"), Some("LC"));

// And/ampersand equivalence
assert_eq!(code("Bosnia and Herzegovina"), Some("BA"));
assert_eq!(code("Bosnia & Herzegovina"), Some("BA"));

// Diacritic handling
assert_eq!(code("Cote d'Ivoire"), Some("CI"));
assert_eq!(code("Côte d'Ivoire"), Some("CI"));

// Partial matching for unique names
assert_eq!(code("Vatican"), Some("VA")); // matches "Holy See (Vatican City State)"
```

### Direct API Functions

For explicit conversions, use the direct API:

```rust
use country_emoji::{code_to_flag, flag_to_code, name_to_code, code_to_name, is_code, is_country_flag};

assert_eq!(code_to_flag("FR"), Some("🇫🇷".to_string()));
assert_eq!(flag_to_code("🇮🇹"), Some("IT"));
assert_eq!(name_to_code("Spain"), Some("ES"));
assert_eq!(code_to_name("BR"), Some("Brazil"));

assert!(is_code(Some("CA")));
assert!(is_country_flag("🇯🇵"));
```

### Error Handling

The library returns `None` for invalid or ambiguous inputs:

```rust
use country_emoji::code;

// Invalid inputs
assert_eq!(code("ZZ"), None);           // Invalid country code
assert_eq!(code("Atlantis"), None);     // Non-existent country

// Ambiguous inputs (prevents false matches)
assert_eq!(code("Korea"), None);        // Could be North or South Korea
assert_eq!(code("United"), None);       // Too ambiguous
```

## Performance

This library is optimized for high performance:

- **Pre-compiled regex patterns** for fast text normalization
- **Cached normalized data** to avoid repeated processing
- **Early exit strategies** in matching algorithms
- **Benchmarked** - Run `cargo bench` to see performance metrics

Typical performance (release build):
- Exact matches: ~40μs for 10 lookups
- Fuzzy matches: ~800μs for 10 complex queries
- Flag operations: ~500ns per conversion

## Country Data

The library includes comprehensive country data:

- **All 249 ISO 3166-1** assigned codes
- **Current country names** (e.g., "North Macedonia", "Eswatini")
- **Alternative names** and historical names
- **Common abbreviations** (UK, UAE, USA, etc.)
- **Territories and dependencies** (Puerto Rico, Guam, etc.)
- **Recent additions** (South Sudan, Curaçao, Sint Maarten)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
country-emoji = "0.3"
```

## Related Libraries

Don't need Rust? Check out these alternatives:

- **JavaScript:** [country-emoji](https://github.com/meeDamian/country-emoji) - Original inspiration
- **Swift:** [SwiftFlags](https://github.com/BubiDevs/SwiftFlags)

## Contributing & Feedback

- **Bug reports:** [GitHub Issues](https://github.com/leodutra/country-emoji/issues/new)
- **Support development:** [Patreon](https://patreon.com/leodutra)
- **Contact:** leodutra.br+foss@gmail.com or [@leodutra](http://twitter.com/leodutra)

## Credits

This library builds upon excellent prior work:

- [country-emoji](https://github.com/meeDamian/country-emoji) (JavaScript) - Original concept and API design
- [flag-emoji-from-country-code](https://github.com/bendodson/flag-emoji-from-country-code) - Flag emoji generation technique



## License

MIT @ [Leo Dutra](https://github.com/leodutra)
