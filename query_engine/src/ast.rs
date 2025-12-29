#[derive(Debug)]
pub enum Query {
    Select(SelectQuery),
}

#[derive(Debug)]
pub struct SelectQuery {
    pub select_exprs: SelectExprs,
    pub table_ref: TableRef,
}

pub type SelectExprs = Vec<SelectExpr>;

#[derive(Debug)]
pub enum SelectExpr {
    All,
    Immediate(Immediate),
}

#[derive(Debug)]
pub enum Immediate {
    String(String),
    Int(i32),
    Float(f32),
}

#[derive(Debug)]
pub enum TableRef {
    String(String),
    Identifier(String),
}
