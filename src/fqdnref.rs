use std::ffi::CStr;
use std::fmt;
use std::fmt::{Formatter, Write};

use crate::*;
use std::hash::{Hash, Hasher};


/// A borrowed FQDN (as a slice).
///
/// [`&Fqdn`](`crate::Fqdn`) is to [`FQDN`](`crate::FQDN`) as [`&str`] is to [`String`]:
/// the former in each pair are borrowed references; the latter are owned data.
#[derive(Debug,Eq)]
pub struct Fqdn(pub(crate) CStr);

impl Fqdn {

    /// Checks if this is the top domain.
    ///
    /// The human-readable representation of the top domain is the single dot `.`.
    #[inline]
    pub fn is_root(&self) -> bool { self.first_label_length() == 0 }

    /// Checks if this is a top level domain (TLD).
    ///
    /// A TLD is the last part of a FQDN, so this test is equivalent
    /// to check is the [depth](Self::depth) of this FQDN equals 1.
    ///
    /// # Example
    /// ```
    /// # use fqdn::*;
    /// assert![ ! fqdn!("github.com.").is_tld() ];
    /// assert![ fqdn!("com").is_tld() ];
    /// ```
    #[inline]
    pub fn is_tld(&self) -> bool
    {
        let index = self.first_label_length();
        // it is safe because of the inner structure of FQDN
        index != 0 && unsafe { *self.as_bytes().get_unchecked(index+1) } == 0
    }

    /// Checks if this domain is an descendant of another one.
    ///
    /// # Example
    /// ```
    /// # use fqdn::*;
    /// assert![ fqdn!("github.com.").is_subdomain_of(&fqdn!("github.com.")) ];
    /// assert![ fqdn!("www.rust-lang.github.com").is_subdomain_of(&fqdn!("github.com.")) ];
    ///
    /// assert![ ! fqdn!("github.com.").is_subdomain_of(&fqdn!("www.rust-lang.github.com")) ];
    /// ```
    #[inline]
    pub fn is_subdomain_of(&self, parent:&Fqdn) -> bool
    {
        // it is safe because of the inner structure of FQDN
        self.as_bytes().len() >= parent.as_bytes().len() && parent.eq(unsafe {
            let diff = self.as_bytes().len() - parent.as_bytes().len();
            &*(&self.0[diff..] as *const CStr as *const Fqdn)
        })
    }

    /// Gets the top level domain.
    ///
    /// The TLD is the last part of a FQDN.
    /// For instance, the TLD of `rust-lang.github.io.` is `io.`
    ///
    /// If the FQDN is already a TLD, `self` is returned.
    /// If the FQDN is the top domain, `None` is returned.
    ///
    /// # Example
    /// ```
    /// # use fqdn::*;
    /// # use std::str::FromStr;
    /// assert_eq![ fqdn!("rust-lang.github.com.").tld(), Some(fqdn!("com.").as_ref()) ];
    /// assert_eq![ fqdn!(".").tld(), None ];
    /// ```
    #[inline]
    pub fn tld(&self) -> Option<&Fqdn> { self.hierarchy().last() }

    /// Extracts a `Fqdn` slice with contains the immediate parent domain.
    ///
    /// The parent is the domain after remaining the first label.
    /// If it is already the top domain, then `None` is returned.
    ///
    /// To iterate over the hierarchy of a FQDN, consider [`Self::hierarchy`]
    ///
    /// # Example
    /// ```
    /// # use fqdn::*;
    /// # use std::str::FromStr;
    /// assert_eq![ fqdn!("github.com").parent(), Some(fqdn!("com").as_ref()) ];
    /// assert_eq![ fqdn!("github.com").parent().unwrap().parent(), None ];
    /// assert_eq![ fqdn!(".").parent(), None ];
    /// ```
    #[inline]
    pub fn parent(&self) -> Option<&Fqdn> { self.hierarchy().nth(1) }


    /// Iterates over the parents of the FQDN.
    /// # Example
    /// ```
    /// # use fqdn::*;
    /// # use std::str::FromStr;
    /// let fqdn = fqdn!("rust-lang.github.com.");
    /// let mut iter = fqdn.hierarchy();
    /// assert_eq![ iter.next(), Some(fqdn!("rust-lang.github.com.").as_ref()) ];
    /// assert_eq![ iter.next(), Some(fqdn!("github.com.").as_ref()) ];
    /// assert_eq![ iter.next(), Some(fqdn!("com.").as_ref()) ];
    /// assert_eq![ iter.next(), None ];
    /// ```
    #[inline]
    pub fn hierarchy(&self) -> impl '_ + Iterator<Item=&Fqdn>
    {
        struct Iter<'a>(&'a Fqdn);

        impl<'a> Iterator for Iter<'a>
        {
            type Item = &'a Fqdn;
            fn next(&mut self) -> Option<<Self as Iterator>::Item>
            {
                match self.0.first_label_length() {
                    0 => None,
                    len => {
                        let current = self.0;
                        // it is safe because of the inner structure of FQDN
                        self.0 = unsafe { &*(&self.0.0[1 + len..] as *const CStr as *const Fqdn) };
                        Some(current)
                    }
                }
            }
        }
        Iter(self)
    }

    /// Computes the depth of this domain (i.e. counts the labels)
    ///
    /// # Example
    /// ```
    /// # use fqdn::*;
    /// # use std::str::FromStr;
    /// assert_eq![ fqdn!("rust-lang.github.com.").depth(), 3 ];
    /// assert_eq![ fqdn!("github.com.").depth(), 2 ];
    /// assert_eq![ fqdn!(".").depth(), 0 ];
    /// ```
    #[inline]
    pub fn depth(&self) -> usize { self.hierarchy().count() }

    /// Builds a FQDN from a byte sequence.
    ///
    /// If the byte sequence does not follow the rules, an error is produced.
    /// See [`Error`] for more details on errors.
    ///
    /// If one is sure that the sequence matches all the rules, then the unchecking version
    ///  [`Self::from_bytes_unchecked`] could be use to be more efficient.
    ///
    /// # Example
    /// ```
    /// # use fqdn::*;
    /// # use std::str::FromStr;
    /// assert_eq![ Fqdn::from_bytes(b"\x06crates\x02io\x00"), Ok(fqdn!("crates.io.").as_ref()) ];
    ///
    /// assert_eq![ Fqdn::from_bytes(b"\x06crates\x02io"),     Err(Error::TrailingNulCharMissing) ];
    /// assert_eq![ Fqdn::from_bytes(b"\x06cr@tes\x02io\x00"), Err(Error::InvalidLabelChar) ];
    /// assert_eq![ Fqdn::from_bytes(b"\x02crates\x02io\x00"), Err(Error::InvalidStructure) ];
    /// ```
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Result<&Self,Error>
    {
        crate::check::check_byte_sequence(bytes)
            .map(|_| unsafe {
                // it is safe because check does the necessary stuff... (including trailing nul char)
                // and because Fqdn is just a wrapper around CStr
                &*(CStr::from_bytes_with_nul_unchecked(bytes) as *const CStr as *const Fqdn)
            })
    }

    /// Builds without any check a FQDN from a byte sequence.
    ///
    /// # Safety
    /// This function is unsafe because it does not check that the bytes passed to it are valid and are well-structured.
    /// It means that:
    /// * each label starts with a byte indicating its length
    /// * the bytes sequence ends with a nul-byte
    /// * only allowed ASCII characters are present in labels
    /// * the size limits should be respected
    ///
    /// If one of these constraints is violated, it may cause memory unsafety issues with future users of the FQDN.
    ///
    /// Consider [`Self::from_bytes`] for a safe version of this function.
    ///
    /// # Example
    /// ```
    /// # use fqdn::*;
    /// let crates = unsafe {
    ///     Fqdn::from_bytes_unchecked(b"\x06crates\x02io\x00")
    /// };
    /// assert_eq![ *crates, fqdn!("crates.io.") ];
    /// ```
    #[inline]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self
    {
        &*(CStr::from_bytes_with_nul_unchecked(bytes) as *const CStr as *const Fqdn)
    }

    /// Returns the complete byte sequence of the FQDN.
    ///
    /// The returned sequence is terminated by the nul byte.
    ///
    /// # Example
    /// ```
    /// # use fqdn::*;
    /// assert_eq![ fqdn!("crates.io.").as_bytes(),  b"\x06crates\x02io\x00" ];
    /// ```
    #[inline]
    pub fn as_bytes(&self) -> &[u8] { self.0.to_bytes_with_nul() }

    /// Returns the FQDN as a C string.
    #[inline]
    pub fn as_c_str(&self) -> &CStr { &self.0 }


    /// Iterates over the labels which constitutes the FQDN.
    /// # Example
    /// ```
    /// # use fqdn::*;
    /// # use std::str::FromStr;
    /// let fqdn = fqdn!("rust-lang.github.com.");
    /// let mut iter = fqdn.labels();
    /// assert_eq![ iter.next(), Some("rust-lang") ];
    /// assert_eq![ iter.next(), Some("github") ];
    /// assert_eq![ iter.next(), Some("com") ];
    /// assert_eq![ iter.next(), None ];
    /// # assert_eq![ iter.next(), None ];
    /// ```
    #[inline]
    pub fn labels(&self) -> impl '_ + Iterator<Item=&str>
    {
        self.hierarchy().map(move |s|
            // it is safe because a FQDN contains only ASCII characters
            unsafe { std::str::from_utf8_unchecked(&s.as_bytes()[1..=s.first_label_length()]) }
        )
    }

    // for internal use
    #[inline]
    fn first_label_length(&self) -> usize {
        // this is safe because of the inner structure of FQDN...
        unsafe { *self.as_bytes().get_unchecked(0) as usize }
    }
}


impl ToOwned for Fqdn {
    type Owned = FQDN;
    #[inline]
    fn to_owned(&self) -> FQDN { FQDN(self.0.to_owned()) }
}

impl fmt::Display for Fqdn
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        if self.as_bytes()[0] == 0 { // root domain
            f.write_char('.')
        } else {
            let mut iter = self.labels();

            #[cfg(feature="domain-name-should-have-trailing-dot")] {
                iter.try_for_each(|s| { f.write_str(s)?; f.write_char('.') })
            }
            #[cfg(not(feature="domain-name-should-have-trailing-dot"))] {
                f.write_str(iter.next().unwrap())?;
                iter.try_for_each(|s| { f.write_char('.')?; f.write_str(s)  })
            }
        }
    }
}

impl PartialEq for Fqdn
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        crate::check::are_equivalent(self.as_bytes(), other.as_bytes())
    }
}

impl PartialEq<FQDN> for Fqdn
{
    #[inline]
    fn eq(&self, other: &FQDN) -> bool { self.eq(other.as_ref()) }
}

impl Hash for Fqdn
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_bytes().iter().for_each(|c| c.to_ascii_lowercase().hash(state))
    }
}


impl AsRef<Fqdn> for &Fqdn
{
    #[inline]
    fn as_ref(&self) -> &Fqdn { self }
}