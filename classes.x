
pub fn create_error_class() -> Stmt {
    let mut instance_fields = HashMap::new();
    let static_fields = HashMap::new();

    instance_fields.insert("message".to_string(), Expr::Literal(Literal::String("No message".to_string())));
    instance_fields.insert("name".to_string(), Expr::Literal(Literal::String("Error".to_string())));

    let constructor = MethodDecl {
        name: "constructor".to_string(),
        params: vec!["message".to_string()],
        vararg: None,
        body: vec![
            Stmt::If { condition: Expr::BinaryOp { op: Operator::Compare(crate::ast::ast::CompareOperator::Eq), left: Box::new(Expr::Identifier("message".to_string())), right: Box::new(Expr::Literal(Literal::Null)) }, then_branch: vec![Stmt::ExprStmt(Expr::Assign { target: Box::new(Expr::Identifier("message".to_string())), op: AssignOperator::Assign, value: Box::new(Expr::BinaryOp { op: Operator::Binary(crate::ast::ast::BinaryOperator::Add), left: Box::new(Expr::BinaryOp { op: Operator::Binary(crate::ast::ast::BinaryOperator::Add), left: Box::new(Expr::Literal(Literal::String("\x1b[31m".to_string()))), right: Box::new(Expr::Literal(Literal::String("Erro de teste".to_string()))) }), right: Box::new(Expr::Literal(Literal::String("\x1b[0m".to_string()))) }) })], else_ifs: vec![], else_branch: None },
            Stmt::ExprStmt(Expr::Call { callee: Box::new(Expr::Identifier("super".to_string())), args: vec![Expr::Identifier("message".to_string()), Expr::Literal(Literal::String("Exc".to_string()))] }),
        ],
        modifiers: vec![],
    };

    let paint = MethodDecl {
        name: "paint".to_string(),
        params: vec!["str".to_string()],
        vararg: None,
        body: vec![
            Stmt::Return(Some(Expr::BinaryOp { op: Operator::Binary(crate::ast::ast::BinaryOperator::Add), left: Box::new(Expr::BinaryOp { op: Operator::Binary(crate::ast::ast::BinaryOperator::Add), left: Box::new(Expr::Literal(Literal::String("\x1b[31m".to_string()))), right: Box::new(Expr::Identifier("str".to_string())) }), right: Box::new(Expr::Literal(Literal::String("\x1b[0m".to_string()))) })),
        ],
        modifiers: vec![],
    };

    let get_message = MethodDecl {
        name: "getMessage".to_string(),
        params: vec![],
        vararg: None,
        body: vec![
            Stmt::Return(Some(Expr::BinaryOp { op: Operator::Binary(crate::ast::ast::BinaryOperator::Add), left: Box::new(Expr::BinaryOp { op: Operator::Binary(crate::ast::ast::BinaryOperator::Add), left: Box::new(Expr::Call { callee: Box::new(Expr::Identifier("paint".to_string())), args: vec![Expr::GetProperty { object: Box::new(Expr::This), property: Box::new(Expr::Identifier("name".to_string())) }] }), right: Box::new(Expr::Literal(Literal::String(": ".to_string()))) }), right: Box::new(Expr::GetProperty { object: Box::new(Expr::This), property: Box::new(Expr::Identifier("message".to_string())) }) })),
        ],
        modifiers: vec![],
    };

    let to_string = MethodDecl {
        name: "toString".to_string(),
        params: vec![],
        vararg: None,
        body: vec![
            Stmt::Return(Some(Expr::Call { callee: Box::new(Expr::Identifier("getMessage".to_string())), args: vec![] })),
        ],
        modifiers: vec![],
    };

    let throw = MethodDecl {
        name: "throw".to_string(),
        params: vec!["message".to_string(), "name".to_string()],
        vararg: None,
        body: vec![
            Stmt::Throw(Expr::New { class_expr: Box::new(Expr::Call { callee: Box::new(Expr::Identifier("Error".to_string())), args: vec![Expr::Identifier("message".to_string()), Expr::Identifier("name".to_string())] }) }),
        ],
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

pub fn create_testsexception_class() -> Stmt {
    let mut instance_fields = HashMap::new();
    let static_fields = HashMap::new();


    let constructor = MethodDecl {
        name: "constructor".to_string(),
        params: vec!["message".to_string(), "name".to_string()],
        vararg: None,
        body: vec![
            Stmt::If { condition: Expr::BinaryOp { op: Operator::Compare(crate::ast::ast::CompareOperator::Ne), left: Box::new(Expr::Identifier("message".to_string())), right: Box::new(Expr::Literal(Literal::Null)) }, then_branch: vec![Stmt::ExprStmt(Expr::Assign { target: Box::new(Expr::GetProperty { object: Box::new(Expr::This), property: Box::new(Expr::Identifier("message".to_string())) }), op: AssignOperator::Assign, value: Box::new(Expr::Identifier("message".to_string())) })], else_ifs: vec![], else_branch: None },
            Stmt::If { condition: Expr::BinaryOp { op: Operator::Compare(crate::ast::ast::CompareOperator::Ne), left: Box::new(Expr::Identifier("name".to_string())), right: Box::new(Expr::Literal(Literal::Null)) }, then_branch: vec![Stmt::ExprStmt(Expr::Assign { target: Box::new(Expr::GetProperty { object: Box::new(Expr::This), property: Box::new(Expr::Identifier("name".to_string())) }), op: AssignOperator::Assign, value: Box::new(Expr::Identifier("name".to_string())) })], else_ifs: vec![], else_branch: None },
        ],
        modifiers: vec![],
    };

    let throw = MethodDecl {
        name: "throw".to_string(),
        params: vec![],
        vararg: None,
        body: vec![
            Stmt::Throw(Expr::This),
        ],
        modifiers: vec![],
    };

    let get_name = MethodDecl {
        name: "getName".to_string(),
        params: vec![],
        vararg: None,
        body: vec![
            Stmt::Return(Some(Expr::GetProperty { object: Box::new(Expr::This), property: Box::new(Expr::Identifier("name".to_string())) })),
        ],
        modifiers: vec![],
    };

    let get_this = MethodDecl {
        name: "getThis".to_string(),
        params: vec![],
        vararg: None,
        body: vec![
            Stmt::Return(Some(Expr::This)),
        ],
        modifiers: vec![],
    };

    let class_stmt = Stmt::ClassDecl {
        name: "TestsException".to_string(),
        superclass: Some(Expr::Identifier("Error".to_string())),
        methods: vec![constructor, throw, get_name, get_this],
        static_fields,
        instance_fields,
    };
    class_stmt
}

