use clap::Parser;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    repo: String,
    #[arg(default_value = ".")]
    dest: String,
}

impl Args {
    fn get_repo(&self) -> &str {
        self.repo.trim_matches('/')
    }
}

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    // let empty_dest = std::fs::read_dir(args.get_repo());

    let repo_archive = smu::github::get_archive_url(args.get_repo());

    if repo_archive.is_err() {
        println!("Error: {}", repo_archive.unwrap_err());
        return Ok(());
    }

    let archive = smu::download_file(repo_archive.unwrap().as_str()).await;

    let extract = smu::github::extract_archive(archive.unwrap(), args.dest);

    if extract.is_err() {
        println!("Error: {}", extract.unwrap_err());
        return Ok(());
    }

    println!("Downloaded...");

    Ok(())
}
