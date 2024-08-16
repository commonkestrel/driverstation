use std::time::Duration;

use driverstation::Robot;

fn main() {
    let mut robot = Robot::new(8891);
    
    let mut enabled = false;
    loop {
        robot.set_enabled(enabled);
        enabled = !enabled;

        std::thread::sleep(Duration::from_secs(1));
    }
}