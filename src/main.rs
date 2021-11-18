extern crate clap;

use clap::{Arg, App};

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
    y: usize, //匹配时使用的states下标,记录着当前匹配到的位置
    buf: Vec<char>,
}

impl Expr {
    fn new() -> Expr {
        Expr {
            name: String::new(),
            states: Vec::new(),
            x: 0,
            y: 0,
            buf: Vec::new(),
        }
    }
    fn reset(&mut self) {
        self.y = 0;
        self.buf.clear();
    }
    fn appendFlag(&mut self, c: char) {
        let state_len = self.states.len();
        let state_point = StatePoint{ target: c, same_pre_index: self.x};
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
    fn appendName(&mut self, c: char) {
        self.name.push(c);
    }

    fn is_matched(&self) -> bool {
        self.y > 0 && self.y == self.states.len()
    } 

    fn feed(&mut self, c: char) -> bool {
        // println!("??? name: ({}), feed: ({}), y: ({}), len: ({})",self.name, c, self.y, self.states.len());
        if self.is_matched() {
            return true;
        }

        self.buf.push(c);
        if (self.states.is_empty()) {
            return false;
        }
        loop {
            let point = &self.states[self.y];
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
        self.is_matched()
    }
    fn flag(&self) -> String {
        let mut flag = String::new();
        for point in &self.states {
            flag.push(point.target);
        };
        flag
    }
    fn value(&self) -> String {
        let len = self.buf.len() - self.states.len();

        let mut result = String::new();
        for i in 0..len {
            result.push(self.buf[i]);
        }
        result
    }

    fn format(&self) -> String {
        let mut result = String::new();
        result += "\"";
        result += &self.name;
        result += "\":\"";
        result += &self.value();
        result += "\"";
        result
    }
}

fn main() -> io::Result<()> {
    let matches = App::new("lp")
    .version("0.0.1")
    .author("allen eden")
    .about("simple log file parser")
    .arg(Arg::with_name("file")
        .help("log file")
        .empty_values(false)
    )
    .arg(Arg::with_name("expr")
        .help("parser expr")
        .empty_values(false)
    )
    .get_matches();

    let expr_str = match matches.value_of("expr") {
        None => panic!("缺少expr"),
        Some(v) => v,
    };
    let file_path = match matches.value_of("file") {
        None => panic!("缺少file"),
        Some(v) => v,
    };

    let mut expr_list = Vec::new();
    parse_expr(expr_str, &mut expr_list);
    /*
    for e in &expr_list {
        println!("name:({}): flag({})", e.name, e.flag());
    }
    panic!("debug");
    
    let result = parse_str(file_path, &mut expr_list);
    println!("{}", result);
    */

    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    println!("[");
    for line in reader.lines() {
        let result = parse_str(&line.unwrap(), &mut expr_list);
        println!("{},", result);
    }
    println!("]");

    Ok(())
}


fn parse_expr(expr_str: &str, expr_list: &mut Vec<Expr>) {
    let mut expr = Expr::new();
    let mut state = 0; // 1: 读取变量 0: 读取flag;
    for c in expr_str.chars() {
        if state == 0 {
            if c == '{' {
                state = 1;
                if expr.states.len() != 0 {
                    expr_list.push(expr);
                } else if expr_list.len() != 0 {
                    panic!("非法的expr: 连续变量");
                }
                expr = Expr::new();
            } else {
                expr.appendFlag(c);
            }
        } else {
            if c == '}' {
                state = 0;
            } else {
                expr.appendName(c);
            }
        }
    };
    if state == 1 {
        panic!("非法的expr: 未正确结尾");
    }
    expr_list.push(expr);
}

fn parse_str(line: &str, expr_list: &mut Vec<Expr>) -> String {
    // println!("start", );
    // println!("{}", &line);

    let mut result: Vec<String> = Vec::new();

    let mut i = 0;

    let mut expr = match expr_list.get_mut(i) {
        None => panic!("invalid index"),
        Some(expr) => expr,
    };
    // println!("try match {}", expr.name);

    for c in line.chars() {
        if expr.feed(c) {
            i += 1;
            expr = match expr_list.get_mut(i) {
                None => break,
                Some(expr) => expr,
            };
            // println!("try match {}", expr.name);
        }
    }

    for e in expr_list {
        if !e.buf.is_empty() && !e.name.is_empty() {
            result.push(e.format());
        }
        e.reset();
    }

    '{'.to_string() + &result.join(",") + &'}'.to_string()
}
