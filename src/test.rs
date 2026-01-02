// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use crate::{
        Matchable, MatchableDerive,
        builder::{MatcherBuilder, field},
        condition::ConditionMode,
        condition::{Condition, ConditionOperator, ConditionSelector},
        matcher::Matcher,
    };

    #[test]
    fn test_matcher_and_mode() {
        let mut matcher: Matcher<&str> = Matcher::new(ConditionMode::AND);
        matcher
            .add_condition(Condition {
                selector: ConditionSelector::Length(5),
                operator: ConditionOperator::GreaterThanOrEqual,
            })
            .add_condition(Condition {
                selector: ConditionSelector::Value("something"),
                operator: ConditionOperator::NotEquals,
            });

        assert_eq!(matcher.run(&"test").unwrap(), false);
        assert_eq!(matcher.run(&"test12345").unwrap(), true);
        assert_eq!(matcher.run(&"something").unwrap(), false);
        assert_eq!(matcher.run(&"somethingelse").unwrap(), true);
    }

    #[test]
    fn test_matcher_or_mode() {
        let mut matcher: Matcher<&str> = Matcher::new(ConditionMode::OR);
        matcher
            .add_condition(Condition {
                selector: ConditionSelector::Length(4),
                operator: ConditionOperator::Equals,
            })
            .add_condition(Condition {
                selector: ConditionSelector::Value("hello"),
                operator: ConditionOperator::Equals,
            });

        assert_eq!(matcher.run(&"test").unwrap(), true);
        assert_eq!(matcher.run(&"hello").unwrap(), true);
        assert_eq!(matcher.run(&"world").unwrap(), false);
    }

    #[test]
    fn test_matcher_xor_mode() {
        let mut matcher: Matcher<&str> = Matcher::new(ConditionMode::XOR);
        matcher
            .add_condition(Condition {
                selector: ConditionSelector::Length(4),
                operator: ConditionOperator::Equals,
            })
            .add_condition(Condition {
                selector: ConditionSelector::Value("test"),
                operator: ConditionOperator::Equals,
            });

        assert_eq!(matcher.run(&"test").unwrap(), false);
        assert_eq!(matcher.run(&"hello").unwrap(), false);
        assert_eq!(matcher.run(&"abcd").unwrap(), true);
    }

    #[test]
    fn test_type_checking() {
        let mut matcher: Matcher<&str> = Matcher::new(ConditionMode::AND);
        matcher.add_condition(Condition {
            selector: ConditionSelector::Type("&str".to_string()),
            operator: ConditionOperator::Equals,
        });

        assert_eq!(matcher.run(&"test").unwrap(), true);
    }

    #[test]
    fn test_field_checking() {
        #[derive(MatchableDerive, PartialEq, Debug)]
        struct TestStruct {
            a: i32,
            b: String,
        }

        let test_value = TestStruct {
            a: 1,
            b: "test".to_string(),
        };

        // Test equals
        let mut matcher: Matcher<TestStruct> = Matcher::new(ConditionMode::AND);
        matcher.add_condition(Condition {
            selector: ConditionSelector::FieldValue("a", &1i32),
            operator: ConditionOperator::Equals,
        });
        assert_eq!(matcher.run(&test_value).unwrap(), true);

        // Test not equals
        let mut matcher2: Matcher<TestStruct> = Matcher::new(ConditionMode::AND);
        matcher2.add_condition(Condition {
            selector: ConditionSelector::FieldValue("a", &2i32),
            operator: ConditionOperator::Equals,
        });
        assert_eq!(matcher2.run(&test_value).unwrap(), false);

        // Test string field
        let mut matcher3: Matcher<TestStruct> = Matcher::new(ConditionMode::AND);
        matcher3.add_condition(Condition {
            selector: ConditionSelector::FieldValue("b", &"test"),
            operator: ConditionOperator::Equals,
        });
        assert_eq!(matcher3.run(&test_value).unwrap(), true);
    }

    #[test]
    fn test_numeric_comparisons_on_fields() {
        #[derive(MatchableDerive, PartialEq, Debug)]
        struct Person {
            age: u32,
            score: f64,
        }

        let person = Person {
            age: 25,
            score: 85.5,
        };

        // Test greater than
        let mut matcher: Matcher<Person> = Matcher::new(ConditionMode::AND);
        matcher.add_condition(Condition {
            selector: ConditionSelector::FieldValue("age", &18u32),
            operator: ConditionOperator::GreaterThan,
        });
        assert!(matcher.run(&person).unwrap());

        // Test less than or equal
        let mut matcher2: Matcher<Person> = Matcher::new(ConditionMode::AND);
        matcher2.add_condition(Condition {
            selector: ConditionSelector::FieldValue("age", &25u32),
            operator: ConditionOperator::LessThanOrEqual,
        });
        assert!(matcher2.run(&person).unwrap());

        // Test float comparison
        let mut matcher3: Matcher<Person> = Matcher::new(ConditionMode::AND);
        matcher3.add_condition(Condition {
            selector: ConditionSelector::FieldValue("score", &80.0f64),
            operator: ConditionOperator::GreaterThan,
        });
        assert!(matcher3.run(&person).unwrap());
    }

    #[test]
    fn test_string_operations() {
        #[derive(MatchableDerive, PartialEq, Debug)]
        struct Email {
            address: String,
        }

        let email = Email {
            address: "user@example.com".to_string(),
        };

        // Test contains
        let mut matcher: Matcher<Email> = Matcher::new(ConditionMode::AND);
        matcher.add_condition(Condition {
            selector: ConditionSelector::FieldValue("address", &"@example"),
            operator: ConditionOperator::Contains,
        });
        assert!(matcher.run(&email).unwrap());

        // Test starts with
        let mut matcher2: Matcher<Email> = Matcher::new(ConditionMode::AND);
        matcher2.add_condition(Condition {
            selector: ConditionSelector::FieldValue("address", &"user@"),
            operator: ConditionOperator::StartsWith,
        });
        assert!(matcher2.run(&email).unwrap());

        // Test ends with
        let mut matcher3: Matcher<Email> = Matcher::new(ConditionMode::AND);
        matcher3.add_condition(Condition {
            selector: ConditionSelector::FieldValue("address", &".com"),
            operator: ConditionOperator::EndsWith,
        });
        assert!(matcher3.run(&email).unwrap());

        // Test not contains
        let mut matcher4: Matcher<Email> = Matcher::new(ConditionMode::AND);
        matcher4.add_condition(Condition {
            selector: ConditionSelector::FieldValue("address", &"@gmail"),
            operator: ConditionOperator::NotContains,
        });
        assert!(matcher4.run(&email).unwrap());
    }

    #[test]
    fn test_detailed_results() {
        let mut matcher: Matcher<&str> = Matcher::new(ConditionMode::AND);
        matcher
            .add_condition(Condition {
                selector: ConditionSelector::Length(4),
                operator: ConditionOperator::Equals,
            })
            .add_condition(Condition {
                selector: ConditionSelector::Value("test"),
                operator: ConditionOperator::Equals,
            });

        let result = matcher.run_detailed(&"test").unwrap();
        assert!(result.is_match());
        assert_eq!(result.passed_conditions().len(), 2);
        assert_eq!(result.failed_conditions().len(), 0);

        let result2 = matcher.run_detailed(&"hello").unwrap();
        assert!(!result2.is_match());
        assert_eq!(result2.passed_conditions().len(), 0);
        assert_eq!(result2.failed_conditions().len(), 2);
    }

    #[test]
    fn test_builder_api() {
        let matcher = MatcherBuilder::<&str>::new()
            .mode(ConditionMode::AND)
            .length_gte(4)
            .value_not_equals("bad")
            .build();

        assert!(matcher.run(&"good").unwrap());
        assert!(!matcher.run(&"bad").unwrap());
        assert!(!matcher.run(&"hi").unwrap());
    }

    #[test]
    fn test_field_builder() {
        #[derive(MatchableDerive, PartialEq, Debug)]
        struct User {
            age: u32,
        }

        let user = User { age: 25 };

        let condition = field::<User>("age").gte(&18u32);
        let mut matcher = Matcher::new(ConditionMode::AND);
        matcher.add_condition(condition);

        assert!(matcher.run(&user).unwrap());
    }

    #[test]
    fn test_convenience_constructors() {
        let and_matcher: Matcher<&str> = Matcher::and();
        assert_eq!(and_matcher.mode, ConditionMode::AND);

        let or_matcher: Matcher<&str> = Matcher::or();
        assert_eq!(or_matcher.mode, ConditionMode::OR);

        let xor_matcher: Matcher<&str> = Matcher::xor();
        assert_eq!(xor_matcher.mode, ConditionMode::XOR);
    }

    #[test]
    fn test_error_on_missing_field() {
        #[derive(MatchableDerive, PartialEq, Debug)]
        struct User {
            name: String,
        }

        let user = User {
            name: "Alice".to_string(),
        };

        let mut matcher: Matcher<User> = Matcher::new(ConditionMode::AND);
        matcher.add_condition(Condition {
            selector: ConditionSelector::FieldValue("nonexistent", &"value"),
            operator: ConditionOperator::Equals,
        });

        let result = matcher.run_detailed(&user).unwrap();
        assert!(!result.is_match());

        let failed = result.failed_conditions();
        assert_eq!(failed.len(), 1);
        assert!(failed[0].error.is_some());
    }

    #[test]
    fn test_not_operator() {
        #[derive(MatchableDerive, PartialEq, Debug)]
        struct Item {
            active: bool,
        }

        let item = Item { active: false };

        // Test NOT operator - should match because NOT(active=true) is true when active=false
        let inner_condition = Condition {
            selector: ConditionSelector::FieldValue("active", &true),
            operator: ConditionOperator::Equals,
        };

        let mut matcher: Matcher<Item> = Matcher::new(ConditionMode::AND);
        matcher.add_condition(Condition {
            selector: ConditionSelector::Not(Box::new(inner_condition)),
            operator: ConditionOperator::Equals, // operator is ignored for NOT
        });

        assert!(matcher.run(&item).unwrap());
    }

    #[test]
    fn test_optional_fields() {
        #[derive(MatchableDerive, PartialEq, Debug)]
        struct Profile {
            name: String,
            nickname: Option<String>,
        }

        let profile_with_nick = Profile {
            name: "Alice".to_string(),
            nickname: Some("Ali".to_string()),
        };

        let profile_without_nick = Profile {
            name: "Bob".to_string(),
            nickname: None,
        };

        // Test matching optional field when present
        let mut matcher: Matcher<Profile> = Matcher::new(ConditionMode::AND);
        matcher.add_condition(Condition {
            selector: ConditionSelector::FieldValue("nickname", &"Ali"),
            operator: ConditionOperator::Equals,
        });

        assert!(matcher.run(&profile_with_nick).unwrap());
        // When None, field access returns None, so the match fails
        assert!(!matcher.run(&profile_without_nick).unwrap());
    }

    #[cfg(feature = "regex")]
    #[test]
    fn test_regex_matching() {
        #[derive(MatchableDerive, PartialEq, Debug)]
        struct Email {
            address: String,
        }

        let email = Email {
            address: "user@example.com".to_string(),
        };

        let mut matcher: Matcher<Email> = Matcher::new(ConditionMode::AND);
        matcher.add_condition(Condition {
            selector: ConditionSelector::FieldValue("address", &r"^[a-z]+@[a-z]+\.[a-z]+$"),
            operator: ConditionOperator::Regex,
        });

        assert!(matcher.run(&email).unwrap());

        // Test non-matching regex
        let bad_email = Email {
            address: "not-an-email".to_string(),
        };
        assert!(!matcher.run(&bad_email).unwrap());
    }
}
