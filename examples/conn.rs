use driverstation::Robot;

fn main() {
    let robot = Robot::new(8891);

    std::io::stdin().read_line(&mut String::new()).unwrap();
}
