//! Test file for D201 and D202 checks on various item types.

///
/// This function has a blank line before the docstring.
pub fn function_with_leading_blank() {}

/// This function has a blank line after the docstring.
///
pub fn function_with_trailing_blank() {}

///
/// This struct has a blank line before the docstring.
pub struct StructWithLeadingBlank {
    x: i32,
}

/// This struct has a blank line after the docstring.
///
pub struct StructWithTrailingBlank {
    y: i32,
}

///
/// This enum has a blank line before the docstring.
pub enum EnumWithLeadingBlank {
    Variant1,
    Variant2,
}

/// This enum has a blank line after the docstring.
///
pub enum EnumWithTrailingBlank {
    Variant1,
    Variant2,
}

///
/// This trait has a blank line before the docstring.
pub trait TraitWithLeadingBlank {
    fn method(&self);
}

/// This trait has a blank line after the docstring.
///
pub trait TraitWithTrailingBlank {
    fn method(&self);
}

///
/// This const has a blank line before the docstring.
pub const CONST_WITH_LEADING_BLANK: i32 = 42;

/// This const has a blank line after the docstring.
///
pub const CONST_WITH_TRAILING_BLANK: i32 = 42;

/// This is properly formatted without blank lines.
pub fn properly_formatted_function() {}

/// This is properly formatted without blank lines.
pub struct ProperlyFormattedStruct {
    value: i32,
}

/// This is properly formatted without blank lines.
pub enum ProperlyFormattedEnum {
    One,
    Two,
}

/// This is properly formatted without blank lines.
pub trait ProperlyFormattedTrait {
    fn method(&self);
}

/// This is properly formatted without blank lines.
pub const PROPERLY_FORMATTED_CONST: i32 = 100;
