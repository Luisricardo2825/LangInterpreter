use std::cell::RefCell;
use std::{collections::HashMap, rc::Rc};

use serde::{Deserialize, Serialize};
use yansi::Color;
use yansi::Paint;

use crate::environment::values::Value;
use crate::environment::Environment;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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
    Export(Box<Stmt>),        // Marca uma declaração como exportável
    ExportDefault(Box<Stmt>), // novo!
    Let {
        name: String,
        value: Expr,
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
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct FunctionStmt {
    pub name: String,
    pub params: Vec<String>,
    pub vararg: Option<String>,
    pub body: Vec<Stmt>,
}
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_string().fmt(f)
    }
}
#[derive(Debug, Clone)]
pub enum ControlFlow<T: std::fmt::Debug> {
    Return(T),
    Break,
    Continue,
    None,
    Error(T),
}

pub fn debug_stmts(stmts: &[Stmt], indent: usize) {
    for stmt in stmts {
        debug_stmt(stmt, indent);
    }
}

fn format_expr(expr: &Expr) -> String {
    format!("{}", expr) // ou implemente um printer real
}

pub fn debug_stmt(stmt: &Stmt, indent: usize) {
    let pad = " ".repeat(indent);

    // Cores distintas por tipo
    let color_import = Color::Blue.bold();
    let color_export = Color::Cyan.bold();
    let color_let = Color::Green.bold();
    let color_func = Color::Fixed(135).bold();
    let color_class = Color::Red.bold();
    let color_method = Color::Fixed(208).bold(); // Laranja
    let color_control = Color::Fixed(214).bold(); // Amarelo queimado
    let color_try = Color::Magenta.bold();
    let color_throw = Color::Fixed(160).bold(); // Vermelho escuro
    let color_expr = Color::White.bold();
    let color_other = Color::Fixed(246); // Cinza claro
    let color_symbol = Color::Fixed(240); // Cinza escuro
    let color_name = Color::Fixed(151); // Verde-limão
    let color_value = Color::Fixed(189);

    match stmt {
        Stmt::ImportNamed { .. } => println!("{pad}{}", "Stmt::ImportNamed".paint(color_import)),
        Stmt::ImportDefault { .. } => {
            println!("{pad}{}", "Stmt::ImportDefault".paint(color_import))
        }
        Stmt::ImportAll { .. } => println!("{pad}{}", "Stmt::ImportAll".paint(color_import)),
        Stmt::ImportMixed { .. } => println!("{pad}{}", "Stmt::ImportMixed".paint(color_import)),

        Stmt::Export(inner) => {
            println!("{pad}{}", "Stmt::Export".paint(color_export));
            debug_stmt(inner, indent + 2);
        }
        Stmt::ExportDefault(inner) => {
            println!("{pad}{}", "Stmt::ExportDefault".paint(color_export));
            debug_stmt(inner, indent + 2);
        }

        Stmt::Let { name, value } => {
            println!(
                "{pad}{} {} = {}",
                "Stmt::Let".paint(color_let),
                name.paint(color_name),
                format_expr(value).paint(color_value)
            );
        }

        Stmt::Return(expr_opt) => {
            if let Some(expr) = expr_opt {
                println!(
                    "{pad}{} {}",
                    "Stmt::Return".paint(color_expr),
                    format_expr(expr).paint(color_value)
                );
            } else {
                println!("{pad}{}", "Stmt::Return".paint(color_expr));
            }
        }

        Stmt::ExprStmt(expr) => {
            println!(
                "{pad}{} {}",
                "Stmt::ExprStmt".paint(color_expr),
                format_expr(expr).paint(color_value)
            );
        }
        Stmt::FuncDecl(func) => println!(
            "{pad}{} ({})",
            "Stmt::FuncDecl".paint(color_func),
            func.name.paint(color_name)
        ),

        Stmt::ClassDecl {
            name: class_name,
            methods,
            ..
        } => {
            println!(
                "{pad}{} ({})",
                "Stmt::ClassDecl".paint(color_class),
                class_name.paint(color_name)
            );
            for m in methods {
                println!(
                    "{pad}  {} {} ({})",
                    "└──".paint(color_symbol),
                    "Stmt::ClassDecl::Method".paint(color_method),
                    m.name.paint(color_name)
                );
            }
        }

        Stmt::Method(m) => println!(
            "{pad}{} ({})",
            "Stmt::Method".paint(color_method),
            m.name.paint(color_name)
        ),

        Stmt::If {
            then_branch,
            else_ifs,
            else_branch,
            ..
        } => {
            println!("{pad}{}", "Stmt::If".paint(color_control));
            for s in then_branch {
                debug_stmt(s, indent + 2);
            }
            for (_cond, block) in else_ifs {
                println!("{pad}  {}", "├── Stmt::ElseIf".paint(color_symbol));
                if let Some(stmts) = block {
                    for s in stmts {
                        debug_stmt(s, indent + 4);
                    }
                }
            }
            if let Some(else_stmts) = else_branch {
                println!("{pad}  {}", "└── Stmt::Else".paint(color_symbol));
                for s in else_stmts {
                    debug_stmt(s, indent + 4);
                }
            }
        }

        Stmt::While { body, .. } => {
            println!("{pad}{}", "Stmt::While".paint(color_control));
            for s in body {
                debug_stmt(s, indent + 2);
            }
        }

        Stmt::For { body, .. } => {
            println!("{pad}{}", "Stmt::For".paint(color_control));
            for s in body {
                debug_stmt(s, indent + 2);
            }
        }

        Stmt::ForIn { body, .. } => {
            println!("{pad}{}", "Stmt::ForIn".paint(color_control));
            for s in body {
                debug_stmt(s, indent + 2);
            }
        }

        Stmt::ForOf { body, .. } => {
            println!("{pad}{}", "Stmt::ForOf".paint(color_control));
            for s in body {
                debug_stmt(s, indent + 2);
            }
        }

        Stmt::TryCatchFinally {
            try_block,
            catch_block,
            finally_block,
        } => {
            println!("{pad}{}", "Stmt::Try".paint(color_try));
            let size = try_block.len();

            for (idx, s) in try_block.iter().enumerate() {
                let ident_command = " ".repeat(indent);

                // if is last
                if idx + 1 == size {
                    print!("{ident_command}└──");
                    debug_stmt(s, 0);
                } else {
                    print!("{ident_command}├──");
                    debug_stmt(s, 0); // +2 a partir do finally_pad
                }
                // debug_stmt(s, indent + 2);
            }

            let has_finally = finally_block.is_some();

            if let Some((_name, catch_block)) = catch_block {
                let catch_indent = indent + 2;
                let catch_pad = " ".repeat(catch_indent);
                let symbol = if has_finally {
                    "├──"
                } else {
                    "└──"
                };
                println!(
                    "{catch_pad}{} {}",
                    symbol.paint(color_symbol),
                    "Stmt::Catch".paint(color_try)
                );

                let size = catch_block.len();
                for (idx, s) in catch_block.iter().enumerate() {
                    let ident_command = " ".repeat(catch_indent + 5);

                    // if is last
                    if idx + 1 == size {
                        print!("{ident_command}└──");
                        debug_stmt(s, 0);
                    } else {
                        print!("{ident_command}├──");
                        debug_stmt(s, 0); // +2 a partir do finally_pad
                    }
                }
            }

            if let Some(finally_block) = finally_block {
                let finally_indent = indent + 2;
                let finally_pad = " ".repeat(finally_indent);
                println!(
                    "{finally_pad}{} {}",
                    "└──".paint(color_symbol),
                    "Stmt::Finally".paint(color_try)
                );
                let size = finally_block.len();
                for (idx, s) in finally_block.iter().enumerate() {
                    let ident_command = " ".repeat(finally_indent + 5);

                    // if is last
                    if idx + 1 == size {
                        print!("{ident_command}└──");
                        debug_stmt(s, 0);
                    } else {
                        print!("{ident_command}├──");
                        debug_stmt(s, 0); // +2 a partir do finally_pad
                    }
                }
            }
        }
        Stmt::Throw(_) => println!("{pad}{}", "Stmt::Throw".paint(color_throw)),
        Stmt::Break => println!("{pad}{}", "Stmt::Break".paint(color_other)),
        Stmt::Continue => println!("{pad}{}", "Stmt::Continue".paint(color_other)),
    }
}
impl<T: std::fmt::Debug + std::convert::From<std::string::String> + From<Value> + Clone>
    ControlFlow<T>
{
    #[track_caller]
    pub fn new_error(env: &mut Rc<RefCell<Environment>>, msg: String) -> ControlFlow<T> {
        let location = std::panic::Location::caller().to_string()
            + " "
            + std::file!()
            + ":"
            + &std::line!().to_string();
        let env = env.borrow();
        let error_class = env.get("Error");
        if error_class.is_none() {
            panic!("Error class not found {msg} {location}")
        }
        let error_class = error_class.unwrap().to_class();
        if error_class.is_none() {
            panic!("Class 'Error' not found");
        }
        let error_class = error_class.unwrap();

        let throw_method = error_class.find_static_method("throw");

        if throw_method.is_none() {
            panic!("throw method not found");
        }
        let throw_method = throw_method.unwrap();

        let error = throw_method.call(vec![Value::Null, Value::String(msg.into())]);

        if error.is_err() {
            panic!("Error creating error object {:?}", error.to_string());
        }
        ControlFlow::Error(error.into())
    }
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

    pub fn is_error(&self) -> bool {
        matches!(self, ControlFlow::Error(_))
    }

    pub fn is_err(&self) -> bool {
        matches!(self, ControlFlow::Error(_))
    }

    pub fn is_break(&self) -> bool {
        matches!(self, ControlFlow::Break)
    }

    pub fn is_continue(&self) -> bool {
        matches!(self, ControlFlow::Continue)
    }

    pub fn err(self) -> Option<T> {
        match self {
            ControlFlow::Error(msg) => Some(msg.into()),
            _ => None,
        }
    }

    #[track_caller]
    pub fn unwrap(self) -> T {
        let location = std::panic::Location::caller().to_string()
            + " "
            + std::file!()
            + ":"
            + &std::line!().to_string();
        match self {
            ControlFlow::Return(value) => value,
            ControlFlow::Error(value) => value,
            other => panic!("Cannot unwrap {other:?} {location}"),
        }
    }

    pub fn is_return(&self) -> bool {
        matches!(self, ControlFlow::Return(_))
    }
    pub fn unwrap_err(self) -> T {
        match self {
            ControlFlow::Error(value) => value,
            other => panic!("Cannot unwrap {other:?}"),
        }
    }

    pub fn as_error(&self) -> Option<ControlFlow<T>> {
        match self {
            ControlFlow::Return(msg) => Some(ControlFlow::Error(msg.to_owned())),
            ControlFlow::Error(msg) => Some(ControlFlow::Error(msg.to_owned())),
            _ => None,
        }
    }
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            ControlFlow::Return(value) => value,
            _ => default,
        }
    }

    pub fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce(ControlFlow<T>) -> T,
    {
        match self {
            ControlFlow::Return(value) => value,
            flow => f(flow),
        }
    }
    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        match self {
            ControlFlow::Return(value) => value,
            _ => Default::default(),
        }
    }

    pub fn error(msg: String) -> ControlFlow<T> {
        ControlFlow::Error(msg.into())
    }

    pub fn name(&self) -> String {
        match self {
            ControlFlow::None => "None".to_string(),
            ControlFlow::Return(_) => "Return".to_string(),
            ControlFlow::Error(_) => "Error".to_string(),
            ControlFlow::Break => "Break".to_string(),
            ControlFlow::Continue => "Continue".to_string(),
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum AssignOperator {
    // =
    Assign,
    // +=
    AddAssign,
    // -=
    SubAssign,
    // *=
    MulAssign,
    // /=
    DivAssign,
    // %=
    ModAssign,
    // **=
    PowAssign,
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Operator {
    Binary(BinaryOperator),
    Compare(CompareOperator),
    Logical(LogicalOperator),
    Unary(UnaryOperator),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Exponentiate,
}

impl BinaryOperator {
    pub fn alias(&self) -> String {
        match self {
            BinaryOperator::Add => "add",
            BinaryOperator::Subtract => "sub",
            BinaryOperator::Multiply => "mul",
            BinaryOperator::Divide => "div",
            BinaryOperator::Modulo => "mod",
            BinaryOperator::Exponentiate => "exp",
        }
        .to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum LogicalOperator {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum UnaryOperator {
    Negative,
    Not,
    Typeof,
    Increment,
    Decrement,
    Positive,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MethodDecl {
    pub name: String,
    pub params: Vec<String>,
    pub vararg: Option<String>,
    pub body: Vec<Stmt>,
    pub modifiers: Vec<Modifiers>,
}

pub trait MethodModifiersOperations {
    fn contains(&self, modifier: Modifiers) -> bool;

    fn contains_all(&self, modifiers: Vec<Modifiers>) -> bool {
        modifiers.iter().all(|m| self.contains(m.clone()))
    }
    fn contains_any(&self, modifiers: Vec<Modifiers>) -> bool {
        modifiers.iter().any(|m| self.contains(m.clone()))
    }
    fn contains_str(&self, modifier: &str) -> bool {
        match modifier.to_lowercase().as_str() {
            "static" => self.contains(Modifiers::Static),
            "operator" => self.contains(Modifiers::Operator),
            "private" => self.contains(Modifiers::Private),
            _ => false,
        }
    }
}
impl MethodModifiersOperations for [Modifiers] {
    fn contains(&self, modifier: Modifiers) -> bool {
        self.contains(&modifier)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Modifiers {
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
            Stmt::Let { name, value } => format!("let {} = {};", name, value.to_string()),
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
