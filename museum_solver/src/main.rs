use itertools::Itertools;
use museum_solver::{solve, Attribute, MemberInfo, Zone};

fn main() {
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
    ),
    Zone::new(
      "综合区-内",
      Attribute::new(80, 80, 80),
      Attribute::new(4, 8, 2),
      Attribute::new(205, 245, 150),
    ),
  ];

  let solve_result = solve(members, zones);

  if let Some(solve_result) = solve_result {
    println!("{:?}", solve_result.0);

    for (zone, members) in solve_result.1 {
      println!(
        "{} {:?}",
        zone.name,
        members.iter().map(|it| &it.name).collect_vec()
      )
    }
  } else {
    println!("no result");
  }
}
