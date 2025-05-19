use crate::{
    ast::ast::{
        BinaryOperator, CompareOperator, Expr, Literal, LogicalOperator, MathOperator, Stmt,
    },
    environment::{Environment, Value},
};

pub struct Interpreter {
    ast: Vec<Stmt>,
    // env: crate::environment::Environment,
}

impl Interpreter {
    pub fn new(ast: Vec<Stmt>) -> Self {
        Self { ast }
    }

    pub fn interpret(&mut self) {
        let ast = self.ast.clone();
        let mut env = Environment::new();
        for stmt in ast {
            if let Some(val) = self.exec_stmt(&stmt, &mut env) {
                println!("{:?}", val);
            }
        }
    }

    pub fn eval_expr(&mut self, expr: &Expr, env: &mut Environment) -> Value {
        match expr {
            Expr::Identifier(name) => {
                let msg = format!("Variable {name} not found");
                env.get(name).cloned().expect(&msg)
            }
            Expr::Literal(lit) => match lit {
                Literal::Number(n) => Value::Number(*n),
                Literal::Bool(b) => Value::Bool(*b),
                Literal::String(s) => Value::String(s.clone()),
                Literal::Null => Value::Null,
                Literal::Void => Value::Void,
                Literal::Object(obj) => {
                    let mut properties = std::collections::HashMap::new();
                    for (key, value) in obj {
                        let val = self.eval_expr(value, env);
                        properties.insert(key.clone(), val);
                    }
                    Value::Object(properties)
                }
                Literal::Array(arr) => {
                    let mut elements = Vec::new();
                    for elem in arr {
                        let val = self.eval_expr(elem, env);
                        elements.push(val);
                    }
                    Value::Array(elements)
                }
            },
            Expr::Block(stmts) => {
                let mut local_env = Environment::new();
                for stmt in stmts {
                    if let Some(ret) = self.exec_stmt(&stmt, &mut local_env) {
                        return ret;
                    }
                }
                Value::Void
            }
            Expr::BinaryOp { op, left, right } => {
                let l = self.eval_expr(left, env);
                let r = self.eval_expr(right, env);
                match (op, l, r) {
                    (BinaryOperator::Math(math_op), Value::Number(a), Value::Number(b)) => {
                        match math_op {
                            MathOperator::Add => Value::Number(a + b),
                            MathOperator::Sub => Value::Number(a - b),
                            MathOperator::Mul => Value::Number(a * b),
                            MathOperator::Div => Value::Number(a / b),
                        }
                    }

                    (BinaryOperator::Compare(comp_op), Value::Number(a), Value::Number(b)) => {
                        match comp_op {
                            CompareOperator::Eq => Value::Bool(a == b),
                            CompareOperator::Ne => Value::Bool(a != b),
                            CompareOperator::Gt => Value::Bool(a > b),
                            CompareOperator::Ge => Value::Bool(a >= b),
                            CompareOperator::Lt => Value::Bool(a < b),
                            CompareOperator::Le => Value::Bool(a <= b),
                        }
                    }

                    (BinaryOperator::Logical(log_op), Value::Bool(a), Value::Bool(b)) => {
                        match log_op {
                            LogicalOperator::And => Value::Bool(a && b),
                            LogicalOperator::Or => Value::Bool(a || b),
                        }
                    }

                    _ => Value::Null, // fallback
                }
            }
            Expr::Call { callee, args } => {
                let name = match callee.as_ref() {
                    Expr::Identifier(name) => name.clone(),
                    Expr::MemberAccess {
                        object: _,
                        property,
                    } => {
                        let property = *(property.clone());
                        match property {
                            Expr::Identifier(name) => name.clone(),
                            _ => return Value::Null,
                        }
                    }
                    _ => {
                        println!("Function {:?} not found", callee);
                        return Value::Void;
                    }
                };

                let arg_values: Vec<_> = args.iter().map(|arg| self.eval_expr(arg, env)).collect();
                let mut global_copy = env; //Cria uma copia do ambiente global no momento atual(Previne que as variaveis locais entrem no escopo global)
                match global_copy.get(&name).cloned() {
                    Some(Value::Function { params, body }) => {
                        let mut local_env = Environment::new();
                        for (param, val) in params.iter().zip(arg_values) {
                            local_env.variables.insert(param.clone(), val);
                        }
                        global_copy.merge(local_env);
                        for stmt in body {
                            if let Some(ret) = self.exec_stmt(&stmt, &mut global_copy) {
                                return ret;
                            }
                        }
                        Value::Null
                    }
                    Some(Value::Builtin(func)) => func(arg_values),
                    _ => Value::Null,
                }
            }
            Expr::Assign { name, value } => {
                let val = self.eval_expr(value, env);
                env.assign(&name, val).unwrap();
                Value::Void
            }
            Expr::MemberAccess { object, property } => {
                let obj = self.eval_expr(object, env);
                let prop = match property.as_ref() {
                    Expr::Identifier(name) => Value::String(name.clone()),
                    Expr::Literal(Literal::Number(n)) => Value::Number(*n),
                    _ => return Value::Null, // ou erro
                };

                match (&obj, &prop) {
                    (Value::Object(obj), Value::String(prop)) => {
                        let msg = format!("Property {prop} not found");
                        obj.get(prop).cloned().expect(&msg)
                    }
                    (Value::Array(arr), Value::Number(index)) => {
                        let index = *index as usize;
                        let msg = format!("Index {index} out of bounds");
                        arr.get(index).cloned().expect(&msg)
                    }
                    (Value::String(arr), Value::Number(index)) => {
                        let index = index.clone() as usize;
                        let msg = format!("Index {index} out of bounds");
                        arr.chars()
                            .nth(index)
                            .map(|ch: char| Value::String(ch.to_string()))
                            .expect(&msg)
                    }
                    _ => panic!(
                        "Cannot access property {:?} of {:?} (type: {:?}, {:?})",
                        prop.to_string(),
                        obj.to_string(),
                        &obj.type_of(),
                        &prop.type_of()
                    ),
                }
            }
            _ => todo!(),
        }
    }

    pub fn exec_stmt(&mut self, stmt: &Stmt, mut env: &mut Environment) -> Option<Value> {
        match stmt {
            Stmt::Let { name, value } => {
                let value = value.clone();
                let expr = match value {
                    Some(expr) => expr,
                    None => Expr::Literal(Literal::Null),
                };

                let val = self.eval_expr(&expr, &mut env);

                env.define(name.clone(), val);
                None
            }
            Stmt::FuncDecl { name, params, body } => {
                env.define(
                    name.clone(),
                    Value::Function {
                        params: params.clone(),
                        body: body.clone(),
                    },
                );
                None
            }
            Stmt::Return(expr) => {
                let value = expr.clone();

                let expr = match value {
                    Some(expr) => expr,
                    None => Expr::Literal(Literal::Null),
                };
                let val = self.eval_expr(&expr, &mut env);
                Some(val)
            }
            Stmt::ExprStmt(expr) => {
                self.eval_expr(expr, &mut env);
                None
            }
            Stmt::For {
                init,
                condition,
                update,
                body,
            } => {
                if let Some(()) = self.try_fast_for(init, condition, update, body, env) {
                    return None;
                }

                // Executa a inicialização no ambiente atual
                self.exec_stmt(init, env);

                while {
                    // Avalia a condição se existir, senão assume verdadeiro
                    match condition {
                        Some(cond_expr) => match self.eval_expr(cond_expr, env) {
                            Value::Bool(b) => b,
                            _ => panic!("For condition must evaluate to boolean"),
                        },
                        None => true,
                    }
                } {
                    // Executa o corpo do for
                    for stmt in body {
                        if let Some(ret) = self.exec_stmt(stmt, env) {
                            return Some(ret); // Retorno precoce (ex: return dentro do for)
                        }
                    }

                    // Executa o update se existir
                    if let Some(update_expr) = update {
                        self.eval_expr(update_expr, env);
                    }
                }

                None
            }
            Stmt::If {
                condition,
                then_branch,
                else_ifs,
                else_branch,
            } => {
                if self.eval_expr(condition, env).to_bool() {
                    for stmt in then_branch {
                        if let Some(ret) = self.exec_stmt(stmt, env) {
                            return Some(ret);
                        }
                    }
                } else {
                    for (cond, branch) in else_ifs {
                        if self.eval_expr(cond, env).to_bool() {
                            if branch.is_none() {
                                return None;
                            }
                            let branch = branch.clone().unwrap_or(vec![]);
                            for stmt in branch {
                                if let Some(ret) = self.exec_stmt(&stmt, env) {
                                    return Some(ret);
                                }
                            }
                            return None;
                        }
                    }

                    if let Some(else_branch) = else_branch {
                        for stmt in else_branch {
                            if let Some(ret) = self.exec_stmt(stmt, env) {
                                return Some(ret);
                            }
                        }
                    }
                }
                None
            }
            _ => todo!(),
        }
    }

    fn try_fast_for(
        &mut self,
        init: &Stmt,
        condition: &Option<Expr>,
        update: &Option<Expr>,
        body: &[Stmt],
        env: &mut Environment,
    ) -> Option<()> {
        // Só funciona para corpo vazio ou muito simples (otimização conservadora)
        if !body.is_empty() {
            return None;
        }

        // Reconhece: let i = 0;
        let (var_name, start) = match init {
            Stmt::Let {
                name,
                value: Some(Expr::Literal(Literal::Number(n))),
            } => (name.as_str(), *n),
            _ => return None,
        };

        // Reconhece: i < LIMITE
        let limit = match condition {
            Some(Expr::BinaryOp {
                op: BinaryOperator::Compare(CompareOperator::Lt),
                left,
                right,
            }) => match (&**left, &**right) {
                (Expr::Identifier(name), Expr::Literal(Literal::Number(n))) if name == var_name => {
                    *n
                }
                _ => return None,
            },
            _ => return None,
        };

        // Reconhece: i = i + 1
        match update {
            Some(Expr::Assign { name, value }) if Self::get_condition(name, var_name, value) => {
                // Executa nativamente
                let mut i = start;
                while i < limit {
                    i += 1.0;
                }
                // Atualiza o valor final de i no ambiente
                env.assign(var_name, Value::Number(i)).ok()?;
                Some(())
            }
            _ => None,
        }
    }

    fn get_condition(name: &String, var_name: &str, value: &Box<Expr>) -> bool {
        let value = value.as_ref();
        match value {
            Expr::BinaryOp { op: _, left, right } => {
                let lhs = left.as_ref().to_string().unwrap();
                let inc = right.as_ref().to_number().unwrap();

                name == var_name && lhs == var_name && inc == 1.0
            }
            _ => false,
        }
    }
}
