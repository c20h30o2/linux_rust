use std::env;
use std::error::Error;
use std::fs;

pub struct Config {
    pub query: String,
    pub filename: String,
    pub case_sensitive: bool,
}

impl Config {
    // 原本尝试将这里的参数  args: &[String]  修改为 args: [String] 但是报错
    //     因为 Rust 中有「动态大小类型（DST）」和「借用」的概念：
    // [T]（比如 [String]）是动态大小类型，不知道编译时大小，所以不能按值传递（函数参数必须有已知大小）。
    // &[T] 是对切片的引用，是一个 fat pointer（指针 + 长度），其大小在编译时已知，因此可以作为函数参数。
    // 注意类型名是 String（大写 S），不是 string。
    pub fn new(args: Vec<String>) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments");
        }
        // 在不消费 Vec 的情况下克隆（有拷贝开销）：
        // let query = args[1].clone();
        // let filename = args[2].clone();

        // 为了避免拷贝开销，尝试使用这样的方法直接转移所有权，但是会报错
        // let query = args[1]; // 注意，对 Vec<T> 使用索引（args[1]）得到的是对元素的借用（Index 返回引用），
        //                      // 不能通过索引直接把元素“挖出”并转移所有权；如果允许这样移动，会在 Vec 中留下空洞，破坏内存安全，
        //                      // 所以编译器不允许。编译器因此提示要么借用（&args[1]）要么 clone，要么用其他会消费 Vec 的方法来取得所有权
        // let filename = args[2];// 因为上述原因，这里需要采用下面的方法来使得所有权转移

        let mut iter = args.into_iter(); // 使用into_iter直接消费整个args
        iter.next(); // 跳过可执行文件名
        let query = iter.next().ok_or("not enough arguments")?;
        let filename = iter.next().ok_or("not enough arguments")?;

        // evc::var 从系统中获取同名的环境变量 可以通过 export CASE_INSENSITIVE=1 来设置
        // 也可以直接在运行时带上 CASE_INSENSITIVE=1 cargo run to poem.txt
        let case_sensitive = env::var("CASE_INSENSITIVE").is_err();

        Ok(Config {
            query,
            filename,
            case_sensitive,
        })
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

pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let mut results = Vec::new();
    for line in contents.lines() {
        if line.contains(query) {
            results.push(line);
        }
    }
    results
}
