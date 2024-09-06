use build_it::Builder;

#[test]
fn pub_fields() {
    #[derive(Default, Builder)]
    struct PubFields {
        pub name: Option<String>,
        age: Option<u32>,
    }
    let builder = PubFields::default().name("Alice".to_string()).age(30);
    assert_eq!(builder.name, Some("Alice".to_string()));
    assert_eq!(builder.age, Some(30));
}

#[test]
fn simple_struct() {
    #[derive(Default, Builder)]
    struct SimpleStruct {
        name: Option<String>,
        age: Option<u32>,
    }
    let builder = SimpleStruct::default().name("Alice".to_string()).age(30);
    assert_eq!(builder.name, Some("Alice".to_string()));
    assert_eq!(builder.age, Some(30));
}

#[test]
fn lifetimes() {
    #[derive(Default, Builder)]
    struct Lifetimes<'a> {
        name: Option<&'a str>,
        age: Option<u32>,
    }
    let builder = Lifetimes::default().name("Alice").age(30);
    assert_eq!(builder.name, Some("Alice"));
    assert_eq!(builder.age, Some(30));
}

#[test]
fn rename() {
    #[derive(Default, Builder)]
    struct Rename {
        #[build_it(rename = "new_name")]
        name: Option<String>,
        age: Option<u32>,
    }
    let builder = Rename::default().new_name("Alice".to_string()).age(30);
    assert_eq!(builder.name, Some("Alice".to_string()));
    assert_eq!(builder.age, Some(30));
}

#[test]
fn skip_fields() {
    #[derive(Default, Builder)]
    struct SkipFields {
        #[skip]
        name: String,
        #[build_it(skip)]
        test: String,
        age: Option<u32>,
    }

    let builder = SkipFields::default().age(30);
    assert_eq!(builder.name, String::default());
    assert_eq!(builder.test, String::default());
    assert_eq!(builder.age, Some(30));
}

#[test]
fn doc_comments() {
    /// A struct with doc comments
    #[derive(Default, Builder)]
    struct DocComments {
        /// Name of the person
        /// This is a multiline doc comment
        /// This is the last line of the multiline doc comment
        name: Option<String>,
        /// Age of the person
        age: Option<u32>,
    }

    let builder = DocComments::default().name("Alice".to_string()).age(30);
    assert_eq!(builder.name, Some("Alice".to_string()));
    assert_eq!(builder.age, Some(30));
}

#[test]
fn into() {
    #[derive(Default, Builder)]
    #[build_it(into)]
    struct Into {
        name: Option<String>,
        age: Option<u32>,
    }

    let builder = Into::default().name("Alice").age(30u32);
    assert_eq!(builder.name, Some("Alice".to_string()));
    assert_eq!(builder.age, Some(30));
}
