pub fn create_error_class() -> Stmt {
    let mut instance_fields = HashMap::new();
    let static_fields = HashMap::new();

    instance_fields.insert("message".to_string(), Expr::Literal(Literal::String("Default error message".to_string())));
    instance_fields.insert("name".to_string(), Expr::Literal(Literal::String("Error".to_string())));

    let constructor = MethodDecl {
        name: "constructor".to_string(),
        params: vec!["self".to_string(), "name".to_string(), "message".to_string()],
        vararg: None,
        body: vec![
            Stmt::ExprStmt(Expr::Assign { target: Box::new(Expr::GetProperty { object: Box::new(Expr::Identifier("self".to_string())), property: Box::new(Expr::Identifier("name".to_string())) }), op: AssignOperator::Assign, value: Box::new(Expr::Identifier("name".to_string())) }),
            Stmt::ExprStmt(Expr::Assign { target: Box::new(Expr::GetProperty { object: Box::new(Expr::Identifier("self".to_string())), property: Box::new(Expr::Identifier("message".to_string())) }), op: AssignOperator::Assign, value: Box::new(Expr::Identifier("message".to_string())) }),
        ],
        modifiers: vec![],
    };

    let throw = MethodDecl {
        name: "throw".to_string(),
        params: vec!["name".to_string(), "message".to_string()],
        vararg: None,
        body: vec![
            Stmt::Return(Some(Expr::New { class_expr: Box::new(Expr::Call { callee: Box::new(Expr::Identifier("Error".to_string())), args: vec![Expr::Identifier("name".to_string()), Expr::Identifier("message".to_string())] }) })),
        ],
        modifiers: vec![Modifiers::Static],
    };

    let paint = MethodDecl {
        name: "paint".to_string(),
        params: vec!["self".to_string()],
        vararg: None,
        body: vec![
            Stmt::Let { name: "redName".to_string(), value: Expr::BinaryOp { op: Operator::Binary(crate::ast::ast::BinaryOperator::Add), left: Box::new(Expr::BinaryOp { op: Operator::Binary(crate::ast::ast::BinaryOperator::Add), left: Box::new(Expr::Literal(Literal::String("\x1b[31m".to_string()))), right: Box::new(Expr::GetProperty { object: Box::new(Expr::Identifier("self".to_string())), property: Box::new(Expr::Identifier("name".to_string())) }) }), right: Box::new(Expr::Literal(Literal::String("\x1b[0m".to_string()))) } },
            Stmt::Return(Some(Expr::BinaryOp { op: Operator::Binary(crate::ast::ast::BinaryOperator::Add), left: Box::new(Expr::BinaryOp { op: Operator::Binary(crate::ast::ast::BinaryOperator::Add), left: Box::new(Expr::Identifier("redName".to_string())), right: Box::new(Expr::Literal(Literal::String(": ".to_string()))) }), right: Box::new(Expr::GetProperty { object: Box::new(Expr::Identifier("self".to_string())), property: Box::new(Expr::Identifier("message".to_string())) }) })),
        ],
        modifiers: vec![],
    };

    let to_string = MethodDecl {
        name: "toString".to_string(),
        params: vec!["self".to_string()],
        vararg: None,
        body: vec![
            Stmt::Return(Some(Expr::Call { callee: Box::new(Expr::GetProperty { object: Box::new(Expr::Identifier("self".to_string())), property: Box::new(Expr::Identifier("paint".to_string())) }), args: vec![] })),
        ],
        modifiers: vec![],
    };

    let value_of = MethodDecl {
        name: "valueOf".to_string(),
        params: vec!["self".to_string()],
        vararg: None,
        body: vec![
            Stmt::Return(Some(Expr::Call { callee: Box::new(Expr::GetProperty { object: Box::new(Expr::Identifier("self".to_string())), property: Box::new(Expr::Identifier("toString".to_string())) }), args: vec![] })),
        ],
        modifiers: vec![],
    };

    let get_message = MethodDecl {
        name: "getMessage".to_string(),
        params: vec!["self".to_string()],
        vararg: None,
        body: vec![
            Stmt::Return(Some(Expr::GetProperty { object: Box::new(Expr::Identifier("self".to_string())), property: Box::new(Expr::Identifier("message".to_string())) })),
        ],
        modifiers: vec![],
    };

    let get_name = MethodDecl {
        name: "getName".to_string(),
        params: vec!["self".to_string()],
        vararg: None,
        body: vec![
            Stmt::Return(Some(Expr::GetProperty { object: Box::new(Expr::Identifier("self".to_string())), property: Box::new(Expr::Identifier("name".to_string())) })),
        ],
        modifiers: vec![],
    };

    let set_name = MethodDecl {
        name: "setName".to_string(),
        params: vec!["self".to_string(), "name".to_string()],
        vararg: None,
        body: vec![
            Stmt::ExprStmt(Expr::Assign { target: Box::new(Expr::GetProperty { object: Box::new(Expr::Identifier("self".to_string())), property: Box::new(Expr::Identifier("name".to_string())) }), op: AssignOperator::Assign, value: Box::new(Expr::Identifier("name".to_string())) }),
        ],
        modifiers: vec![],
    };

    let set_message = MethodDecl {
        name: "setMessage".to_string(),
        params: vec!["self".to_string(), "message".to_string()],
        vararg: None,
        body: vec![
            Stmt::ExprStmt(Expr::Assign { target: Box::new(Expr::GetProperty { object: Box::new(Expr::Identifier("self".to_string())), property: Box::new(Expr::Identifier("message".to_string())) }), op: AssignOperator::Assign, value: Box::new(Expr::Identifier("message".to_string())) }),
        ],
        modifiers: vec![],
    };

    let class_stmt = Stmt::ClassDecl {
        name: "Error".to_string(),
        superclass: None,
        methods: vec![constructor, throw, paint, to_string, value_of, get_message, get_name, set_name, set_message],
        static_fields,
        instance_fields,
    };
    class_stmt
}

pub fn create_error_class() -> Stmt {
    let mut instance_fields = HashMap::new();
    let static_fields = HashMap::new();

    instance_fields.insert("name".to_string(), Expr::Literal(Literal::String("Error".to_string())));
    instance_fields.insert("message".to_string(), Expr::Literal(Literal::String("Default error message".to_string())));

    let constructor = MethodDecl {
        name: "constructor".to_string(),
        params: vec!["self".to_string(), "name".to_string(), "message".to_string()],
        vararg: None,
        body: vec![
            Stmt::If { condition: Expr::BinaryOp { op: Operator::Compare(crate::ast::ast::CompareOperator::Ne), left: Box::new(Expr::Identifier("name".to_string())), right: Box::new(Expr::Literal(Literal::Null)) }, then_branch: vec![Stmt::ExprStmt(Expr::Assign { target: Box::new(Expr::GetProperty { object: Box::new(Expr::Identifier("self".to_string())), property: Box::new(Expr::Identifier("name".to_string())) }), op: AssignOperator::Assign, value: Box::new(Expr::Identifier("name".to_string())) })], else_ifs: vec![], else_branch: None },
            Stmt::If { condition: Expr::BinaryOp { op: Operator::Compare(crate::ast::ast::CompareOperator::Ne), left: Box::new(Expr::Identifier("message".to_string())), right: Box::new(Expr::Literal(Literal::Null)) }, then_branch: vec![Stmt::ExprStmt(Expr::Assign { target: Box::new(Expr::GetProperty { object: Box::new(Expr::Identifier("self".to_string())), property: Box::new(Expr::Identifier("message".to_string())) }), op: AssignOperator::Assign, value: Box::new(Expr::Identifier("message".to_string())) })], else_ifs: vec![], else_branch: None },
        ],
        modifiers: vec![],
    };

    let throw = MethodDecl {
        name: "throw".to_string(),
        params: vec!["name".to_string(), "message".to_string()],
        vararg: None,
        body: vec![
            Stmt::Return(Some(Expr::New { class_expr: Box::new(Expr::Call { callee: Box::new(Expr::Identifier("Error".to_string())), args: vec![Expr::Identifier("name".to_string()), Expr::Identifier("message".to_string())] }) })),
        ],
        modifiers: vec![Modifiers::Static],
    };

    let paint = MethodDecl {
        name: "paint".to_string(),
        params: vec!["self".to_string()],
        vararg: None,
        body: vec![
            Stmt::Let { name: "redName".to_string(), value: Expr::BinaryOp { op: Operator::Binary(crate::ast::ast::BinaryOperator::Add), left: Box::new(Expr::BinaryOp { op: Operator::Binary(crate::ast::ast::BinaryOperator::Add), left: Box::new(Expr::Literal(Literal::String("\x1b[31m".to_string()))), right: Box::new(Expr::GetProperty { object: Box::new(Expr::Identifier("self".to_string())), property: Box::new(Expr::Identifier("name".to_string())) }) }), right: Box::new(Expr::Literal(Literal::String("\x1b[0m".to_string()))) } },
            Stmt::Return(Some(Expr::BinaryOp { op: Operator::Binary(crate::ast::ast::BinaryOperator::Add), left: Box::new(Expr::BinaryOp { op: Operator::Binary(crate::ast::ast::BinaryOperator::Add), left: Box::new(Expr::Identifier("redName".to_string())), right: Box::new(Expr::Literal(Literal::String(": ".to_string()))) }), right: Box::new(Expr::GetProperty { object: Box::new(Expr::Identifier("self".to_string())), property: Box::new(Expr::Identifier("message".to_string())) }) })),
        ],
        modifiers: vec![],
    };

    let to_string = MethodDecl {
        name: "toString".to_string(),
        params: vec!["self".to_string()],
        vararg: None,
        body: vec![
            Stmt::Return(Some(Expr::Call { callee: Box::new(Expr::GetProperty { object: Box::new(Expr::Identifier("self".to_string())), property: Box::new(Expr::Identifier("paint".to_string())) }), args: vec![] })),
        ],
        modifiers: vec![],
    };

    let value_of = MethodDecl {
        name: "valueOf".to_string(),
        params: vec!["self".to_string()],
        vararg: None,
        body: vec![
            Stmt::Return(Some(Expr::Call { callee: Box::new(Expr::GetProperty { object: Box::new(Expr::Identifier("self".to_string())), property: Box::new(Expr::Identifier("toString".to_string())) }), args: vec![] })),
        ],
        modifiers: vec![],
    };

    let get_message = MethodDecl {
        name: "getMessage".to_string(),
        params: vec!["self".to_string()],
        vararg: None,
        body: vec![
            Stmt::Return(Some(Expr::GetProperty { object: Box::new(Expr::Identifier("self".to_string())), property: Box::new(Expr::Identifier("message".to_string())) })),
        ],
        modifiers: vec![],
    };

    let get_name = MethodDecl {
        name: "getName".to_string(),
        params: vec!["self".to_string()],
        vararg: None,
        body: vec![
            Stmt::Return(Some(Expr::GetProperty { object: Box::new(Expr::Identifier("self".to_string())), property: Box::new(Expr::Identifier("name".to_string())) })),
        ],
        modifiers: vec![],
    };

    let set_name = MethodDecl {
        name: "setName".to_string(),
        params: vec!["self".to_string(), "name".to_string()],
        vararg: None,
        body: vec![
            Stmt::ExprStmt(Expr::Assign { target: Box::new(Expr::GetProperty { object: Box::new(Expr::Identifier("self".to_string())), property: Box::new(Expr::Identifier("name".to_string())) }), op: AssignOperator::Assign, value: Box::new(Expr::Identifier("name".to_string())) }),
        ],
        modifiers: vec![],
    };

    let set_message = MethodDecl {
        name: "setMessage".to_string(),
        params: vec!["self".to_string(), "message".to_string()],
        vararg: None,
        body: vec![
            Stmt::ExprStmt(Expr::Assign { target: Box::new(Expr::GetProperty { object: Box::new(Expr::Identifier("self".to_string())), property: Box::new(Expr::Identifier("message".to_string())) }), op: AssignOperator::Assign, value: Box::new(Expr::Identifier("message".to_string())) }),
        ],
        modifiers: vec![],
    };

    let class_stmt = Stmt::ClassDecl {
        name: "Error".to_string(),
        superclass: None,
        methods: vec![constructor, throw, paint, to_string, value_of, get_message, get_name, set_name, set_message],
        static_fields,
        instance_fields,
    };
    class_stmt
}

