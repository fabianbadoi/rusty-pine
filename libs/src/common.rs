#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum UnaryFilterType {
    IsNull,
    IsNotNull,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum BinaryFilterType {
    LesserThan,
    LesserThanOrEquals,
    Equals,
    NotEquals,
    GreaterThan,
    GreaterThanOrEquals,
}
