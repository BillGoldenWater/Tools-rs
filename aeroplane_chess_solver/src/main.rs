use aeroplane_chess_solver::Board;

fn main() {
    let mut board = Board::new();
    board.takeoff(1);
    let p = 1;
    println!("{:?}", board.flying().iter().collect::<Vec<_>>());
    println!("{:#?}", board.preview(p));
}
