use okane::data;
use okane::import;
use okane::import::config;
use okane::import::config::{FieldKey, FieldPos};

use indoc::indoc;
use maplit::hashmap;
use pretty_assertions::assert_eq;
use rust_decimal_macros::dec;

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn test_import_csv_index() {
    init();
    let config = config::ConfigEntry {
        path: "/path/to/match".into(),
        encoding: config::Encoding(encoding_rs::UTF_8),
        account: "Liabilities:Okane Card".to_string(),
        account_type: config::AccountType::Liability,
        commodity: "USD".to_string(),
        format: config::FormatSpec {
            date: "%Y-%m-%d".to_string(),
            fields: hashmap! {
                FieldKey::Date => FieldPos::Index(0),
                FieldKey::Amount => FieldPos::Index(1),
                FieldKey::Payee => FieldPos::Index(2),
            },
        },
        rewrite: vec![
            config::RewriteRule {
                payee: r#"Debit Card (?P<code>\d+) (?P<payee>.*)"#.to_string(),
                account: None,
            },
            config::RewriteRule {
                payee: r#"cashback"#.to_string(),
                account: Some("Incomes:Misc".to_string()),
            },
            config::RewriteRule {
                payee: r#"Migros"#.to_string(),
                account: Some("Expenses:Grocery".to_string()),
            },
        ],
    };
    let input = indoc! {r#"
        header
        2021-9-1,-50.00,cashback
        2021-9-2,28.00,Debit Card 31415 Migros
        2021-9-3,1.45,Debit Card 14142 FooBar
        ,,,,
    "#};
    let mut r = input.as_bytes();
    let transactions = import::import(&mut r, import::Format::CSV, &config).unwrap();
    let want = vec![
        data::Transaction {
            date: chrono::NaiveDate::from_ymd(2021, 9, 1),
            clear_state: data::ClearState::Cleared,
            code: None,
            payee: "cashback".to_string(),
            posts: vec![
                data::Post {
                    account: "Incomes:Misc".to_string(),
                    clear_state: data::ClearState::Uncleared,
                    amount: data::Amount {
                        value: dec!(-50.00),
                        commodity: "USD".to_string(),
                    },
                    balance: None,
                },
                data::Post {
                    account: "Liabilities:Okane Card".to_string(),
                    clear_state: data::ClearState::Uncleared,
                    amount: data::Amount {
                        value: dec!(50.00),
                        commodity: "USD".to_string(),
                    },
                    balance: None,
                },
            ],
        },
        data::Transaction {
            date: chrono::NaiveDate::from_ymd(2021, 9, 2),
            clear_state: data::ClearState::Cleared,
            code: Some("31415".to_string()),
            payee: "Migros".to_string(),
            posts: vec![
                data::Post {
                    account: "Liabilities:Okane Card".to_string(),
                    clear_state: data::ClearState::Uncleared,
                    amount: data::Amount {
                        value: dec!(-28.00),
                        commodity: "USD".to_string(),
                    },
                    balance: None,
                },
                data::Post {
                    account: "Expenses:Grocery".to_string(),
                    clear_state: data::ClearState::Uncleared,
                    amount: data::Amount {
                        value: dec!(28.00),
                        commodity: "USD".to_string(),
                    },
                    balance: None,
                },
            ],
        },
        data::Transaction {
            date: chrono::NaiveDate::from_ymd(2021, 9, 3),
            clear_state: data::ClearState::Cleared,
            code: Some("14142".to_string()),
            payee: "FooBar".to_string(),
            posts: vec![
                data::Post {
                    account: "Liabilities:Okane Card".to_string(),
                    clear_state: data::ClearState::Uncleared,
                    amount: data::Amount {
                        value: dec!(-1.45),
                        commodity: "USD".to_string(),
                    },
                    balance: None,
                },
                data::Post {
                    account: "Expenses:Unknown".to_string(),
                    clear_state: data::ClearState::Pending,
                    amount: data::Amount {
                        value: dec!(1.45),
                        commodity: "USD".to_string(),
                    },
                    balance: None,
                },
            ],
        },
    ];
    assert_eq!(want, transactions);
}

#[test]
fn test_import_csv_label() {
    init();
    let config = config::ConfigEntry {
        path: "/path/to/match".into(),
        encoding: config::Encoding(encoding_rs::UTF_8),
        account: "Assets:Okane Bank".to_string(),
        account_type: config::AccountType::Asset,
        commodity: "USD".to_string(),
        format: config::FormatSpec {
            date: "%Y/%m/%d".to_string(),
            fields: hashmap! {
                FieldKey::Date => FieldPos::Label("日付".to_string()),
                FieldKey::Payee => FieldPos::Label("摘要".to_string()),
                FieldKey::Debit => FieldPos::Label("引き出し額".to_string()),
                FieldKey::Credit => FieldPos::Label("預け入れ額".to_string()),
                FieldKey::Balance => FieldPos::Label("口座残高".to_string()),
            },
        },
        rewrite: vec![
            config::RewriteRule {
                payee: r#"Debit Card (?P<code>\d+) (?P<payee>.*)"#.to_string(),
                account: None,
            },
            config::RewriteRule {
                payee: r#"五反田ATM"#.to_string(),
                account: Some("Assets:Cash".to_string()),
            },
            config::RewriteRule {
                payee: r#"Migros"#.to_string(),
                account: Some("Expenses:Grocery".to_string()),
            },
        ],
    };
    let input = indoc! {r#"
        日付,摘要,預け入れ額,引き出し額,口座残高
        2021/09/01,五反田ATM,50.00,,123.45
        2021/09/02,Debit Card 31415 Migros,,28.00,95.45
        2021/09/03,Debit Card 14142 FooBar,,1.45,94.00
    "#};
    let mut r = input.as_bytes();
    let transactions = import::import(&mut r, import::Format::CSV, &config).unwrap();
    let want = vec![
        data::Transaction {
            date: chrono::NaiveDate::from_ymd(2021, 9, 1),
            clear_state: data::ClearState::Cleared,
            code: None,
            payee: "五反田ATM".to_string(),
            posts: vec![
                data::Post {
                    account: "Assets:Cash".to_string(),
                    clear_state: data::ClearState::Uncleared,
                    amount: data::Amount {
                        value: dec!(-50.00),
                        commodity: "USD".to_string(),
                    },
                    balance: None,
                },
                data::Post {
                    account: "Assets:Okane Bank".to_string(),
                    clear_state: data::ClearState::Uncleared,
                    amount: data::Amount {
                        value: dec!(50.00),
                        commodity: "USD".to_string(),
                    },
                    balance: Some(data::Amount {
                        value: dec!(123.45),
                        commodity: "USD".to_string(),
                    }),
                },
            ],
        },
        data::Transaction {
            date: chrono::NaiveDate::from_ymd(2021, 9, 2),
            clear_state: data::ClearState::Cleared,
            code: Some("31415".to_string()),
            payee: "Migros".to_string(),
            posts: vec![
                data::Post {
                    account: "Assets:Okane Bank".to_string(),
                    clear_state: data::ClearState::Uncleared,
                    amount: data::Amount {
                        value: dec!(-28.00),
                        commodity: "USD".to_string(),
                    },
                    balance: Some(data::Amount {
                        value: dec!(95.45),
                        commodity: "USD".to_string(),
                    }),
                },
                data::Post {
                    account: "Expenses:Grocery".to_string(),
                    clear_state: data::ClearState::Uncleared,
                    amount: data::Amount {
                        value: dec!(28.00),
                        commodity: "USD".to_string(),
                    },
                    balance: None,
                },
            ],
        },
        data::Transaction {
            date: chrono::NaiveDate::from_ymd(2021, 9, 3),
            clear_state: data::ClearState::Cleared,
            code: Some("14142".to_string()),
            payee: "FooBar".to_string(),
            posts: vec![
                data::Post {
                    account: "Assets:Okane Bank".to_string(),
                    clear_state: data::ClearState::Uncleared,
                    amount: data::Amount {
                        value: dec!(-1.45),
                        commodity: "USD".to_string(),
                    },
                    balance: Some(data::Amount {
                        value: dec!(94.00),
                        commodity: "USD".to_string(),
                    }),
                },
                data::Post {
                    account: "Expenses:Unknown".to_string(),
                    clear_state: data::ClearState::Pending,
                    amount: data::Amount {
                        value: dec!(1.45),
                        commodity: "USD".to_string(),
                    },
                    balance: None,
                },
            ],
        },
    ];
    assert_eq!(want, transactions);
}
