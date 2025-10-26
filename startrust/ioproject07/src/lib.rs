use std::error::Error;
use std::fs;
use std::env;

pub struct Config {
    pub query: String,
    pub filename: String,
    pub case_sensitive: bool,
}


// 此处的new由ioproject05修改而来，起初这里需要 clone 的原因是参数 args 中有一个 String 元素的 slice，
// 而 new 函数并不拥有 args。为了能够返回 Config 实例的所有权，我们需要克隆 Config 中字段 query 和 filename 的值，这样 Config 实例就能拥有这些值。
// 在学习了迭代器之后，我们可以将 new 函数改为获取一个有所有权的迭代器作为参数而不是借用 slice。我们将使用迭代器功能之前检查 slice 长度和索引特定位置的代码。
// 这会明确 Config::new 的工作因为迭代器会负责访问这些值。
// 一旦 Config::new 获取了迭代器的所有权并不再使用借用的索引操作，就可以将迭代器中的 String 值移动到 Config 中，而不是调用 clone 分配新的空间。
// env::args 函数的标准库文档显示，它返回的迭代器的类型为 std::env::Args
// 因为我们拥有 args 的所有权，并且将通过对其进行迭代来改变 args，所以我们可以将 mut 关键字添加到 args 参数的规范中以使其可变。
// 接下来，我们将修改 Config::new 的内容。标准库文档还提到 std::env::Args 实现了 Iterator trait，因此我们知道可以对其调用 next 方法！
// 请记住 env::args 返回值的第一个值是程序的名称。我们希望忽略它并获取下一个值，所以首先调用 next 并不对返回值做任何操作。
impl Config {
    pub fn new(mut args:std::env::Args ) -> Result<Config, &'static str> {
        args.next();

        let query = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a query string"),
        };

        let filename = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a file name"),
        };

        let case_sensitive = env::var("CASE_INSENSITIVE").is_err();

        Ok(Config { query, filename, case_sensitive })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.filename)?;

    let results = if config.case_sensitive {
        search(&config.query, &contents)
    } else {
        search_case_insensitive(&config.query, &contents)
    };

    for line in results {
        println!("{}", line);
    }

    Ok(())
}

pub fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let query = query.to_lowercase();
    let mut results = Vec::new();

    for line in contents.lines() {
        if line.to_lowercase().contains(&query) {
            results.push(line);
        }
    }

    results
}
// 测试函数

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_sensitive() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.";

        assert_eq!(vec!["safe, fast, productive."], search(query, contents));
    }

    #[test]
    fn case_insensitive() {
        let query = "rUsT";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

        assert_eq!(
            vec!["Rust:", "Trust me."],
            search_case_insensitive(query, contents)
        );
    }
}

// 相比于使用这里的for循环方法，可以用使用迭代器的方法来替换
// pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
//     let mut results = Vec::new();
//     for line in contents.lines() {
//         if line.contains(query) {
//             results.push(line);
//         }
//     }
//     results
// }

// 可以通过使用迭代器适配器方法来编写更简明的代码。这也避免了一个可变的中间 results vector 的使用。
// 函数式编程风格倾向于最小化可变状态的数量来使代码更简洁。去掉可变状态可能会使得将来进行并行搜索的增强变得更容易，因为我们不必管理 results vector 的并发访问
// 最终返回的是contents中的不可变引用
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    contents.lines()
        .filter(|line| line.contains(query))
        .collect()
}

