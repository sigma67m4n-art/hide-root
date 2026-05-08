use serde::Deserialize;
use std::process::Command;
use std::fs;
use std::io::copy;

#[derive(Deserialize)]
struct Asset { browser_download_url: String }

#[derive(Deserialize)]
struct Release { assets: Vec<Asset> }

struct Repo {
    owner: &'static str,
    name: &'static str,
    label: &'static str,
    index: usize,
}

fn main() {
    let repos = vec![
        Repo { owner: "Dr-TSNG", name: "ZygiskNext", label: "Zygisk Next", index: 0 },
        Repo { owner: "KOWX712", name: "PlayIntegrityFix", label: "Play Integrity Fix", index: 0 },
        Repo { owner: "5ec1cff", name: "TrickyStore", label: "Tricky Store", index: 0 },
        Repo { owner: "KOWX712", name: "Tricky-Addon-Update-Target-List", label: "Tricky Addon", index: 0 },
        Repo { owner: "JingMatrix", name: "Vector", label: "Vector", index: 1 },
        Repo { owner: "frknkrc44", name: "HMA-OSS", label: "HMA-OSS", index: 1 },
        Repo { owner: "dpejoh", name: "specter", label: "Specter", index: 0 },
    ];

    let client = reqwest::blocking::Client::new();
    let mut failed = Vec::new();

    let install_cmd = if Command::new("ksud").arg("-V").output().is_ok() {
        vec!["ksud", "module", "install"]
    } else if Command::new("magisk").arg("-v").output().is_ok() {
        vec!["magisk", "--install-module"]
    } else {
        vec!["pm", "install"]
    };

    for repo in repos {
        println!("[*] Downloading {}...", repo.label);

        let url = format!("https://api.github.com/repos/{}/{}/releases/latest", repo.owner, repo.name);
        
        let dl_url = client.get(&url).send()
            .ok()
            .and_then(|r| r.json::<Release>().ok())
            .and_then(|r| r.assets.get(repo.index)
                .or(r.assets.first())
                .map(|a| a.browser_download_url.clone()));

        if let Some(url) = dl_url {
            let tmp_path = format!("/data/local/tmp/{}.tmp", repo.name);
            let is_apk = url.ends_with(".apk");

            if let Ok(mut resp) = client.get(&url).send() {
                if let Ok(mut f) = fs::File::create(&tmp_path) {
                    if copy(&mut resp, &mut f).is_ok() {
                        println!("[*] Installing {}...", repo.label);
                        
                        let status = if is_apk {
                            Command::new("pm").args(["install", &tmp_path]).status()
                        } else {
                            Command::new(install_cmd[0]).args(&install_cmd[1..]).arg(&tmp_path).status()
                        };

                        if status.map_or(true, |s| !s.success()) {
                            failed.push(repo.label);
                        }
                        let _ = fs::remove_file(&tmp_path);
                        continue;
                    }
                }
            }
        }
        failed.push(repo.label);
    }

    if failed.is_empty() {
        println!("[+] All resources downloaded");
    } else {
        println!("[!] Can't install {}", failed.join(", "));
    }
}