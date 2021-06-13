fn main() {
    use std::io;

    let _: () = bincode::deserialize_from(std::io::stdin()).unwrap();
    bincode::serialize_into(std::io::stdout(), &Some("ok")).unwrap();
}
