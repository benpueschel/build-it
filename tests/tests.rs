use simple_builder::Builder;

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
fn skip_fields() {
    #[derive(Default, Builder)]
    struct SkipFields {
        #[skip]
        name: String,
        age: Option<u32>,
    }

    let builder = SkipFields::default().age(30);
    assert_eq!(builder.name, String::default());
    assert_eq!(builder.age, Some(30));
}
