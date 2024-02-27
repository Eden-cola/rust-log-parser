extern crate clap;
use clap::{Arg, Command};

use std::fs::File;
use std::io::{self, prelude::*, BufReader};

struct StatePoint {
    target: char,
    same_pre_index: usize,
}

struct Expr {
    name: String,
    states: Vec<StatePoint>,
    x: usize, //初始化状态机使用的states下标,记录着和上一个字符具有最大相同前缀的字符
}

struct Matcher<'a> {
    expr: &'a Expr,
    y: usize, //匹配时使用的states下标,记录着当前匹配到的位置
    buf: Vec<char>,
}

impl Matcher<'_> {
    fn new<'a, 'b> (expr: &'a Expr) -> Matcher<'a> {
        Matcher{
            expr,
            y: 0,
            buf: Vec::new()
        }
    }

    fn is_matched(&self) -> bool {
        self.y > 0 && self.y == self.expr.states.len()
    }

    fn feed(&mut self, c: char) -> Option<String> {
        // println!("??? name: ({}), feed: ({}), y: ({}), len: ({})",self.name, c, self.y, self.states.len());
        if self.is_matched() {
            return Some(self.value());
        }

        self.buf.push(c);
        if self.expr.states.is_empty() {
            return None;
        }
        loop {
            let point = &self.expr.states[self.y];
            if point.target == c {
                // println!("name: ({}), feed: ({}), target: ({})",self.name, c, point.target);
                self.y += 1;
                break;
            }
            if self.y <= 0 {
                break;
            }
            self.y = point.same_pre_index;
        }
        if self.is_matched() { Some(self.value()) } else { None }
    }

    fn value(&self) -> String {
        let len = self.buf.len() - self.expr.states.len();

        let mut result = String::new();
        for i in 0..len {
            result.push(self.buf[i]);
        }
        result
    }

}

impl Expr {
    fn new() -> Expr {
        Expr {
            name: String::new(),
            states: Vec::new(),
            x: 0,
        }
    }
    fn append_flag(&mut self, c: char) {
        let _state_len = self.states.len();
        let state_point = StatePoint {
            target: c,
            same_pre_index: self.x,
        };
        self.states.push(state_point);
        while self.x > 0 {
            let same_pre_point = &self.states[self.x];
            if c == same_pre_point.target {
                self.x += 1;
                break;
            }
            self.x = same_pre_point.same_pre_index;
        }
    }
    fn append_name(&mut self, c: char) {
        self.name.push(c);
    }

    fn get_matcher(&self) -> Matcher {
        Matcher::new(&self)
    }

    fn _flag(&self) -> String {
        let mut flag = String::new();
        for point in &self.states {
            flag.push(point.target);
        }
        flag
    }

    fn format(&self, value: String) -> String {
        let mut result = String::new();
        result += "\"";
        result += &self.name;
        result += "\":\"";
        result += &value;
        result += "\"";
        result
    }
}

fn main() -> io::Result<()> {
    let matches = Command::new("lp")
        .version("0.0.1")
        .author("allen eden")
        .about("simple log file parser")
        .arg(
            Arg::new("file")
            .short('f')
            .help("log file")
        )
        .arg(
            Arg::new("expr")
            .short('e')
            .help("parser expr")
        )
        .get_matches();

    let expr_str = match matches.get_one::<String>("expr") {
        None => panic!("缺少expr"),
        Some(v) => v,
    };
    let file_path = match matches.get_one::<String>("file") {
        None => panic!("缺少file"),
        Some(v) => v,
    };

    let expr_list = parse_expr(expr_str);
    /*
    for e in &expr_list {
        println!("name:({}): flag({})", e.name, e._flag());
    }
    panic!("debug");

    let result = parse_str(file_path, &mut expr_list);
    println!("{}", result);
    */

    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    println!("[");
    for line in reader.lines() {
        if let Some(result) = parse_str(&line.unwrap(), &expr_list) {
            println!("{},", result);
        }
    }
    println!("]");

    Ok(())
}

enum State {
    FlagChar,
    NameChar,
}

fn parse_expr(expr_str: &str) -> Vec<Expr> {
    let mut expr_list = Vec::new();
    let mut expr = Expr::new();
    let mut state = State::FlagChar; // 1: 读取变量 0: 读取flag;

    for c in expr_str.chars() {
        let expr_to_save: Option<Expr>;
        (state, expr, expr_to_save) = match (&state, c) {
            (&State::FlagChar, '{') => (State::NameChar, Expr::new(), Some(expr)),
            (&State::FlagChar, c) => { expr.append_flag(c); (State::FlagChar, expr, None) },
            (&State::NameChar, '}') => (State::FlagChar, expr, None),
            (&State::NameChar, c) => { expr.append_name(c); (State::NameChar, expr, None)},
        };
        if let Some(expr_to_save) = expr_to_save {
            if expr_to_save.states.len() != 0 {
                expr_list.push(expr_to_save);
            } else if expr_list.len() != 0 {
                panic!("非法的expr: 连续变量");
            }
        };
    }
    if let State::NameChar = state {
        panic!("非法的expr: 未正确结尾");
    }
    expr_list.push(expr);
    expr_list
}

fn parse_str(line: &str, expr_list: &Vec<Expr>) -> Option<String> {
    // println!("start", );
    // println!("{}", &line);

    let mut result: Vec<String> = Vec::new();

    let mut char_iter = line.chars().into_iter();

    for expr in expr_list {
        let mut matcher = expr.get_matcher();
        while let Some(c) = char_iter.next() {
            if let Some(value) = matcher.feed(c) {
                if expr.name.len() > 0 && value.len() > 0 {
                    result.push(expr.format(value));
                }
                break
            }
        }
    }

    if result.len() > 0 {
        Some('{'.to_string() + &result.join(",") + &'}'.to_string())
    } else {
        None
    }
}
