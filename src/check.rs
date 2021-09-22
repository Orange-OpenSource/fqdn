
// value of the _ depends if we apply the rfc strictly or not
#[cfg(feature="domain-name-without-special-chars")] const __: u8 = 0;
#[cfg(not(feature="domain-name-without-special-chars"))] const __: u8 = 38;

// size of the alphabet
#[cfg(feature="domain-name-without-special-chars")]
#[allow(dead_code)] pub(crate) const ALPHABET_SIZE: usize = 38; // 26 letters + 10 digits + '-' + others (0)
#[cfg(not(feature="domain-name-without-special-chars"))]
#[allow(dead_code)] pub(crate) const ALPHABET_SIZE: usize = 39; // we should also count the '_'

// in order to decrease the necessary memory, this table reduces the search space only
// to allowed chars in FQDN, i.e. a-zA-Z, 0-9 and -.
// -> underscore is exceptionnally added since it often appears (control plane ?)
// all the others are treated equally (i.e. as a dot)
// this is case insensitive (lower and upper case give the same index)

pub(crate) const ALPHABET: [u8;256] = [
    0, 0, 0, 0, 0, 0, 0, 0,     0, 0, 0, 0, 0, 0, 0, 0,   //  16
    0, 0, 0, 0, 0, 0, 0, 0,     0, 0, 0, 0, 0, 0, 0, 0,   //  32
    0, 0, 0, 0, 0, 0, 0, 0,     0, 0, 0, 0, 0,37, 0, 0,   //  48 (-)
    27,28,29,30,31,32,33,34,    35,36, 0, 0, 0, 0, 0, 0,   //  64 (0-9)
    0, 1, 2, 3, 4, 5, 6, 7,     8, 9,10,11,12,13,14,15,   //  80 (A-O)
    16,17,18,19,20,21,22,23,    24,25,26, 0, 0, 0, 0,__,   //  96 (P-Z et _)
    0, 1, 2, 3, 4, 5, 6, 7,     8, 9,10,11,12,13,14,15,   // 112 (a-o)
    16,17,18,19,20,21,22,23,    24,25,26, 0, 0, 0, 0, 0,   // 128 (p-z)
    0, 0, 0, 0, 0, 0, 0, 0,     0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,     0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,     0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,     0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,     0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,     0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,     0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,     0, 0, 0, 0, 0, 0, 0, 0
];

pub(crate) fn are_equivalent(bytes1:&[u8], bytes2:&[u8]) -> bool
{
    let mut i1 = bytes1.iter();
    let mut i2 = bytes2.iter();

    loop {
        match i1.next() {
            None => return i2.next().is_none(),
            Some(&step) =>  match i2.next() {
                None => return false, // fqdn have different number of labels
                Some(&n) if n != step => return false, // labels have different sizes
                Some(_) => if (0..step as usize).into_iter()// check label characters (according to alphabet)
                    .any(|_| ALPHABET[*i1.next().unwrap() as usize] != ALPHABET[*i2.next().unwrap() as usize]) { return false; }
            }
        }
    }
}

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

    /// One label does not start with a letter
    ///
    /// The returned error contains the start position of the involved label
    LabelDoesNotStartWithLetter,

    /// One label is empty (e.g. starting dot as `.github.com.` or two following dots as `github..com.`)
    EmptyLabel
}

// Checks if the bytes are really a FQDN (without nul char)
pub(crate) fn check_byte_sequence(bytes: &[u8]) -> Result<(),Error>
{
    // check against 256 since we have the trailing char and the first label length to consider
    #[cfg(feature="domain-name-length-limited-to-255")]
    if bytes.len() > 256 {
        return Err(Error::TooLongDomainName)
    }
    // if unlimited, then the radix trie limits it to u32::MAX
    #[cfg(not(feature="domain-name-length-limited-to-255"))]
    if bytes.len() > u32::MAX as usize {
        return Err(Error::TooLongDomainName)
    }

    match bytes.last() {
        Some(0) => {
            let mut iter = bytes.iter();
            let remaining = bytes.len() - 1;
            while let Some(&c) = iter.next() {
                if remaining < c as usize {
                    return Err(Error::InvalidStructure)
                }
                #[cfg(feature="domain-label-length-limited-to-63")]
                if c > 63 {
                    return Err(Error::TooLongLabel)
                }
                (0..c as usize).into_iter().try_for_each(|c| Ok({check_char(c==0, *iter.next().unwrap())?;}))?;
            }
            Ok(())
        }
        Some(_) => Err(Error::TrailingNulCharMissing),
        None => Err(Error::TrailingNulCharMissing),
    }

}

pub(crate) fn check_char(_first: bool, c: u8) -> Result<u8,Error>
{
    match ALPHABET[c as usize] {
        0 => Err(Error::InvalidLabelChar),
        #[cfg(feature="domain-label-should-start-with-letter")]
        _ if _first && !c.is_ascii_alphabetic() => Err(Error::LabelDoesNotStartWithLetter),
        n => Ok(n)
    }
}
