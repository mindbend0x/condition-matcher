//! Basic usage example for condition-matcher

use condition_matcher::{
    Condition, ConditionOperator, ConditionSelector, Matchable, MatchableDerive, Matcher,
    MatcherMode,
};

// Define a struct and derive Matchable automatically!
#[derive(MatchableDerive, PartialEq, Debug)]
struct User {
    id: i32,
    name: String,
    age: u32,
    is_active: bool,
}

fn main() {
    let user = User {
        id: 42,
        name: "Alice".to_string(),
        age: 30,
        is_active: true,
    };

    // Example 1: Match by field value (automatic field access!)
    let mut matcher = Matcher::new(MatcherMode::AND);
    matcher
        .add_condition(Condition {
            selector: ConditionSelector::FieldValue("id", &42i32),
            operator: ConditionOperator::Equals,
        })
        .add_condition(Condition {
            selector: ConditionSelector::FieldValue("is_active", &true),
            operator: ConditionOperator::Equals,
        });

    println!(
        "User matches ID=42 AND is_active=true: {}",
        matcher.run(&user).unwrap()
    );

    // Example 2: Match by string field
    let expected_name = "Alice".to_string();
    let mut name_matcher = Matcher::new(MatcherMode::AND);
    name_matcher.add_condition(Condition {
        selector: ConditionSelector::FieldValue("name", &expected_name),
        operator: ConditionOperator::Equals,
    });

    println!(
        "User name is 'Alice': {}",
        name_matcher.run(&user).unwrap()
    );

    // Example 3: OR mode - match either condition
    let mut or_matcher = Matcher::new(MatcherMode::OR);
    or_matcher
        .add_condition(Condition {
            selector: ConditionSelector::FieldValue("id", &100i32),
            operator: ConditionOperator::Equals,
        })
        .add_condition(Condition {
            selector: ConditionSelector::FieldValue("age", &30u32),
            operator: ConditionOperator::Equals,
        });

    println!(
        "User matches ID=100 OR age=30: {}",
        or_matcher.run(&user).unwrap()
    );

    // Example 4: Numeric comparisons (NEW!)
    let mut age_matcher = Matcher::new(MatcherMode::AND);
    age_matcher.add_condition(Condition {
        selector: ConditionSelector::FieldValue("age", &18u32),
        operator: ConditionOperator::GreaterThanOrEqual,
    });

    println!(
        "User is 18 or older: {}",
        age_matcher.run(&user).unwrap()
    );

    // Example 5: String operations (NEW!)
    let mut name_contains = Matcher::new(MatcherMode::AND);
    name_contains.add_condition(Condition {
        selector: ConditionSelector::FieldValue("name", &"lic"),
        operator: ConditionOperator::Contains,
    });

    println!(
        "User name contains 'lic': {}",
        name_contains.run(&user).unwrap()
    );

    // Example 6: Complex nested struct
    #[derive(MatchableDerive, PartialEq, Debug)]
    struct Address {
        city: String,
        zip: i32,
    }

    let address = Address {
        city: "New York".to_string(),
        zip: 10001,
    };

    let mut address_matcher = Matcher::new(MatcherMode::AND);
    address_matcher.add_condition(Condition {
        selector: ConditionSelector::FieldValue("zip", &10001i32),
        operator: ConditionOperator::Equals,
    });

    println!(
        "Address ZIP is 10001: {}",
        address_matcher.run(&address).unwrap()
    );

    // Example 7: Detailed results (NEW!)
    let result = matcher.run_detailed(&user).unwrap();
    println!("\n=== Detailed Results ===");
    println!("Overall match: {}", result.is_match());
    println!("Conditions passed: {}", result.passed_conditions().len());
    println!("Conditions failed: {}", result.failed_conditions().len());

    // Example 8: Builder API (NEW!)
    use condition_matcher::MatcherBuilder;

    let builder_matcher = MatcherBuilder::<&str>::new()
        .mode(MatcherMode::AND)
        .length_gte(4)
        .value_not_equals("bad")
        .build();

    println!(
        "\nBuilder matcher on 'good': {}",
        builder_matcher.run(&"good").unwrap()
    );
    println!(
        "Builder matcher on 'bad': {}",
        builder_matcher.run(&"bad").unwrap()
    );
}
