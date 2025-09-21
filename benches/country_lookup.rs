use country_emoji::{code, flag, name};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_exact_matches(c: &mut Criterion) {
    let exact_matches = vec![
        "United States",
        "Canada",
        "Brazil",
        "Germany",
        "Japan",
        "Australia",
        "France",
        "Italy",
        "Spain",
        "Mexico",
    ];

    c.bench_function("exact_matches", |b| {
        b.iter(|| {
            for country in &exact_matches {
                black_box(code(black_box(country)));
            }
        })
    });
}

fn benchmark_fuzzy_matches(c: &mut Criterion) {
    let fuzzy_queries = vec![
        "USA",                 // Should match United States
        "UK",                  // Should match United Kingdom
        "Russia",              // Should match Russian Federation
        "Vatican",             // Should match Holy See (Vatican City State)
        "UAE",                 // Should match United Arab Emirates
        "Congo",               // Should match Democratic Republic of Congo
        "Korea",               // Ambiguous - should return None
        "Guinea",              // Should match Guinea exactly
        "Virgin Islands",      // Ambiguous without qualifier
        "Republic of Moldova", // Should match Moldova
    ];

    c.bench_function("fuzzy_matches", |b| {
        b.iter(|| {
            for query in &fuzzy_queries {
                black_box(code(black_box(query)));
            }
        })
    });
}

fn benchmark_saint_st_normalization(c: &mut Criterion) {
    let saint_queries = vec![
        "Saint Lucia",
        "St. Lucia",
        "St Lucia",
        "Saint Martin",
        "St. Martin",
        "St Martin",
        "Saint Helena",
        "St. Helena",
        "St Helena",
        "Saint Kitts & Nevis",
        "St. Kitts & Nevis",
        "Saint Pierre & Miquelon",
    ];

    c.bench_function("saint_st_normalization", |b| {
        b.iter(|| {
            for query in &saint_queries {
                black_box(code(black_box(query)));
            }
        })
    });
}

fn benchmark_and_ampersand_normalization(c: &mut Criterion) {
    let and_queries = vec![
        "Bosnia and Herzegovina",
        "Bosnia & Herzegovina",
        "Trinidad and Tobago",
        "Trinidad & Tobago",
        "Antigua and Barbuda",
        "Antigua & Barbuda",
        "São Tomé and Príncipe",
        "São Tomé & Príncipe",
        "Turks and Caicos Islands",
        "Turks & Caicos Islands",
    ];

    c.bench_function("and_ampersand_normalization", |b| {
        b.iter(|| {
            for query in &and_queries {
                black_box(code(black_box(query)));
            }
        })
    });
}

fn benchmark_comma_reversal(c: &mut Criterion) {
    let comma_queries = vec![
        "Virgin Islands, British",
        "British Virgin Islands",
        "Korea, Democratic People's Republic of",
        "Democratic People's Republic of Korea",
        "Congo, The Democratic Republic of the",
        "Democratic Republic of the Congo",
        "Moldova, Republic of",
        "Republic of Moldova",
    ];

    c.bench_function("comma_reversal", |b| {
        b.iter(|| {
            for query in &comma_queries {
                black_box(code(black_box(query)));
            }
        })
    });
}

fn benchmark_government_patterns(c: &mut Criterion) {
    let government_queries = vec![
        "Republic of France",           // Should match France
        "Kingdom of Spain",             // Should match Spain
        "Islamic Republic of Iran",     // Should match Iran
        "Democratic Republic of Congo", // Should match Congo-Kinshasa
        "United Kingdom",               // Exact match
        "Federal Republic of Germany",  // Should match Germany
        "Republic of Italy",            // Should match Italy
        "Commonwealth of Australia",    // Should match Australia
    ];

    c.bench_function("government_patterns", |b| {
        b.iter(|| {
            for query in &government_queries {
                black_box(code(black_box(query)));
            }
        })
    });
}

fn benchmark_flag_operations(c: &mut Criterion) {
    let country_codes = vec!["US", "CA", "BR", "DE", "JP", "AU", "FR", "IT", "ES", "MX"];
    let flag_emojis = vec!["🇺🇸", "🇨🇦", "🇧🇷", "🇩🇪", "🇯🇵", "🇦🇺", "🇫🇷", "🇮🇹", "🇪🇸", "🇲🇽"];

    c.bench_function("code_to_flag", |b| {
        b.iter(|| {
            for code in &country_codes {
                black_box(flag(black_box(code)));
            }
        })
    });

    c.bench_function("flag_to_code", |b| {
        b.iter(|| {
            for flag_emoji in &flag_emojis {
                black_box(code(black_box(flag_emoji)));
            }
        })
    });

    c.bench_function("code_to_name", |b| {
        b.iter(|| {
            for code in &country_codes {
                black_box(name(black_box(code)));
            }
        })
    });
}

fn benchmark_edge_cases(c: &mut Criterion) {
    let edge_cases = vec![
        "",                           // Empty string
        "   ",                        // Whitespace only
        "Atlantis",                   // Non-existent country
        "X",                          // Single character
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ", // Very long invalid
        "123",                        // Numbers
        "🇺🇸",                         // Flag emoji
        "US",                         // Country code
    ];

    c.bench_function("edge_cases", |b| {
        b.iter(|| {
            for query in &edge_cases {
                black_box(code(black_box(query)));
            }
        })
    });
}

fn benchmark_diacritic_handling(c: &mut Criterion) {
    let diacritic_queries = vec![
        "Côte d'Ivoire",
        "Cote d'Ivoire", // Without diacritic
        "Curaçao",
        "Curacao", // Without diacritic
        "São Tomé & Príncipe",
        "Sao Tome & Principe", // Without diacritics
        "Åland Islands",
        "Aland Islands", // Without diacritic
    ];

    c.bench_function("diacritic_handling", |b| {
        b.iter(|| {
            for query in &diacritic_queries {
                black_box(code(black_box(query)));
            }
        })
    });
}

criterion_group!(
    benches,
    benchmark_exact_matches,
    benchmark_fuzzy_matches,
    benchmark_saint_st_normalization,
    benchmark_and_ampersand_normalization,
    benchmark_comma_reversal,
    benchmark_government_patterns,
    benchmark_flag_operations,
    benchmark_edge_cases,
    benchmark_diacritic_handling
);

criterion_main!(benches);
