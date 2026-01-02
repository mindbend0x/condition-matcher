//! Basic usage example for condition-matcher

use condition_matcher::{
    Condition, ConditionMode, ConditionOperator, ConditionSelector, Evaluate, Matchable,
    MatchableDerive, Matcher, RuleMatcher,
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
    let mut matcher = RuleMatcher::new(ConditionMode::AND);
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
        matcher.matches(&user)
    );

    // Example 2: Match by string field
    let expected_name = "Alice".to_string();
    let mut name_matcher = RuleMatcher::new(ConditionMode::AND);
    name_matcher.add_condition(Condition {
        selector: ConditionSelector::FieldValue("name", &expected_name),
        operator: ConditionOperator::Equals,
    });

    println!("User name is 'Alice': {}", name_matcher.matches(&user));

    // Example 3: OR mode - match either condition
    let mut or_matcher = RuleMatcher::new(ConditionMode::OR);
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
        or_matcher.matches(&user)
    );

    // Example 4: Numeric comparisons
    let mut age_matcher = RuleMatcher::new(ConditionMode::AND);
    age_matcher.add_condition(Condition {
        selector: ConditionSelector::FieldValue("age", &18u32),
        operator: ConditionOperator::GreaterThanOrEqual,
    });

    println!("User is 18 or older: {}", age_matcher.matches(&user));

    // Example 5: String operations
    let mut name_contains = RuleMatcher::new(ConditionMode::AND);
    name_contains.add_condition(Condition {
        selector: ConditionSelector::FieldValue("name", &"lic"),
        operator: ConditionOperator::Contains,
    });

    println!(
        "User name contains 'lic': {}",
        name_contains.matches(&user)
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

    let mut address_matcher = RuleMatcher::new(ConditionMode::AND);
    address_matcher.add_condition(Condition {
        selector: ConditionSelector::FieldValue("zip", &10001i32),
        operator: ConditionOperator::Equals,
    });

    println!("Address ZIP is 10001: {}", address_matcher.matches(&address));

    // Example 7: Detailed results
    let result = matcher.evaluate(&user);
    println!("\n=== Detailed Results ===");
    println!("Overall match: {}", result.is_match());
    println!("Conditions passed: {}", result.passed_conditions().len());
    println!("Conditions failed: {}", result.failed_conditions().len());

    // Example 8: Builder API
    use condition_matcher::MatcherBuilder;

    let builder_matcher = MatcherBuilder::<&str>::new()
        .mode(ConditionMode::AND)
        .length_gte(4)
        .value_not_equals("bad")
        .build();

    println!(
        "\nBuilder matcher on 'good': {}",
        builder_matcher.matches(&"good")
    );
    println!(
        "Builder matcher on 'bad': {}",
        builder_matcher.matches(&"bad")
    );
}
