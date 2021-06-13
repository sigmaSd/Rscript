fn main() {
    use std::io;

    loop {
        let _hook: () = bincode::deserialize_from(std::io::stdin()).unwrap();
        //match on the hook and answer back
        bincode::serialize_into(std::io::stdout(), &Some("ok")).unwrap();
    }
}
