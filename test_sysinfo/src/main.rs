use byte_unit::Byte;
use sysinfo::{RefreshKind, SystemExt};

fn main() {
    let sys = sysinfo::System::new_with_specifics(RefreshKind::new().with_memory());
    let total_mem = sys.total_memory();
    let free_mem = sys.free_memory();
    println!(
        "Total memory: {}",
        Byte::from_bytes(total_mem as u128).get_appropriate_unit(true)
    );
    println!(
        "Free memory: {}",
        Byte::from_bytes(free_mem as u128).get_appropriate_unit(true)
    );
    println!("Core count: {}", sys.physical_core_count().unwrap_or(0));
}
