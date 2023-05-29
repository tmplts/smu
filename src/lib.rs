pub mod github {
    pub fn get_archive_url(repo_path: &str) -> Result<String, git2::Error> {
        let base_url = "https://github.com";
        let url = format!("{}/{}.git", base_url, repo_path);
        let mut repo = git2::Remote::create_detached(url).unwrap();

        repo.connect(git2::Direction::Fetch).unwrap();

        let list = repo
            .list()?
            .iter()
            .map(|x| (x.name().to_string(), x.oid().to_string()))
            .filter(|x| x.0 == "HEAD" || x.0.contains("main") || x.0.contains("master"))
            .collect::<Vec<_>>();

        if list.len() == 0 {
            return Err(git2::Error::from_str("No heads found"));
        }

        // Get a head in order of preference (HEAD, main, master)
        let head = list
            .iter()
            .find(|x| x.0 == "HEAD")
            .or_else(|| list.iter().find(|x| x.0.contains("main")))
            .or_else(|| list.iter().find(|x| x.0.contains("master")));

        if head.is_none() {
            return Err(git2::Error::from_str("No heads found"));
        }

        Ok(format!(
            "{}/{}/archive/{}.tar.gz",
            base_url,
            repo_path,
            head.unwrap().1
        ))
    }

    pub fn extract_archive(archive: Vec<u8>, dest: String) -> std::io::Result<()> {
        let tar = flate2::read::GzDecoder::new(archive.as_slice());
        let mut archive = tar::Archive::new(tar);

        if dest != "." {
            std::fs::create_dir_all(dest.clone())?;
        }

        archive
            .entries()?
            .filter_map(|e| e.ok())
            .map(|mut entry| -> std::io::Result<String> {
                let entry_path = entry.path()?;
                let mut path = entry_path
                    .to_str()
                    .unwrap()
                    .split("/")
                    .collect::<Vec<&str>>();

                // Replace the first element with the destination directory.
                path[0] = dest.trim_end_matches('/');

                let path = path.join("/");

                entry.unpack(&path)?;
                Ok(path)
            })
            .all(|e| e.is_ok());

        Ok(())
    }
}

pub async fn download_file(url: &str) -> surf::Result<Vec<u8>> {
    println!("Downloading: {}", url);
    let mut archive = surf::get(url).await?;

    let mut redirects = 0;
    while archive.status() == 301 || archive.status() == 302 {
        let redirect = archive.header("location").unwrap().get(0).unwrap().as_str();

        println!("Redirected to: {}", redirect);

        archive = surf::get(redirect).await.unwrap();

        redirects += 1;
        if redirects > 5 {
            panic!("Too many redirects");
        }
    }

    archive.body_bytes().await
}
