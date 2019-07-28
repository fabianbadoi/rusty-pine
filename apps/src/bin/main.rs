
fn main() {
    //    use pine_transpiler::{MySqlTranspiler, Transpiler};
    //
    //    let transpiler = MySqlTranspiler::default();
    //
    //    // normal flow
    //    println!("------------------------------");
    //    println!(
    //        "{}",
    //        transpiler.transpile("from: users | where: id = 3").unwrap()
    //    );
    //    println!("------------------------------");
    //
    //    // faulty limit
    //    println!("------------------------------");
    //    println!(
    //        "{}",
    //        transpiler
    //            .transpile("from: users | l: 500000000000000000000000000")
    //            .unwrap_err()
    //    );
    //    println!("------------------------------");
    //
    //    // syntax error 1
    //    println!("------------------------------");
    //    println!(
    //        "{}",
    //        transpiler
    //            .transpile("from: users | filter: id = 3 | select: id")
    //            .unwrap_err()
    //    );
    //    println!("------------------------------");
    //
    //    // syntax erro 2
    //    println!("------------------------------");
    //    println!(
    //        "{}",
    //        transpiler
    //            .transpile("from: users | where: id  3 3 id | select: id")
    //            .unwrap_err()
    //    );
    //    println!("------------------------------");
    //
    //    // query builder flow
    //    println!("------------------------------");
    //    println!(
    //        "{}",
    //        transpiler
    //            .transpile("where: id = 3 | select: id")
    //            .unwrap_err()
    //    );
    //    println!("------------------------------");
    //
    //    println!("------------------------------");
    //    println!("{}", transpiler.transpile("users 3").unwrap());
    //    println!("------------------------------");
}