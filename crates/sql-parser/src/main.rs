fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use sqlparser::{
        ast::{visit_statements, Expr, Query, Select, Statement},
        dialect::MySqlDialect,
        parser::Parser,
    };

    #[test]
    fn test_parse_sql() {
        // let sql = "SELECT a, b FROM table_1  WHERE a > b AND b < 100  ORDER BY a DESC, b";
        let sql = "SELECT t1.id as t_id,t2.name FROM table_1 t1 left join table_2 t2 on t1.id = t2.id where t1.age>10  ORDER BY t1.id desc";

        let dialect = MySqlDialect {};
        let mut ast = Parser::parse_sql(&dialect, sql).unwrap();

        println!("AST: {:?}", &ast);

        let statement = ast.pop().unwrap();

        match statement {
            Statement::Query(query) => {
                println!("query:{:#?}", &query);
            }
            Statement::Insert(_insert) => todo!(),
            Statement::Update {
                table: _,
                assignments: _,
                from: _,
                selection: _,
                returning: _,
            } => todo!(),
            Statement::Delete(_delete) => todo!(),
            _ => todo!(),
        }
    }
}
