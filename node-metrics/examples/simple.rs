use sysinfo::{System, SystemExt, DiskExt, ProcessorExt};
fn main () {

let mut sys = System::new_all();

// We display the disks:
println!("=> Disk list:");
for disk in sys.disks() {
    println!("Name : {:?}", disk.name());
    println!("total space: {} bytes", disk.total_space());
    println!("free space : {} bytes", disk.available_space());
}

// Memory information:
println!("=> Memory");
println!("total memory: {} KB", sys.total_memory());
println!("free memory : {} KB", sys.total_memory() - sys.used_memory());

// Number of processors
println!("=> CPU");
sys.refresh_all();

let cpu_amount = sys.processors().len();
println!("Number of CPU: {}", cpu_amount);
let mut avg_cpu_usage = 0.0;
for cpu in sys.processors() {
    let cpu_usage = cpu.cpu_usage();
    println!("{:?}%", cpu_usage);
    avg_cpu_usage += cpu_usage;
}
avg_cpu_usage /= cpu_amount as f32;
println!("avg_cpu_usage {:?}%", avg_cpu_usage);

// To refresh all system information:
}