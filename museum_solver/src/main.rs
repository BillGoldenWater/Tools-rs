use std::fmt::Display;
use std::io::{stdin, stdout, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use museum_solver::{solve, Attribute, MemberInfo, Zone};

#[derive(Serialize, Deserialize)]
struct State {
  members: Vec<MemberInfo>,
  zones: Vec<Zone>,
}

impl State {
  pub fn solve_print(&self) {
    let members = self.members.iter().cloned().collect_vec();
    let zones = self.zones.iter().cloned().collect_vec();

    let solve_result = solve(members, zones);

    if let Some(solve_result) = solve_result {
      print!("结果:");
      if solve_result.0.require > 0 || solve_result.0.overflow > 0 {
        print!(" (");
        if solve_result.0.require > 0 {
          print!(" 差距总和: {}", solve_result.0.require);
        }
        if solve_result.0.overflow > 0 {
          print!(" 溢出总和: {}", solve_result.0.overflow);
        }
        print!(" )")
      }
      println!();

      for (zone, members) in solve_result.1 {
        let attr = zone.calc_detail(&members);
        indent(2);
        println!("{}: ", zone.name);
        indent(4);
        println!(
          "助理: {}",
          members
            .iter()
            .map(|it| &it.name)
            .cloned()
            .collect_vec()
            .join(", ")
        );

        println_attr(4, &"游览时长", attr.time, zone.require.time);
        println_attr(4, &"科普价值", attr.value, zone.require.value);
        println_attr(4, &"吸引人流", attr.popularity, zone.require.popularity);
      }
    } else {
      println!("计算失败");
    }
  }

  pub fn clear(&mut self) {
    self.members.clear();
    self.zones.clear();
  }

  pub fn print(&self) {
    if !self.members.is_empty() {
      println!("助理: ");
      for member in &self.members {
        indent(2);
        println!("{}: ", member.name);

        indent(4);
        print!(" 游览时长: {:<2}", member.attr.time);
        print!(" 科普价值: {:<2}", member.attr.value);
        print!(" 吸引人流: {:<2}", member.attr.popularity);
        println!()
      }
    }

    if !self.members.is_empty() {
      println!("区域: ");
      for zone @ Zone {
        name,
        sub_level,
        require,
        ..
      } in &self.zones
      {
        indent(2);
        println!("{}:", name);

        let base = zone.calc_detail(&[]);
        println_attr(4, &"游览时长", base.time, require.time);
        println_attr(4, &"科普价值", base.value, require.value);
        println_attr(4, &"吸引人流", base.popularity, require.popularity);

        indent(4);
        println!("游览导引: {}", sub_level.time);
        indent(4);
        println!("科普物料: {}", sub_level.value);
        indent(4);
        println!("游客宣传: {}", sub_level.popularity);
      }
    }

    if self.members.is_empty() && self.zones.is_empty() {
      println!("[空]");
    }
  }

  pub fn add_member(&mut self, name: impl Into<String>, attr: Attribute) {
    let name = name.into();
    self.del_member(&name);
    self.members.push(MemberInfo::new(name, attr));
  }

  pub fn del_member(&mut self, name: impl Into<String>) {
    let name = name.into();
    self.members.retain(|it| it.name != name)
  }

  pub fn add_zone(
    &mut self,
    name: impl Into<String>,
    base: Attribute,
    sub_level: Attribute,
    require: Attribute,
  ) {
    let name = name.into();
    self.del_zone(&name);
    self
      .zones
      .push(Zone::new(name, base, sub_level, require, 100))
  }

  pub fn del_zone(&mut self, name: impl Into<String>) {
    let name = name.into();
    self.zones.retain(|it| it.name != name);
  }

  pub fn update_zone_level(
    &mut self,
    name: impl Into<String>,
    sub_level: Attribute,
  ) -> Result<(), bool> {
    if let Some(zone) = self.zone_by_name_mut(name) {
      zone.sub_level = sub_level;
      Ok(())
    } else {
      unknown_usage()
    }
  }

  pub fn update_zone_require(
    &mut self,
    name: impl Into<String>,
    require: Attribute,
  ) -> Result<(), bool> {
    if let Some(zone) = self.zone_by_name_mut(name) {
      zone.require = require;
      Ok(())
    } else {
      unknown_usage()
    }
  }

  pub fn update_zone_scaler(&mut self, name: impl Into<String>, scaler: u64) -> Result<(), bool> {
    if let Some(zone) = self.zone_by_name_mut(name) {
      zone.base_scaler = scaler;
      Ok(())
    } else {
      unknown_usage()
    }
  }

  fn zone_by_name_mut(&mut self, name: impl Into<String>) -> Option<&mut Zone> {
    let name = name.into();
    self.zones.iter_mut().find(|it| it.name == name)
  }

  pub fn load(&mut self, path: impl AsRef<Path>) -> std::io::Result<()> {
    let content = std::fs::read(path)?;
    *self = serde_json::from_slice(&content).expect("failed to parse state");
    Ok(())
  }

  pub fn save(&self, path: impl AsRef<Path>) {
    if let Some(parent) = path.as_ref().parent() {
      std::fs::create_dir_all(parent).expect("failed to create parent folder");
    }
    std::fs::write(path, serde_json::to_string(self).unwrap()).expect("failed to write state");
  }
}

fn main() {
  let mut state = State::default();
  let mut refresh = true;

  let _ = state.load(PathBuf::from("state.json"));

  loop {
    let result = loop_once(&mut state, &mut refresh);
    if let Err(break_loop) = result {
      if break_loop {
        break;
      }
      refresh = false;
    }
  }
}

fn loop_once(state: &mut State, refresh: &mut bool) -> Result<(), bool> {
  if *refresh {
    state.print();
    print_help_short();
  } else {
    *refresh = true;
  }

  let (command, args) = read_command();

  match command.as_str() {
    "addm" => {
      check_arg(&args, 2)?;

      state.add_member(&args[0], parse_attr(&args[1])?)
    }
    "delm" => {
      check_arg(&args, 1)?;
      state.del_member(&args[0]);
    }

    "addz" => {
      check_arg(&args, 7)?;
      if args[1] != "base" || args[3] != "lvl" || args[5] != "req" {
        return unknown_usage();
      }

      state.add_zone(
        &args[0],
        parse_attr(&args[2])?,
        parse_attr(&args[4])?,
        parse_attr(&args[6])?,
      )
    }
    "delz" => {
      check_arg(&args, 1)?;

      state.del_zone(&args[0]);
    }
    "updz" => {
      check_arg(&args, 3)?;

      match args[1].as_str() {
        "lvl" => state.update_zone_level(&args[0], parse_attr(&args[2])?),
        "req" => state.update_zone_require(&args[0], parse_attr(&args[2])?),
        "scaler" => state.update_zone_scaler(&args[0], parse_i64(&args[2])? as u64),
        _ => return unknown_usage(),
      }?;
    }

    "load" => {
      let result = state.load(PathBuf::from(
        &args
          .get(0)
          .cloned()
          .unwrap_or_else(|| String::from("state.json")),
      ));

      if let Err(err) = result {
        println!("加载失败: {err:?}");
        return Err(false);
      }
    }
    "save" => {
      state.save(PathBuf::from(
        &args
          .get(0)
          .cloned()
          .unwrap_or_else(|| String::from("state.json")),
      ));
      println!("保存成功");
      *refresh = false;
    }
    "clear" => state.clear(),

    "solve" => {
      state.solve_print();
      *refresh = false;
    }

    "help" => {
      print_help();
      *refresh = false;
    }
    "exit" => return Err(true),
    &_ => {
      println!("未知的命令");
      print_help_short();
      return Err(false);
    }
  }

  Ok(())
}

fn check_arg(args: &[String], len: usize) -> Result<(), bool> {
  if args.len() != len {
    return unknown_usage();
  }

  Ok(())
}

fn unknown_usage() -> Result<(), bool> {
  println!("未知用法");
  print_help_short();
  Err(false)
}

fn parse_attr(input: impl AsRef<str>) -> Result<Attribute, bool> {
  let attr = input
    .as_ref()
    .split('/')
    .map(parse_i64)
    .collect::<Result<Vec<_>, _>>()?;

  if attr.len() != 3 {
    unknown_usage()?;
  }

  Ok(Attribute::new(attr[0], attr[1], attr[2]))
}

fn parse_i64(input: impl AsRef<str>) -> Result<i64, bool> {
  i64::from_str(input.as_ref()).map_err(|_| unknown_usage().unwrap_err())
}

fn read_command() -> (String, Vec<String>) {
  print!(">");
  stdout().flush().expect("failed to flush stdout");
  let mut buf = String::new();
  stdin()
    .read_line(&mut buf)
    .expect("failed to read user input");

  let mut split = buf.trim().split(' ');

  let command = if let Some(cmd) = split.next() {
    cmd.to_string()
  } else {
    println!("请输入正确的命令");
    return read_command();
  };

  (command, split.map(str::to_string).collect())
}

fn print_help() {
  println!(
    r#"
命令:
  solve
    计算结果

  addm <名字> <游览时长>/<科普价值>/<吸引人流>
    添加助理
  delm <名字>
    移除助理

  addz <名字> base <游览时长>/<科普价值>/<吸引人流> lvl <游览导引>/<科普物料>/<游客宣传> req <游览时长>/<科普价值>/<吸引人流>
    添加区域
      base 后方为基础数值, 可通过 "无助理时的值 - 相应等级 * 10" 计算得出
      lvl 后方为当前的等级
      req 后方为需要的数值
      例: addz 综合区-内 base 80/80/80 lvl 3/6/2 req 205/245/150
  delz <名字>
    移除区域
  updz <名字> lvl <游览导引>/<科普物料>/<游客宣传>
    更新区域等级
  updz <名字> req <游览时长>/<科普价值>/<吸引人流>
    更新区域需求
  updz <名字> scaler <百分比>
    更新区域基础数值缩放

  load <路径>
    从文件加载状态 默认 "state.json" (启动时自动尝试从 "state.json" 加载)
  save <路径>
    保存状态至文件 默认 "state.json"
  
  clear
    清空状态

  exit
    退出  
"#
  )
}

fn print_help_short() {
  println!("输入 help 查看帮助");
}

fn indent(num: usize) {
  print!("{0:>1$}", "", num);
}

fn println_attr(indent_size: usize, name: &dyn Display, current: i64, require: i64) {
  indent(indent_size);
  print!("{name}: ");
  print!("{}/{}", current, require);
  if current != require {
    print!(" {:+}", current - require);
  }
  println!()
}

impl Default for State {
  fn default() -> Self {
    let members = vec![
      MemberInfo::new("娜塔莎", Attribute::new(56, 15, 64)),
      MemberInfo::new("希露瓦", Attribute::new(68, 57, 10)),
      MemberInfo::new("帕金斯", Attribute::new(42, 22, 65)),
      MemberInfo::new("匹克", Attribute::new(44, 54, 10)),
      MemberInfo::new("罗希", Attribute::new(54, 54, 0)),
      MemberInfo::new("狡猾的小孩子", Attribute::new(8, 58, 42)),
      MemberInfo::new("尤利安", Attribute::new(52, 20, 36)),
      MemberInfo::new("吉尔伯特", Attribute::new(36, 40, 20)),
      MemberInfo::new("莉拉", Attribute::new(52, 14, 30)),
      MemberInfo::new("费斯曼", Attribute::new(50, 29, 17)),
      MemberInfo::new("佩拉", Attribute::new(30, 30, 30)),
      MemberInfo::new("亚诺", Attribute::new(40, 8, 30)),
      MemberInfo::new("伊蕾恩", Attribute::new(26, 26, 26)),
      MemberInfo::new("希露瓦的狂热粉丝", Attribute::new(20, 44, 14)),
    ];

    let zones = vec![
      Zone::new(
        "综合区-外",
        Attribute::new(30, 30, 30),
        Attribute::new(10, 10, 10),
        Attribute::new(256, 220, 255),
        100,
      ),
      Zone::new(
        "综合区-内",
        Attribute::new(80, 80, 80),
        Attribute::new(4, 8, 2),
        Attribute::new(205, 245, 150),
        100,
      ),
    ];

    Self { members, zones }
  }
}
