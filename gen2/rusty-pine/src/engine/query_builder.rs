use crate::engine::syntax::{Position, Stage4Rep};

mod stage5;

pub fn build_query(input: Stage4Rep<'_>) -> Query {
    stage5::Stage5Builder {}.try_build(input).unwrap()
}

#[derive(Debug)]
pub struct Query {
    pub input: String,
    pub from: Sourced<Table>,
    // pub select: Vec<Select>,
}

#[derive(Debug)]
pub struct Table {
    pub name: Sourced<TableName>,
    pub db: Option<Sourced<DatabaseName>>,
}

#[derive(Debug)]
pub struct TableName(String);
#[derive(Debug)]
pub struct DatabaseName(String);

#[derive(Debug)]
pub enum Source {
    Input(Position),
}

#[derive(Debug)]
pub struct Sourced<T: Sized> {
    pub it: T,
    pub source: Source,
}

trait ToSource<D> {
    fn as_it(&self) -> D;
    fn as_source(&self) -> Source;

    fn to_sourced(self) -> Sourced<D>
    where
        Self: Sized,
    {
        let it = self.as_it();
        let source = self.as_source();

        Sourced { it, source }
    }
}

impl<T, D> ToSource<D> for T
where
    for<'a> &'a T: Into<D>,
    for<'a> &'a T: Into<Position>,
{
    fn as_it(&self) -> D {
        self.into()
    }

    fn as_source(&self) -> Source {
        Source::Input(self.into())
    }
}

impl From<Position> for Source {
    fn from(value: Position) -> Self {
        Source::Input(value)
    }
}

impl<T> From<&T> for TableName
where
    for<'a> &'a T: Into<String>,
{
    fn from(value: &T) -> Self {
        TableName(value.into())
    }
}

impl<T> From<&T> for DatabaseName
where
    for<'a> &'a T: Into<String>,
{
    fn from(value: &T) -> Self {
        DatabaseName(value.into())
    }
}
