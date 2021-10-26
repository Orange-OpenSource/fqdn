//! A fully qualified domain name representation
//!
//! Notice that a fully qualified domain name (or FQDN) is case-insensitive.
//! So the implementation of traits like `Hash` or `PartialEq` do the same.
//!
//! # Crate features
//! Some limitations are enforced by the Internet RFC but some of them are defaultly relaxed to fit
//! with more applicative contexts. Features are available in order to activate or not these
//! limitations, depending on applicative purposes.
//!
//! These features control how the parsing of a String should be done.
//! Violation of one of these activated limitations raises an error (see [`Error`]).
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
//! The internet standards specifies that a FQDN should only contains digits, letters and hyphen (`-`).
//! But, many network equipment accepts also `_` (underscore) without problems. If this crate is used to design
//! something like a firewall, it could be necessary to deal with this, so do this crate.
//! At the contrary, the activation of this feature refuses such special characters.
//!
//! ### `domain-label-should-start-with-letter`
//! The internet standards specifies that FQDN should always start with a letter (nor a digit, nor a hyphen).
//! By default, this crate accept any of theses characters event at the first position.
//! The activation of this feature enforces the use of a letter at the beginning of FQDN.
//!
//! ### `domain-label-should-have-trailing-dot`
//! The internet standards specifies that the human readable representation of FQDN should always end with a dot.
//! If this feature is activated, then parsing or printing a FQDN strictly apply this rule. By default,
//! these behaviors are more laxist.
//!
//! # RFC 1035
//! The RFC 1035 has some restrictions that are not activated by default.
//! The feature `strict-rfc-1035` activates all of them:
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
        $( str += "."; str += $args; )*
        match str.as_str().as_bytes().last() {
            None => $crate::FQDN::default(),
            Some(b'.') => str[1..].parse::<$crate::FQDN>().unwrap(),
            _ => (str + ".")[1..].parse::<$crate::FQDN>().unwrap(),
        }
    }}
}

pub use crate::fqdn::FQDN;
pub use fqdnref::Fqdn;
pub use check::Error;

#[cfg(test)]
mod tests {
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
    }

    #[test]
    fn parsing_bytes()
    {
        assert!(Fqdn::from_bytes(b"\x06github\x03com\x00").is_ok());

        assert_eq!(Fqdn::from_bytes(b"\x06github\x03com"), Err(fqdn::Error::TrailingNulCharMissing));
        assert_eq!(Fqdn::from_bytes(b"\x06g|thub\x03com\x00"), Err(fqdn::Error::InvalidLabelChar));

        #[cfg(feature="domain-label-should-start-with-letter")]
        assert_eq!(Fqdn::from_bytes(b"\x04yeah\x0512345\x03com\x00"), Err(fqdn::Error::LabelDoesNotStartWithLetter));
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
}

