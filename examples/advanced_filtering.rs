//! Advanced filtering example demonstrating real-world use cases

use condition_matcher::{
    field, Condition, ConditionOperator, ConditionSelector, Matchable, MatchableDerive, Matcher,
    MatcherMode,
};

// Complex example with multiple struct types
#[derive(MatchableDerive, PartialEq, Debug)]
struct Product {
    id: i32,
    name: String,
    price: f64,
    in_stock: bool,
    quantity: u32,
}

#[derive(MatchableDerive, PartialEq, Debug)]
struct Order {
    order_id: i32,
    customer_name: String,
    total_amount: f64,
    is_paid: bool,
}

fn main() {
    println!("=== Product Filtering Examples ===\n");

    let products = vec![
        Product {
            id: 1,
            name: "Laptop".to_string(),
            price: 999.99,
            in_stock: true,
            quantity: 5,
        },
        Product {
            id: 2,
            name: "Mouse".to_string(),
            price: 29.99,
            in_stock: true,
            quantity: 50,
        },
        Product {
            id: 3,
            name: "Keyboard".to_string(),
            price: 79.99,
            in_stock: false,
            quantity: 0,
        },
    ];

    // Example 1: Find products that are in stock
    println!("1. Finding products in stock:");
    let mut in_stock_matcher = Matcher::new(MatcherMode::AND);
    in_stock_matcher.add_condition(Condition {
        selector: ConditionSelector::FieldValue("in_stock", &true),
        operator: ConditionOperator::Equals,
    });

    for product in &products {
        if in_stock_matcher.run(product).unwrap_or(false) {
            println!(
                "   ✓ {} (${}) - Qty: {}",
                product.name, product.price, product.quantity
            );
        }
    }

    // Example 2: Find expensive products (price > $50) that are in stock using numeric comparison
    println!("\n2. Finding products with price > $50:");
    let mut expensive_matcher = Matcher::new(MatcherMode::AND);
    expensive_matcher.add_condition(Condition {
        selector: ConditionSelector::FieldValue("price", &50.0f64),
        operator: ConditionOperator::GreaterThan,
    });

    for product in &products {
        if expensive_matcher.run(product).unwrap_or(false) {
            println!("   ✓ {} - ${}", product.name, product.price);
        }
    }

    // Example 3: Using OR mode - products that are either out of stock OR have quantity < 10
    println!("\n3. Products out of stock OR low quantity (<10):");
    let mut or_matcher = Matcher::new(MatcherMode::OR);
    or_matcher
        .add_condition(Condition {
            selector: ConditionSelector::FieldValue("in_stock", &false),
            operator: ConditionOperator::Equals,
        })
        .add_condition(Condition {
            selector: ConditionSelector::FieldValue("quantity", &10u32),
            operator: ConditionOperator::LessThan,
        });

    for product in &products {
        if or_matcher.run(product).unwrap_or(false) {
            println!(
                "   ⚠ {} - Stock: {}, Qty: {}",
                product.name, product.in_stock, product.quantity
            );
        }
    }

    // Example 4: String operations - find products with names containing certain text
    println!("\n4. Products with 'boa' in name:");
    let mut name_matcher = Matcher::new(MatcherMode::AND);
    name_matcher.add_condition(Condition {
        selector: ConditionSelector::FieldValue("name", &"boa"),
        operator: ConditionOperator::Contains,
    });

    for product in &products {
        if name_matcher.run(product).unwrap_or(false) {
            println!("   ✓ {}", product.name);
        } else {
            println!("   ✗ {} (no match)", product.name);
        }
    }

    // Example 5: Orders with complex conditions
    println!("\n=== Order Filtering Examples ===\n");

    let orders = vec![
        Order {
            order_id: 101,
            customer_name: "Alice".to_string(),
            total_amount: 150.50,
            is_paid: true,
        },
        Order {
            order_id: 102,
            customer_name: "Bob".to_string(),
            total_amount: 75.25,
            is_paid: false,
        },
        Order {
            order_id: 103,
            customer_name: "Charlie".to_string(),
            total_amount: 200.00,
            is_paid: true,
        },
    ];

    // Find paid orders
    println!("5. Finding paid orders:");
    let mut paid_matcher = Matcher::new(MatcherMode::AND);
    paid_matcher.add_condition(Condition {
        selector: ConditionSelector::FieldValue("is_paid", &true),
        operator: ConditionOperator::Equals,
    });

    for order in &orders {
        if paid_matcher.run(order).unwrap_or(false) {
            println!(
                "   ✓ Order #{} - {} - ${}",
                order.order_id, order.customer_name, order.total_amount
            );
        }
    }

    // Find unpaid orders with high amounts
    println!("\n6. Finding unpaid orders over $50:");
    let mut high_unpaid_matcher = Matcher::new(MatcherMode::AND);
    high_unpaid_matcher
        .add_condition(Condition {
            selector: ConditionSelector::FieldValue("is_paid", &false),
            operator: ConditionOperator::Equals,
        })
        .add_condition(Condition {
            selector: ConditionSelector::FieldValue("total_amount", &50.0f64),
            operator: ConditionOperator::GreaterThan,
        });

    for order in &orders {
        if high_unpaid_matcher.run(order).unwrap_or(false) {
            println!(
                "   ⚠ Order #{} - {} - ${} - NEEDS FOLLOW-UP",
                order.order_id, order.customer_name, order.total_amount
            );
        }
    }

    // Example 7: Using XOR mode
    println!("\n7. Demonstrating XOR mode:");
    let test_product = &products[0];

    let mut xor_matcher = Matcher::new(MatcherMode::XOR);
    xor_matcher
        .add_condition(Condition {
            selector: ConditionSelector::FieldValue("in_stock", &true),
            operator: ConditionOperator::Equals,
        })
        .add_condition(Condition {
            selector: ConditionSelector::FieldValue("id", &999i32),
            operator: ConditionOperator::Equals,
        });

    println!(
        "   Product '{}' matches XOR (exactly one condition): {}",
        test_product.name,
        xor_matcher.run(test_product).unwrap_or(false)
    );

    // Example 8: Using the field builder API
    println!("\n8. Using field builder API:");
    let condition = field::<Product>("price").gte(&100.0f64);
    let mut field_matcher = Matcher::new(MatcherMode::AND);
    field_matcher.add_condition(condition);

    for product in &products {
        if field_matcher.run(product).unwrap_or(false) {
            println!("   ✓ {} costs $100 or more", product.name);
        }
    }

    // Example 9: Getting detailed match results
    println!("\n9. Detailed match results for Laptop:");
    let laptop = &products[0];
    let detailed = expensive_matcher.run_detailed(laptop).unwrap();

    println!("   Overall match: {}", detailed.is_match());
    for (i, result) in detailed.condition_results.iter().enumerate() {
        println!(
            "   Condition {}: {} - {}",
            i + 1,
            if result.passed { "PASS" } else { "FAIL" },
            result.description
        );
        if let Some(actual) = &result.actual_value {
            println!("      Actual: {}", actual);
        }
        if let Some(expected) = &result.expected_value {
            println!("      Expected: {}", expected);
        }
    }

    println!("\n=== Summary ===");
    println!("Total products: {}", products.len());
    println!(
        "In stock: {}",
        products.iter().filter(|p| p.in_stock).count()
    );
    println!("Paid orders: {}", orders.iter().filter(|o| o.is_paid).count());
}
