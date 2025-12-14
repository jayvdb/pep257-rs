#[doc = "Calculate the sum of two numbers."]
fn add_with_doc_attr(a: i32, b: i32) -> i32 {
    a + b
}

/// Standard doc comment style.
fn add_standard(a: i32, b: i32) -> i32 {
    a + b
}

#[doc = "Represents a point in space."]
struct PointWithDocAttr {
    x: f64,
    y: f64,
}
