use fqdn::*;

fn main() {
    let fqdn = fqdn!("foo.bar");
    println!("fqdn = {fqdn:?} => {fqdn}");

    // Convert the FQDN to a JSON string.
    let serialized = postcard::to_stdvec(&fqdn).unwrap();

    // Prints serialized = [3,102,111,111,3,98,97,114]
    println!("serialized = {serialized:?}");

    // Convert the JSON string back to a FQDN.
    let deserialized: FQDN = postcard::from_bytes(&serialized).unwrap();

    // Prints deserialized
    println!("deserialized = {deserialized:?} => {deserialized}");

    assert_eq!(fqdn, deserialized);
}