// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use crate::{
        builder::{field, MatcherBuilder},
        condition::ConditionMode,
        condition::{Condition, ConditionOperator, ConditionSelector},
        matchers::RuleMatcher,
        traits::{Evaluate, Matcher},
        Matchable, MatchableDerive,
    };

    #[test]
    fn test_matcher_and_mode() {
        let mut matcher: RuleMatcher<&str> = RuleMatcher::new(ConditionMode::AND);
        matcher
            .add_condition(Condition {
                selector: ConditionSelector::Length(5),
                operator: ConditionOperator::GreaterThanOrEqual,
            })
            .add_condition(Condition {
                selector: ConditionSelector::Value("something"),
                operator: ConditionOperator::NotEquals,
            });

        assert_eq!(matcher.matches(&"test"), false);
        assert_eq!(matcher.matches(&"test12345"), true);
        assert_eq!(matcher.matches(&"something"), false);
        assert_eq!(matcher.matches(&"somethingelse"), true);
    }

    #[test]
    fn test_matcher_or_mode() {
        let mut matcher: RuleMatcher<&str> = RuleMatcher::new(ConditionMode::OR);
        matcher
            .add_condition(Condition {
                selector: ConditionSelector::Length(4),
                operator: ConditionOperator::Equals,
            })
            .add_condition(Condition {
                selector: ConditionSelector::Value("hello"),
                operator: ConditionOperator::Equals,
            });

        assert_eq!(matcher.matches(&"test"), true);
        assert_eq!(matcher.matches(&"hello"), true);
        assert_eq!(matcher.matches(&"world"), false);
    }

    #[test]
    fn test_matcher_xor_mode() {
        let mut matcher: RuleMatcher<&str> = RuleMatcher::new(ConditionMode::XOR);
        matcher
            .add_condition(Condition {
                selector: ConditionSelector::Length(4),
                operator: ConditionOperator::Equals,
            })
            .add_condition(Condition {
                selector: ConditionSelector::Value("test"),
                operator: ConditionOperator::Equals,
            });

        assert_eq!(matcher.matches(&"test"), false);
        assert_eq!(matcher.matches(&"hello"), false);
        assert_eq!(matcher.matches(&"abcd"), true);
    }

    #[test]
    fn test_type_checking() {
        let mut matcher: RuleMatcher<&str> = RuleMatcher::new(ConditionMode::AND);
        matcher.add_condition(Condition {
            selector: ConditionSelector::Type("&str".to_string()),
            operator: ConditionOperator::Equals,
        });

        assert_eq!(matcher.matches(&"test"), true);
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
        let mut matcher: RuleMatcher<TestStruct> = RuleMatcher::new(ConditionMode::AND);
        matcher.add_condition(Condition {
            selector: ConditionSelector::FieldValue("a", &1i32),
            operator: ConditionOperator::Equals,
        });
        assert_eq!(matcher.matches(&test_value), true);

        // Test not equals
        let mut matcher2: RuleMatcher<TestStruct> = RuleMatcher::new(ConditionMode::AND);
        matcher2.add_condition(Condition {
            selector: ConditionSelector::FieldValue("a", &2i32),
            operator: ConditionOperator::Equals,
        });
        assert_eq!(matcher2.matches(&test_value), false);

        // Test string field
        let mut matcher3: RuleMatcher<TestStruct> = RuleMatcher::new(ConditionMode::AND);
        matcher3.add_condition(Condition {
            selector: ConditionSelector::FieldValue("b", &"test"),
            operator: ConditionOperator::Equals,
        });
        assert_eq!(matcher3.matches(&test_value), true);
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
        let mut matcher: RuleMatcher<Person> = RuleMatcher::new(ConditionMode::AND);
        matcher.add_condition(Condition {
            selector: ConditionSelector::FieldValue("age", &18u32),
            operator: ConditionOperator::GreaterThan,
        });
        assert!(matcher.matches(&person));

        // Test less than or equal
        let mut matcher2: RuleMatcher<Person> = RuleMatcher::new(ConditionMode::AND);
        matcher2.add_condition(Condition {
            selector: ConditionSelector::FieldValue("age", &25u32),
            operator: ConditionOperator::LessThanOrEqual,
        });
        assert!(matcher2.matches(&person));

        // Test float comparison
        let mut matcher3: RuleMatcher<Person> = RuleMatcher::new(ConditionMode::AND);
        matcher3.add_condition(Condition {
            selector: ConditionSelector::FieldValue("score", &80.0f64),
            operator: ConditionOperator::GreaterThan,
        });
        assert!(matcher3.matches(&person));
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
        let mut matcher: RuleMatcher<Email> = RuleMatcher::new(ConditionMode::AND);
        matcher.add_condition(Condition {
            selector: ConditionSelector::FieldValue("address", &"@example"),
            operator: ConditionOperator::Contains,
        });
        assert!(matcher.matches(&email));

        // Test starts with
        let mut matcher2: RuleMatcher<Email> = RuleMatcher::new(ConditionMode::AND);
        matcher2.add_condition(Condition {
            selector: ConditionSelector::FieldValue("address", &"user@"),
            operator: ConditionOperator::StartsWith,
        });
        assert!(matcher2.matches(&email));

        // Test ends with
        let mut matcher3: RuleMatcher<Email> = RuleMatcher::new(ConditionMode::AND);
        matcher3.add_condition(Condition {
            selector: ConditionSelector::FieldValue("address", &".com"),
            operator: ConditionOperator::EndsWith,
        });
        assert!(matcher3.matches(&email));

        // Test not contains
        let mut matcher4: RuleMatcher<Email> = RuleMatcher::new(ConditionMode::AND);
        matcher4.add_condition(Condition {
            selector: ConditionSelector::FieldValue("address", &"@gmail"),
            operator: ConditionOperator::NotContains,
        });
        assert!(matcher4.matches(&email));
    }

    #[test]
    fn test_detailed_results() {
        let mut matcher: RuleMatcher<&str> = RuleMatcher::new(ConditionMode::AND);
        matcher
            .add_condition(Condition {
                selector: ConditionSelector::Length(4),
                operator: ConditionOperator::Equals,
            })
            .add_condition(Condition {
                selector: ConditionSelector::Value("test"),
                operator: ConditionOperator::Equals,
            });

        let result = matcher.evaluate(&"test");
        assert!(result.is_match());
        assert_eq!(result.passed_conditions().len(), 2);
        assert_eq!(result.failed_conditions().len(), 0);

        let result2 = matcher.evaluate(&"hello");
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

        assert!(matcher.matches(&"good"));
        assert!(!matcher.matches(&"bad"));
        assert!(!matcher.matches(&"hi"));
    }

    #[test]
    fn test_field_builder() {
        #[derive(MatchableDerive, PartialEq, Debug)]
        struct User {
            age: u32,
        }

        let user = User { age: 25 };

        let condition = field::<User>("age").gte(&18u32);
        let mut matcher = RuleMatcher::new(ConditionMode::AND);
        matcher.add_condition(condition);

        assert!(matcher.matches(&user));
    }

    #[test]
    fn test_convenience_constructors() {
        let and_matcher: RuleMatcher<&str> = RuleMatcher::and();
        assert_eq!(and_matcher.mode, ConditionMode::AND);

        let or_matcher: RuleMatcher<&str> = RuleMatcher::or();
        assert_eq!(or_matcher.mode, ConditionMode::OR);

        let xor_matcher: RuleMatcher<&str> = RuleMatcher::xor();
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

        let mut matcher: RuleMatcher<User> = RuleMatcher::new(ConditionMode::AND);
        matcher.add_condition(Condition {
            selector: ConditionSelector::FieldValue("nonexistent", &"value"),
            operator: ConditionOperator::Equals,
        });

        let result = matcher.evaluate(&user);
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

        let mut matcher: RuleMatcher<Item> = RuleMatcher::new(ConditionMode::AND);
        matcher.add_condition(Condition {
            selector: ConditionSelector::Not(Box::new(inner_condition)),
            operator: ConditionOperator::Equals, // operator is ignored for NOT
        });

        assert!(matcher.matches(&item));
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
        let mut matcher: RuleMatcher<Profile> = RuleMatcher::new(ConditionMode::AND);
        matcher.add_condition(Condition {
            selector: ConditionSelector::FieldValue("nickname", &"Ali"),
            operator: ConditionOperator::Equals,
        });

        assert!(matcher.matches(&profile_with_nick));
        // When None, field access returns None, so the match fails
        assert!(!matcher.matches(&profile_without_nick));
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

        let mut matcher: RuleMatcher<Email> = RuleMatcher::new(ConditionMode::AND);
        matcher.add_condition(Condition {
            selector: ConditionSelector::FieldValue("address", &r"^[a-z]+@[a-z]+\.[a-z]+$"),
            operator: ConditionOperator::Regex,
        });

        assert!(matcher.matches(&email));

        // Test non-matching regex
        let bad_email = Email {
            address: "not-an-email".to_string(),
        };
        assert!(!matcher.matches(&bad_email));
    }

    // ========================================================================
    // New tests for the trait-based API
    // ========================================================================

    #[test]
    fn test_matcher_ext_filter() {
        use crate::traits::MatcherExt;

        let matcher = MatcherBuilder::<i32>::new().value_equals(42).build();

        let values = vec![40, 41, 42, 43, 42, 44];
        let matches = matcher.filter(&values);

        assert_eq!(matches.len(), 2);
        assert!(matches.iter().all(|&&v| v == 42));
    }

    #[test]
    fn test_matcher_ext_matches_all() {
        use crate::traits::MatcherExt;

        let matcher = MatcherBuilder::<i32>::new().value_equals(42).build();

        let values = vec![40, 42, 43];
        let results = matcher.matches_all(&values);

        assert_eq!(results, vec![false, true, false]);
    }

    #[cfg(feature = "json_condition")]
    #[test]
    fn test_json_matcher() {
        use crate::matchers::JsonMatcher;

        #[derive(MatchableDerive, PartialEq, Debug)]
        struct User {
            name: String,
            age: u32,
        }

        let user = User {
            name: "Alice".to_string(),
            age: 25,
        };

        let json = r#"{"mode": "AND", "rules": [{"field": "age", "operator": "greater_than_or_equal", "value": 18}]}"#;
        let matcher = JsonMatcher::from_json(json).unwrap();

        assert!(matcher.matches(&user));
    }

    #[cfg(feature = "json_condition")]
    #[test]
    fn test_json_matcher_complex() {
        use crate::matchers::JsonMatcher;

        #[derive(MatchableDerive, PartialEq, Debug)]
        struct Product {
            name: String,
            price: f64,
            in_stock: bool,
        }

        let product = Product {
            name: "Widget".to_string(),
            price: 29.99,
            in_stock: true,
        };

        // Test OR condition
        let json = r#"{
            "mode": "OR",
            "rules": [
                {"field": "price", "operator": "less_than", "value": 20.0},
                {"field": "in_stock", "operator": "equals", "value": true}
            ]
        }"#;
        let matcher = JsonMatcher::from_json(json).unwrap();
        assert!(matcher.matches(&product));

        // Test AND condition that should fail
        let json2 = r#"{
            "mode": "AND",
            "rules": [
                {"field": "price", "operator": "less_than", "value": 20.0},
                {"field": "in_stock", "operator": "equals", "value": true}
            ]
        }"#;
        let matcher2 = JsonMatcher::from_json(json2).unwrap();
        assert!(!matcher2.matches(&product));
    }

    #[test]
    fn test_batch_operations() {
        use crate::batch;

        let matcher = MatcherBuilder::<i32>::new().value_equals(42).build();

        // Test count_matching
        let count = batch::count_matching(&42, &[&matcher]);
        assert_eq!(count, 1);

        // Test any_matches
        assert!(batch::any_matches(&42, &[&matcher]));
        assert!(!batch::any_matches(&41, &[&matcher]));
    }
}
