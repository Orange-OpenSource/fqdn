# fqdn

[![Crates.io](https://img.shields.io/crates/v/fqdn?style=flat-square)](https://crates.io/crates/fqdn)
[![Crates.io](https://img.shields.io/crates/d/fqdn?style=flat-square)](https://crates.io/crates/fqdn)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://crates.io/crates/fqdn)


**Fully Qualified Domain Name**

This crate allows basic manipulation of FQDN with an inner representation 
compatible with the [RFC 1035](https://tools.ietf.org/html/rfc1035).

So, all comparisons between character strings (e.g., labels, domain names, etc.)
are done in a case-insensitive manner. Of course, FQDN hashing should follow this behaviour.

Notice that this RFC introduces some size limits which are not defaulty
set by this crate. 
The feature `strict-rfc-1035` activates all of them 
but each of them could be activated independently of the others:
- labels are limited to 63 octets (`domain-label-length-limited-to-63`)
- names are limited to 255 octets (`domain-name-length-limited-to-255`)
- labels should start with a letter (`domain-label-should-start-with-letter`)
- labels should only contain letters, digits and hyphen (`domain-name-without-special-chars`)
