include!("./data.inc.rs");

fn new() {
    let x = 1;
}

#[cfg(test)]
mod test {
    use super::DB;

    #[test]
    fn test_db() {
        let doc = DB.get("array._array_reconstructor");
        assert!(doc.is_some());
    }
}
