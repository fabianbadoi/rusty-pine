//! Stage 2 representation has a list of one node per "Pine"
//! For example "users 1 | s: id" would be represented by:
//!
//! # Examples
//! ```ignore
//! # let (pine1, pine2) = (0,0); // ignore this
//! vec![
//!     pine1,
//!     pine2,
//! ]
//! ```
//!
//! The data is not yet hierarchical.
//!
//! Since this is just a more convenient way of representing the source Pest info, it's not possible
//! to fail to parse.
use crate::engine::syntax::stage1::{Rule, Stage1Rep};
use crate::engine::syntax::stage2::fn_calls::translate_fn_call;
use crate::engine::syntax::stage2::identifiers::translate_column;
use crate::engine::syntax::{Computation, Stage2LiteralValue, TableInput};
use crate::engine::{
    Comparison, ConditionHolder, JoinConditions, JoinHolder, JoinType, Position, SelectableHolder,
    Sourced,
};
use pest::iterators::{Pair, Pairs};
use pest::Span;

mod fn_calls;
/// We split up identifier (databases, tables, and columns) into its own module to keep things clean.
mod identifiers;

/// It's a pattern we have that every stage keeps a ref to the input string + whatever data we
/// processed.
///
/// ```ignore
/// # use crate::syntax::stage1::parse_stage1;
/// # let stage1_rep = parse_stage1("name").unwrap();
/// let stage2_rep = stage2_rep.into();
/// ```
///
pub struct Stage2Rep<'a> {
    pub input: &'a str,
    /// Our syntax is formed by a chain of "pines".
    ///
    /// We use an iterator here instead of a vector to avoid passing over all the pines in each stage.
    /// This is not really a problem given how few stages we have, and how few pines input will
    /// actually have. I just thought it would be interesting, and I had never done it.
    /// Looking back, generators would have made things easier, but they were experimental.
    pub pines: PestIterator<'a>,
}

/// Our inputs are made up of a chain of "pines".
///
/// The stage1 rep just uses the Pest out as a representation. That's ok, but it does not map to
/// something we can directly work with.
///
/// Since each pine can be of a certain limited number of types, and each holds specific data and
/// references, enums are a very good choice to represent them.
///
/// Each pine will be one of these variants, and hold its own data that can be of use different
/// types.
#[derive(Debug, Clone)]
pub enum Stage2Pine<'a> {
    /// All pines start with a base pine that can never repeat. This specified the original
    /// table we'll be working with.
    Base { table: Sourced<TableInput<'a>> },
    /// Selects one or more computations from the previous table.
    Select(Vec<Sourced<Stage2Selectable<'a>>>),
    /// Specify exactly how to join another table.
    ExplicitJoin(Sourced<Stage2Join<'a>>),
    /// Join a table, we'll figure out how for you.
    ExplicitAutoJoin(Sourced<Stage2ExplicitAutoJoin<'a>>),
}

pub type Stage2Selectable<'a> = SelectableHolder<Stage2Condition<'a>, Computation<'a>>;
pub type Stage2Condition<'a> = ConditionHolder<Computation<'a>>;

pub type Stage2Join<'a> = JoinHolder<TableInput<'a>, Stage2Condition<'a>>;

#[derive(Debug, Clone)]
pub struct Stage2ExplicitAutoJoin<'a> {
    pub join_type: Sourced<JoinType>,
    pub target_table: Sourced<TableInput<'a>>,
}

/// The From implementation allows us to write stage1_rep.into() to get a stage2 rep.
///
/// Since Pest will guarantee that our input is valid, this process cannot fail. If we need to have
/// a process that could fail, we could have used TryFrom instead of From, which returns a
/// Result<OK, Error>.
impl<'a> From<Stage1Rep<'a>> for Stage2Rep<'a> {
    fn from(stage1: Stage1Rep<'a>) -> Self {
        let pines = translate_root(stage1.pest);

        Stage2Rep {
            input: stage1.input,
            pines,
        }
    }
}

/// This struct will go through all of our syntax tree and "emit" a pine whenever it can. Code using
/// the iterator will be optimized on compiling for the "release" target.
///
/// Theoretically, this means our code that goes through multiple stages will compile to something
/// like a large for loop that does all the operations one by one.
/// I have not bothered to double check this.
pub struct PestIterator<'a> {
    inners: Pairs<'a, Rule>,
    /// We made our syntax have a special meaning for the base pair. Keeping track that we've
    /// already processed it is helpful.
    base_done: bool,
}

impl<'a> PestIterator<'a> {
    fn new(base: Pairs<'a, Rule>) -> PestIterator {
        Self {
            base_done: false,
            inners: base,
        }
    }
}

/// Iterator is perhaps the biggest trait in rust, in terms of how many methods you get for just
/// implementing next().
/// It allows code to use PestIterator just like a loop, but also gives it about a gorillion other
/// methods.
impl<'a> Iterator for PestIterator<'a> {
    // The Item type is the type of item you iterate over.
    // In our case, we iterate over Positioned elements, if we ever need to explain why something
    // happened, we highlight the relevant part of the input.
    type Item = Sourced<Stage2Pine<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inners.next();

        // By having this if here, we simplify our translate_pine function
        if !self.base_done {
            self.base_done = true;

            return Some(translate_base(next.expect("Guaranteed by syntax")));
        }

        match next {
            None => None, // When we return none, it means the iterator is now done
            Some(pair) => translate_pine(pair),
        }
    }
}

// a list of functions will follow that will just parse every type of Pine we have.

fn translate_root(mut pairs: Pairs<Rule>) -> PestIterator {
    let root_pair = pairs.next().expect("Impossible due to pest parsing");

    // Asserts like this are optimized out of release builds, but are really helpful for debugging
    // and when running tests.
    // If the assert condition is not met, the thread panics with a helpful message. You can see
    // these messages in the test output and figure out what went wrong.
    assert_eq!(Rule::root, root_pair.as_rule());
    // pairs.next().is_none() means there are no more language tokens *after* this one.
    // Of course, this token has tokens underneath it. It is a tree.
    assert!(pairs.next().is_none());

    PestIterator::new(root_pair.into_inner())
}

fn translate_base(base_pair: Pair<Rule>) -> Sourced<Stage2Pine> {
    assert_eq!(Rule::base, base_pair.as_rule());

    let span = base_pair.as_span();
    let table_name = identifiers::translate_table(base_pair.into_inner().next().unwrap());

    Sourced::from_input(span, Stage2Pine::Base { table: table_name })
}

fn translate_pine(pair: Pair<Rule>) -> Option<Sourced<Stage2Pine>> {
    // Normally, you would use an exhaustive list here. Meaning you would put in all the possible
    // types of rules. Then, if a new rule were to be added, the compiler will let you know there's
    // a case you missed.
    // Pest decided to disallow that, so we have to have that catch-all case at the bottom.
    let span = pair.as_span();
    let pine = match pair.as_rule() {
        Rule::select_pine => translate_select(pair),
        Rule::explicit_join_pine => translate_explicit_join(pair),
        Rule::explicit_auto_join_pine => translate_explicit_auto_join(pair),
        // Rule::join_pine => Some(todo!()),
        Rule::EOI => return None, // EOI is End Of Input
        _ => panic!("Unknown pine {:#?}", pair),
    };

    Some(Sourced::from_input(span, pine))
}

fn translate_select(select: Pair<Rule>) -> Stage2Pine {
    assert_eq!(Rule::select_pine, select.as_rule());

    let mut columns = Vec::new();

    for column_pair in select.into_inner() {
        let column = translate_selectable(column_pair);
        columns.push(column);
    }

    Stage2Pine::Select(columns)
}

fn translate_explicit_join(join: Pair<Rule>) -> Stage2Pine {
    assert_eq!(Rule::explicit_join_pine, join.as_rule());

    let span = join.as_span();
    let mut inners = join.into_inner();

    let target_table = identifiers::translate_table(
        inners
            .next()
            .expect("explicit join target table should be present because of pest syntax"),
    );

    let conditions: Vec<_> = inners.map(translate_condition).collect();

    assert!(
        !conditions.is_empty(),
        "Pest grammar prevents explicit joins without conditions"
    );

    let conditions = JoinConditions::Explicit(conditions);

    Stage2Pine::ExplicitJoin(Sourced::from_input(
        span,
        Stage2Join {
            join_type: Sourced::implicit(JoinType::Left),
            target_table,
            conditions,
        },
    ))
}

fn translate_explicit_auto_join(join: Pair<Rule>) -> Stage2Pine {
    assert_eq!(Rule::explicit_auto_join_pine, join.as_rule());

    let span = join.as_span();
    let mut inners = join.into_inner();

    let target_table = identifiers::translate_table(
        inners
            .next()
            .expect("explicit join target table should be present because of pest syntax"),
    );

    Stage2Pine::ExplicitAutoJoin(Sourced::from_input(
        span,
        Stage2ExplicitAutoJoin {
            join_type: Sourced::implicit(JoinType::Left),
            target_table,
        },
    ))
}

fn translate_selectable(selectable: Pair<Rule>) -> Sourced<Stage2Selectable> {
    assert_eq!(Rule::selectable, selectable.as_rule());

    let span = selectable.as_span();

    let mut inners = selectable.into_inner();
    let inner = inners.next().expect("Has to be valid syntax");
    assert!(inners.next().is_none());

    use SelectableHolder::{Computation, Condition};
    Sourced::from_input(
        span,
        match inner.as_rule() {
            Rule::computation => Computation(translate_computation(inner)),
            Rule::condition => Condition(translate_condition(inner)),
            unexpected_rule => panic!(
                "Unexpected rule while processing selectable: Rule::{:?}",
                unexpected_rule
            ),
        },
    )
}

fn translate_computation(computation: Pair<Rule>) -> Sourced<Computation> {
    assert_eq!(Rule::computation, computation.as_rule());

    let mut inners = computation.into_inner();
    let inner = inners.next().expect("Has to be valid syntax");
    assert!(inners.next().is_none());

    Sourced::from_input(
        inner.as_span(),
        match inner.as_rule() {
            Rule::column => Computation::Column(translate_column(inner)),
            Rule::function_call => Computation::FunctionCall(translate_fn_call(inner)),
            Rule::literal_value => Computation::Value(translate_value(inner)),
            unsupported_rule => panic!("Unexpected rule: Rule::{:?}", unsupported_rule),
        },
    )
}

fn translate_condition(condition: Pair<Rule>) -> Sourced<Stage2Condition> {
    assert_eq!(Rule::condition, condition.as_rule());

    let span = condition.as_span();

    let mut inners = condition.into_inner();
    let left = inners
        .next()
        .expect("Valid condition must have left operand");
    let left = translate_computation(left);

    let comparison = inners
        .next()
        .expect("Valid condition must have comparison operator");
    let comparison = translate_comparison(comparison);

    let right = inners
        .next()
        .expect("Valid condition must have right operand");
    let right = translate_computation(right);

    assert!(inners.next().is_none());

    Sourced::from_input(
        span,
        Stage2Condition {
            left,
            comparison,
            right,
        },
    )
}

impl From<Span<'_>> for Position {
    fn from(span: Span) -> Self {
        Position {
            start: span.start(),
            end: span.end(),
        }
    }
}

fn translate_value(pair: Pair<Rule>) -> Sourced<Stage2LiteralValue> {
    assert_eq!(Rule::literal_value, pair.as_rule());

    let span = pair.as_span();

    let inner = pair
        .into_inner()
        .next()
        .expect("Rule::value has inner number or string");

    let value = match inner.as_rule() {
        Rule::numeric_value => Stage2LiteralValue::Number(inner.as_str().trim()),
        Rule::string_value => Stage2LiteralValue::String(inner.as_str()),
        unexpected_rule => panic!("Unexpected rule for value: Rule::{:?}", unexpected_rule),
    };

    Sourced::from_input(span, value)
}

fn translate_comparison(pair: Pair<Rule>) -> Sourced<Comparison> {
    assert_eq!(Rule::comparison_symbol, pair.as_rule());

    Sourced::from_input(
        pair.as_span(),
        match pair.as_str() {
            "=" => Comparison::Equals,
            "!=" => Comparison::NotEquals,
            ">" => Comparison::GreaterThan,
            ">=" => Comparison::GreaterOrEqual,
            "<" => Comparison::LesserThan,
            "<=" => Comparison::LesserOrEqual,
            other_comparison_symbol => {
                panic!("Unknown comparison symbol '{other_comparison_symbol}")
            }
        },
    )
}

#[cfg(test)]
mod test {
    use crate::engine::syntax::stage1::parse_stage1;
    use crate::engine::syntax::stage2::{Stage2Pine, Stage2Rep};
    use crate::engine::syntax::{OptionalInput, SqlIdentifierInput, TableInput};
    use crate::engine::{Position, Source, Sourced};

    // You might be asking why I write so few tests. It's because writing out the structs for these
    // stages is a PITA. In this case, I'll just write some integration tests at the end and compare
    // Pine to SQL query.

    #[test]
    fn test_simple_parse() {
        let stage1 = parse_stage1("name").unwrap();
        let mut stage2: Stage2Rep = stage1.into();

        assert_eq!("name", stage2.input);

        let base = &stage2.pines.next().unwrap();
        assert!(matches!(
            base.it,
            // This is what us professionals like to call "FUGLY"
            Stage2Pine::Base {
                table: Sourced {
                    it: TableInput {
                        database: OptionalInput::Implicit,
                        table: Sourced {
                            it: SqlIdentifierInput { name: "name" },
                            source: Source::Input(Position { start: 0, end: 4 })
                        },
                    },
                    source: Source::Input(Position { start: 0, end: 4 })
                }
            }
        ));

        // 0..4 represents a Range. It holds start and end values, and implements the Iterator
        // trait. This means you could use it a for loop.
        // How can we compare to our Position struct? Normally you can't compare different types,
        // even if in cases like this you have types that hold the same data.
        // Look for where the Position struct is defined, you will see it also implements the
        // PartialEq trait for ranges.
        // I did that just to save some key strokes.
        assert_eq!(0..4, base.source);
    }
}
