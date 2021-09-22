use std::ffi::CStr;
use std::fmt;
use std::fmt::{Formatter, Write};

use crate::*;
use std::hash::{Hash, Hasher};


/// A borrowed FQDN (as a slice).
#[derive(Debug,Eq)]
pub struct Fqdn(pub(crate) CStr);

impl Fqdn {

    /// Checks if this is the top domain.
    ///
    /// The human-readable representation of the top domain is the single dot `.`.
    #[inline]
    pub fn is_root(&self) -> bool { self.first_label_length() == 0 }

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
    /// let fqdn = fqdn!("rust-lang.github.com.");
    /// assert_eq![ fqdn.tld(), Some(fqdn!("com.").as_ref()) ];
    /// assert_eq![ FQDN::default().tld(), None ];
    /// ```
    #[inline]
    pub fn tld(&self) -> Option<&Fqdn> { self.hierarchy().last() }

    #[inline]
    pub fn is_tld(&self) -> bool
    {
        let index = self.first_label_length();
        // it is safe because of the inner structure of FQDN
        index != 0 && unsafe { *self.as_bytes().get_unchecked(index) } == 0
    }

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
    /// let fqdn = fqdn!("github.com");
    /// assert_eq![ fqdn.parent(), Some(fqdn!("com").as_ref()) ];
    /// assert_eq![ fqdn.parent().unwrap().parent(), None ];
    /// assert_eq![ FQDN::default().parent(), None ];
    /// ```
    #[inline]
    pub fn parent(&self) -> Option<&Fqdn> { self.hierarchy().skip(1).next() }


    /// Iterates over the parents of the FQDN.
    /// # Example
    /// ```
    /// # use fqdn::*;
    /// # use std::str::FromStr;
    /// let fqdn = "rust-lang.github.com.".parse::<FQDN>().unwrap();
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
        Iter(&self)
    }

    #[inline]
    pub fn is_subdomain_of(&self, suffix:&Fqdn) -> bool
    {
        // it is safe because of the inner structure of FQDN
        self.as_bytes().len() >= suffix.as_bytes().len() && suffix.eq(unsafe {
            let diff = self.as_bytes().len() - suffix.as_bytes().len();
            &*(&self.0[diff..] as *const CStr as *const Fqdn)
        })
    }

    /// Computes the depth of this domain (i.e. counts the labels)
    #[inline]
    pub fn depth(&self) -> usize { self.hierarchy().count() }

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

    #[inline]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self
    {
        &*(CStr::from_bytes_with_nul_unchecked(bytes) as *const CStr as *const Fqdn)
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] { self.0.to_bytes_with_nul() }

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
        let bytes = self.as_bytes();
        if bytes[0] == 0 { // root domain
            f.write_char('.')
        } else {
            let mut iter = self.labels();
            iter.try_for_each(|s| {
                f.write_str(s)?;
                f.write_char('.')
            })
        }
    }
}

impl PartialEq for Fqdn
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        crate::check::are_equivalent(&self.as_bytes(), &other.as_bytes())
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