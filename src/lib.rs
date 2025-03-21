//! A fully qualified domain name representation
//!
//! Notice that a fully qualified domain name (FQDN) is case-insensitive.
//! So the implementation of traits `Hash`, `PartialEq`, `Eq`, `PartialOrd` and `Ord` do the same.
//!
//! # Crate features
//! These features control how the parsing of a string should be done.
//! Violation of one of these activated limitations raises an error (see [`Error`]).
//!
//! Some limitations are set by the Internet RFC but, by default, some of them are relaxed to fit
//! with more applicative contexts. The following features are available in order to activate or not these
//! limitations, depending on applicative purposes.
//!
//!
//! ### `domain-label-length-limited-to-63`
//! The internet standards specifies that each label of a FQDN is limited to 63 characters.
//! By default, this crate allows up to 256 characters but the 63 limitation could be set
//! through the activation of this feature.
//!
//! ### `domain-name-length-limited-to-255`
//! The internet standards specifies that the total length of a FQDN is limited to 255 characters.
//! By default, the only limit is the available memory but the 255 limitation could be set
//! through the activation of this feature.
//!
//! ### `domain-name-without-special-chars`
//! The internet standards specifies that a FQDN should only contain digits, letters and hyphens (`-`).
//! But, many network equipment accept also `_` (underscore) without any problem. If this crate is used to design
//! something like a firewall, it could be necessary to deal with this, so do this feature.
//! At the contrary, the activation of this feature refuses these special characters.
//!
//! ### `domain-label-should-start-with-letter`
//! The internet standards specifies that FQDN should always start with a letter (nor a digit, nor a hyphen).
//! By default, this crate accept any of these characters event at the first position.
//! The activation of this feature forces the use of a letter at the beginning of FQDN.
//!
//! ### `domain-label-should-have-trailing-dot`
//! The internet standards specifies that the human-readable representation of FQDN should always end with a dot.
//! If this feature is activated, then parsing or printing a FQDN strictly apply this rule. By default,
//! the parsing behavior is more lenient (i.e. the trailing dot could miss).
//!
//! ### `punycode`
//! Allow the use of Unicode characters in FQDN. They are automatically encoded
//! by using [Punycode](https://en.wikipedia.org/wiki/Punycode).
//! When activated, the macro `fqdn!` implements the punycode, so do the trait `FromStr`
//!
//! ### `serde`
//! Provide serialization and deseriatization for FQDN.
//!
//! # RFC related to FQDN
//! The related RFC define some restrictions that are not activated by default.
//! The feature `strict-rfc` activates all of them, which are:
//! * `domain-label-length-limited-to-63`
//! * `domain-name-length-limited-to-255`
//! * `domain-name-without-special-chars`
//! * `domain-label-should-start-with-letter`
//! * `domain-label-should-have-trailing-dot`
//!
//! See above for more details.
//!
mod fqdnref;
mod fqdn;
mod check;
mod eqcmp;

#[cfg(feature = "punycode")] mod punycode;


/// Parses a list of strings and creates an new
/// FQDN by concatenating them.
///
/// If the trailing dot is missing, it is automatically added.
///
/// # Examples
/// ```
/// use fqdn::fqdn;
///
/// let fqdn = fqdn!("rust-lang", "github.io");
/// ```
/// # Panics
/// If one of the elements is not a valid symbol, the macro panics.
/// ```should_panic
/// use fqdn::fqdn;
///
/// let s = fqdn!("w@w","fr"); // panics !!
/// ```
#[macro_export]
macro_rules! fqdn {
    ($($args:expr),*) => {{
        #[allow(unused_mut)]
        let mut str = std::string::String::new();
        $( str += $args; str += "."; )*
        if str.as_bytes().len() <= 1 {
            $crate::FQDN::default()
        } else {
            let penultimate = str.as_bytes().len() - 2;
            // SAFETY: the length is checked just before
            let str = match unsafe { str.as_bytes().get_unchecked(penultimate) } {
                b'.' => &str[..(penultimate+1)],
                _ => &str
            };
            str.parse::<$crate::FQDN>().unwrap()
        }
    }}
}

pub use crate::fqdn::FQDN;
pub use fqdnref::Fqdn;
pub use check::Error;

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashSet};
    use crate as fqdn;
    use fqdn::*;

    #[test]
    fn parsing_string()
    {
        assert!(FQDN::default().is_root());
        assert!("github.com.".parse::<FQDN>().is_ok());

        #[cfg(feature="domain-name-should-have-trailing-dot")]
        assert_eq!("crates.io".parse::<FQDN>(), Err(fqdn::Error::TrailingDotMissing));

        #[cfg(not(feature="domain-name-should-have-trailing-dot"))]
        assert_eq!("crates.io".parse::<FQDN>().map(|fqdn| fqdn.to_string()), Ok("crates.io".to_string()));

        assert_eq!("github..com.".parse::<FQDN>(), Err(fqdn::Error::EmptyLabel));
        assert_eq!(".github.com.".parse::<FQDN>(), Err(fqdn::Error::EmptyLabel));
        assert_eq!("git@ub.com.".parse::<FQDN>(), Err(fqdn::Error::InvalidLabelChar));

        const LENGTH_256: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.";
        const LENGTH_255: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.";

        #[cfg(feature="domain-name-length-limited-to-255")]
        assert_eq!(LENGTH_256.parse::<FQDN>(), Err(fqdn::Error::TooLongDomainName));

        #[cfg(not(feature="domain-name-length-limited-to-255"))]
        assert!(LENGTH_256.parse::<FQDN>().is_ok());

        assert!(LENGTH_255.parse::<FQDN>().is_ok());

        #[cfg(not(feature="domain-name-should-have-trailing-dot"))]
        {
            #[cfg(feature="domain-name-length-limited-to-255")]
            assert_eq!(LENGTH_256[..LENGTH_256.len() - 1].parse::<FQDN>(), Err(fqdn::Error::TooLongDomainName));

            #[cfg(not(feature="domain-name-length-limited-to-255"))]
            assert!(LENGTH_256[..LENGTH_256.len() - 1].parse::<FQDN>().is_ok());

            assert!(LENGTH_255[..LENGTH_255.len() - 1].parse::<FQDN>().is_ok());
        }
    }

    #[test]
    fn parsing_bytes()
    {
        assert!(Fqdn::from_bytes(b"\x06github\x03com\x00").is_ok());

        assert_eq!(Fqdn::from_bytes(b"\x06github\x03com"), Err(fqdn::Error::TrailingNulCharMissing));
        assert_eq!(Fqdn::from_bytes(b"\x06g|thub\x03com\x00"), Err(fqdn::Error::InvalidLabelChar));

        #[cfg(feature = "domain-label-cannot-start-or-end-with-hyphen")] {
            assert_eq!(Fqdn::from_bytes(b"\x05-yeah\x0512345\x03com\x00"), Err(fqdn::Error::LabelCannotStartWithHyphen));
            assert_eq!(Fqdn::from_bytes(b"\x05yeah-\x0512345\x03com\x00"), Err(fqdn::Error::LabelCannotEndWithHyphen));
        }

        const LENGTH_256: &[u8; 256] = b"\x3faaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\x3faaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\x3faaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\x3eaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\x00";
        const LENGTH_255: &[u8; 255] = b"\x3faaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\x3faaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\x3faaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\x3daaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\x00";

        #[cfg(feature="domain-name-length-limited-to-255")]
        assert_eq!(Fqdn::from_bytes(LENGTH_256), Err(fqdn::Error::TooLongDomainName));

        #[cfg(not(feature="domain-name-length-limited-to-255"))]
        assert!(Fqdn::from_bytes(LENGTH_256).is_ok());

        assert!(Fqdn::from_bytes(LENGTH_255).is_ok());
    }

    #[test]
    fn check_bytes()
    {
        let fqdn = Fqdn::from_bytes(b"\x06github\x03com\x00").unwrap();
        assert_eq!( fqdn.tld().unwrap().as_bytes(), b"\x03com\x00");
        assert_eq!( &fqdn.as_bytes()[fqdn.as_bytes().len() - 5..], b"\x03com\x00");

        assert_eq!( Ok(FQDN::default()), FQDN::from_vec(vec![]) );

        assert_eq!( Err(Error::InvalidStructure), FQDN::from_vec(vec![1]) );
        assert_eq!( Ok(fqdn!("a.fr")), FQDN::from_vec(vec![1, b'a', 2, b'f', b'r']) );
        assert_eq!( Ok(fqdn!("a.fr")), FQDN::from_vec(vec![1, b'a', 2, b'f', b'r', 0]) );
    }

    #[test]
    fn check_bytes_label_with_lowercase()
    {
        let fqdnref = Fqdn::from_bytes(b"\x06github\x03com\x00").unwrap();

        #[cfg(feature="domain-name-should-have-trailing-dot")] {
            assert_eq!(fqdnref.to_string(), "github.com.");
        }
        #[cfg(not(feature="domain-name-should-have-trailing-dot"))] {
            assert_eq!(fqdnref.to_string(), "github.com");
        }

        let fqdn = FQDN::from_vec(b"\x06GitHUB\x03com\x00".to_vec()).unwrap();
        assert_eq!( fqdn, *fqdnref);

        let fqdn = FQDN::from_vec(b"\x06GitHUB\x03com".to_vec()).unwrap();
        assert_eq!( fqdn, *fqdnref);
    }

    #[test]
    fn check_bytes_label_with_single_char()
    {
        assert! (Fqdn::from_bytes(b"\x01a\x02fr\x00").is_ok() );
    }


    #[test]
    fn depth()
    {
        assert_eq!(".".parse::<FQDN>().map(|f| f.is_root()), Ok(true));
        assert_eq!(".".parse::<FQDN>().map(|f| f.depth()), Ok(0));
        assert_eq!("github.com.".parse::<FQDN>().map(|f| f.depth()), Ok(2));
        assert_eq!("rust-lang.github.com.".parse::<FQDN>().map(|f| f.depth()), Ok(3));
    }

    #[test]
    fn subdomains()
    {
        let a = "rust-lang.github.com.".parse::<FQDN>().unwrap();
        let b = "GitHub.com.".parse::<FQDN>().unwrap();

        assert!( a.is_subdomain_of(&a));
        assert!( a.is_subdomain_of(&b));
        assert!( !b.is_subdomain_of(&a));

        assert!( fqdn!("com").is_tld() );
        assert_eq!( a, fqdn!("rust-lang","github","com") );
    }

    #[test]
    fn equivalence()
    {
        let fqdn1 = "github.com.".parse::<FQDN>().unwrap();
        let fqdn2 = "GitHub.com.".parse::<FQDN>().unwrap();
        assert_eq!(fqdn1, fqdn2);
        assert_eq!(&fqdn1, &fqdn2);
        assert_eq!(fqdn1.as_ref(), fqdn2.as_ref());
        assert_eq!(&fqdn1, fqdn2.as_ref());
        assert_eq!(fqdn1.as_ref(), &fqdn2);
        assert_eq!(fqdn1, *fqdn2.as_ref());
    }

    #[test]
    fn string_compare()
    {
        let fqdn = "GitHub.com.".parse::<FQDN>().unwrap();
        assert_eq!(fqdn, "github.com.");
        assert_eq!("github.com.", fqdn);
        assert_eq!(fqdn, "github.COM.");
        assert_ne!(fqdn, "git=hub.COM.");

        #[cfg(feature="domain-name-should-have-trailing-dot")] {
            assert_ne!(fqdn, "github.com");
            assert_ne!(fqdn, "github.COM");
        }

        #[cfg(not(feature="domain-name-should-have-trailing-dot"))] {
            assert_eq!(fqdn, "github.com");
            assert_eq!(fqdn, "github.COM");
        }
    }

    #[test]
    fn ordering()
    {
        assert!("a.github.com.".parse::<FQDN>().unwrap() < "aa.GitHub.com.".parse::<FQDN>().unwrap());
        assert!("ab.github.com.".parse::<FQDN>().unwrap() > "aa.github.com.".parse::<FQDN>().unwrap());
        assert!("ab.GitHub.com.".parse::<FQDN>().unwrap() > "aa.github.com.".parse::<FQDN>().unwrap());
        assert!("ab.GitHub.com.".parse::<FQDN>().unwrap() > "aa.github.co.".parse::<FQDN>().unwrap());

        let items = ["github.com.", "a.Github.com.", "a.GitHub.com.", "a.github.com.", "aa.github.com."];

        let ordered = items.iter().map(|s| s.parse::<FQDN>().unwrap())
            .collect::<BTreeSet<_>>();

        let unordered = items.iter().map(|s| s.parse::<FQDN>().unwrap())
            .collect::<HashSet<_>>();

        assert_eq!(ordered.len(), unordered.len());
    }

    #[test]
    #[cfg(not(feature="domain-name-without-special-chars"))]
    fn special_chars()
    {
        assert!("git_hub.com.".parse::<FQDN>().is_ok());
        assert!("git#hub.com.".parse::<FQDN>().is_ok());
    }
}

