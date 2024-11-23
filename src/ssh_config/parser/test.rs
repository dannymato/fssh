use super::*;

#[test]
fn test_argument_quoted() {
    let test_argument = "\"arg1 with spaces\" arg2";

    let result = argument::<()>(test_argument);

    assert_eq!(result, Ok((" arg2", "arg1 with spaces")))
}

#[test]
fn test_argument() {
    let test_argument = "arg1 arg2";

    let result = argument::<()>(test_argument);

    assert_eq!(result, Ok((" arg2", "arg1")))
}

#[test]
fn test_keyword() {
    let test_keyword = "host";

    assert_eq!(keyword_string::<()>(test_keyword), Ok(("", Keyword::Host)))
}

#[test]
fn test_keyword_hostname() {
    let test_keyword = "hOstName";

    assert_eq!(
        keyword_string::<()>(test_keyword),
        Ok(("", Keyword::Hostname))
    )
}

#[test]
fn test_keyword_other() {
    let test_keyword = "otherKeyword";

    assert_eq!(
        keyword_string::<()>(test_keyword),
        Ok(("", Keyword::Other("otherkeyword".to_string())))
    )
}

#[test]
fn test_key_value_simple() {
    let test_keyword = "key value";

    assert_eq!(
        key_value::<()>(test_keyword),
        Ok(("", (Keyword::Other("key".to_string()), vec!["value"],)))
    )
}

#[test]
fn test_key_value_complex() {
    let test_keyword = "key =value \"value2\" \"value 3\"";

    assert_eq!(
        key_value::<()>(test_keyword),
        Ok((
            "",
            (
                Keyword::Other("key".to_string()),
                vec!["value", "value2", "value 3"],
            )
        ))
    )
}

#[test]
fn test_config_values() {
    let test_values = "key value\nanother value";

    assert_eq!(
        ssh_config_value_parser::<()>(test_values),
        Ok((
            "",
            vec![
                (Keyword::Other("key".to_string()), vec!["value"]),
                (Keyword::Other("another".to_string()), vec!["value"]),
            ]
        ))
    );
}
#[test]
fn test_config_values_spaced() {
    let test_values = "key value\n\n\n\n      host value \"value 2\"\nhostName thishost";

    assert_eq!(
        ssh_config_value_parser::<()>(test_values),
        Ok((
            "",
            vec![
                (Keyword::Other("key".to_string()), vec!["value"]),
                (Keyword::Host, vec!["value", "value 2"]),
                (Keyword::Hostname, vec!["thishost"]),
            ]
        ))
    );
}
