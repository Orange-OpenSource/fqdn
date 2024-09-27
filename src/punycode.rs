use std::ffi::CString;
use crate::{Error, Fqdn, FQDN};
use crate::check::check_byte_sequence;

impl FQDN {

    pub fn punyencode<S: AsRef<str>>(fqdn: S) -> Result<Self, Error>
    {
        if fqdn.as_ref() == "." || (cfg!(not(feature="domain-name-should-have-trailing-dot")) && fqdn.as_ref().is_empty()) {
            Ok(Self::default())
        } else if fqdn.as_ref().starts_with('.') || fqdn.as_ref().contains("..") {
            Err(Error::EmptyLabel)
        } else {
            fqdn.as_ref()
                .split('.')
                .take_while(|s| !s.is_empty())
                .map(|s| s.to_lowercase())
                .try_fold(String::new(), |mut fqdn, label| {
                    let puny = punycode::encode(&label)
                        .map_err(|_| Error::InvalidLabelChar)?;
                    if puny.ends_with('-') {
                        fqdn.push(label.len() as u8 as char);
                        fqdn.push_str(&label);
                    } else {
                        fqdn.push((4 + puny.len()) as u8 as char);
                        fqdn.push_str("xn--");
                        fqdn.push_str(&puny);
                    }
                    Ok(fqdn)
                })
                .and_then(|mut fqdn| {
                    fqdn.push(0 as char);
                    check_byte_sequence(fqdn.as_bytes())
                        .map(|_| unsafe { // SAFETY: just checked above
                            Self(CString::from_vec_with_nul_unchecked(fqdn.into_bytes()))
                        })
                })
        }
    }
}

impl Fqdn {

    pub fn punydecode(&self) -> String
    {
        let mut fqdn = self.labels()
            .fold(String::with_capacity(self.as_bytes().len()),
                  |mut acc, label| {
                      if label.starts_with("xn--") {
                          acc.push_str(&punycode::decode(&label[4..]).unwrap());
                      } else {
                          acc.push_str(label);
                      }
                      acc.push('.');
                      acc
                  });
        #[cfg(not(feature = "domain-name-should-have-trailing-dot"))]
        fqdn.pop();
        fqdn
    }
}


#[cfg(test)]
mod tests {
    use crate as fqdn;
    use fqdn::*;

    #[test]
    fn punycode()
    {
       let fqdn = fqdn!("www.académie-Française.fr");
        assert_eq!(fqdn, "www.xn--acadmie-franaise-npb1a.fr");

        assert_eq!(FQDN::punyencode("www.académie-française.fr").unwrap(), fqdn);

        #[cfg(not(feature = "domain-name-should-have-trailing-dot"))]
        assert_eq!(fqdn.punydecode(), "www.académie-française.fr".to_string());

        #[cfg(feature = "domain-name-should-have-trailing-dot")]
        assert_eq!(fqdn.punydecode(), "www.académie-française.fr.".to_string());
    }
}