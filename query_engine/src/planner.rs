use crate::ast::{self, SelectExprs, TableRef};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub enum Value {
    Boolean(bool),
    String(String),
    Int(i32),
    Float(f32),
}

impl From<&ast::Immediate> for Value {
    fn from(val: &ast::Immediate) -> Self {
        match val {
            ast::Immediate::String(s) => Value::String(s.clone()),
            ast::Immediate::Int(i) => Value::Int(*i),
            ast::Immediate::Float(f) => Value::Float(*f),
        }
    }
}

#[derive(Debug)]
pub enum Expr {
    Value(Value), // Literal Value
}

#[derive(Debug)]
pub enum ProjectionColumn {
    All,            // *
    Expr(Expr),     // Arbitrary expressions
    Column(String), // Column name
}

#[derive(Debug)]
pub enum Operation {
    Scan(ast::TableRef),            // FROM Clauses
    Filter(Expr),                   // WHERE Clauses
    Project(Vec<ProjectionColumn>), // SELECT columns
}

pub type NodeId = usize;

#[derive(Debug)]
pub struct PlanNode {
    id: NodeId,
    operator: Operation,
    dependencies: Vec<NodeId>, // Nodes this depends on (inputs)
}

#[derive(Debug)]
pub struct ExecutionPlan {
    pub nodes: HashMap<NodeId, PlanNode>,
    pub root: NodeId,
}

impl ExecutionPlan {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root: 0,
        }
    }

    pub fn add_node(&mut self, operation: Operation, dependencies: Vec<NodeId>) -> NodeId {
        let id = self.nodes.len();
        self.nodes.insert(
            id,
            PlanNode {
                id,
                operator: operation,
                dependencies,
            },
        );
        id
    }

    // Get nodes that have no pending dependencies
    pub fn get_ready_nodes(&self, completed: &HashSet<NodeId>) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter(|(id, node)| {
                !completed.contains(id) && // Node is not completed AND
                    node.dependencies.iter().all(|dep| completed.contains(dep)) // All dependencies are completed
            })
            .map(|(id, _)| *id) // Extract ID
            .collect()
    }

    pub fn get_max_node_id(&self) -> Option<&usize> {
        self.nodes.keys().max()
    }
}

impl From<ast::Query> for ExecutionPlan {
    fn from(val: ast::Query) -> Self {
        match val {
            ast::Query::Select(select_query) => select_query.into(),
        }
    }
}

impl From<ast::SelectQuery> for ExecutionPlan {
    fn from(val: ast::SelectQuery) -> Self {
        let ast::SelectQuery {
            select_exprs,
            table_ref,
        } = val;

        // scan_op is Some if the query has a FROM clause
        let scan_op = table_ref.map(Operation::Scan);
        let proj_op: Operation = select_exprs.into();

        let mut execution_plan = ExecutionPlan::new();

        // If there is a FROM clause, scan -> projection
        if let Some(scan_op) = scan_op {
            let scan_id = execution_plan.add_node(scan_op, vec![]);
            let proj_id = execution_plan.add_node(proj_op, vec![scan_id]);
            execution_plan.root = proj_id;
        // If there is no FROM clause, there is just a projection op
        } else {
            let proj_id = execution_plan.add_node(proj_op, vec![]);
            execution_plan.root = proj_id;
        };
        execution_plan
    }
}

impl From<SelectExprs> for Operation {
    fn from(val: SelectExprs) -> Self {
        let projection_columns: Vec<_> = val
            .iter()
            .map(|s| match s {
                ast::SelectExpr::All => ProjectionColumn::All,
                ast::SelectExpr::Immediate(i) => ProjectionColumn::Expr(Expr::Value(i.into())),
                ast::SelectExpr::Column(c) => ProjectionColumn::Column(c.clone()),
            })
            .collect();
        Operation::Project(projection_columns)
    }
}
