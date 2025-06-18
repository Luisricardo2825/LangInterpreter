use std::collections::HashMap;

use crate::ast::ast::{
    AssignOperator, BinaryOperator, CompareOperator, Expr, FunctionStmt, Literal, LogicalOperator,
    MethodDecl, MethodModifiers, ObjectEntry, Operator, Stmt, UnaryOperator,
};
use std::fmt::Write;

pub struct ClassGenerator;

const CRATE_NAME: &str = env!("CARGO_PKG_NAME");

impl ClassGenerator {
    pub fn get_prop_from_this(ident: String) -> Expr {
        return crate::ast::ast::Expr::GetProperty {
            object: Box::new(Expr::This),
            property: Box::new(Expr::Identifier(ident)),
        };
    }
    pub fn set_prop_from_this(ident: String, value: Expr) -> Expr {
        return crate::ast::ast::Expr::SetProperty {
            object: Box::new(Expr::This),
            property: Box::new(Expr::Identifier(ident)),
            value: Box::new(value),
        };
    }

    pub fn create_error_class() -> Stmt {
        let mut instance_fields = HashMap::new();
        let static_fields = HashMap::new();

        instance_fields.insert(
            "name".to_string(),
            Expr::Literal(Literal::String("Error".to_string())),
        );
        instance_fields.insert(
            "message".to_string(),
            Expr::Literal(Literal::String("No message".to_string())),
        );

        let constructor = MethodDecl {
            name: "constructor".to_string(),
            params: vec!["message".to_string(), "name".to_string()],
            vararg: None,
            body: vec![
                Stmt::If {
                    condition: Expr::BinaryOp {
                        op: Operator::Compare(crate::ast::ast::CompareOperator::Ne),
                        left: Box::new(Expr::Identifier("message".to_string())),
                        right: Box::new(Expr::Literal(Literal::Null)),
                    },
                    then_branch: vec![Stmt::ExprStmt(Expr::Assign {
                        target: Box::new(Expr::GetProperty {
                            object: Box::new(Expr::This),
                            property: Box::new(Expr::Identifier("message".to_string())),
                        }),
                        op: AssignOperator::Assign,
                        value: Box::new(Expr::Identifier("message".to_string())),
                    })],
                    else_ifs: vec![],
                    else_branch: None,
                },
                Stmt::If {
                    condition: Expr::BinaryOp {
                        op: Operator::Compare(crate::ast::ast::CompareOperator::Ne),
                        left: Box::new(Expr::Identifier("name".to_string())),
                        right: Box::new(Expr::Literal(Literal::Null)),
                    },
                    then_branch: vec![Stmt::ExprStmt(Expr::Assign {
                        target: Box::new(Expr::GetProperty {
                            object: Box::new(Expr::This),
                            property: Box::new(Expr::Identifier("name".to_string())),
                        }),
                        op: AssignOperator::Assign,
                        value: Box::new(Expr::Identifier("name".to_string())),
                    })],
                    else_ifs: vec![],
                    else_branch: None,
                },
            ],
            modifiers: vec![],
        };

        let paint = MethodDecl {
            name: "paint".to_string(),
            params: vec!["str".to_string()],
            vararg: None,
            body: vec![Stmt::Return(Some(Expr::BinaryOp {
                op: Operator::Binary(crate::ast::ast::BinaryOperator::Add),
                left: Box::new(Expr::BinaryOp {
                    op: Operator::Binary(crate::ast::ast::BinaryOperator::Add),
                    left: Box::new(Expr::Literal(Literal::String("\x1b[31m".to_string()))),
                    right: Box::new(Expr::Identifier("str".to_string())),
                }),
                right: Box::new(Expr::Literal(Literal::String("\x1b[0m".to_string()))),
            }))],
            modifiers: vec![],
        };

        let get_message = MethodDecl {
            name: "getMessage".to_string(),
            params: vec![],
            vararg: None,
            body: vec![Stmt::Return(Some(Expr::BinaryOp {
                op: Operator::Binary(crate::ast::ast::BinaryOperator::Add),
                left: Box::new(Expr::BinaryOp {
                    op: Operator::Binary(crate::ast::ast::BinaryOperator::Add),
                    left: Box::new(Expr::Call {
                        callee: Box::new(Expr::Identifier("paint".to_string())),
                        args: vec![Expr::GetProperty {
                            object: Box::new(Expr::This),
                            property: Box::new(Expr::Identifier("name".to_string())),
                        }],
                    }),
                    right: Box::new(Expr::Literal(Literal::String(": ".to_string()))),
                }),
                right: Box::new(Expr::GetProperty {
                    object: Box::new(Expr::This),
                    property: Box::new(Expr::Identifier("message".to_string())),
                }),
            }))],
            modifiers: vec![],
        };

        let to_string = MethodDecl {
            name: "toString".to_string(),
            params: vec![],
            vararg: None,
            body: vec![Stmt::Return(Some(Expr::Call {
                callee: Box::new(Expr::Identifier("getMessage".to_string())),
                args: vec![],
            }))],
            modifiers: vec![],
        };

        let throw = MethodDecl {
            name: "throw".to_string(),
            params: vec!["message".to_string(), "name".to_string()],
            vararg: None,
            body: vec![Stmt::Throw(Expr::New {
                class_expr: Box::new(Expr::Call {
                    callee: Box::new(Expr::Identifier("Error".to_string())),
                    args: vec![
                        Expr::Identifier("message".to_string()),
                        Expr::Identifier("name".to_string()),
                    ],
                }),
            })],
            modifiers: vec![MethodModifiers::Static],
        };

        let class_stmt = Stmt::ClassDecl {
            name: "Error".to_string(),
            superclass: None,
            methods: vec![constructor, paint, get_message, to_string, throw],
            static_fields,
            instance_fields,
        };
        class_stmt
    }

    pub fn generate_class_function(decl: &Stmt) -> Option<String> {
        if let Stmt::ClassDecl {
            name,
            superclass,
            methods,
            instance_fields,
            ..
        } = decl
        {
            let mut out = String::new();

            writeln!(
                &mut out,
                "pub fn create_{}_class() -> Stmt {{",
                name.to_lowercase()
            )
            .unwrap();
            writeln!(&mut out, "    let mut instance_fields = HashMap::new();").unwrap();
            writeln!(&mut out, "    let static_fields = HashMap::new();\n").unwrap();

            for (k, v) in instance_fields {
                writeln!(
                    &mut out,
                    "    instance_fields.insert(\"{}\".to_string(), {});",
                    k,
                    Self::expr_to_code(v)
                )
                .unwrap();
            }

            writeln!(&mut out).unwrap();

            for m in methods {
                writeln!(
                    &mut out,
                    "    let {} = MethodDecl {{",
                    Self::camel_to_snake(&m.name)
                )
                .unwrap();
                writeln!(&mut out, "        name: \"{}\".to_string(),", m.name).unwrap();

                writeln!(
                    &mut out,
                    "        params: vec![{}],",
                    m.params
                        .iter()
                        .map(|p| format!("\"{}\".to_string()", p))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
                .unwrap();

                writeln!(
                    &mut out,
                    "        vararg: {},",
                    match &m.vararg {
                        Some(v) => format!("Some(\"{}\".to_string())", v),
                        None => "None".to_string(),
                    }
                )
                .unwrap();

                writeln!(&mut out, "        body: vec![").unwrap();
                for stmt in &m.body {
                    writeln!(&mut out, "            {},", Self::stmt_to_code(stmt)).unwrap();
                }
                writeln!(&mut out, "        ],").unwrap();

                writeln!(
                    &mut out,
                    "        modifiers: vec![{}],",
                    m.modifiers
                        .iter()
                        .map(|m| format!("MethodModifiers::{:?}", m))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
                .unwrap();

                writeln!(&mut out, "    }};\n").unwrap();
            }

            writeln!(
                &mut out,
                "    let class_stmt = Stmt::ClassDecl {{
        name: \"{}\".to_string(),
        superclass: {},
        methods: vec![{}],
        static_fields,
        instance_fields,
    }};",
                name,
                match superclass {
                    Some(e) => format!("Some({})", Self::expr_to_code(e)),
                    None => "None".to_string(),
                },
                methods
                    .iter()
                    .map(|m| Self::camel_to_snake(&m.name))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
            .unwrap();

            writeln!(&mut out, "    class_stmt\n}}\n").unwrap();

            Some(Self::escape_string_class_generator(&out))
        } else {
            None
        }
    }

    pub fn escape_string_class_generator(input: &str) -> String {
        let mut result = String::new();
        for c in input.chars() {
            match c {
                '\n' => result.push_str("\n"),
                '\t' => result.push_str("\\t"),
                '\r' => result.push_str("\\r"),
                '\0' => result.push_str("\\0"),
                '\"' => result.push_str("\""),
                '\'' => result.push_str("\\'"),
                '\\' => result.push_str("\\\\"),
                c if c.is_control() || c == '\x1b' => {
                    result.push_str(&format!("\\x{:02x}", c as u8));
                }
                _ => result.push(c),
            }
        }
        result
    }
    pub fn expr_to_code(expr: &Expr) -> String {
        match expr {
        Expr::Literal(lit) => match lit {
            Literal::Null => "Expr::Literal(Literal::Null)".to_string(),
            Literal::String(s) => format!("Expr::Literal(Literal::String(\"{}\".to_string()))", s),
            Literal::Number(n) => format!("Expr::Literal(Literal::Number({:?}))", n),
            Literal::Bool(b) => format!("Expr::Literal(Literal::Bool({}))", b),
            Literal::Void =>"Expr::Literal(Literal::Void)".to_string(),
            Literal::Array(exprs) =>  format!("Expr::Literal(Literal::Array(vec![{}]))", exprs.iter().map(Self::expr_to_code).collect::<Vec<_>>().join(", ")),
            Literal::Object(items) => {
    let entries_code = items.iter().map(|entry| {
        match entry {
            ObjectEntry::Property { key, value } => {
                format!("ObjectEntry::Property {{ key: \"{}\".to_string(), value: {} }}", key, Self::expr_to_code(value))
            }
            ObjectEntry::Shorthand(name) => {
                format!("ObjectEntry::Shorthand(\"{}\".to_string())", name)
            }
            ObjectEntry::Spread(expr) => {
                format!("ObjectEntry::Spread({})", Self::expr_to_code(expr))
            }
        }
    }).collect::<Vec<_>>().join(", ");

    format!("Expr::Literal(Literal::Object(vec![{}]))", entries_code)
},
        },
        Expr::Identifier(s) => format!("Expr::Identifier(\"{}\".to_string())", s),
        Expr::Assign { target, op, value } => format!(
            "Expr::Assign {{ target: Box::new({}), op: AssignOperator::{:?}, value: Box::new({}) }}",
            Self::expr_to_code(target),
            op,
            Self::expr_to_code(value)
        ),
        Expr::BinaryOp { op, left, right } => format!(
            "Expr::BinaryOp {{ op: {}, left: Box::new({}), right: Box::new({}) }}",
            Self::operator_to_code(op),
            Self::expr_to_code(left),
            Self::expr_to_code(right)
        ),
        Expr::GetProperty { object, property } => format!(
            "Expr::GetProperty {{ object: Box::new({}), property: Box::new({}) }}",
            Self::expr_to_code(object),
            Self::expr_to_code(property)
        ),
        Expr::SetProperty { object, property, value } => format!(
            "Expr::SetProperty {{ object: Box::new({}), property: Box::new({}), value: Box::new({}) }}",
            Self::expr_to_code(object),
            Self::expr_to_code(property),
            Self::expr_to_code(value)
        ),
        Expr::BracketAccess { object, property } => format!(
            "Expr::BracketAccess {{ object: Box::new({}), property: Box::new({}) }}",
            Self::expr_to_code(object),
            Self::expr_to_code(property)
        ),
        Expr::UnaryOp { op, expr, postfix } => format!(
            "Expr::UnaryOp {{ op: {}, expr: Box::new({}), postfix: {} }}",
            Self::unary_operator_to_code(op),
            Self::expr_to_code(expr),
            postfix
        ),
        Expr::Call { callee, args } => format!(
            "Expr::Call {{ callee: Box::new({}), args: vec![{}] }}",
            Self::expr_to_code(callee),
            args.iter().map(Self::expr_to_code).collect::<Vec<_>>().join(", ")
        ),
        Expr::New { class_expr } => format!(
            "Expr::New {{ class_expr: Box::new({}) }}",
            Self::expr_to_code(class_expr)
        ),
        Expr::This => "Expr::This".to_string(),
        Expr::Block(stmts) => format!(
            "Expr::Block(vec![{}])",
            stmts.iter().map(Self::stmt_to_code).collect::<Vec<_>>().join(", ")
        ),
        Expr::Spread(expr) => format!("Expr::Spread(Box::new({}))", Self::expr_to_code(expr)),
    }
    }

    fn stmt_to_code(stmt: &Stmt) -> String {
        match stmt {
            Stmt::ImportNamed { items, from } => format!(
                    "Stmt::ImportNamed {{ items: vec![{}], from: \"{}\".to_string() }}",
                    items.iter()
                        .map(|(e, l)| format!("(\"{}\".to_string(), \"{}\".to_string())", e, l))
                        .collect::<Vec<_>>()
                        .join(", "),
                    from
                ),
            Stmt::ImportDefault { local_name, from } => format!(
                    "Stmt::ImportDefault {{ local_name: \"{}\".to_string(), from: \"{}\".to_string() }}",
                    local_name, from
                ),
            Stmt::ImportAll { local_name, from } => format!(
                    "Stmt::ImportAll {{ local_name: \"{}\".to_string(), from: \"{}\".to_string() }}",
                    local_name, from
                ),
            Stmt::ImportMixed { default, items, from } => format!(
                    "Stmt::ImportMixed {{ default: \"{}\".to_string(), items: vec![{}], from: \"{}\".to_string() }}",
                    default,
                    items.iter()
                        .map(|(e, l)| format!("(\"{}\".to_string(), \"{}\".to_string())", e, l))
                        .collect::<Vec<_>>()
                        .join(", "),
                    from
                ),
            Stmt::Export(inner) => format!("Stmt::Export(Rc::new({}))", Self::stmt_to_code(inner)),
            Stmt::ExportDefault(inner) => format!("Stmt::ExportDefault(Rc::new({}))", Self::stmt_to_code(inner)),
            Stmt::Let { name, value } => match value {
                    Some(v) => format!(
                        "Stmt::Let {{ name: \"{}\".to_string(), value: Some({}) }}",
                        name,
                        Self::expr_to_code(v)
                    ),
                    None => format!(
                        "Stmt::Let {{ name: \"{}\".to_string(), value: None }}",
                        name
                    ),
                },
            Stmt::Throw(expr) => format!("Stmt::Throw({})", Self::expr_to_code(expr)),
            Stmt::ExprStmt(expr) => format!("Stmt::ExprStmt({})", Self::expr_to_code(expr)),
            Stmt::Return(Some(expr)) => format!("Stmt::Return(Some({}))", Self::expr_to_code(expr)),
            Stmt::Return(None) => "Stmt::Return(None)".to_string(),
            Stmt::Break => "Stmt::Break".to_string(),
            Stmt::Continue => "Stmt::Continue".to_string(),
 Stmt::FuncDecl(func) => {
            // Supondo que você tenha um func_to_code implementado
            format!("Stmt::FuncDecl({})", Self::func_to_code(func))
        }
        Stmt::ClassDecl { name, superclass, methods, static_fields, instance_fields } => {
            let superclass_code = if let Some(sc) = superclass {
                format!("Some({})", Self::expr_to_code(sc))
            } else {
                "None".to_string()
            };

            let methods_code = methods.iter()
                .map(|m| Self::method_to_code(m))
                .collect::<Vec<_>>()
                .join(", ");

            let static_fields_code = static_fields.iter()
                .map(|(k, v)| format!("(\"{}\".to_string(), {})", k, Self::expr_to_code(v)))
                .collect::<Vec<_>>()
                .join(", ");

            let instance_fields_code = instance_fields.iter()
                .map(|(k, v)| format!("(\"{}\".to_string(), {})", k, Self::expr_to_code(v)))
                .collect::<Vec<_>>()
                .join(", ");

            format!(
                "Stmt::ClassDecl {{ name: \"{}\".to_string(), superclass: {}, methods: vec![{}], static_fields: std::collections::HashMap::from([{}]), instance_fields: std::collections::HashMap::from([{}]) }}",
                name,
                superclass_code,
                methods_code,
                static_fields_code,
                instance_fields_code
            )
        }
        Stmt::Method(method_decl) => {
            format!("Stmt::Method({})", Self::method_to_code(method_decl))
        }
        Stmt::If { condition, then_branch, else_ifs, else_branch } => {
            let then_code = Self::stmt_vec_to_code(then_branch);
            let else_ifs_code = else_ifs.iter()
                .map(|(cond, stmts_opt)| {
                    let stmts_code = stmts_opt.as_ref()
                        .map(|stmts| Self::stmt_vec_to_code(stmts))
                        .unwrap_or_else(|| "None".to_string());
                    format!("({}, {})", Self::expr_to_code(cond), if stmts_opt.is_some() { format!("Some(vec![{}])", stmts_code) } else { "None".to_string() })
                })
                .collect::<Vec<_>>()
                .join(", ");

            let else_code = else_branch.as_ref()
                .map(|stmts| format!("Some(vec![{}])", Self::stmt_vec_to_code(stmts)))
                .unwrap_or_else(|| "None".to_string());

            format!(
                "Stmt::If {{ condition: {}, then_branch: vec![{}], else_ifs: vec![{}], else_branch: {} }}",
                Self::expr_to_code(condition),
                then_code,
                else_ifs_code,
                else_code
            )
        }
        Stmt::While { condition, body } => {
            let body_code = Self::stmt_vec_to_code(body);
            format!(
                "Stmt::While {{ condition: {}, body: vec![{}] }}",
                Self::expr_to_code(condition),
                body_code
            )
        }
        Stmt::For { init, condition, update, body } => {
            let cond_code = condition.as_ref()
                .map(|c| Self::expr_to_code(c))
                .unwrap_or_else(|| "None".to_string());
            let update_code = update.as_ref()
                .map(|u| Self::expr_to_code(u))
                .unwrap_or_else(|| "None".to_string());
            let body_code = Self::stmt_vec_to_code(body);
            format!(
                "Stmt::For {{ init: Box::new({}), condition: {}, update: {}, body: vec![{}] }}",
                Self::stmt_to_code(init),
                if cond_code == "None" { "None".to_string() } else { format!("Some({})", cond_code) },
                if update_code == "None" { "None".to_string() } else { format!("Some({})", update_code) },
                body_code
            )
        }
        Stmt::ForIn { target, object, body } => {
            let body_code = Self::stmt_vec_to_code(body);
            format!(
                "Stmt::ForIn {{ target: {}, object: {}, body: vec![{}] }}",
                Self::expr_to_code(target),
                Self::expr_to_code(object),
                body_code
            )
        }
        Stmt::ForOf { target, iterable, body } => {
            let body_code = Self::stmt_vec_to_code(body);
            format!(
                "Stmt::ForOf {{ target: {}, iterable: {}, body: vec![{}] }}",
                Self::expr_to_code(target),
                Self::expr_to_code(iterable),
                body_code
            )
        }
        Stmt::TryCatchFinally { try_block, catch_block, finally_block } => {
            let try_code = Self::stmt_vec_to_code(try_block);

            let catch_code = catch_block.as_ref()
                .map(|(param, block)| format!("Some((\"{}\".to_string(), vec![{}]))", param, Self::stmt_vec_to_code(block)))
                .unwrap_or_else(|| "None".to_string());

            let finally_code = finally_block.as_ref()
                .map(|block| format!("Some(vec![{}])", Self::stmt_vec_to_code(block)))
                .unwrap_or_else(|| "None".to_string());

            format!(
                "Stmt::TryCatchFinally {{ try_block: vec![{}], catch_block: {}, finally_block: {} }}",
                try_code,
                catch_code,
                finally_code
            )
        }
        }
    }
    pub fn stmt_vec_to_code(stmts: &[Stmt]) -> String {
        stmts
            .iter()
            .map(|s| Self::stmt_to_code(s))
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn method_to_code(method: &MethodDecl) -> String {
        let params_code = method
            .params
            .iter()
            .map(|p| format!("\"{}\".to_string()", p))
            .collect::<Vec<_>>()
            .join(", ");

        let body_code = Self::stmt_vec_to_code(&method.body);

        let modifiers_code = method
            .modifiers
            .iter()
            .map(|m| format!("Modifier::{:?}", m))
            .collect::<Vec<_>>()
            .join(", ");

        let vararg_code = match &method.vararg {
            Some(v) => format!("Some(\"{}\".to_string())", v),
            None => "None".to_string(),
        };

        format!(
            "MethodDecl {{ name: \"{}\".to_string(), params: vec![{}], body: vec![{}], modifiers: vec![{}], vararg: {} }}",
            method.name,
            params_code,
            body_code,
            modifiers_code,
            vararg_code,
        )
    }

    pub fn func_to_code(func: &FunctionStmt) -> String {
        let params_code = func
            .params
            .iter()
            .map(|p| format!("\"{}\".to_string()", p))
            .collect::<Vec<_>>()
            .join(", ");

        let body_code = Self::stmt_vec_to_code(&func.body);

        let vararg_code = match &func.vararg {
            Some(v) => format!("Some(\"{}\".to_string())", v),
            None => "None".to_string(),
        };

        format!(
            "FunctionStmt {{ name: \"{}\".to_string(), params: vec![{}], body: vec![{}], vararg: {} }}",
            func.name,
            params_code,
            body_code,
            vararg_code,
        )
    }

    pub fn camel_to_snake(s: &str) -> String {
        let mut result = String::new();
        let mut prev_char = ' ';
        for c in s.chars() {
            if c.is_uppercase() && prev_char != ' ' && prev_char != '_' {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
            prev_char = c;
        }
        result
    }

    pub fn get_full_type_name<T>(_: &T) -> String {
        let full = std::any::type_name::<T>();
        full.replace(CRATE_NAME, "crate")
    }
    pub fn operator_to_code(op: &Operator) -> String {
        match op {
            Operator::Binary(b) => format!(
                "Operator::Binary({}::{})",
                Self::get_full_type_name(b),
                Self::binary_operator_to_code(b)
            ),
            Operator::Compare(c) => format!(
                "Operator::Compare({}::{})",
                Self::get_full_type_name(c),
                Self::compare_operator_to_code(c)
            ),
            Operator::Logical(l) => {
                format!(
                    "Operator::Logical({}::{})",
                    Self::get_full_type_name(l),
                    Self::logical_operator_to_code(l)
                )
            }
            Operator::Unary(u) => format!(
                "Operator::Unary({}::{})",
                Self::get_full_type_name(u),
                Self::unary_operator_to_code(u)
            ),
        }
    }

    pub fn binary_operator_to_code(op: &BinaryOperator) -> &'static str {
        match op {
            BinaryOperator::Add => "Add",
            BinaryOperator::Subtract => "Subtract",
            BinaryOperator::Multiply => "Multiply",
            BinaryOperator::Divide => "Divide",
            BinaryOperator::Modulo => "Modulo",
            BinaryOperator::Exponentiate => "Exponentiate",
        }
    }

    pub fn compare_operator_to_code(op: &CompareOperator) -> &'static str {
        match op {
            CompareOperator::Eq => "Eq",
            CompareOperator::Ne => "Ne",
            CompareOperator::Gt => "Gt",
            CompareOperator::Ge => "Ge",
            CompareOperator::Lt => "Lt",
            CompareOperator::Le => "Le",
            CompareOperator::InstanceOf => "InstanceOf",
            CompareOperator::In => "In",
        }
    }

    pub fn logical_operator_to_code(op: &LogicalOperator) -> &'static str {
        match op {
            LogicalOperator::And => "And",
            LogicalOperator::Or => "Or",
        }
    }

    pub fn unary_operator_to_code(op: &UnaryOperator) -> &'static str {
        match op {
            UnaryOperator::Negative => "Negative",
            UnaryOperator::Not => "Not",
            UnaryOperator::Typeof => "Typeof",
            UnaryOperator::Increment => "Increment",
            UnaryOperator::Decrement => "Decrement",
            UnaryOperator::Positive => "Positive",
        }
    }
}
