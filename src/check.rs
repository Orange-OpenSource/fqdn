

/// Error when FQDN parsing goes wrong
#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub enum Error {

    /// The trailing dot of the FQDN string is missing.
    ///
    /// A valid FQDN string should be ended by a dot (e.g. `github.com.`).
    TrailingDotMissing,

    /// The trailing nul byte of the FQDN bytes is missing.
    ///
    /// A valid FQDN array of bytes should be ended by the nul byte (e.g. `b"\x06github\x03com\x00"`)
    TrailingNulCharMissing,

    /// An invalid character is found in a label of the FQDN.
    ///
    /// The allowed characters in a FQDN label are letters, digits and `'-'`.
    /// By default, this crate also accepts `'_'` in FQDN but this behavior could be deactivated with
    /// the `strict-rfc-1035` feature.
    InvalidLabelChar,

    /// The analysed bytes are not consistent with a FQDN sequence of bytes.
    ///
    /// Typically, the length bytes of labels are not consistent.
    InvalidStructure,

    /// The name of the domain is too long
    ///
    /// By default, there is no limit except if the `strict-rfc-1035` feature is selected and
    /// then, the domain name should be less than 255 characters (including the trailing dot).
    TooLongDomainName,

    /// One label of the FQDN is too long
    ///
    /// The returned error contains the excessive length.
    ///
    /// By default, the limit is set to 255 characters but if the `strict-rfc-1035` feature is selected,
    /// then this limit is set to `63` (as said in the RFC).
    TooLongLabel,

    /// One label cannot start with a hyphen
    ///
    /// The returned error contains the start position of the involved label
    LabelCannotStartWithHyphen,

    /// One label cannot end with a hyphen
    ///
    /// The returned error contains the start position of the involved label
    LabelCannotEndWithHyphen,

    /// One label is empty (e.g. starting dot as `.github.com.` or two following dots as `github..com.`)
    EmptyLabel
}

impl std::error::Error for Error { }

use std::fmt;
use std::fmt::Debug;

impl fmt::Display for Error {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            match self {
                Error::TrailingDotMissing => "the trailing dot of the FQDN string is missing",
                Error::TrailingNulCharMissing => "the trailing nul byte of the FQDN bytes is missing",
                Error::InvalidLabelChar => "invalid char found in FQDN",
                Error::InvalidStructure => "invalid FQDN byte sequence",
                Error::TooLongDomainName => "too long FQDN",
                Error::TooLongLabel => "too long label found in FQDN",
                Error::LabelCannotStartWithHyphen => "FQDN label can’t start with a hyphen",
                Error::LabelCannotEndWithHyphen => "FQDN label can’t end with a hyphen",
                Error::EmptyLabel => "empty label found in FQDN",
            })
    }
}

// Checks if the bytes are really a FQDN (with lower cases and a trailing nul char)
pub(crate) fn check_byte_sequence(bytes: &[u8]) -> Result<(),Error>
{
    // stop immediately if the trailing nul char is missing
    match bytes.last() {
        Some(0) => { /* ok, continue */ }
        _ => return Err(Error::TrailingNulCharMissing)
    }

    #[cfg(feature="domain-name-length-limited-to-255")]
    if bytes.len() > 255 {
        return Err(Error::TooLongDomainName)
    }
    // if unlimited, then the radix trie limits it to u32::MAX
    #[cfg(not(feature="domain-name-length-limited-to-255"))]
    if bytes.len() > u32::MAX as usize {
        return Err(Error::TooLongDomainName)
    }

    let mut iter = bytes.iter();
    let mut remaining = bytes.len() - 1;

    while remaining > 0 {
        match iter.next() {
            // sublen does not match with available bytes
            None | Some(&0) => return Err(Error::InvalidStructure),

            Some(&sublen) if sublen as usize > remaining => {
                return Err(Error::InvalidStructure)
            }

            #[cfg(feature="domain-label-length-limited-to-63")]
            Some(&sublen) if sublen > 63 => {
                return Err(Error::TooLongLabel)
            }

            #[cfg(feature="domain-label-cannot-start-or-end-with-hyphen")]
            Some(&1) => { // label with only one single char
                if check_any_char(*iter.next().unwrap())? == b'-' {
                    return Err(Error::LabelCannotStartWithHyphen);
                }
                remaining -= 2;
            }

            #[cfg(feature="domain-label-cannot-start-or-end-with-hyphen")]
            Some(&sublen) => {
                if check_any_char(*iter.next().unwrap())? == b'-' {
                    return Err(Error::LabelCannotStartWithHyphen);
                }
                for _ in 1..sublen - 1 {
                    check_any_char(*iter.next().unwrap())?;
                }
                if check_any_char(*iter.next().unwrap())? == b'-' {
                    return Err(Error::LabelCannotEndWithHyphen);
                }
                remaining -= sublen as usize + 1;
            }

            #[cfg(not(feature="domain-label-cannot-start-or-end-with-hyphen"))]
            Some(&sublen) => {
                for _ in 0..sublen {
                    check_any_char(*iter.next().unwrap())?;
                }
                remaining -= sublen as usize + 1;
            }
        }
    }
    debug_assert_eq!( iter.next(), Some(&0));
    debug_assert!( iter.next().is_none() );
    Ok(())
}


fn check_any_char(c: u8) -> Result<u8,Error>
{
    match c {
        b'a'..=b'z' | b'-' | b'0'..=b'9' => Ok(c),
        #[cfg(not(feature="domain-name-without-special-chars"))]
        b'_' | b'#' => Ok(c),
        _ => Err(Error::InvalidLabelChar),
    }
}

pub(crate) fn check_and_lower_any_char(c: u8) -> Result<u8,Error>
{
    /// If the 6th bit is set, ascii is lower case.
    const ASCII_CASE_MASK: u8 = 0b0010_0000;

    match c {
        b'a'..=b'z' | b'-' | b'0'..=b'9' => Ok(c),
        #[cfg(not(feature="domain-name-without-special-chars"))]
        b'_' | b'#' => Ok(c),
        b'A'..=b'Z' => Ok(c | ASCII_CASE_MASK), // to lowercase
        _ => Err(Error::InvalidLabelChar),
    }
}
