use std::process::exit;

pub struct Parameter {
    pub wait: u16,
    pub range_min: u32,
    pub range_max: u32,
    pub interface: String
}

impl Parameter {
    pub fn init() -> Self {
        Parameter {
            wait: 3,
            range_min: 1,
            range_max: 4094,
            interface: "eth0".to_string()
        }
    }
}

pub fn handle_arguments(args:Vec<String>) -> Parameter {
    let mut i = 1;
    let mut param = Parameter::init();
    while i < args.len() {
        let st = args[i].as_str();
        println!("{}", st);
        match args[i].as_str() {
            "-r" | "--range" => {
                if i + 1 < args.len() && args[i+1].starts_with("-") == false {
                    let min_max = args[i+1].split_once('-');
                    match min_max {
                        None => {
                            print_help()
                        }
                        Some(minmax) => {
                            param.range_min = minmax.0.parse().unwrap();
                            param.range_max = minmax.1.parse().unwrap();
                        }
                    }
                }
                i += 1;
            }
            "-i" | "--interface" => {
                if i + 1 < args.len() && args[i+1].starts_with("-") == false {
                    param.interface = args[i+1].to_string();
                    i += 1;
                } else {
                    println!("interface name has to be given");
                    print_help()
                }
            }
            "-w" | "--wait" => {
                if i + 1 < args.len() && args[i+1].starts_with("-") == false {
                    param.wait = args[i + 1].parse::<u16>().unwrap_or_else(|_| {
                        eprintln!("Error: -w wait time needs a valid number between 1 and 10.");
                        exit(2);
                    });
                } else {
                    println!("wait time must be set");
                    print_help()
                }
                i += 1;
            }
            _ => {
                print_help();
                exit(0);
            }
        }
        i += 1;
    }

    param
}


fn print_help() {
    println!(
        "
==========================================================
          VLAN Scanner
==========================================================
  A basic vlan scanning tool

Version: 1.0.0
Usage: vlan_scanner [OPTIONS]

Options:
  -h, --help          Print this help information
  -r , --range        vlan id range (f.e. -r 200-210)
  -i , --interface    interface name (f.e. -i eth1)
  -w, --wait          wait time definition in seconds

----------------------------------------------------------
  Visit https://github.com/r4ruk/vlan_scanner for more information or updates.
==========================================================
"
    );
}