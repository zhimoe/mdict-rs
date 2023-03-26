fn main() {
    let words = r#"abc
    def
    "#;
    let words_list: Vec<&str> = words.lines().collect();
    println!("{:?}", words_list);
}

