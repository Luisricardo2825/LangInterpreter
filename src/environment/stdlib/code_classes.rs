use std::collections::HashMap;

use crate::ast::ast::{AssignOperator, Expr, Literal, MethodDecl, Operator, Stmt};

pub fn create_testsexception_class() -> Stmt {
    let mut instance_fields = HashMap::new();
    let static_fields = HashMap::new();

    instance_fields.insert(
        "teste".to_string(),
        Expr::Literal(Literal::Number(10.4234324)),
    );

    let constructor = MethodDecl {
        name: "constructor".to_string(),
        params: vec!["message".to_string()],
        vararg: None,
        body: vec![
            Stmt::If {
                condition: Expr::BinaryOp {
                    op: Operator::Compare(crate::ast::ast::CompareOperator::Eq),
                    left: Box::new(Expr::Identifier("message".to_string())),
                    right: Box::new(Expr::Literal(Literal::Null)),
                },
                then_branch: vec![Stmt::ExprStmt(Expr::Assign {
                    target: Box::new(Expr::Identifier("message".to_string())),
                    op: AssignOperator::Assign,
                    value: Box::new(Expr::BinaryOp {
                        op: Operator::Binary(crate::ast::ast::BinaryOperator::Add),
                        left: Box::new(Expr::BinaryOp {
                            op: Operator::Binary(crate::ast::ast::BinaryOperator::Add),
                            left: Box::new(Expr::Literal(Literal::String("\x1b[31m".to_string()))),
                            right: Box::new(Expr::Literal(Literal::String(
                                "Erro de teste".to_string(),
                            ))),
                        }),
                        right: Box::new(Expr::Literal(Literal::String("\x1b[0m".to_string()))),
                    }),
                })],
                else_ifs: vec![],
                else_branch: None,
            },
            Stmt::ExprStmt(Expr::Call {
                callee: Box::new(Expr::Identifier("super".to_string())),
                args: vec![
                    Expr::Identifier("message".to_string()),
                    Expr::Literal(Literal::String("Exc".to_string())),
                ],
            }),
        ],
        modifiers: vec![],
    };

    let get_teste = MethodDecl {
        name: "getTeste".to_string(),
        params: vec![],
        vararg: None,
        body: vec![Stmt::Return(Some(Expr::GetProperty {
            object: Box::new(Expr::This),
            property: Box::new(Expr::Identifier("teste".to_string())),
        }))],
        modifiers: vec![],
    };

    let throw = MethodDecl {
        name: "throw".to_string(),
        params: vec![],
        vararg: None,
        body: vec![Stmt::Throw(Expr::This)],
        modifiers: vec![],
    };

    let class_stmt = Stmt::ClassDecl {
        name: "TestsException".to_string(),
        superclass: Some(Expr::Identifier("Error".to_string())),
        methods: vec![constructor, get_teste, throw],
        static_fields,
        instance_fields,
    };
    class_stmt
}
