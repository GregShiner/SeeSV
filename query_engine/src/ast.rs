#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum Query {
    Select(SelectQuery),
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct SelectQuery {
    pub select_exprs: SelectExprs,
    pub table_ref: Option<TableRef>,
}

pub type SelectExprs = Vec<SelectExpr>;

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum SelectExpr {
    All,
    Literal(Literal),
    Column(Identifier),
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum Literal {
    String(String),
    Int(i32),
    Float(f32),
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum TableRef {
    String(String),
    Identifier(Identifier),
}

pub type Identifier = String;
