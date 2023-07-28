
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File to be read and executed.
    #[arg(short, long)]
    file : String,

    /// Name of targets to be executed.
    #[arg(short, long)]
    targets : Vec<String>,

    /// How many threads we should use.
    #[arg(short, long, default_value_t = 1)]
    jobs: u32,
}

fn main() {
    let args = Args::parse();

    let bytes = std::fs::read(&args.file).unwrap();

    // time begin
    let now = std::time::Instant::now();

    let ret = remake_lib::parser::parse_from_bytes(bytes);

    if ret.is_err(){
        eprintln!("Failed to parse the `{}`:{}", &args.file,ret.err().unwrap());
        std::process::exit(1);
    }

    let ret = ret.unwrap();

    let mut executer = remake_lib::executer::Executer::new(args.jobs,ret.targets);

    let executed =    executer.execute(&args.targets);

    let errors =  executed.lock();

    // time end

    if errors.len() != 0{
        for error in errors.iter() {
            eprintln!("Runtime Error:{}",error)
        }
    }
    else{
        println!("Finished")
    }

    let used = now.elapsed();
    println!("Cost {}s {}ms",used.as_secs(),used.subsec_millis());

}
