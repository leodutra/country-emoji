# country-emoji

Converts between country names, ISO 3166-1 codes and flag emojis.

## Usage

```rust
use country_emoji::{flag, code, name, countries};

flag("CL")
 // ~> 🇨🇱

code("🇨🇦")
 // ~> CA

name("🇶🇦")
 // ~> Qatar

// can extract name from string…
flag("Taiwan number one!")
 // ~> 🇹🇼

// …but only if there"s no ambiguity
flag("Congo and Burma")
 // ~> undefined

flag("Republic of Tanzania")
 // ~> 🇹🇿

flag("Tanzania, United Republic of")
 // ~> 🇹🇿

code("Australia")
 // ~> AU

code("UAE")
 // ~> AE

name("AE")
 // ~> United Arab Emirates

code("UK")
 // ~> GB
```

### Don't want Rust?

Check out the following:

* **JavaScript:** [country-emoji](https://github.com/meeDamian/country-emoji)
* **Swift:** [SwiftFlags](https://github.com/BubiDevs/SwiftFlags) (ref: [#16](https://github.com/meeDamian/country-emoji/issues/16))

## Bugs and feedback

If you discover a bug please report it [here](https://github.com/leodutra/country-emoji/issues/new). Express gratitude [here](https://patreon.com/leodutra).

Mail me at leodutra.br+foss@gmail.com, or on twitter [@leodutra](http://twitter.com/leodutra).

## Credits

This library is based on the work of two existing library:

* [country-emoji](https://github.com/meeDamian/country-emoji/blob/master/src/lib.js), available for JavaScript
* [flag-emoji-from-country-code](https://github.com/bendodson/flag-emoji-from-country-code), a great snippet to get the emoji flag from an ISO 3166-1 region code

Thanks guys for your work!

## License

MIT @ [Leo Dutra](https://github.com/leodutra)