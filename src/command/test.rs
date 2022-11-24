use super::parse;
use crate::db::Evictor;
use crate::{
    command::{Command, Create, Set},
    frame::Frame,
};
use bytes::Bytes;
use std::ops::Add;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn get_frame_from_str(str: &'static str) -> Frame {
    Frame::String(Bytes::from(str))
}

#[test]
fn parse_given_unknown_command_returns_error() {
    let command = vec![get_frame_from_str("foo")];
    assert!(parse(Frame::Array(command)).is_err())
}

#[test]
fn parse_given_command_with_non_string_tokens_returns_error() {
    let command = vec![get_frame_from_str("create"), Frame::Integer(100)];
    assert!(parse(Frame::Array(command)).is_err())
}

#[test]
fn parse_given_command_with_non_array_container_returns_error() {
    assert!(parse(get_frame_from_str("create")).is_err())
}

#[test]
fn parse_given_create_command_without_keyspace_returns_error() {
    let command = vec![get_frame_from_str("create")];
    assert!(parse(Frame::Array(command)).is_err())
}

#[test]
fn parse_given_create_command_with_keyspace_and_no_config_returns_create() {
    let command = vec![get_frame_from_str("create"), get_frame_from_str("foo")];
    assert_eq!(
        parse(Frame::Array(command)).unwrap(),
        Command::Create(Create {
            evictor: Evictor::Nop,
            if_not_exists: false,
            keyspace: Bytes::from("foo")
        })
    );
}

#[test]
fn parse_given_create_command_with_no_evictor_value_returns_error() {
    let command = vec![
        get_frame_from_str("create"),
        get_frame_from_str("foo"),
        get_frame_from_str("evictor"),
    ];
    assert!(parse(Frame::Array(command)).is_err())
}

#[test]
fn parse_given_create_command_with_invalid_evictor_value_returns_error() {
    let command = vec![
        get_frame_from_str("create"),
        get_frame_from_str("foo"),
        get_frame_from_str("evictor"),
        get_frame_from_str("bar"),
    ];
    assert!(parse(Frame::Array(command)).is_err())
}

#[test]
fn parse_given_create_command_with_lru_evictor_returns_create() {
    let command = vec![
        get_frame_from_str("create"),
        get_frame_from_str("foo"),
        get_frame_from_str("evictor"),
        get_frame_from_str("lru"),
    ];
    assert_eq!(
        parse(Frame::Array(command)).unwrap(),
        Command::Create(Create {
            evictor: Evictor::Lru,
            if_not_exists: false,
            keyspace: Bytes::from("foo")
        })
    );
}

#[test]
fn parse_given_create_command_with_random_evictor_returns_create() {
    let command = vec![
        get_frame_from_str("create"),
        get_frame_from_str("foo"),
        get_frame_from_str("evictor"),
        get_frame_from_str("random"),
    ];
    assert_eq!(
        parse(Frame::Array(command)).unwrap(),
        Command::Create(Create {
            evictor: Evictor::Random,
            if_not_exists: false,
            keyspace: Bytes::from("foo")
        })
    );
}

#[test]
fn parse_given_create_command_with_nop_evictor_returns_create() {
    let command = vec![
        get_frame_from_str("create"),
        get_frame_from_str("foo"),
        get_frame_from_str("evictor"),
        get_frame_from_str("nop"),
    ];
    assert_eq!(
        parse(Frame::Array(command)).unwrap(),
        Command::Create(Create {
            evictor: Evictor::Nop,
            if_not_exists: false,
            keyspace: Bytes::from("foo")
        })
    );
}

#[test]
fn parse_given_create_command_with_incomplete_if_flag_1_returns_error() {
    let command = vec![
        get_frame_from_str("create"),
        get_frame_from_str("foo"),
        get_frame_from_str("evictor"),
        get_frame_from_str("lru"),
        get_frame_from_str("if"),
    ];
    assert!(parse(Frame::Array(command)).is_err())
}

#[test]
fn parse_given_create_command_with_incomplete_if_flag_2_returns_error() {
    let command = vec![
        get_frame_from_str("create"),
        get_frame_from_str("foo"),
        get_frame_from_str("evictor"),
        get_frame_from_str("lru"),
        get_frame_from_str("if"),
        get_frame_from_str("not"),
    ];
    assert!(parse(Frame::Array(command)).is_err())
}

#[test]
fn parse_given_create_command_with_invalid_if_flag_returns_error() {
    let command = vec![
        get_frame_from_str("create"),
        get_frame_from_str("foo"),
        get_frame_from_str("evictor"),
        get_frame_from_str("lru"),
        get_frame_from_str("if"),
        get_frame_from_str("exists"),
    ];
    assert!(parse(Frame::Array(command)).is_err())
}

#[test]
fn parse_given_create_command_with_if_not_exists_flag_returns_create() {
    let command = vec![
        get_frame_from_str("create"),
        get_frame_from_str("foo"),
        get_frame_from_str("evictor"),
        get_frame_from_str("lru"),
        get_frame_from_str("if"),
        get_frame_from_str("not"),
        get_frame_from_str("exists"),
    ];
    assert_eq!(
        parse(Frame::Array(command)).unwrap(),
        Command::Create(Create {
            evictor: Evictor::Lru,
            if_not_exists: true,
            keyspace: Bytes::from("foo")
        })
    );
}

#[test]
fn parse_given_create_command_with_if_not_exists_flag_and_evictor_position_reversed_returns_create()
{
    let command = vec![
        get_frame_from_str("create"),
        get_frame_from_str("foo"),
        get_frame_from_str("if"),
        get_frame_from_str("not"),
        get_frame_from_str("exists"),
        get_frame_from_str("evictor"),
        get_frame_from_str("lru"),
    ];
    assert_eq!(
        parse(Frame::Array(command)).unwrap(),
        Command::Create(Create {
            evictor: Evictor::Lru,
            if_not_exists: true,
            keyspace: Bytes::from("foo")
        })
    );
}

#[test]
fn parse_given_create_command_with_if_not_exists_flag_and_default_evictor_returns_create() {
    let command = vec![
        get_frame_from_str("create"),
        get_frame_from_str("foo"),
        get_frame_from_str("if"),
        get_frame_from_str("not"),
        get_frame_from_str("exists"),
    ];
    assert_eq!(
        parse(Frame::Array(command)).unwrap(),
        Command::Create(Create {
            evictor: Evictor::Nop,
            if_not_exists: true,
            keyspace: Bytes::from("foo")
        })
    );
}

#[test]
fn parse_given_create_command_with_invalid_if_flag_2_returns_error() {
    let command = vec![
        get_frame_from_str("create"),
        get_frame_from_str("foo"),
        get_frame_from_str("if"),
        get_frame_from_str("not"),
        get_frame_from_str("evictor"),
        get_frame_from_str("exists"),
        get_frame_from_str("lru"),
    ];
    assert!(parse(Frame::Array(command)).is_err())
}

#[test]
fn parse_given_set_command_without_keyspace_returns_error() {
    let command = vec![get_frame_from_str("set")];
    assert!(parse(Frame::Array(command)).is_err())
}

#[test]
fn parse_given_set_command_without_key_returns_error() {
    let command = vec![get_frame_from_str("set"), get_frame_from_str("my_keyspace")];
    assert!(parse(Frame::Array(command)).is_err())
}

#[test]
fn parse_given_set_command_without_value_returns_error() {
    let command = vec![
        get_frame_from_str("set"),
        get_frame_from_str("my_keyspace"),
        get_frame_from_str("foo"),
    ];
    assert!(parse(Frame::Array(command)).is_err())
}

#[test]
fn parse_given_set_command_without_config_returns_set() {
    let command = vec![
        get_frame_from_str("set"),
        get_frame_from_str("my_keyspace"),
        get_frame_from_str("foo"),
        get_frame_from_str("bar"),
    ];
    assert_eq!(
        parse(Frame::Array(command)).unwrap(),
        Command::Set(Set {
            keyspace: Bytes::from("my_keyspace"),
            key: Bytes::from("foo"),
            value: Bytes::from("bar"),
            if_not_exists: false,
            expire_at: None,
            if_exists: false
        })
    );
}

#[test]
fn parse_given_set_command_with_incomplete_expire_at_returns_error() {
    let command = vec![
        get_frame_from_str("set"),
        get_frame_from_str("my_keyspace"),
        get_frame_from_str("foo"),
        get_frame_from_str("bar"),
        get_frame_from_str("expire"),
    ];
    assert!(parse(Frame::Array(command)).is_err())
}

#[test]
fn parse_given_set_command_without_expire_at_value_returns_error() {
    let command = vec![
        get_frame_from_str("set"),
        get_frame_from_str("my_keyspace"),
        get_frame_from_str("foo"),
        get_frame_from_str("bar"),
        get_frame_from_str("expire"),
        get_frame_from_str("at"),
    ];
    assert!(parse(Frame::Array(command)).is_err())
}

#[test]
fn parse_given_set_command_with_invalid_expire_at_value_returns_error() {
    let command = vec![
        get_frame_from_str("set"),
        get_frame_from_str("my_keyspace"),
        get_frame_from_str("foo"),
        get_frame_from_str("bar"),
        get_frame_from_str("expire"),
        get_frame_from_str("at"),
        get_frame_from_str("baz"),
    ];
    assert!(parse(Frame::Array(command)).is_err())
}

#[test]
fn parse_given_set_command_with_expire_at_returns_set() {
    let command = vec![
        get_frame_from_str("set"),
        get_frame_from_str("my_keyspace"),
        get_frame_from_str("foo"),
        get_frame_from_str("bar"),
        get_frame_from_str("expire"),
        get_frame_from_str("at"),
        get_frame_from_str("1667041052"),
    ];
    assert_eq!(
        parse(Frame::Array(command)).unwrap(),
        Command::Set(Set {
            keyspace: Bytes::from("my_keyspace"),
            key: Bytes::from("foo"),
            value: Bytes::from("bar"),
            if_not_exists: false,
            expire_at: Some(1667041052),
            if_exists: false
        })
    );
}

#[test]
fn parse_given_set_command_with_incomplete_expire_after_returns_error() {
    let command = vec![
        get_frame_from_str("set"),
        get_frame_from_str("my_keyspace"),
        get_frame_from_str("foo"),
        get_frame_from_str("bar"),
        get_frame_from_str("expire"),
    ];
    assert!(parse(Frame::Array(command)).is_err())
}

#[test]
fn parse_given_set_command_without_expire_after_value_returns_error() {
    let command = vec![
        get_frame_from_str("set"),
        get_frame_from_str("my_keyspace"),
        get_frame_from_str("foo"),
        get_frame_from_str("bar"),
        get_frame_from_str("expire"),
        get_frame_from_str("after"),
    ];
    assert!(parse(Frame::Array(command)).is_err())
}

#[test]
fn parse_given_set_command_with_invalid_expire_after_value_returns_error() {
    let command = vec![
        get_frame_from_str("set"),
        get_frame_from_str("my_keyspace"),
        get_frame_from_str("foo"),
        get_frame_from_str("bar"),
        get_frame_from_str("expire"),
        get_frame_from_str("after"),
        get_frame_from_str("baz"),
    ];
    assert!(parse(Frame::Array(command)).is_err())
}

// FIXME: this is a finicky test, find a better way to test the timestamp
#[test]
fn parse_given_set_command_with_expire_after_returns_set() {
    let command = vec![
        get_frame_from_str("set"),
        get_frame_from_str("my_keyspace"),
        get_frame_from_str("foo"),
        get_frame_from_str("bar"),
        get_frame_from_str("expire"),
        get_frame_from_str("after"),
        get_frame_from_str("60000"),
    ];

    let timestamp = SystemTime::now()
        .add(Duration::from_millis(60000))
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    assert_eq!(
        parse(Frame::Array(command)).unwrap(),
        Command::Set(Set {
            keyspace: Bytes::from("my_keyspace"),
            key: Bytes::from("foo"),
            value: Bytes::from("bar"),
            if_not_exists: false,
            expire_at: Some(timestamp),
            if_exists: false
        })
    );
}

#[test]
fn parse_given_set_command_with_both_expire_at_and_expire_after_returns_error() {
    let command = vec![
        get_frame_from_str("set"),
        get_frame_from_str("my_keyspace"),
        get_frame_from_str("foo"),
        get_frame_from_str("bar"),
        get_frame_from_str("expire"),
        get_frame_from_str("after"),
        get_frame_from_str("100"),
        get_frame_from_str("expire"),
        get_frame_from_str("at"),
        get_frame_from_str("1667041052"),
    ];
    assert!(parse(Frame::Array(command)).is_err())
}

#[test]
fn parse_given_set_command_with_incomplete_if_flag_1_returns_error() {
    let command = vec![
        get_frame_from_str("set"),
        get_frame_from_str("my_keyspace"),
        get_frame_from_str("foo"),
        get_frame_from_str("bar"),
        get_frame_from_str("expire"),
        get_frame_from_str("after"),
        get_frame_from_str("100"),
        get_frame_from_str("if"),
    ];
    assert!(parse(Frame::Array(command)).is_err())
}

#[test]
fn parse_given_set_command_with_invalid_if_flag_1_returns_error() {
    let command = vec![
        get_frame_from_str("set"),
        get_frame_from_str("my_keyspace"),
        get_frame_from_str("foo"),
        get_frame_from_str("bar"),
        get_frame_from_str("expire"),
        get_frame_from_str("after"),
        get_frame_from_str("100"),
        get_frame_from_str("if"),
        get_frame_from_str("foo"),
    ];
    assert!(parse(Frame::Array(command)).is_err())
}

#[test]
fn parse_given_set_command_with_if_exists_flag_returns_set() {
    let command = vec![
        get_frame_from_str("set"),
        get_frame_from_str("my_keyspace"),
        get_frame_from_str("foo"),
        get_frame_from_str("bar"),
        get_frame_from_str("if"),
        get_frame_from_str("exists"),
    ];

    assert_eq!(
        parse(Frame::Array(command)).unwrap(),
        Command::Set(Set {
            keyspace: Bytes::from("my_keyspace"),
            key: Bytes::from("foo"),
            value: Bytes::from("bar"),
            if_not_exists: false,
            expire_at: None,
            if_exists: true
        })
    );
}

#[test]
fn parse_given_set_command_with_incomplete_if_flag_2_returns_error() {
    let command = vec![
        get_frame_from_str("set"),
        get_frame_from_str("my_keyspace"),
        get_frame_from_str("foo"),
        get_frame_from_str("bar"),
        get_frame_from_str("expire"),
        get_frame_from_str("after"),
        get_frame_from_str("100"),
        get_frame_from_str("if"),
        get_frame_from_str("not"),
    ];
    assert!(parse(Frame::Array(command)).is_err())
}

#[test]
fn parse_given_set_command_with_invalid_if_flag_2_returns_error() {
    let command = vec![
        get_frame_from_str("set"),
        get_frame_from_str("my_keyspace"),
        get_frame_from_str("foo"),
        get_frame_from_str("bar"),
        get_frame_from_str("expire"),
        get_frame_from_str("after"),
        get_frame_from_str("100"),
        get_frame_from_str("if"),
        get_frame_from_str("not"),
        get_frame_from_str("foo"),
    ];
    assert!(parse(Frame::Array(command)).is_err())
}

#[test]
fn parse_given_set_command_with_if_not_exists_flag_returns_set() {
    let command = vec![
        get_frame_from_str("set"),
        get_frame_from_str("my_keyspace"),
        get_frame_from_str("foo"),
        get_frame_from_str("bar"),
        get_frame_from_str("if"),
        get_frame_from_str("not"),
        get_frame_from_str("exists"),
    ];

    assert_eq!(
        parse(Frame::Array(command)).unwrap(),
        Command::Set(Set {
            keyspace: Bytes::from("my_keyspace"),
            key: Bytes::from("foo"),
            value: Bytes::from("bar"),
            if_not_exists: true,
            expire_at: None,
            if_exists: false
        })
    );
}

#[test]
fn parse_given_set_command_with_both_if_exists_and_if_not_exists_returns_error() {
    let command = vec![
        get_frame_from_str("set"),
        get_frame_from_str("my_keyspace"),
        get_frame_from_str("foo"),
        get_frame_from_str("bar"),
        get_frame_from_str("expire"),
        get_frame_from_str("after"),
        get_frame_from_str("100"),
        get_frame_from_str("if"),
        get_frame_from_str("not"),
        get_frame_from_str("exists"),
        get_frame_from_str("if"),
        get_frame_from_str("exists"),
    ];
    assert!(parse(Frame::Array(command)).is_err())
}

#[test]
fn parse_given_set_command_with_if_not_exists_and_tll_returns_set() {
    let command = vec![
        get_frame_from_str("set"),
        get_frame_from_str("my_keyspace"),
        get_frame_from_str("foo"),
        get_frame_from_str("bar"),
        get_frame_from_str("expire"),
        get_frame_from_str("at"),
        get_frame_from_str("1667041052"),
        get_frame_from_str("if"),
        get_frame_from_str("not"),
        get_frame_from_str("exists"),
    ];

    assert_eq!(
        parse(Frame::Array(command)).unwrap(),
        Command::Set(Set {
            keyspace: Bytes::from("my_keyspace"),
            key: Bytes::from("foo"),
            value: Bytes::from("bar"),
            if_not_exists: true,
            expire_at: Some(1667041052),
            if_exists: false
        })
    );
}

#[test]
fn parse_given_set_command_with_if_exists_and_tll_returns_set() {
    let command = vec![
        get_frame_from_str("set"),
        get_frame_from_str("my_keyspace"),
        get_frame_from_str("foo"),
        get_frame_from_str("bar"),
        get_frame_from_str("expire"),
        get_frame_from_str("at"),
        get_frame_from_str("1667041052"),
        get_frame_from_str("if"),
        get_frame_from_str("exists"),
    ];

    assert_eq!(
        parse(Frame::Array(command)).unwrap(),
        Command::Set(Set {
            keyspace: Bytes::from("my_keyspace"),
            key: Bytes::from("foo"),
            value: Bytes::from("bar"),
            if_not_exists: false,
            expire_at: Some(1667041052),
            if_exists: true
        })
    );
}

#[test]
fn parse_given_set_command_with_if_not_exists_and_tll_reverse_order_returns_set() {
    let command = vec![
        get_frame_from_str("set"),
        get_frame_from_str("my_keyspace"),
        get_frame_from_str("foo"),
        get_frame_from_str("bar"),
        get_frame_from_str("if"),
        get_frame_from_str("not"),
        get_frame_from_str("exists"),
        get_frame_from_str("expire"),
        get_frame_from_str("at"),
        get_frame_from_str("1667041052"),
    ];

    assert_eq!(
        parse(Frame::Array(command)).unwrap(),
        Command::Set(Set {
            keyspace: Bytes::from("my_keyspace"),
            key: Bytes::from("foo"),
            value: Bytes::from("bar"),
            if_not_exists: true,
            expire_at: Some(1667041052),
            if_exists: false
        })
    );
}

#[test]
fn parse_given_set_command_with_if_exists_and_tll_reverse_order_returns_set() {
    let command = vec![
        get_frame_from_str("set"),
        get_frame_from_str("my_keyspace"),
        get_frame_from_str("foo"),
        get_frame_from_str("bar"),
        get_frame_from_str("if"),
        get_frame_from_str("exists"),
        get_frame_from_str("expire"),
        get_frame_from_str("at"),
        get_frame_from_str("1667041052"),
    ];

    assert_eq!(
        parse(Frame::Array(command)).unwrap(),
        Command::Set(Set {
            keyspace: Bytes::from("my_keyspace"),
            key: Bytes::from("foo"),
            value: Bytes::from("bar"),
            if_not_exists: false,
            expire_at: Some(1667041052),
            if_exists: true
        })
    );
}
