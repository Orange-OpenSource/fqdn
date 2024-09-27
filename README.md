# fqdn

[![Crates.io](https://img.shields.io/crates/v/fqdn?style=flat)](https://crates.io/crates/fqdn)
[![Crates.io](https://img.shields.io/crates/d/fqdn?style=flat)](https://crates.io/crates/fqdn)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat)](https://crates.io/crates/fqdn)
[![Docs](https://img.shields.io/docsrs/fqdn)](https://docs.rs/fqdn)

**Fully Qualified Domain Name**

This crate allows basic manipulation of FQDN with an inner representation 
compatible with the [RFC 1035](https://tools.ietf.org/html/rfc1035).

So, all comparisons between character strings (e.g., labels, domain names, etc.)
are done in a case-insensitive manner. Of course, FQDN hashing follows this behaviour.
Note that FQDN are internally converted to lowercase.

Notice that this RFC introduces some size limits which are not defaulty
set by this crate. 
The feature `strict-rfc` activates all of them 
but each of them could be activated independently of the others:
- labels are limited to 63 chars (`domain-label-length-limited-to-63`)
- names are limited to 255 chars (`domain-name-length-limited-to-255`)
- labels should start with a letter (`domain-label-should-start-with-letter`)
- labels should only contain letters, digits and hyphens (`domain-name-without-special-chars`)
- FQDN should end with a period (`domain-name-should-have-trailing-dot`): notice that activating this feature
modifies the behaviour of `Display` which adds a period at the end of the FQDN.