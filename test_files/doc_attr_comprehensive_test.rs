#[doc = "Calculate sum correctly."]
fn good_doc_attr(a: i32, b: i32) -> i32 {
    a + b
}

#[doc = "calculate without period"]
fn bad_doc_attr_no_period(a: i32, b: i32) -> i32 {
    a + b
}

#[doc = "this is not capitalized."]
fn bad_doc_attr_not_capitalized(a: i32, b: i32) -> i32 {
    a + b
}

#[doc = "gets the value from storage"]
fn bad_doc_attr_non_imperative(a: i32) -> i32 {
    a
}

/// This is a standard comment for comparison.
fn standard_comment_good(a: i32) -> i32 {
    a
}

#[doc = "Calculate the sum."]
#[doc = ""]
#[doc = "This is a multi-line description."]
fn multiline_doc_attrs(a: i32, b: i32) -> i32 {
    a + b
}

#[doc = "Represents a point."]
struct GoodDocAttrStruct {
    x: f64,
    y: f64,
}

#[doc = "missing period"]
enum BadDocAttrEnum {
    Variant1,
    Variant2,
}