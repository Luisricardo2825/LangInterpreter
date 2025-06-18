use std::{collections::HashMap, rc::Rc};

use crate::environment::values::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    ImportNamed {
        items: Vec<(String, String)>, // (exported_name, local_name)
        from: String,
    },
    ImportDefault {
        local_name: String,
        from: String,
    },
    ImportAll {
        local_name: String,
        from: String,
    },
    ImportMixed {
        default: String,
        items: Vec<(String, String)>,
        from: String,
    },
    Export(Rc<Stmt>),        // Marca uma declaração como exportável
    ExportDefault(Rc<Stmt>), // novo!
    Let {
        name: String,
        value: Option<Expr>,
    },
    FuncDecl(FunctionStmt),
    ClassDecl {
        name: String,
        superclass: Option<Expr>, // para herança, se suportar
        methods: Vec<MethodDecl>, // (Nome, estatico)
        static_fields: HashMap<String, Expr>,
        instance_fields: HashMap<String, Expr>, // FuncDecl ou algo similar
    },
    Method(MethodDecl),
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_ifs: Vec<(Expr, Option<Vec<Stmt>>)>,
        else_branch: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    For {
        init: Box<Stmt>,
        condition: Option<Expr>,
        update: Option<Expr>,
        body: Vec<Stmt>,
    },
    ForIn {
        target: Expr,
        object: Expr,
        body: Vec<Stmt>,
    },
    ForOf {
        target: Expr,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    TryCatchFinally {
        try_block: Vec<Stmt>,
        catch_block: Option<(String, Vec<Stmt>)>,
        finally_block: Option<Vec<Stmt>>,
    },
    Throw(Expr),
    ExprStmt(Expr),
    Return(Option<Expr>),
    Break,
    Continue,
}
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionStmt {
    pub name: String,
    pub params: Vec<String>,
    pub vararg: Option<String>,
    pub body: Vec<Stmt>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    Assign {
        target: Box<Expr>,
        op: AssignOperator,
        value: Box<Expr>,
    },
    BinaryOp {
        op: Operator,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    GetProperty {
        object: Box<Expr>,
        property: Box<Expr>,
    },
    SetProperty {
        object: Box<Expr>,
        property: Box<Expr>,
        value: Box<Expr>,
    },
    BracketAccess {
        object: Box<Expr>,
        property: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOperator,
        expr: Box<Expr>,
        postfix: bool,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    New {
        class_expr: Box<Expr>,
        // args: Vec<Expr>,
    },

    This,
    Block(Vec<Stmt>),
    Spread(Box<Expr>),
}

#[derive(Debug)]
pub enum ControlFlow<T: std::fmt::Debug> {
    Return(T),
    Break,
    Continue,
    None,
    Error(Value),
}

pub fn debug_stmts(stmts: &[Stmt], indent: usize) {
    for stmt in stmts {
        debug_stmt(stmt, indent);
    }
}

pub fn debug_stmt(stmt: &Stmt, indent: usize) {
    let pad = " ".repeat(indent);
    match stmt {
        Stmt::ImportNamed { .. } => println!("{pad}Stmt::ImportNamed"),
        Stmt::ImportDefault { .. } => println!("{pad}Stmt::ImportDefault"),
        Stmt::ImportAll { .. } => println!("{pad}Stmt::ImportAll"),
        Stmt::ImportMixed { .. } => println!("{pad}Stmt::ImportMixed"),
        Stmt::Export(inner) => {
            println!("{pad}Stmt::Export");
            debug_stmt(inner, indent + 2);
        }
        Stmt::ExportDefault(inner) => {
            println!("{pad}Stmt::ExportDefault");
            debug_stmt(inner, indent + 2);
        }
        Stmt::Let { .. } => println!("{pad}Stmt::Let"),
        Stmt::FuncDecl(_) => println!("{pad}Stmt::FuncDecl"),
        Stmt::ClassDecl { name, methods, .. } => {
            println!("{pad}Stmt::ClassDecl ({name})");
            for m in methods {
                println!("{pad}  └── Stmt::ClassDecl::Method ({})", m.name);
            }
        }
        Stmt::Method(m) => println!("{pad}Stmt::Method ({})", m.name),
        Stmt::If {
            then_branch,
            else_ifs,
            else_branch,
            ..
        } => {
            println!("{pad}Stmt::If");
            for s in then_branch {
                debug_stmt(s, indent + 2);
            }
            for (_cond, block) in else_ifs {
                println!("{pad}  ├── Stmt::ElseIf");
                if let Some(stmts) = block {
                    for s in stmts {
                        debug_stmt(s, indent + 4);
                    }
                }
            }
            if let Some(else_stmts) = else_branch {
                println!("{pad}  └── Stmt::Else");
                for s in else_stmts {
                    debug_stmt(s, indent + 4);
                }
            }
        }
        Stmt::While { body, .. } => {
            println!("{pad}Stmt::While");
            for s in body {
                debug_stmt(s, indent + 2);
            }
        }
        Stmt::For { body, .. } => {
            println!("{pad}Stmt::For");
            for s in body {
                debug_stmt(s, indent + 2);
            }
        }
        Stmt::ForIn { body, .. } => {
            println!("{pad}Stmt::ForIn");
            for s in body {
                debug_stmt(s, indent + 2);
            }
        }
        Stmt::ForOf { body, .. } => {
            println!("{pad}Stmt::ForOf");
            for s in body {
                debug_stmt(s, indent + 2);
            }
        }
        Stmt::TryCatchFinally {
            try_block,
            catch_block,
            finally_block,
        } => {
            println!("{pad}Stmt::Try");
            for s in try_block {
                debug_stmt(s, indent + 2);
            }
            if let Some((_name, catch_block)) = catch_block {
                println!("{pad}  ├── Stmt::Catch");
                for s in catch_block {
                    debug_stmt(s, indent + 4);
                }
            }
            if let Some(finally_block) = finally_block {
                println!("{pad}  └── Stmt::Finally");
                for s in finally_block {
                    debug_stmt(s, indent + 4);
                }
            }
        }
        Stmt::Throw(_) => println!("{pad}Stmt::Throw"),
        Stmt::ExprStmt(_) => println!("{pad}Stmt::ExprStmt"),
        Stmt::Return(_) => println!("{pad}Stmt::Return"),
        Stmt::Break => println!("{pad}Stmt::Break"),
        Stmt::Continue => println!("{pad}Stmt::Continue"),
    }
}

impl<T: std::fmt::Debug> ControlFlow<T> {
    pub fn is_none(&self) -> bool {
        match self {
            ControlFlow::None => true,
            _ => false,
        }
    }
    pub fn is_some(&self) -> bool {
        match self {
            ControlFlow::None => false,
            _ => true,
        }
    }
    pub fn unwrap(self) -> T {
        match self {
            ControlFlow::Return(value) => value,
            other => panic!("Cannot unwrap {other:?}"),
        }
    }
}

impl Expr {
    pub fn is_literal(&self) -> bool {
        matches!(self, Expr::Literal(_))
    }
    pub fn to_number(&self) -> Option<f64> {
        match self {
            Expr::Literal(Literal::Number(n)) => Some(*n),
            _ => None,
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            Expr::Literal(lit) => lit.to_string(),
            Expr::Identifier(name) => name.clone(),
            Expr::Assign { target, op, value } => {
                let op_str = match op {
                    AssignOperator::Assign => "=",
                    AssignOperator::AddAssign => "+=",
                    AssignOperator::SubAssign => "-=",
                    AssignOperator::MulAssign => "*=",
                    AssignOperator::DivAssign => "/=",
                    AssignOperator::ModAssign => "%=",
                    AssignOperator::PowAssign => "**=",
                };
                format!("{} {} {}", target.to_string(), op_str, value.to_string())
            }
            Expr::BinaryOp { op, left, right } => {
                let op_str = match op {
                    Operator::Binary(b) => match b {
                        BinaryOperator::Add => "+",
                        BinaryOperator::Subtract => "-",
                        BinaryOperator::Multiply => "*",
                        BinaryOperator::Divide => "/",
                        BinaryOperator::Modulo => "%",
                        BinaryOperator::Exponentiate => "**",
                    },
                    Operator::Compare(c) => match c {
                        CompareOperator::Eq => "==",
                        CompareOperator::Ne => "!=",
                        CompareOperator::Gt => ">",
                        CompareOperator::Ge => ">=",
                        CompareOperator::Lt => "<",
                        CompareOperator::Le => "<=",
                        CompareOperator::In => "in",
                        CompareOperator::InstanceOf => "instanceof",
                    },
                    Operator::Logical(l) => match l {
                        LogicalOperator::And => "&&",
                        LogicalOperator::Or => "||",
                    },
                    Operator::Unary(_) => unreachable!("UnaryOp should not appear here"),
                };
                format!("({} {} {})", left.to_string(), op_str, right.to_string())
            }
            Expr::UnaryOp { op, expr, postfix } => {
                let op_str = match op {
                    UnaryOperator::Negative => "-",
                    UnaryOperator::Not => "!",
                    UnaryOperator::Typeof => "typeof ",
                    UnaryOperator::Increment => "++",
                    UnaryOperator::Decrement => "--",
                    UnaryOperator::Positive => "+",
                };
                if *postfix {
                    format!("{}{}", expr.to_string(), op_str)
                } else {
                    format!("{}{}", op_str, expr.to_string())
                }
            }
            Expr::GetProperty { object, property } => {
                format!("{}.{}", object.to_string(), property.to_string())
            }
            Expr::SetProperty {
                object,
                property,
                value,
            } => {
                format!(
                    "{}.{} = {}",
                    object.to_string(),
                    property.to_string(),
                    value.to_string()
                )
            }
            Expr::BracketAccess { object, property } => {
                format!("{}[{}]", object.to_string(), property.to_string())
            }
            Expr::Call { callee, args } => {
                let args_str = args
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", callee.to_string(), args_str)
            }
            Expr::New { class_expr } => {
                let class_name = class_expr.to_string();
                // let args_str = args
                //     .iter()
                //     .map(|a| a.to_string())
                //     .collect::<Vec<_>>()
                //     .join(", ");
                format!("new {}", class_name)
            }
            Expr::This => "this".to_string(),
            Expr::Block(stmts) => {
                let body = stmts
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{{\n{}\n}}", body)
            }
            Expr::Spread(expr) => {
                format!("...{}", expr.to_string())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignOperator {
    Assign,    // =
    AddAssign, // +=
    SubAssign, // -=
    MulAssign, // *=
    DivAssign, // /=
    ModAssign, // %=
    PowAssign, // **=
}
impl AssignOperator {
    pub fn from_op(op: &str) -> Option<Self> {
        match op {
            "=" => Some(AssignOperator::Assign),
            "+=" => Some(AssignOperator::AddAssign),
            "-=" => Some(AssignOperator::SubAssign),
            "*=" => Some(AssignOperator::MulAssign),
            "/=" => Some(AssignOperator::DivAssign),
            "%=" => Some(AssignOperator::ModAssign),
            "**=" => Some(AssignOperator::PowAssign),
            _ => None,
        }
    }
}

impl std::fmt::Display for AssignOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssignOperator::Assign => write!(f, "="),
            AssignOperator::AddAssign => write!(f, "+="),
            AssignOperator::SubAssign => write!(f, "-="),
            AssignOperator::MulAssign => write!(f, "*="),
            AssignOperator::DivAssign => write!(f, "/="),
            AssignOperator::ModAssign => write!(f, "%="),
            AssignOperator::PowAssign => write!(f, "**="),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operator {
    Binary(BinaryOperator),
    Compare(CompareOperator),
    Logical(LogicalOperator),
    Unary(UnaryOperator),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Exponentiate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompareOperator {
    Eq,         // ==
    Ne,         // !=
    Gt,         // >
    Ge,         // >=
    Lt,         // <
    Le,         // <=
    InstanceOf, // instanceof
    In,         // in
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogicalOperator {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOperator {
    Negative,
    Not,
    Typeof,
    Increment,
    Decrement,
    Positive,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Void,
    /// null.
    Null,
    /// true or false.
    Bool(bool),
    /// Any floating point number.
    Number(f64),
    /// Any quoted string.
    String(String),
    /// An array of values
    Array(Vec<Expr>),
    /// An dictionary mapping keys and values.
    Object(Vec<ObjectEntry>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectEntry {
    Property { key: String, value: Expr },
    Shorthand(String),
    Spread(Expr), // `...obj`
}
impl Literal {
    pub fn to_string(&self) -> String {
        match self {
            Literal::Void => "void".to_string(),
            Literal::Null => "null".to_string(),
            Literal::Bool(b) => b.to_string(),
            Literal::Number(n) => n.to_string(),
            Literal::String(s) => format!("\"{}\"", s),
            Literal::Array(a) => {
                let mut s = "[".to_string();
                for (i, e) in a.iter().enumerate() {
                    s += &e.to_string();
                    if i < a.len() - 1 {
                        s += ", ";
                    }
                }
                s += "]";
                s
            }
            Literal::Object(o) => {
                let mut s = "{".to_string();
                for (i, e) in o.iter().enumerate() {
                    match e {
                        ObjectEntry::Property { key, value } => {
                            s += &format!("{}: {}", key, value.to_string());
                        }
                        ObjectEntry::Shorthand(key) => {
                            s += &key.to_string();
                        }
                        ObjectEntry::Spread(expr) => {
                            s += &format!("...{}", expr.to_string());
                        }
                    }
                    if i < o.len() - 1 {
                        s += ", ";
                    }
                }
                s += "}";
                s
            }
        }
    }
    pub fn from_value(value: &Value) -> Literal {
        match value {
            Value::Void => Literal::Void,
            Value::Null => Literal::Null,
            Value::Bool(b) => Literal::Bool(*b),
            Value::Number(n) => Literal::Number(n.get_value()),
            Value::String(s) => Literal::String(s.clone().to_string()),
            Value::Array(a) => {
                let mut arr = Vec::new();
                for v in a.get_value().borrow().clone() {
                    arr.push(Expr::Literal(Literal::from_value(&v)));
                }
                Literal::Array(arr)
            }
            Value::Object(o) => {
                let mut obj = Vec::new();
                for (k, v) in o.borrow().clone() {
                    obj.push(ObjectEntry::Property {
                        key: k.clone(),
                        value: Expr::Literal(Literal::from_value(&v)),
                    });
                }
                Literal::Object(obj)
            }
            _ => unreachable!("Cannot convert {:?} to Literal", value),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodDecl {
    pub name: String,
    pub params: Vec<String>,
    pub vararg: Option<String>,
    pub body: Vec<Stmt>,
    pub modifiers: Vec<MethodModifiers>,
}

pub trait MethodModifiersOperations {
    fn contains(&self, modifier: MethodModifiers) -> bool;

    fn contains_all(&self, modifiers: Vec<MethodModifiers>) -> bool {
        modifiers.iter().all(|m| self.contains(m.clone()))
    }
    fn contains_any(&self, modifiers: Vec<MethodModifiers>) -> bool {
        modifiers.iter().any(|m| self.contains(m.clone()))
    }
    fn contains_str(&self, modifier: &str) -> bool {
        match modifier.to_lowercase().as_str() {
            "static" => self.contains(MethodModifiers::Static),
            "operator" => self.contains(MethodModifiers::Operator),
            "private" => self.contains(MethodModifiers::Private),
            _ => false,
        }
    }
}
impl MethodModifiersOperations for [MethodModifiers] {
    fn contains(&self, modifier: MethodModifiers) -> bool {
        self.contains(&modifier)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodModifiers {
    Static,
    Operator,
    Private,
}

impl std::fmt::Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Stmt {
    pub fn to_string(&self) -> String {
        match self {
            Stmt::Let { name, value } => match value {
                Some(expr) => format!("let {} = {};", name, expr.to_string()),
                None => format!("let {};", name),
            },
            Stmt::Return(Some(expr)) => format!("return {};", expr.to_string()),
            Stmt::Return(None) => "return;".to_string(),
            Stmt::ExprStmt(expr) => format!("{};", expr.to_string()),
            Stmt::Break => "break;".to_string(),
            Stmt::Continue => "continue;".to_string(),
            Stmt::If {
                condition,
                then_branch,
                else_ifs,
                else_branch,
            } => {
                let mut s = format!("if ({}) {{\n", condition.to_string());
                for stmt in then_branch {
                    s += &format!("  {}\n", stmt.to_string());
                }
                s += "}";
                for (cond, block) in else_ifs {
                    s += &format!(" else if ({}) {{\n", cond.to_string());
                    if let Some(stmts) = block {
                        for stmt in stmts {
                            s += &format!("  {}\n", stmt.to_string());
                        }
                    }
                    s += "}";
                }
                if let Some(else_block) = else_branch {
                    s += " else {\n";
                    for stmt in else_block {
                        s += &format!("  {}\n", stmt.to_string());
                    }
                    s += "}";
                }
                s
            }
            Stmt::While { condition, body } => {
                let mut s = format!("while ({}) {{\n", condition.to_string());
                for stmt in body {
                    s += &format!("  {}\n", stmt.to_string());
                }
                s += "}";
                s
            }
            Stmt::FuncDecl(func) => {
                let params = func.params.join(", ");
                let mut s = format!("function {}({}) {{\n", func.name, params);
                for stmt in &func.body {
                    s += &format!("  {}\n", stmt.to_string());
                }
                s += "}";
                s
            }
            Stmt::ImportNamed { items, from } => {
                let imports = items
                    .iter()
                    .map(|(exported, local)| {
                        if exported == local {
                            exported.clone()
                        } else {
                            format!("{} as {}", exported, local)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("import {{ {} }} from '{}';", imports, from)
            }
            Stmt::ImportDefault { local_name, from } => {
                format!("import {} from '{}';", local_name, from)
            }
            Stmt::ImportAll { local_name, from } => {
                format!("import * as {} from '{}';", local_name, from)
            }
            Stmt::ImportMixed {
                default,
                items,
                from,
            } => {
                let named = items
                    .iter()
                    .map(|(exported, local)| {
                        if exported == local {
                            exported.clone()
                        } else {
                            format!("{} as {}", exported, local)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("import {}, {{ {} }} from '{}';", default, named, from)
            }
            Stmt::Export(stmt) => format!("export {};", stmt.to_string()),
            Stmt::ExportDefault(stmt) => format!("export default {};", stmt.to_string()),
            other => format!("{:?}", other), // fallback para casos não tratados
        }
    }
}
